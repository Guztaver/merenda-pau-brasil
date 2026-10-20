use crate::validation::{sql, R};
use rusqlite::{params, Connection};
pub mod exports;
pub mod governance;
pub mod inventory;
pub mod nutrition;
pub mod purchasing;
mod queries;
mod reports;
pub mod security;
#[cfg(test)]
mod tests;
pub struct Db {
    c: Connection,
    actor: Option<security::User>,
}
impl Db {
    pub fn open(path: &std::path::Path) -> R<Self> {
        let c = Connection::open(path).map_err(sql)?;
        let version: i64 = c
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(sql)?;
        let has_tables: bool = c
            .query_row(
                "SELECT COUNT(*)>0 FROM sqlite_master WHERE type='table'",
                [],
                |r| r.get(0),
            )
            .map_err(sql)?;
        if version < 2 && has_tables {
            let backup = path.with_extension(format!(
                "antes-v2-{}.sqlite",
                chrono::Utc::now().format("%Y%m%d-%H%M%S%f")
            ));
            c.backup(rusqlite::DatabaseName::Main, &backup, None)
                .map_err(sql)?;
        }
        Self::init(c)
    }
    fn init(mut c: Connection) -> R<Self> {
        c.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=DELETE;")
            .map_err(sql)?;
        let version: i64 = c
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(sql)?;
        if version > 3 {
            return Err("Banco criado por uma versão mais recente. Atualize o aplicativo.".into());
        }
        if version < 2 {
            let t = c.transaction().map_err(sql)?;
            t.execute_batch(include_str!("schema.sql")).map_err(sql)?;
            t.execute_batch(include_str!("migration_v2.sql"))
                .map_err(sql)?;
            for table in [
                "audit",
                "operations",
                "receipts",
                "payments",
                "resources",
                "technical_cards",
                "menus",
                "servings",
                "documents",
                "food_assessments",
                "lot_evidence",
            ] {
                t.execute_batch(&format!("CREATE TRIGGER immutable_{table}_update BEFORE UPDATE ON {table} BEGIN SELECT RAISE(ABORT,'Registro imutável. Crie uma revisão ou estorno.'); END; CREATE TRIGGER immutable_{table}_delete BEFORE DELETE ON {table} BEGIN SELECT RAISE(ABORT,'Registro imutável.'); END;")).map_err(sql)?;
            }
            t.commit().map_err(sql)?;
        }
        if version < 3 {
            let t = c.transaction().map_err(sql)?;
            t.execute_batch(include_str!("migration_v3.sql"))
                .map_err(sql)?;
            t.commit().map_err(sql)?;
        }
        Ok(Self { c, actor: None })
    }
    pub fn names(&self, table: &str) -> Vec<(i64, String)> {
        if self.require(&[]).is_err()
            || !["items", "schools", "recipes", "suppliers"].contains(&table)
        {
            return Vec::new();
        }
        let Ok(mut s) = self
            .c
            .prepare(&format!("SELECT id,name FROM {table} ORDER BY name"))
        else {
            return Vec::new();
        };
        let Ok(rows) = s.query_map([], |r| Ok((r.get(0)?, r.get(1)?))) else {
            return Vec::new();
        };
        rows.filter_map(Result::ok).collect()
    }
    pub fn add(&mut self, table: &str, name: &str) -> R<()> {
        let actor = self.require(if table == "recipes" {
            &["Nutricionista"]
        } else {
            &["Estoque"]
        })?;
        if !["schools", "recipes"].contains(&table) || name.trim().is_empty() {
            return Err("Informe um nome válido.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        t.execute(
            &format!("INSERT INTO {table}(name) VALUES (?)"),
            [name.trim()],
        )
        .map_err(sql)?;
        security::audit(&t, Some(actor), "CADASTRAR", table, name.trim())?;
        t.commit().map_err(sql)
    }
    pub(super) fn operation(
        c: &Connection,
        actor: i64,
        kind: &str,
        date: &str,
        reason: &str,
    ) -> R<i64> {
        governance::open_date(c, date)?;
        c.execute(
            "INSERT INTO operations(kind,date,actor,reason) VALUES (?,?,?,?)",
            params![kind, date, actor, reason],
        )
        .map_err(sql)?;
        Ok(c.last_insert_rowid())
    }
}
