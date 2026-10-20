use super::{security::audit, Db};
use crate::validation::{date, sql, R};
use rusqlite::{params, Connection};
#[derive(Clone)]
pub struct Lot {
    pub id: i64,
    pub label: String,
    pub balance: i64,
    pub legacy: bool,
    pub expiry: String,
    pub location: String,
}
pub(super) fn available(c: &Connection, lot: i64, day: &str) -> R<i64> {
    let mut s = c
        .prepare("SELECT date,SUM(grams) FROM movements WHERE lot_id=? GROUP BY date ORDER BY date")
        .map_err(sql)?;
    let rows = s
        .query_map([lot], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .map_err(sql)?;
    let mut balance = 0;
    let mut at_day = 0;
    let mut minimum = i64::MAX;
    for row in rows {
        let (d, g) = row.map_err(sql)?;
        balance += g;
        if d.as_str() <= day {
            at_day = balance;
        } else {
            minimum = minimum.min(balance);
        }
    }
    Ok(at_day.min(minimum).max(0))
}
pub(super) fn check_history(c: &Connection, lot: i64) -> R<()> {
    let mut s = c
        .prepare("SELECT date,SUM(grams) FROM movements WHERE lot_id=? GROUP BY date ORDER BY date")
        .map_err(sql)?;
    let rows = s
        .query_map([lot], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .map_err(sql)?;
    let mut balance = 0;
    for row in rows {
        let (d, g) = row.map_err(sql)?;
        balance += g;
        if balance < 0 {
            return Err(format!(
                "Lote {lot}: estoque insuficiente em {d}. O lançamento foi desfeito."
            ));
        }
    }
    Ok(())
}
#[allow(clippy::too_many_arguments)]
pub(super) fn movement(
    c: &Connection,
    op: i64,
    day: &str,
    item: i64,
    lot: i64,
    grams: i64,
    note: &str,
    serving: Option<i64>,
) -> R<()> {
    c.execute("INSERT INTO movements(date,item,lot_id,operation_id,grams,note,serving) VALUES (?,?,?,?,?,?,?)",params![day,item,lot,op,grams,note,serving]).map_err(sql)?;
    check_history(c, lot)
}
impl Db {
    pub fn stock(&self, day: &str) -> Vec<(String, i64, i64)> {
        if self.require(&[]).is_err() || date(day).is_err() {
            return Vec::new();
        }
        let Ok(mut s)=self.c.prepare("SELECT i.name,COALESCE(SUM(CASE WHEN m.date<=? THEN m.grams ELSE 0 END),0),i.minimum FROM items i LEFT JOIN movements m ON m.item=i.id GROUP BY i.id ORDER BY i.name")else{return Vec::new()};
        let Ok(rows) = s.query_map([day], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))) else {
            return Vec::new();
        };
        rows.filter_map(Result::ok).collect()
    }
    pub fn lots(&self, day: &str) -> R<Vec<Lot>> {
        self.require(&[])?;
        date(day)?;
        let mut s=self.c.prepare("SELECT l.id,l.item,i.name,l.batch,l.location,l.legacy,COALESCE(l.expiry,''),COALESCE(SUM(CASE WHEN m.date<=? THEN m.grams ELSE 0 END),0) FROM lots l JOIN items i ON i.id=l.item LEFT JOIN movements m ON m.lot_id=l.id GROUP BY l.id ORDER BY l.expiry,l.id").map_err(sql)?;
        let rows = s
            .query_map([day], |r| {
                Ok(Lot {
                    id: r.get(0)?,
                    label: format!(
                        "#{} • {} • {}",
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?
                    ),
                    location: r.get(4)?,
                    legacy: r.get(5)?,
                    expiry: r.get(6)?,
                    balance: r.get(7)?,
                })
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }
    pub fn loss(&mut self, day: &str, lot: i64, grams: i64, reason: &str) -> R<()> {
        let actor = self.require(&["Estoque"])?;
        if grams <= 0 || grams > 1_000_000_000 || reason.trim().is_empty() {
            return Err("Informe peso positivo e justificativa da saída.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        let item: i64 = t
            .query_row("SELECT item FROM lots WHERE id=?", [lot], |r| r.get(0))
            .map_err(sql)?;
        let op = Self::operation(&t, actor, "PERDA", day, reason)?;
        movement(&t, op, day, item, lot, -grams, reason, None)?;
        audit(
            &t,
            Some(actor),
            "PERDA",
            "operations",
            &format!("op={op}; lote={lot}; g={grams}; {reason}"),
        )?;
        t.commit().map_err(sql)
    }
    pub fn transfer(
        &mut self,
        day: &str,
        lot: i64,
        grams: i64,
        school: i64,
        reason: &str,
    ) -> R<()> {
        let actor = self.require(&["Estoque"])?;
        if grams <= 0 || grams > 1_000_000_000 || reason.trim().is_empty() {
            return Err("Informe o peso e a justificativa da transferência.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        let target = if school == 0 {
            "central".to_string()
        } else {
            t.query_row("SELECT id FROM schools WHERE id=?", [school], |r| {
                r.get::<_, i64>(0)
            })
            .map_err(sql)?;
            format!("escola:{school}")
        };
        let (item,receipt,batch,expiry,location,legacy):(i64,Option<i64>,String,String,String,bool)=t.query_row("SELECT item,receipt,batch,COALESCE(expiry,''),location,legacy FROM lots WHERE id=?",[lot],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).map_err(sql)?;
        if location == target || legacy || expiry.as_str() < day {
            return Err("Selecione outro destino e um lote rastreável dentro da validade.".into());
        }
        let op = Self::operation(&t, actor, "TRANSFERENCIA", day, reason)?;
        t.execute(
            "INSERT INTO lots(item,receipt,batch,expiry,location,parent) VALUES (?,?,?,?,?,?)",
            params![item, receipt, batch, expiry, target, lot],
        )
        .map_err(sql)?;
        let destination = t.last_insert_rowid();
        movement(
            &t,
            op,
            day,
            item,
            lot,
            -grams,
            "Transferência de saída",
            None,
        )?;
        movement(
            &t,
            op,
            day,
            item,
            destination,
            grams,
            "Transferência de entrada",
            None,
        )?;
        audit(
            &t,
            Some(actor),
            "TRANSFERIR",
            "operations",
            &format!("op={op}; lote={lot}; destino={target}; g={grams}; {reason}"),
        )?;
        t.commit().map_err(sql)
    }
    pub fn serve(&mut self, day: &str, school: i64, menu: i64, n: i64, central: bool) -> R<()> {
        let actor = self.require(&["Estoque"])?;
        if n <= 0 || n > 1_000_000 {
            return Err("Informe entre 1 e 1.000.000 de porções.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        let op = Self::operation(&t, actor, "REFEICAO", day, "")?;
        let (card,recipe,snapshot):(i64,i64,String)=t.query_row("SELECT c.id,c.recipe,m.snapshot FROM menus m JOIN technical_cards c ON c.id=m.card WHERE m.id=? AND m.school=? AND m.date=? AND NOT EXISTS(SELECT 1 FROM menus n WHERE n.supersedes=m.id)",params![menu,school,day],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(|_|"Selecione um cardápio aprovado para a escola e data do consumo.".to_string())?;
        let data: super::nutrition::Card =
            serde_json::from_str(&snapshot).map_err(|e| e.to_string())?;
        t.execute("INSERT INTO servings(date,school,recipe,portions,operation_id,card_id,menu_id) VALUES (?,?,?,?,?,?,?)",params![day,school,recipe,n,op,card,menu]).map_err(sql)?;
        let serving = t.last_insert_rowid();
        let location = if central {
            "central".to_string()
        } else {
            format!("escola:{school}")
        };
        for ingredient in data.ingredients {
            let mut needed = ingredient
                .grams
                .checked_mul(n)
                .ok_or("Peso total excede o limite.")?;
            let lots = {
                let mut s=t.prepare("SELECT id FROM lots WHERE item=? AND location=? AND legacy=0 AND expiry>=? ORDER BY expiry,id").map_err(sql)?;
                let rows = s
                    .query_map(params![ingredient.id, location, day], |r| {
                        r.get::<_, i64>(0)
                    })
                    .map_err(sql)?;
                rows.collect::<Result<Vec<_>, _>>().map_err(sql)?
            };
            for lot in lots {
                let take = needed.min(available(&t, lot, day)?);
                if take > 0 {
                    movement(
                        &t,
                        op,
                        day,
                        ingredient.id,
                        lot,
                        -take,
                        "Consumo de refeição",
                        Some(serving),
                    )?;
                    needed -= take;
                }
                if needed == 0 {
                    break;
                }
            }
            if needed > 0 {
                return Err(format!("Estoque rastreável e válido insuficiente para {} em {day}. Nenhuma baixa foi salva.",ingredient.name));
            }
        }
        audit(&t,Some(actor),"SERVIR","operations",&format!("op={op}; escola={school}; ficha={card}; cardápio={menu}; porções={n}; origem={location}"))?;
        t.commit().map_err(sql)
    }
}
