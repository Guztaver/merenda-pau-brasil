use super::{
    nutrition::Card,
    security::{audit, digest, stamp},
    Db,
};
use crate::validation::{currency, sql, R};
use rusqlite::params;
use std::path::{Path, PathBuf};
pub fn csv(s: &str) -> String {
    let protected = if s.trim_start().starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{s}")
    } else {
        s.to_string()
    };
    format!("\"{}\"", protected.replace('"', "\"\""))
}
pub fn html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\n', "<br>")
}
pub fn year(s: &str) -> R<()> {
    if s.len() != 4
        || s.parse::<u16>()
            .map_or(true, |n| !(2000..=2200).contains(&n))
    {
        return Err("Informe um ano entre 2000 e 2200.".into());
    }
    Ok(())
}
fn io(e: std::io::Error) -> String {
    e.to_string()
}
fn page(title: &str, body: &str) -> String {
    format!("<!doctype html><html lang='pt-BR'><meta charset='utf-8'><title>{}</title><style>body{{font:15px Arial;margin:40px;color:#233530}}h1,h2{{color:#1b7056}}table{{border-collapse:collapse;width:100%}}td,th{{border:1px solid #ddd;padding:8px;text-align:left}}p{{line-height:1.6}}@media print{{body{{margin:10mm}}}}</style><h1>Prefeitura Municipal de Pau Brasil</h1><h2>{}</h2>{body}</html>",html(title),html(title))
}
fn card_body(c: &Card) -> String {
    let ingredients = c
        .ingredients
        .iter()
        .map(|i| format!("<tr><td>{}</td><td>{} g</td></tr>", html(&i.name), i.grams))
        .collect::<String>();
    let rt =
        c.rt.as_ref()
            .map(|r| format!("{} • CRN {}", html(&r.name), html(&r.crn)))
            .unwrap_or_default();
    let n = &c.nutrients;
    format!("<h2>{}</h2><p>Etapa/faixa etária: {}<br>Dieta: {}<br>Consistência: {}<br>Rendimento por porção: {} g • Custo por porção: R$ {}</p><table><tr><th>Ingrediente por porção</th><th>Peso de estoque</th></tr>{ingredients}</table><h3>Modo de preparo</h3><p>{}</p><h3>Apresentação</h3><p>{}</p><h3>Informação nutricional por porção</h3><p>Energia: {} kcal • Proteínas: {} g • Carboidratos: {} g • Lipídios: {} g • Fibras: {} g<br>Sódio: {} mg • Ferro: {} mg • Vitamina A: {} µg RAE • Vitamina C: {} mg • Cálcio: {} mg<br>Fonte dos valores: {}</p><p>Responsável pela aprovação interna: {rt}</p><p>Assinatura do nutricionista: ____________________________________</p><small>Este documento registra aprovação interna. Não contém assinatura digital certificada nem validação automática da adequação nutricional.</small>",html(&c.name),html(&c.age_group),html(&c.diet),html(&c.consistency),c.yield_grams,currency(c.cost_cents),html(&c.preparation),html(&c.presentation),n.kcal,n.protein,n.carbs,n.fat,n.fiber,n.sodium,n.iron,n.vitamin_a,n.vitamin_c,n.calcium,html(&c.nutrient_source))
}
impl Db {
    pub fn export_card(&self, id: i64, path: &Path) -> R<()> {
        let c = self.card(id)?;
        std::fs::write(path, page(&format!("Ficha técnica #{id}"), &card_body(&c))).map_err(io)
    }
    pub fn export_menus(&self, school: i64, start: &str, end: &str, path: &Path) -> R<()> {
        self.require(&[])?;
        crate::validation::date(start)?;
        crate::validation::date(end)?;
        if start > end {
            return Err("Período inválido.".into());
        }
        let school_name: String = self
            .c
            .query_row("SELECT name FROM schools WHERE id=?", [school], |r| {
                r.get(0)
            })
            .map_err(sql)?;
        let mut body = format!(
            "<p>Escola: {} • Período: {} a {}</p>",
            html(&school_name),
            html(start),
            html(end)
        );
        let mut s=self.c.prepare("SELECT m.date,m.meal,m.planned,m.snapshot,m.stamp,u.name,u.crn FROM menus m JOIN users u ON u.id=m.actor WHERE m.school=? AND m.date BETWEEN ? AND ? AND NOT EXISTS(SELECT 1 FROM menus n WHERE n.supersedes=m.id) ORDER BY m.date,m.meal").map_err(sql)?;
        let rows = s
            .query_map(params![school, start, end], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                ))
            })
            .map_err(sql)?;
        let mut count = 0;
        for row in rows {
            let (day, meal, n, snapshot, approved, name, crn) = row.map_err(sql)?;
            let c: Card = serde_json::from_str(&snapshot).map_err(|e| e.to_string())?;
            body+=&format!("<hr><h2>{} • {}</h2><p>Porções planejadas: {n}<br>Aprovação interna do cardápio: {} • CRN {} • UTC {}</p>{}",html(&day),html(&meal),html(&name),html(&crn),html(&approved),card_body(&c));
            count += 1;
        }
        if count == 0 {
            return Err("Não há cardápios aprovados nesse período para a escola.".into());
        }
        std::fs::write(path, page("Cardápios para divulgação e conferência", &body)).map_err(io)
    }
    pub fn attach_document(&mut self, entity: &str, id: i64, source: &Path, dir: &Path) -> R<()> {
        let actor = self.require(&["Estoque", "Nutricionista"])?;
        let table = match entity {
            "Operação" => "operations",
            "Fornecedor familiar" => {
                self.require(&["Administrador"])?;
                let family: bool = self
                    .c
                    .query_row("SELECT family FROM suppliers WHERE id=?", [id], |r| {
                        r.get(0)
                    })
                    .map_err(sql)?;
                if !family {
                    return Err(
                        "Anexos desta seção exigem fornecedor marcado como agricultura familiar."
                            .into(),
                    );
                }
                "suppliers"
            }
            "Ficha técnica" => {
                self.require_rt()?;
                "technical_cards"
            }
            "Cardápio" => {
                self.require_rt()?;
                "menus"
            }
            _ => return Err("Escolha uma entidade válida.".into()),
        };
        self.c
            .query_row(&format!("SELECT id FROM {table} WHERE id=?"), [id], |r| {
                r.get::<_, i64>(0)
            })
            .map_err(sql)?;
        let metadata = std::fs::metadata(source).map_err(io)?;
        if !metadata.is_file() || metadata.len() > 25 * 1024 * 1024 {
            return Err("Anexe um arquivo de até 25 MB.".into());
        }
        let extension = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !["pdf", "xml", "png", "jpg", "jpeg", "txt", "csv", "docx"].contains(&extension.as_str())
        {
            return Err("Formatos aceitos: PDF, XML, PNG, JPG, TXT, CSV e DOCX.".into());
        }
        let bytes = std::fs::read(source).map_err(io)?;
        let hash = digest(&bytes);
        let name = source
            .file_name()
            .ok_or("Nome inválido.")?
            .to_string_lossy()
            .to_string();
        let stored = format!("{hash}.{extension}");
        let folder = dir.join("documentos");
        std::fs::create_dir_all(&folder).map_err(io)?;
        let target = folder.join(&stored);
        if target.exists() {
            if digest(&std::fs::read(&target).map_err(io)?) != hash {
                return Err("O arquivo armazenado apresenta divergência de hash.".into());
            }
        } else {
            std::fs::write(&target, &bytes).map_err(io)?;
        }
        let t = self.c.transaction().map_err(sql)?;
        t.execute("INSERT INTO documents(entity,entity_id,name,stored_name,hash,actor,stamp) VALUES (?,?,?,?,?,?,?)",params![entity,id,name,stored,hash,actor,stamp()]).map_err(sql)?;
        audit(
            &t,
            Some(actor),
            "ANEXAR_DOCUMENTO",
            entity,
            &format!("id={id}; arquivo={name}; SHA256={hash}"),
        )?;
        t.commit().map_err(sql)
    }
    pub fn finance_csv(&self, year: &str) -> R<String> {
        let v = self.finance_totals(year)?;
        let mut out=format!("\u{feff}Conferência financeira auxiliar — {year}\nIndicador;Valor em reais\nRepasses FNDE do exercício;{}\nSaldo reprogramado (não somado à base dos percentuais);{}\nPagamentos FNDE;{}\nPagamentos agricultura familiar com FNDE;{}\nPagamentos com recursos próprios;{}\nIn natura/minimamente processados FNDE;{}\nProcessados e ultraprocessados FNDE;{}\nIngredientes culinários FNDE;{}\n",currency(v.transfers),currency(v.carryover),currency(v.federal_paid),currency(v.family_paid),currency(v.own_paid),currency(v.natural_paid),currency(v.processed_paid),currency(v.culinary_paid));
        out+="\nData;Operação;Tipo;Nota;Fornecedor;Fonte;Categoria;Valor em reais;Documento do pagamento\n";
        let mut s=self.c.prepare("SELECT o.date,o.id,o.kind,r.invoice,s.name,r.funding,r.category,CASE WHEN o.reversal_of IS NULL THEN p.cents ELSE -p.cents END,p.document FROM operations o JOIN payments p ON p.operation_id=COALESCE(o.reversal_of,o.id) JOIN receipts r ON r.id=p.receipt JOIN suppliers s ON s.id=r.supplier WHERE substr(o.date,1,4)=? ORDER BY o.date,o.id").map_err(sql)?;
        let rows = s
            .query_map([year], |r| {
                Ok(vec![
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?.to_string(),
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                    currency(r.get(7)?),
                    r.get::<_, String>(8)?,
                ])
            })
            .map_err(sql)?;
        for row in rows {
            out += &row
                .map_err(sql)?
                .iter()
                .map(|v| csv(v))
                .collect::<Vec<_>>()
                .join(";");
            out.push('\n');
        }
        Ok(out)
    }
    pub fn traceability_csv(&self) -> R<String> {
        self.require(&[])?;
        let mut out="\u{feff}Movimento;Data;Alimento;Gramas;Lote;Validade;Local;Fornecedor;NF/documento histórico;Operação;Autor;Legado;Justificativa\n".to_string();
        let mut s=self.c.prepare("SELECT m.id,m.date,i.name,m.grams,l.batch,COALESCE(l.expiry,''),COALESCE(l.location,''),COALESCE(s.name,ls.name,''),COALESCE(r.invoice,le.invoice,''),COALESCE(m.operation_id,0),COALESCE(u.name,'Histórico anterior à v2'),COALESCE(l.legacy,1),m.note FROM movements m JOIN items i ON i.id=m.item LEFT JOIN lots l ON l.id=m.lot_id LEFT JOIN receipts r ON r.id=l.receipt LEFT JOIN suppliers s ON s.id=r.supplier LEFT JOIN lot_evidence le ON le.lot=COALESCE(l.parent,l.id) LEFT JOIN suppliers ls ON ls.id=le.supplier LEFT JOIN operations o ON o.id=m.operation_id LEFT JOIN users u ON u.id=o.actor ORDER BY m.date,m.id").map_err(sql)?;
        let rows = s
            .query_map([], |r| {
                Ok(vec![
                    r.get::<_, i64>(0)?.to_string(),
                    r.get(1)?,
                    r.get(2)?,
                    r.get::<_, i64>(3)?.to_string(),
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                    r.get::<_, i64>(9)?.to_string(),
                    r.get(10)?,
                    r.get::<_, bool>(11)?.to_string(),
                    r.get(12)?,
                ])
            })
            .map_err(sql)?;
        for row in rows {
            out += &row
                .map_err(sql)?
                .iter()
                .map(|v| csv(v))
                .collect::<Vec<_>>()
                .join(";");
            out.push('\n');
        }
        Ok(out)
    }
    pub fn export_dossier(&self, year: &str, dir: &Path) -> R<PathBuf> {
        self.require(&["Administrador"])?;
        super::exports::year(year)?;
        self.verify_audit()?;
        let folder = dir.join(format!(
            "dossie-{year}-{}",
            chrono::Utc::now().format("%Y%m%d-%H%M%S%f")
        ));
        std::fs::create_dir(&folder).map_err(io)?;
        self.backup(&folder.join("banco.sqlite"))?;
        std::fs::write(folder.join("auditoria.csv"), self.audit_csv()?).map_err(io)?;
        std::fs::write(folder.join("financeiro.csv"), self.finance_csv(year)?).map_err(io)?;
        std::fs::write(
            folder.join("conferencia-pnae.csv"),
            self.compliance_csv(year)?,
        )
        .map_err(io)?;
        std::fs::write(
            folder.join("avaliacao-alimentos.csv"),
            self.assess_foods_csv()?,
        )
        .map_err(io)?;
        std::fs::write(folder.join("rastreabilidade.csv"), self.traceability_csv()?).map_err(io)?;
        std::fs::write(
            folder.join("estoque-refeicoes.csv"),
            self.report(&format!("{year}-01-01"), &format!("{year}-12-31"))?,
        )
        .map_err(io)?;
        for (id, _) in self.cards()? {
            self.export_card(id, &folder.join(format!("ficha-{id}.html")))?;
        }
        for (school, _) in self.names("schools") {
            if !self
                .menus(&format!("{year}-01-01"), &format!("{year}-12-31"))?
                .is_empty()
            {
                let has: bool = self
                    .c
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM menus WHERE school=? AND substr(date,1,4)=?)",
                        params![school, year],
                        |r| r.get(0),
                    )
                    .map_err(sql)?;
                if has {
                    self.export_menus(
                        school,
                        &format!("{year}-01-01"),
                        &format!("{year}-12-31"),
                        &folder.join(format!("cardapios-escola-{school}.html")),
                    )?;
                }
            }
        }
        let documents = folder.join("documentos");
        std::fs::create_dir(&documents).map_err(io)?;
        let mut manifest = "Entidade;ID;Nome original;Arquivo armazenado;SHA256\n".to_string();
        let mut s = self
            .c
            .prepare("SELECT entity,entity_id,name,stored_name,hash FROM documents ORDER BY id")
            .map_err(sql)?;
        let rows = s
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })
            .map_err(sql)?;
        for row in rows {
            let (entity, id, name, stored, hash) = row.map_err(sql)?;
            if Path::new(&stored).file_name().and_then(|n| n.to_str()) != Some(stored.as_str()) {
                return Err("Caminho de anexo inválido.".into());
            }
            let bytes = std::fs::read(dir.join("documentos").join(&stored)).map_err(io)?;
            if digest(&bytes) != hash {
                return Err(format!(
                    "Anexo {name} ausente ou alterado. Dossiê incompleto."
                ));
            }
            std::fs::write(documents.join(&stored), bytes).map_err(io)?;
            manifest += &[entity, id.to_string(), name, stored, hash]
                .iter()
                .map(|s| csv(s))
                .collect::<Vec<_>>()
                .join(";");
            manifest.push('\n');
        }
        std::fs::write(folder.join("manifesto-documentos.csv"), manifest).map_err(io)?;
        std::fs::write(folder.join("LEIA-ANTES.txt"),"Dossiê auxiliar local, não enviado ao FNDE. Inclui dados pessoais e banco com credenciais protegidas por hash: mantenha acesso restrito. O backup e as fichas incluem histórico completo para rastreabilidade; os relatórios financeiros usam o ano selecionado. Percentuais são indicadores sobre repasses FNDE do exercício e pagamentos registrados, sem certificação da legalidade das compras. Valide com o RT, contabilidade e controle interno. Consulte os canais oficiais vigentes, incluindo a transição para SIGPC Ágil.").map_err(io)?;
        Ok(folder)
    }
}
