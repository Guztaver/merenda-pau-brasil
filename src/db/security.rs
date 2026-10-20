use super::Db;
use crate::validation::{sql, R};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const ROLES: [&str; 4] = ["Administrador", "Estoque", "Nutricionista", "Consulta"];
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub crn: String,
}
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(super) fn stamp() -> String {
    chrono::Utc::now().to_rfc3339()
}
pub(super) fn audit(
    c: &Connection,
    actor: Option<i64>,
    action: &str,
    entity: &str,
    detail: &str,
) -> R<()> {
    let previous: String = c
        .query_row("SELECT hash FROM audit ORDER BY id DESC LIMIT 1", [], |r| {
            r.get(0)
        })
        .optional()
        .map_err(sql)?
        .unwrap_or_default();
    let time = stamp();
    let body = serde_json::to_vec(&(&previous, &time, actor, action, entity, detail))
        .map_err(|e| e.to_string())?;
    c.execute(
        "INSERT INTO audit(stamp,actor,action,entity,detail,previous,hash) VALUES (?,?,?,?,?,?,?)",
        params![time, actor, action, entity, detail, previous, digest(&body)],
    )
    .map_err(sql)?;
    Ok(())
}
fn password_hash(password: &str) -> R<String> {
    if password.chars().count() < 12 || password.len() > 256 {
        return Err("A senha deve ter pelo menos 12 caracteres e no máximo 256 bytes.".into());
    }
    Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}
