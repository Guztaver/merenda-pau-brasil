use super::{inventory::movement, security::audit, Db};
use crate::validation::{sql, R};
use rusqlite::{params, OptionalExtension};
pub const CATEGORIES: [&str; 4] = [
    "In natura/minimamente processado",
    "Ingrediente culinário",
    "Processado",
    "Ultraprocessado",
];
pub const METHODS: [&str; 4] = [
    "Pregão",
    "Chamada pública",
    "Chamada pública PCT",
    "Outra hipótese legal documentada",
];
pub(super) fn check_payment_history(c: &rusqlite::Connection, receipt: i64) -> R<()> {
    let total: i64 = c
        .query_row("SELECT cents FROM receipts WHERE id=?", [receipt], |r| {
            r.get(0)
        })
        .map_err(sql)?;
    let mut s=c.prepare("SELECT o.date,SUM(CASE WHEN o.reversal_of IS NULL THEN p.cents ELSE -p.cents END) FROM operations o JOIN payments p ON p.operation_id=COALESCE(o.reversal_of,o.id) WHERE p.receipt=? GROUP BY o.date ORDER BY o.date").map_err(sql)?;
    let rows = s
        .query_map([receipt], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })
        .map_err(sql)?;
    let mut paid = 0;
    for row in rows {
        let (day, value) = row.map_err(sql)?;
        paid += value;
        if paid < 0 || paid > total {
            return Err(format!("O lançamento ultrapassaria o valor da nota ou deixaria pagamentos negativos em {day}."));
        }
    }
    Ok(())
}
#[derive(Default)]
pub struct ReceiptInput {
    pub day: String,
    pub supplier: i64,
    pub item: i64,
    pub invoice: String,
    pub process: String,
    pub contract: String,
    pub method: String,
    pub funding: String,
    pub category: String,
    pub grams: i64,
    pub cents: i64,
    pub batch: String,
    pub expiry: String,
}
#[derive(Clone)]
pub struct Supplier {
    pub id: i64,
    pub name: String,
    pub document: String,
    pub family: bool,
    pub caf: String,
    pub verified: bool,
    pub verification: String,
}
#[derive(Default)]
pub struct FinanceTotals {
    pub transfers: i64,
    pub carryover: i64,
    pub family_paid: i64,
    pub federal_paid: i64,
    pub own_paid: i64,
    pub natural_paid: i64,
    pub processed_paid: i64,
    pub culinary_paid: i64,
}
#[derive(Default, Clone)]
pub struct FundingReport {
    pub year: String,
    pub federal_transfer_cents: i64,
    pub carryover_cents: i64,
    pub federal_paid_cents: i64,
    pub family_paid_cents: i64,
    pub natural_paid_cents: i64,
    pub processed_paid_cents: i64,
    pub culinary_paid_cents: i64,
    pub family_percent: Option<f64>,
    pub natural_percent: Option<f64>,
    pub processed_percent: Option<f64>,
    pub culinary_percent: Option<f64>,
    pub sufficient_base: bool,
}
pub fn document_valid(document: &str) -> bool {
    let digits: Vec<u32> = document.chars().filter_map(|c| c.to_digit(10)).collect();
    if ![11, 14].contains(&digits.len()) || digits.iter().all(|d| *d == digits[0]) {
        return false;
    }
    let verify = |end: usize, weights: &[u32]| {
        let remainder = digits[..end]
            .iter()
            .zip(weights)
            .map(|(d, w)| d * w)
            .sum::<u32>()
            % 11;
        digits[end] == if remainder < 2 { 0 } else { 11 - remainder }
    };
    if digits.len() == 11 {
        verify(9, &[10, 9, 8, 7, 6, 5, 4, 3, 2]) && verify(10, &[11, 10, 9, 8, 7, 6, 5, 4, 3, 2])
    } else {
        verify(12, &[5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2])
            && verify(13, &[6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2])
    }
}
impl Db {
    pub fn supplier_choices(&self) -> R<Vec<(i64, String)>> {
        self.require(&[])?;
        let mut s = self
            .c
            .prepare("SELECT id,name,document FROM suppliers ORDER BY name")
            .map_err(sql)?;
        let rows = s
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    format!(
                        "{} • CPF/CNPJ {}",
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?
                    ),
                ))
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }
    pub fn update_supplier(
        &mut self,
        id: i64,
        name: &str,
        document: &str,
        family: bool,
        caf: &str,
    ) -> R<()> {
        let actor = self.require(&["Estoque"])?;
        if id <= 0
            || name.trim().is_empty()
            || !document_valid(document)
            || (family && caf.trim().is_empty())
        {
            return Err(
                "Informe nome, CPF/CNPJ válido e CAF para fornecedor da agricultura familiar."
                    .into(),
            );
        }
        let clean: String = document.chars().filter(char::is_ascii_digit).collect();
        let t = self.c.transaction().map_err(sql)?;
        let (old_document, old_family, old_caf): (String, bool, String) = t
            .query_row(
                "SELECT document,family,caf FROM suppliers WHERE id=?",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(sql)?;
        let verification_still_valid =
            old_document == clean && old_family == family && old_caf == caf.trim();
        t.execute("UPDATE suppliers SET name=?,document=?,family=?,caf=?,verified=CASE WHEN ? THEN verified ELSE 0 END,verification=CASE WHEN ? THEN verification ELSE NULL END,verified_by=CASE WHEN ? THEN verified_by ELSE NULL END,verified_at=CASE WHEN ? THEN verified_at ELSE NULL END WHERE id=?", params![name.trim(),clean,family,caf.trim(),verification_still_valid,verification_still_valid,verification_still_valid,verification_still_valid,id]).map_err(sql)?;
        audit(&t,Some(actor),"EDITAR_FORNECEDOR","suppliers",&format!("id={id}; nome={}; documento atualizado; agricultura familiar={family}; CAF declarado atualizado",name.trim()))?;
        let still_family: bool = t
            .query_row("SELECT family FROM suppliers WHERE id=?", [id], |r| {
                r.get(0)
            })
            .map_err(sql)?;
        if old_family != still_family {
            audit(&t,Some(actor),"ALTERAR_ENQUADRAMENTO_FORNECEDOR","suppliers",&format!("id={id}; agricultura familiar={still_family}; registros de recebimento preservam fotografia histórica"))?;
        }
        t.commit().map_err(sql)
    }
    pub fn delete_supplier(&mut self, id: i64) -> R<()> {
        let actor = self.require(&["Administrador"])?;
        let t = self.c.transaction().map_err(sql)?;
        let name: String = t
            .query_row("SELECT name FROM suppliers WHERE id=?", [id], |r| r.get(0))
            .map_err(sql)?;
        let receipts: i64 = t
            .query_row(
                "SELECT COUNT(*) FROM receipts WHERE supplier=?",
                [id],
                |r| r.get(0),
            )
            .map_err(sql)?;
        let documents: i64 = t
            .query_row(
                "SELECT COUNT(*) FROM documents WHERE entity='Fornecedor familiar' AND entity_id=?",
                [id],
                |r| r.get(0),
            )
            .map_err(sql)?;
        let lot_evidence: i64 = t
            .query_row(
                "SELECT COUNT(*) FROM lot_evidence WHERE supplier=?",
                [id],
                |r| r.get(0),
            )
            .map_err(sql)?;
        if receipts > 0 || documents > 0 || lot_evidence > 0 {
            return Err("Este fornecedor já possui recebimentos ou documentos vinculados. Edite o cadastro para manter o histórico íntegro.".into());
        }
        t.execute("DELETE FROM suppliers WHERE id=?", [id])
            .map_err(sql)?;
        audit(
            &t,
            Some(actor),
            "EXCLUIR_FORNECEDOR",
            "suppliers",
            &format!("id={id}; nome={name}; sem operações ou documentos vinculados"),
        )?;
        t.commit().map_err(sql)
    }
    pub fn add_supplier(&mut self, name: &str, document: &str, family: bool, caf: &str) -> R<()> {
        let actor = self.require(&["Estoque"])?;
        if name.trim().is_empty() || !document_valid(document) || (family && caf.trim().is_empty())
        {
            return Err(
                "Informe nome, CPF/CNPJ válido e CAF para fornecedor da agricultura familiar."
                    .into(),
            );
        }
        let clean: String = document.chars().filter(char::is_ascii_digit).collect();
        let t = self.c.transaction().map_err(sql)?;
        t.execute(
            "INSERT INTO suppliers(name,document,family,caf) VALUES (?,?,?,?)",
            params![name.trim(), clean, family, caf.trim()],
        )
        .map_err(sql)?;
        audit(
            &t,
            Some(actor),
            "CADASTRAR_FORNECEDOR",
            "suppliers",
            &format!(
                "id={}; nome={name}; familiar={family}; CAF declarado={caf}",
                t.last_insert_rowid()
            ),
        )?;
        t.commit().map_err(sql)
    }
    pub fn verify_family_supplier(&mut self, id: i64, reference: &str, reason: &str) -> R<()> {
        let actor = self.require(&["Administrador"])?;
        if reference.trim().is_empty() || reason.trim().len() < 5 {
            return Err("Informe referência documental e conferência por um administrador.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        let family: bool = t
            .query_row("SELECT family FROM suppliers WHERE id=?", [id], |r| {
                r.get(0)
            })
            .map_err(sql)?;
        if !family {
            return Err("O fornecedor não está marcado como agricultura familiar.".into());
        }
        t.execute(
            "UPDATE suppliers SET verified=1,verification=?,verified_by=?,verified_at=? WHERE id=?",
            params![
                format!("{} — {}", reference.trim(), reason.trim()),
                actor,
                super::security::stamp(),
                id
            ],
        )
        .map_err(sql)?;
        audit(
            &t,
            Some(actor),
            "VERIFICAR_AGRICULTURA_FAMILIAR",
            "suppliers",
            &format!("fornecedor={id}; referência={reference}; {reason}"),
        )?;
        t.commit().map_err(sql)
    }
    pub fn receive(&mut self, f: &ReceiptInput) -> R<()> {
        let actor = self.require(&["Estoque"])?;
        let day = crate::validation::iso_date(&f.day)?;
        let expiry = crate::validation::iso_date(&f.expiry)?;
        if expiry < day
            || f.grams <= 0
            || f.grams > 1_000_000_000
            || f.cents <= 0
            || f.cents > 1_000_000_000_000
            || [&f.invoice, &f.process, &f.contract, &f.batch]
                .iter()
                .any(|s| s.trim().is_empty())
            || !METHODS.contains(&f.method.as_str())
            || !["FNDE", "Próprios"].contains(&f.funding.as_str())
            || !CATEGORIES.contains(&f.category.as_str())
        {
            return Err("Preencha fornecedor, alimento, nota fiscal, processo, contrato, lote, validade e valores positivos válidos.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        let family: bool = t
            .query_row(
                "SELECT family FROM suppliers WHERE id=?",
                [f.supplier],
                |r| r.get(0),
            )
            .map_err(sql)?;
        if f.funding == "FNDE" {
            let assessed:Option<(String,bool)>=t.query_row("SELECT category,allowed FROM food_assessments WHERE item=? ORDER BY id DESC LIMIT 1",[f.item],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(sql)?;
            match assessed {Some((category,true)) if category==f.category=>{},_=>return Err("Compra FNDE exige classificação do alimento e autorização de elegibilidade registradas pelo nutricionista, coincidentes com a categoria informada.".into())}
        }
        if family && f.funding == "FNDE" && !f.method.starts_with("Chamada pública") {
            return Err(
                "Compra FNDE da agricultura familiar: registre a chamada pública correspondente."
                    .into(),
            );
        }
        let verified: bool = t
            .query_row(
                "SELECT verified FROM suppliers WHERE id=?",
                [f.supplier],
                |r| r.get(0),
            )
            .map_err(sql)?;
        if family && f.funding == "FNDE" && !verified {
            return Err("Fornecedor familiar ainda não verificado. Registre a conferência do CAF/documento antes da compra com FNDE.".into());
        }
        let duplicate:bool=t.query_row("SELECT EXISTS(SELECT 1 FROM receipts r JOIN operations o ON o.id=r.operation_id WHERE supplier=? AND invoice=? AND item=? AND NOT EXISTS(SELECT 1 FROM operations rev WHERE rev.reversal_of=o.id))",params![f.supplier,f.invoice.trim(),f.item],|r|r.get(0)).map_err(sql)?;
        if duplicate {
            return Err("Este alimento já foi recebido nessa nota fiscal. Para entregas parciais, identifique também a parcela na nota/referência.".into());
        }
        let op = Self::operation(&t, actor, "RECEBIMENTO", &day, &f.invoice)?;
        let snapshot: (String, String, String) = t
            .query_row(
                "SELECT name,document,caf FROM suppliers WHERE id=?",
                [f.supplier],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(sql)?;
        t.execute("INSERT INTO receipts(supplier,item,invoice,process,contract,method,funding,category,grams,cents,operation_id,supplier_name,supplier_document,supplier_family,supplier_caf) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",params![f.supplier,f.item,f.invoice.trim(),f.process.trim(),f.contract.trim(),f.method,f.funding,f.category,f.grams,f.cents,op,snapshot.0,snapshot.1,family,snapshot.2]).map_err(sql)?;
        let receipt = t.last_insert_rowid();
        t.execute(
            "INSERT INTO lots(item,receipt,batch,expiry,location) VALUES (?,?,?,?,'central')",
            params![f.item, receipt, f.batch.trim(), expiry],
        )
        .map_err(sql)?;
        let lot = t.last_insert_rowid();
        movement(
            &t,
            op,
            &day,
            f.item,
            lot,
            f.grams,
            "Recebimento de alimentos",
            None,
        )?;
        audit(&t,Some(actor),"RECEBER","operations",&format!("op={op}; recebimento={receipt}; lote={lot}; fornecedor={}; NF={}; processo={}; contrato={}; fonte={}; g={}; centavos={}",f.supplier,f.invoice,f.process,f.contract,f.funding,f.grams,f.cents))?;
        t.commit().map_err(sql)
    }
    pub fn receipt_choices(&self) -> R<Vec<(i64, String)>> {
        self.require(&[])?;
        let mut s=self.c.prepare("SELECT r.id,r.supplier_name,r.invoice,r.cents FROM receipts r WHERE NOT EXISTS(SELECT 1 FROM operations rev WHERE rev.reversal_of=r.operation_id) ORDER BY r.id DESC").map_err(sql)?;
        let rows = s
            .query_map([], |r| {
                Ok((
                    r.get(0)?,
                    format!(
                        "#{} • {} • NF {} • R$ {}",
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        crate::validation::currency(r.get(3)?)
                    ),
                ))
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }
    pub fn record_payment(&mut self, day: &str, receipt: i64, cents: i64, document: &str) -> R<()> {
        let actor = self.require(&["Administrador"])?;
        let day = crate::validation::iso_date(day)?;
        if cents <= 0 || document.trim().is_empty() {
            return Err("Informe valor positivo e documento do pagamento.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        let (total,received):(i64,String)=t.query_row("SELECT r.cents,o.date FROM receipts r JOIN operations o ON o.id=r.operation_id WHERE r.id=? AND NOT EXISTS(SELECT 1 FROM operations rev WHERE rev.reversal_of=o.id)",[receipt],|r|Ok((r.get(0)?,r.get(1)?))).map_err(sql)?;
        let paid:i64=t.query_row("SELECT COALESCE(SUM(p.cents),0) FROM payments p WHERE p.receipt=? AND NOT EXISTS(SELECT 1 FROM operations rev WHERE rev.reversal_of=p.operation_id)",[receipt],|r|r.get(0)).map_err(sql)?;
        if day < received || cents > total - paid {
            return Err(
                "Pagamento anterior ao recebimento ou acima do saldo da nota fiscal.".into(),
            );
        }
        let duplicate: Option<i64> = t
            .query_row(
                "SELECT id FROM payments WHERE document=?",
                [document.trim()],
                |r| r.get(0),
            )
            .optional()
            .map_err(sql)?;
        if duplicate.is_some() {
            return Err(
                "Documento de pagamento já utilizado. Identifique a parcela quando aplicável."
                    .into(),
            );
        }
        let op = Self::operation(&t, actor, "PAGAMENTO", &day, document)?;
        t.execute(
            "INSERT INTO payments(receipt,cents,document,operation_id) VALUES (?,?,?,?)",
            params![receipt, cents, document.trim(), op],
        )
        .map_err(sql)?;
        check_payment_history(&t, receipt)?;
        audit(
            &t,
            Some(actor),
            "PAGAR",
            "operations",
            &format!("op={op}; recebimento={receipt}; centavos={cents}; documento={document}"),
        )?;
        t.commit().map_err(sql)
    }
    pub fn record_resource(&mut self, day: &str, cents: i64, document: &str, kind: &str) -> R<()> {
        let actor = self.require(&["Administrador"])?;
        if cents <= 0
            || cents > 1_000_000_000_000
            || document.trim().is_empty()
            || !["Repasse FNDE", "Saldo reprogramado"].contains(&kind)
        {
            return Err("Informe tipo, valor positivo e documento do recurso.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        let op = Self::operation(&t, actor, "RECURSO", day, document)?;
        t.execute(
            "INSERT INTO resources(cents,document,kind,operation_id) VALUES (?,?,?,?)",
            params![cents, document.trim(), kind, op],
        )
        .map_err(sql)?;
        audit(
            &t,
            Some(actor),
            "RECURSO",
            "operations",
            &format!("op={op}; tipo={kind}; centavos={cents}; documento={document}"),
        )?;
        t.commit().map_err(sql)
    }
    pub fn finance_totals(&self, year: &str) -> R<FinanceTotals> {
        self.require(&[])?;
        super::exports::year(year)?;
        let mut v = FinanceTotals::default();
        let mut s=self.c.prepare("SELECT r.kind,SUM(CASE WHEN o.reversal_of IS NULL THEN r.cents ELSE -r.cents END) FROM operations o JOIN resources r ON r.operation_id=COALESCE(o.reversal_of,o.id) WHERE substr(o.date,1,4)=? GROUP BY r.kind").map_err(sql)?;
        for row in s
            .query_map([year], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            })
            .map_err(sql)?
        {
            let (k, c) = row.map_err(sql)?;
            if k == "Repasse FNDE" {
                v.transfers = c
            } else {
                v.carryover = c
            }
        }
        let mut s=self.c.prepare("SELECT r.funding,r.supplier_family,r.category,SUM(CASE WHEN o.reversal_of IS NULL THEN p.cents ELSE -p.cents END) FROM operations o JOIN payments p ON p.operation_id=COALESCE(o.reversal_of,o.id) JOIN receipts r ON r.id=p.receipt WHERE substr(o.date,1,4)=? GROUP BY r.funding,r.supplier_family,r.category").map_err(sql)?;
        for row in s
            .query_map([year], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, bool>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })
            .map_err(sql)?
        {
            let (funding, family, category, c) = row.map_err(sql)?;
            if funding == "FNDE" {
                v.federal_paid += c;
                if family {
                    v.family_paid += c;
                }
                match category.as_str() {
                    "In natura/minimamente processado" => v.natural_paid += c,
                    "Ingrediente culinário" => v.culinary_paid += c,
                    _ => v.processed_paid += c,
                }
            } else {
                v.own_paid += c;
            }
        }
        Ok(v)
    }
    pub fn compliance_report(&self, year: &str) -> R<FundingReport> {
        self.require(&[])?;
        super::exports::year(year)?;
        let v = self.finance_totals(year)?;
        let base = v.transfers;
        let pct = |amount: i64| (base > 0).then(|| amount as f64 * 100. / base as f64);
        let report = FundingReport {
            year: year.to_string(),
            federal_transfer_cents: v.transfers,
            carryover_cents: v.carryover,
            federal_paid_cents: v.federal_paid,
            family_paid_cents: v.family_paid,
            natural_paid_cents: v.natural_paid,
            processed_paid_cents: v.processed_paid,
            culinary_paid_cents: v.culinary_paid,
            family_percent: pct(v.family_paid),
            natural_percent: pct(v.natural_paid),
            processed_percent: pct(v.processed_paid),
            culinary_percent: pct(v.culinary_paid),
            sufficient_base: base > 0,
        };
        Ok(report)
    }
    pub fn compliance_csv(&self, year: &str) -> R<String> {
        let v = self.compliance_report(year)?;
        let pct = |v: Option<f64>| v.map_or_else(|| "sem base".to_string(), |n| format!("{n:.2}%"));
        let mut out = format!(
            "Conferência auxiliar PNAE;{}\nIndicador;Valor\nRepasses FNDE no exercício (base dos percentuais);{}\nSaldo reprogramado (informativo, fora da base);{}\nPagamentos FNDE registrados;{}\nAgricultura familiar paga com FNDE;{}\nPercentual agricultura familiar;{}\nIn natura/minimamente processados pagos com FNDE;{}\nPercentual in natura/minimamente processados;{}\nProcessados/ultraprocessados pagos com FNDE;{}\nPercentual processados/ultraprocessados;{}\nIngredientes culinários pagos com FNDE;{}\nPercentual ingredientes culinários;{}\n",
            v.year,
            crate::validation::currency(v.federal_transfer_cents),
            crate::validation::currency(v.carryover_cents),
            crate::validation::currency(v.federal_paid_cents),
            crate::validation::currency(v.family_paid_cents),
            pct(v.family_percent),
            crate::validation::currency(v.natural_paid_cents),
            pct(v.natural_percent),
            crate::validation::currency(v.processed_paid_cents),
            pct(v.processed_percent),
            crate::validation::currency(v.culinary_paid_cents),
            pct(v.culinary_percent),
        );
        out.push_str("\nATENÇÃO;Indicadores auxiliares calculados apenas sobre pagamentos e repasses cadastrados. Conferir extratos, estornos, elegibilidade, classificações, exercício e regras vigentes com contabilidade, controle interno e RT. Este arquivo não é prestação de contas oficial nem valida regularidade da aquisição.\n\nPagamento;Data;Fornecedor;CPF/CNPJ;Agricultura familiar declarada;CAF informado;Nota;Processo;Contrato;Método;Alimento;Categoria;Fonte;Valor em reais;Documento\n");
        let mut s = self.c.prepare("SELECT o.id,o.date,r.supplier_name,r.supplier_document,r.supplier_family,r.supplier_caf,r.invoice,r.process,r.contract,r.method,i.name,r.category,r.funding,CASE WHEN o.reversal_of IS NULL THEN p.cents ELSE -p.cents END,p.document FROM operations o JOIN payments p ON p.operation_id=COALESCE(o.reversal_of,o.id) JOIN receipts r ON r.id=p.receipt JOIN items i ON i.id=r.item WHERE substr(o.date,1,4)=? ORDER BY o.date,o.id").map_err(sql)?;
        let rows = s
            .query_map([year], |r| {
                Ok(vec![
                    r.get::<_, i64>(0)?.to_string(),
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get::<_, bool>(4)?.to_string(),
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                    r.get(9)?,
                    r.get(10)?,
                    r.get(11)?,
                    r.get(12)?,
                    crate::validation::currency(r.get(13)?),
                    r.get(14)?,
                ])
            })
            .map_err(sql)?;
        for row in rows {
            out += &row
                .map_err(sql)?
                .iter()
                .map(|x| super::exports::csv(x))
                .collect::<Vec<_>>()
                .join(";");
            out.push('\n');
        }
        Ok(out)
    }
}
