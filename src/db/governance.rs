use super::{
    inventory::{check_history, movement},
    security::{audit, stamp},
    Db,
};
use crate::validation::{date, sql, R};
use rusqlite::{params, Connection, OptionalExtension};
pub(super) fn unlocked(c: &Connection, day: &str) -> R<()> {
    date(day)?;
    let closed: bool = c
        .query_row(
            "SELECT COALESCE((SELECT closed FROM periods WHERE month=?),0)",
            [&day[..7]],
            |r| r.get(0),
        )
        .map_err(sql)?;
    if closed {
        return Err(format!(
            "Competência {} fechada. Somente o administrador pode reabri-la com justificativa.",
            &day[..7]
        ));
    }
    Ok(())
}
pub(super) fn open_date(c: &Connection, day: &str) -> R<()> {
    unlocked(c, day)?;
    if day > chrono::Local::now().format("%Y-%m-%d").to_string().as_str() {
        return Err(
            "Lançamentos realizados não podem ter data futura. Use Cardápios para planejar.".into(),
        );
    }
    Ok(())
}
impl Db {
    pub fn set_period(&mut self, month: &str, closed: bool, reason: &str) -> R<()> {
        let actor = self.require(&["Administrador"])?;
        date(&format!("{month}-01"))?;
        if reason.trim().len() < 5
            || month > chrono::Local::now().format("%Y-%m").to_string().as_str()
        {
            return Err("Informe competência atual/anterior e uma justificativa de pelo menos 5 caracteres.".into());
        }
        self.verify_audit()?;
        let t = self.c.transaction().map_err(sql)?;
        t.execute("INSERT INTO periods(month,closed,reason,actor,stamp) VALUES (?,?,?,?,?) ON CONFLICT(month) DO UPDATE SET closed=excluded.closed,reason=excluded.reason,actor=excluded.actor,stamp=excluded.stamp",params![month,closed,reason.trim(),actor,stamp()]).map_err(sql)?;
        audit(
            &t,
            Some(actor),
            if closed {
                "FECHAR_COMPETENCIA"
            } else {
                "REABRIR_COMPETENCIA"
            },
            "periods",
            &format!("mês={month}; {reason}"),
        )?;
        t.commit().map_err(sql)
    }
    pub fn periods(&self) -> R<Vec<String>> {
        self.require(&[])?;
        let mut s = self
            .c
            .prepare("SELECT month,closed,reason FROM periods ORDER BY month DESC")
            .map_err(sql)?;
        let rows = s
            .query_map([], |r| {
                Ok(format!(
                    "{} • {} • {}",
                    r.get::<_, String>(0)?,
                    if r.get::<_, bool>(1)? {
                        "Fechada"
                    } else {
                        "Aberta"
                    },
                    r.get::<_, String>(2)?
                ))
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }
    pub fn operations(&self) -> R<Vec<(i64, String)>> {
        self.require(&[])?;
        let mut s=self.c.prepare("SELECT o.id,o.date,o.kind,o.reason,u.name FROM operations o JOIN users u ON u.id=o.actor WHERE o.reversal_of IS NULL AND NOT EXISTS(SELECT 1 FROM operations rev WHERE rev.reversal_of=o.id) ORDER BY o.id DESC LIMIT 200").map_err(sql)?;
        let rows = s
            .query_map([], |r| {
                Ok((
                    r.get(0)?,
                    format!(
                        "#{} • {} • {} • {} • {}",
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?
                    ),
                ))
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }
    pub fn reverse(&mut self, original: i64, day: &str, reason: &str) -> R<()> {
        let actor = self.require(&["Administrador"])?;
        if reason.trim().len() < 5 {
            return Err("Informe a justificativa do estorno com pelo menos 5 caracteres.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        let (original_day, kind, reversed): (String, String, Option<i64>) = t
            .query_row(
                "SELECT date,kind,reversal_of FROM operations WHERE id=?",
                [original],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(sql)?;
        if reversed.is_some() || day < original_day.as_str() {
            return Err(
                "Não é permitido estornar um estorno nem usar data anterior à operação original."
                    .into(),
            );
        }
        let already: bool = t
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM operations WHERE reversal_of=?)",
                [original],
                |r| r.get(0),
            )
            .map_err(sql)?;
        if already {
            return Err("Operação já estornada.".into());
        }
        if kind == "RECEBIMENTO" {
            let paid:bool=t.query_row("SELECT EXISTS(SELECT 1 FROM payments p JOIN receipts r ON r.id=p.receipt WHERE r.operation_id=? AND NOT EXISTS(SELECT 1 FROM operations rev WHERE rev.reversal_of=p.operation_id))",[original],|r|r.get(0)).map_err(sql)?;
            if paid {
                return Err(
                    "Estorne os pagamentos dessa nota antes de estornar o recebimento.".into(),
                );
            }
            let last_payment:String=t.query_row("SELECT COALESCE(MAX(o.date),'') FROM operations o JOIN payments p ON p.operation_id=COALESCE(o.reversal_of,o.id) JOIN receipts r ON r.id=p.receipt WHERE r.operation_id=?",[original],|r|r.get(0)).map_err(sql)?;
            if day < last_payment.as_str() {
                return Err("A data de estorno do recebimento deve ser igual ou posterior aos estornos de seus pagamentos.".into());
            }
        }
        open_date(&t, day)?;
        t.execute(
            "INSERT INTO operations(kind,date,actor,reason,reversal_of) VALUES ('ESTORNO',?,?,?,?)",
            params![day, actor, reason.trim(), original],
        )
        .map_err(sql)?;
        let op = t.last_insert_rowid();
        let rows = {
            let mut s = t
                .prepare("SELECT item,lot_id,grams FROM movements WHERE operation_id=?")
                .map_err(sql)?;
            let q = s
                .query_map([original], |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, i64>(1)?,
                        r.get::<_, i64>(2)?,
                    ))
                })
                .map_err(sql)?;
            q.collect::<Result<Vec<_>, _>>().map_err(sql)?
        };
        for (item, lot, g) in rows {
            movement(&t, op, day, item, lot, -g, reason, None)?;
        }
        if kind == "PAGAMENTO" {
            let receipt: i64 = t
                .query_row(
                    "SELECT receipt FROM payments WHERE operation_id=?",
                    [original],
                    |r| r.get(0),
                )
                .map_err(sql)?;
            super::purchasing::check_payment_history(&t, receipt)?;
        }
        audit(
            &t,
            Some(actor),
            "ESTORNAR",
            "operations",
            &format!("op={op}; original={original}; data={day}; {reason}"),
        )?;
        t.commit().map_err(sql)
    }
    pub fn regularize_legacy(
        &mut self,
        lot: i64,
        supplier: i64,
        invoice: &str,
        batch: &str,
        expiry: &str,
        reason: &str,
    ) -> R<()> {
        let actor = self.require(&["Administrador"])?;
        date(expiry)?;
        if [invoice, batch, reason].iter().any(|s| s.trim().is_empty()) {
            return Err("Informe fornecedor, documento histórico, lote, validade conferida e justificativa.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        let original: Option<i64> = t
            .query_row("SELECT id FROM lots WHERE id=? AND legacy=1", [lot], |r| {
                r.get(0)
            })
            .optional()
            .map_err(sql)?;
        if original.is_none() {
            return Err("Selecione um lote legado ainda não regularizado.".into());
        }
        check_history(&t, lot)?;
        t.execute("INSERT INTO lot_evidence(lot,supplier,invoice,reason,actor,stamp) VALUES (?,?,?,?,?,?)",params![lot,supplier,invoice.trim(),reason.trim(),actor,stamp()]).map_err(sql)?;
        t.execute(
            "UPDATE lots SET batch=?,expiry=?,legacy=0 WHERE id=?",
            params![batch.trim(), expiry, lot],
        )
        .map_err(sql)?;
        audit(&t,Some(actor),"REGULARIZAR_LEGADO","lots",&format!("lote={lot}; fornecedor={supplier}; documento={invoice}; lote conferido={batch}; validade={expiry}; {reason}"))?;
        t.commit().map_err(sql)
    }
}
