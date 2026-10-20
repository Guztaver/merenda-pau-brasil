use super::Db;
use crate::validation::{kg, sql, R};
use rusqlite::params;
impl Db {
    pub fn report(&self, a: &str, b: &str) -> R<String> {
        self.require(&[])?;
        let a = crate::validation::iso_date(a)?;
        let b = crate::validation::iso_date(b)?;
        if a > b {
            return Err("A data inicial deve vir antes da final.".into());
        }
        let mut out=String::from("\u{feff}Prefeitura de Pau Brasil — Merenda escolar\nAlimento;Saldo inicial (kg);Entradas (kg);Saídas (kg);Saldo final (kg)\n");
        let mut s=self.c.prepare("SELECT i.name,COALESCE(SUM(CASE WHEN date<?1 THEN grams ELSE 0 END),0),COALESCE(SUM(CASE WHEN date BETWEEN ?1 AND ?2 AND grams>0 THEN grams ELSE 0 END),0),COALESCE(SUM(CASE WHEN date BETWEEN ?1 AND ?2 AND grams<0 THEN -grams ELSE 0 END),0) FROM items i LEFT JOIN movements m ON m.item=i.id GROUP BY i.id ORDER BY i.name").map_err(sql)?;
        for r in s
            .query_map(params![a, b], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })
            .map_err(sql)?
        {
            let (n, x, y, z) = r.map_err(sql)?;
            out += &format!(
                "{};{};{};{};{}\n",
                super::exports::csv(&n),
                kg(x),
                kg(y),
                kg(z),
                kg(x + y - z)
            );
        }
        out += "\nData;Escola;Refeição;Porções\n";
        let mut s=self.c.prepare("SELECT date,school,recipe,portions FROM (SELECT v.date,s.name school,r.name recipe,v.portions FROM servings v JOIN schools s ON s.id=v.school JOIN recipes r ON r.id=v.recipe UNION ALL SELECT o.date,s.name,r.name,-v.portions FROM operations o JOIN servings v ON v.operation_id=o.reversal_of JOIN schools s ON s.id=v.school JOIN recipes r ON r.id=v.recipe) WHERE date BETWEEN ? AND ? ORDER BY date").map_err(sql)?;
        for r in s
            .query_map(params![a, b], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })
            .map_err(sql)?
        {
            let (d, s, r, n) = r.map_err(sql)?;
            out += &format!(
                "{};{};{};{}\n",
                d,
                super::exports::csv(&s),
                super::exports::csv(&r),
                n
            );
        }
        Ok(out)
    }
}