fn validate_user(name: &str, login: &str, role: &str, crn: &str) -> R<()> {
    if name.trim().is_empty()
        || login.len() < 3
        || !login
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err("Informe o nome e um login de pelo menos 3 caracteres (letras, números, ponto, hífen ou sublinhado).".into());
    }
    if !ROLES.contains(&role) || (role == "Nutricionista" && crn.trim().is_empty()) {
        return Err("Selecione um perfil válido. Nutricionista precisa informar o CRN.".into());
    }
    Ok(())
}
impl Db {
    pub fn needs_setup(&self) -> bool {
        self.c
            .query_row("SELECT COUNT(*)=0 FROM users", [], |r| r.get(0))
            .unwrap_or(false)
    }
    pub fn user(&self) -> Option<&User> {
        self.actor.as_ref()
    }
    pub(super) fn require(&self, roles: &[&str]) -> R<i64> {
        let id = self
            .actor
            .as_ref()
            .ok_or("Entre com seu usuário para continuar.")?
            .id;
        let current: Option<(String, bool)> = self
            .c
            .query_row("SELECT role,active FROM users WHERE id=?", [id], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .optional()
            .map_err(sql)?;
        match current {
            Some((role, true)) if role == "Administrador" || roles.contains(&role.as_str()) => {
                Ok(id)
            }
            _ => Err("Seu perfil não tem permissão para esta operação.".into()),
        }
    }
    pub(super) fn require_rt(&self) -> R<User> {
        let id = self.require(&[])?;
        let u = self
            .c
            .query_row(
                "SELECT id,name,role,crn FROM users WHERE id=? AND active=1",
                [id],
                |r| {
                    Ok(User {
                        id: r.get(0)?,
                        name: r.get(1)?,
                        role: r.get(2)?,
                        crn: r.get(3)?,
                    })
                },
            )
            .map_err(sql)?;
        if u.role != "Nutricionista" || u.crn.trim().is_empty() {
            return Err("A aprovação exige o perfil Nutricionista e o CRN cadastrado.".into());
        }
        Ok(u)
    }
    pub fn setup_admin(&mut self, name: &str, login: &str, password: &str) -> R<()> {
        validate_user(name, login, "Administrador", "")?;
        let hash = password_hash(password)?;
        let t = self.c.transaction().map_err(sql)?;
        if t.query_row("SELECT COUNT(*) FROM users", [], |r| r.get::<_, i64>(0))
            .map_err(sql)?
            != 0
        {
            return Err("A configuração inicial já foi concluída.".into());
        }
        t.execute(
            "INSERT INTO users(name,login,password_hash,role) VALUES (?,?,?,'Administrador')",
            params![name.trim(), login, hash],
        )
        .map_err(sql)?;
        let id = t.last_insert_rowid();
        audit(&t, Some(id), "CONFIGURAR_ADMIN", "users", name.trim())?;
        t.commit().map_err(sql)?;
        self.actor = Some(User {
            id,
            name: name.trim().into(),
            role: "Administrador".into(),
            crn: String::new(),
        });
        Ok(())
    }
    pub fn login(&mut self, login: &str, password: &str) -> R<()> {
        self.actor = None;
        let row = self
            .c
            .query_row(
                "SELECT id,name,role,crn,password_hash FROM users WHERE login=? AND active=1",
                [login.trim()],
                |r| {
                    Ok((
                        User {
                            id: r.get(0)?,
                            name: r.get(1)?,
                            role: r.get(2)?,
                            crn: r.get(3)?,
                        },
                        r.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()
            .map_err(sql)?;
        if let Some((user, hash)) = row {
            if PasswordHash::new(&hash).ok().is_some_and(|h| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &h)
                    .is_ok()
            }) {
                audit(&self.c, Some(user.id), "ENTRAR", "users", login.trim())?;
                self.actor = Some(user);
                if let Err(error) = self.verify_audit() {
                    self.actor = None;
                    return Err(error);
                }
                return Ok(());
            }
        }
        audit(&self.c, None, "LOGIN_RECUSADO", "users", login.trim())?;
        Err("Login ou senha inválidos.".into())
    }
    pub fn logout(&mut self) {
        if let Some(u) = self.actor.take() {
            let _ = audit(&self.c, Some(u.id), "SAIR", "users", "");
        }
    }
    pub fn create_user(
        &mut self,
        name: &str,
        login: &str,
        password: &str,
        role: &str,
        crn: &str,
    ) -> R<()> {
        let actor = self.require(&["Administrador"])?;
        validate_user(name, login, role, crn)?;
        let hash = password_hash(password)?;
        let t = self.c.transaction().map_err(sql)?;
        t.execute(
            "INSERT INTO users(name,login,password_hash,role,crn) VALUES (?,?,?,?,?)",
            params![name.trim(), login, hash, role, crn.trim()],
        )
        .map_err(sql)?;
        audit(
            &t,
            Some(actor),
            "CRIAR_USUARIO",
            "users",
            &format!("{login}; {role}; {crn}"),
        )?;
        t.commit().map_err(sql)
    }
    pub fn set_user_active(&mut self, id: i64, active: bool, reason: &str) -> R<()> {
        let actor = self.require(&["Administrador"])?;
        if reason.trim().is_empty() || actor == id {
            return Err("Informe a justificativa. Não desative seu próprio usuário.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        if t.execute("UPDATE users SET active=? WHERE id=?", params![active, id])
            .map_err(sql)?
            != 1
        {
            return Err("Usuário não encontrado.".into());
        }
        audit(
            &t,
            Some(actor),
            "ATIVAR_DESATIVAR",
            "users",
            &format!("{id}; ativo={active}; {reason}"),
        )?;
        t.commit().map_err(sql)
    }
    pub fn users(&self) -> R<Vec<(i64, String, String, bool)>> {
        self.require(&["Administrador"])?;
        let mut s = self
            .c
            .prepare("SELECT id,name,role,active FROM users ORDER BY name")
            .map_err(sql)?;
        let rows = s
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }
    pub fn reset_user_password(&mut self, id: i64, password: &str, reason: &str) -> R<()> {
        let actor = self.require(&["Administrador"])?;
        if reason.trim().len() < 5 {
            return Err("Informe a justificativa da redefinição.".into());
        }
        let hash = password_hash(password)?;
        let t = self.c.transaction().map_err(sql)?;
        if t.execute(
            "UPDATE users SET password_hash=? WHERE id=?",
            params![hash, id],
        )
        .map_err(sql)?
            != 1
        {
            return Err("Usuário não encontrado.".into());
        }
        audit(
            &t,
            Some(actor),
            "REDEFINIR_SENHA",
            "users",
            &format!("id={id}; {reason}"),
        )?;
        t.commit().map_err(sql)
    }
    pub fn verify_audit(&self) -> R<usize> {
        let actor = self
            .actor
            .as_ref()
            .ok_or("Entre com seu usuário para continuar.")?;
        let active: bool = self
            .c
            .query_row("SELECT active FROM users WHERE id=?", [actor.id], |r| {
                r.get(0)
            })
            .map_err(sql)?;
        if !active {
            return Err("Seu perfil não tem permissão para esta operação.".into());
        }
        let mut previous = String::new();
        let mut count = 0;
        let mut s = self
            .c
            .prepare("SELECT stamp,actor,action,entity,detail,previous,hash FROM audit ORDER BY id")
            .map_err(sql)?;
        let rows = s
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<i64>>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                ))
            })
            .map_err(sql)?;
        for row in rows {
            let (time, actor, action, entity, detail, prev, hash) = row.map_err(sql)?;
            let body = serde_json::to_vec(&(&prev, &time, actor, &action, &entity, &detail))
                .map_err(|e| e.to_string())?;
            if prev != previous || digest(&body) != hash {
                return Err(format!(
                    "Falha de integridade no evento {}. Interrompa o uso e confira o backup.",
                    count + 1
                ));
            }
            previous = hash;
            count += 1;
        }
        Ok(count)
    }
    pub fn audit_csv(&self) -> R<String> {
        self.require(&[])?;
        self.verify_audit()?;
        let mut out =
            "\u{feff}ID;Data UTC;Usuário;Ação;Entidade;Detalhes;Hash anterior;Hash\n".to_string();
        let mut s=self.c.prepare("SELECT a.id,a.stamp,COALESCE(u.name,'Sem sessão'),a.action,a.entity,a.detail,a.previous,a.hash FROM audit a LEFT JOIN users u ON u.id=a.actor ORDER BY a.id").map_err(sql)?;
        let rows = s
            .query_map([], |r| {
                let mut v = Vec::new();
                v.push(r.get::<_, i64>(0)?.to_string());
                for i in 1..8 {
                    v.push(r.get::<_, String>(i)?);
                }
                Ok(v)
            })
            .map_err(sql)?;
        for r in rows {
            out += &r
                .map_err(sql)?
                .iter()
                .map(|s| super::exports::csv(s))
                .collect::<Vec<_>>()
                .join(";");
            out.push('\n');
        }
        Ok(out)
    }
}
