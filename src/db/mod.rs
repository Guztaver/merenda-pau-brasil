use crate::validation::{date, sql, R};
use rusqlite::{params, Connection};
mod queries;
mod reports;
#[cfg(test)]
mod tests;
pub struct Db {
    c: Connection,
}
impl Db {
    pub fn open(path: &std::path::Path) -> R<Self> {
        let c = Connection::open(path).map_err(sql)?;
        Self::init(c)
    }
    fn init(c: Connection) -> R<Self> {
        c.execute_batch(include_str!("schema.sql")).map_err(sql)?;
        Ok(Self { c })
    }
    pub fn names(&self, table: &str) -> Vec<(i64, String)> {
        let mut s = self
            .c
            .prepare(&format!("SELECT id,name FROM {table} ORDER BY name"))
            .unwrap();
        s.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .filter_map(Result::ok)
            .collect()
    }
    pub fn add(&self, table: &str, name: &str) -> R<()> {
        if name.trim().is_empty() {
            return Err("Informe o nome.".into());
        }
        self.c
            .execute(
                &format!("INSERT INTO {table}(name) VALUES (?)"),
                [name.trim()],
            )
            .map_err(sql)?;
        Ok(())
    }
    fn check(c: &Connection, item: i64) -> R<()> {
        let mut s = c
            .prepare(
                "SELECT date,SUM(grams) FROM movements WHERE item=? GROUP BY date ORDER BY date",
            )
            .map_err(sql)?;
        let rows = s
            .query_map([item], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            })
            .map_err(sql)?;
        let mut balance = 0;
        for r in rows {
            let (d, g) = r.map_err(sql)?;
            balance += g;
            if balance < 0 {
                return Err(format!(
                    "Estoque insuficiente em {d}. Revise o peso ou registre a entrada anterior."
                ));
            }
        }
        Ok(())
    }
    pub fn movement(&mut self, d: &str, item: i64, g: i64, note: &str) -> R<()> {
        date(d)?;
        if item == 0 || g == 0 || note.trim().is_empty() {
            return Err("Selecione um alimento e informe a origem ou justificativa.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        t.execute(
            "INSERT INTO movements(date,item,grams,note) VALUES (?,?,?,?)",
            params![d, item, g, note.trim()],
        )
        .map_err(sql)?;
        Self::check(&t, item)?;
        t.commit().map_err(sql)
    }
    pub fn serve(&mut self, d: &str, school: i64, recipe: i64, n: i64) -> R<()> {
        date(d)?;
        if school == 0 || recipe == 0 || n <= 0 || n > 1_000_000 {
            return Err("Selecione a escola, a refeição e entre 1 e 1.000.000 de porções.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        let ingredients = {
            let mut s = t
                .prepare("SELECT item,grams FROM ingredients WHERE recipe=?")
                .map_err(sql)?;
            let v = s
                .query_map([recipe], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))
                .map_err(sql)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(sql)?;
            v
        };
        if ingredients.is_empty() {
            return Err("Adicione os ingredientes da receita primeiro.".into());
        }
        t.execute(
            "INSERT INTO servings(date,school,recipe,portions) VALUES (?,?,?,?)",
            params![d, school, recipe, n],
        )
        .map_err(sql)?;
        let id = t.last_insert_rowid();
        for (item, g) in ingredients {
            t.execute("INSERT INTO movements(date,item,grams,note,serving) VALUES (?,?,?,'Consumo de refeição',?)",params![d,item,-g*n,id]).map_err(sql)?;
            Self::check(&t, item)?;
        }
        t.commit().map_err(sql)
    }
    pub fn stock(&self, d: &str) -> Vec<(String, i64, i64)> {
        let mut s=self.c.prepare("SELECT i.name,COALESCE(SUM(CASE WHEN m.date<=? THEN m.grams ELSE 0 END),0),i.minimum FROM items i LEFT JOIN movements m ON m.item=i.id GROUP BY i.id ORDER BY i.name").unwrap();
        s.query_map([d], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .filter_map(Result::ok)
            .collect()
    }
}
