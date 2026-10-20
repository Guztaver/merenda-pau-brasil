use super::{security::audit, Db};
use crate::validation::{sql, R};
use rusqlite::params;
pub struct Movement {
    pub date: String,
    pub food: String,
    pub grams: i64,
    pub note: String,
}
pub struct Serving {
    pub date: String,
    pub school: String,
    pub recipe: String,
    pub portions: i64,
}
pub struct Ingredient {
    pub id: i64,
    pub name: String,
    pub grams: i64,
}
impl Db {
    pub fn add_food(&mut self, name: &str, minimum: &str) -> R<()> {
        let actor = self.require(&["Estoque"])?;
        let min = minimum
            .trim()
            .parse::<i64>()
            .map_err(|_| "O mínimo deve ser um número inteiro de gramas.".to_string())?;
        if name.trim().is_empty() || !(0..=1_000_000_000).contains(&min) {
            return Err("Informe nome e mínimo entre 0 e 1.000.000.000 g.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        t.execute(
            "INSERT INTO items(name,minimum) VALUES (?,?)",
            params![name.trim(), min],
        )
        .map_err(sql)?;
        audit(
            &t,
            Some(actor),
            "CADASTRAR_ALIMENTO",
            "items",
            &format!("id={}; nome={name}; mínimo={min}", t.last_insert_rowid()),
        )?;
        t.commit().map_err(sql)
    }
    pub fn ingredients(&self, recipe: i64) -> R<Vec<Ingredient>> {
        self.require(&[])?;
        let mut s=self.c.prepare("SELECT i.id,i.name,g.grams FROM ingredients g JOIN items i ON i.id=g.item WHERE g.recipe=? ORDER BY i.name").map_err(sql)?;
        let rows = s
            .query_map([recipe], |r| {
                Ok(Ingredient {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    grams: r.get(2)?,
                })
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }
    pub fn save_ingredient(&mut self, recipe: i64, item: i64, grams: i64) -> R<()> {
        let actor = self.require(&["Nutricionista"])?;
        if recipe == 0 || item == 0 || grams <= 0 || grams > 1_000_000 {
            return Err(
                "Selecione receita, ingrediente e peso entre 1 e 1.000.000 g por porção.".into(),
            );
        }
        let t = self.c.transaction().map_err(sql)?;
        t.execute("INSERT INTO ingredients(recipe,item,grams) VALUES (?,?,?) ON CONFLICT(recipe,item) DO UPDATE SET grams=excluded.grams",params![recipe,item,grams]).map_err(sql)?;
        t.execute("UPDATE recipes SET approved_card=NULL WHERE id=?", [recipe])
            .map_err(sql)?;
        audit(
            &t,
            Some(actor),
            "ALTERAR_INGREDIENTE",
            "recipes",
            &format!(
                "receita={recipe}; item={item}; gramas={grams}; aprovação anterior invalidada"
            ),
        )?;
        t.commit().map_err(sql)
    }
    pub fn remove_ingredient(&mut self, recipe: i64, item: i64) -> R<()> {
        let actor = self.require(&["Nutricionista"])?;
        let t = self.c.transaction().map_err(sql)?;
        t.execute(
            "DELETE FROM ingredients WHERE recipe=? AND item=?",
            params![recipe, item],
        )
        .map_err(sql)?;
        t.execute("UPDATE recipes SET approved_card=NULL WHERE id=?", [recipe])
            .map_err(sql)?;
        audit(
            &t,
            Some(actor),
            "REMOVER_INGREDIENTE",
            "recipes",
            &format!("receita={recipe}; item={item}; aprovação anterior invalidada"),
        )?;
        t.commit().map_err(sql)
    }
    pub fn movements(&self) -> R<Vec<Movement>> {
        self.require(&[])?;
        let mut s=self.c.prepare("SELECT m.date,i.name,m.grams,m.note FROM movements m JOIN items i ON i.id=m.item ORDER BY m.id DESC LIMIT 50").map_err(sql)?;
        let rows = s
            .query_map([], |r| {
                Ok(Movement {
                    date: r.get(0)?,
                    food: r.get(1)?,
                    grams: r.get(2)?,
                    note: r.get(3)?,
                })
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }
    pub fn servings(&self) -> R<Vec<Serving>> {
        self.require(&[])?;
        let mut s=self.c.prepare("SELECT date,school,recipe,portions FROM (SELECT v.date,s.name school,r.name recipe,v.portions,v.id sort FROM servings v JOIN schools s ON s.id=v.school JOIN recipes r ON r.id=v.recipe UNION ALL SELECT o.date,s.name,r.name,-v.portions,o.id FROM operations o JOIN servings v ON v.operation_id=o.reversal_of JOIN schools s ON s.id=v.school JOIN recipes r ON r.id=v.recipe) ORDER BY date DESC,sort DESC LIMIT 50").map_err(sql)?;
        let rows = s
            .query_map([], |r| {
                Ok(Serving {
                    date: r.get(0)?,
                    school: r.get(1)?,
                    recipe: r.get(2)?,
                    portions: r.get(3)?,
                })
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }
    pub fn portions_on(&self, day: &str) -> R<i64> {
        self.require(&[])?;
        self.c.query_row("SELECT (SELECT COALESCE(SUM(portions),0) FROM servings WHERE date=?1)-(SELECT COALESCE(SUM(v.portions),0) FROM operations o JOIN servings v ON v.operation_id=o.reversal_of WHERE o.date=?1)",[day],|r|r.get(0)).map_err(sql)
    }
    pub fn backup(&self, path: &std::path::Path) -> R<()> {
        self.require(&["Administrador"])?;
        self.c
            .backup(rusqlite::DatabaseName::Main, path, None)
            .map_err(sql)
    }
}
