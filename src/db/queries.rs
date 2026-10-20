use super::Db;
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
    pub fn add_food(&self, name: &str, minimum: &str) -> R<()> {
        if name.trim().is_empty() {
            return Err("Informe o nome do alimento.".into());
        }
        let minimum = minimum
            .trim()
            .parse::<i64>()
            .map_err(|_| "O mínimo deve ser um número inteiro de gramas.".to_string())?;
        if minimum < 0 {
            return Err("O mínimo não pode ser negativo.".into());
        }
        self.c
            .execute(
                "INSERT INTO items(name,minimum) VALUES (?,?)",
                params![name.trim(), minimum],
            )
            .map_err(sql)?;
        Ok(())
    }

    pub fn ingredients(&self, recipe: i64) -> R<Vec<Ingredient>> {
        let mut s = self.c.prepare("SELECT i.id,i.name,g.grams FROM ingredients g JOIN items i ON i.id=g.item WHERE g.recipe=? ORDER BY i.name").map_err(sql)?;
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
    pub fn save_ingredient(&self, recipe: i64, item: i64, grams: i64) -> R<()> {
        if recipe == 0 || item == 0 {
            return Err("Selecione a receita e o ingrediente.".into());
        }
        self.c.execute("INSERT INTO ingredients(recipe,item,grams) VALUES (?,?,?) ON CONFLICT(recipe,item) DO UPDATE SET grams=excluded.grams",params![recipe,item,grams]).map_err(sql)?;
        Ok(())
    }
    pub fn remove_ingredient(&self, recipe: i64, item: i64) -> R<()> {
        self.c
            .execute(
                "DELETE FROM ingredients WHERE recipe=? AND item=?",
                params![recipe, item],
            )
            .map_err(sql)?;
        Ok(())
    }
    pub fn movements(&self) -> R<Vec<Movement>> {
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
        let mut s=self.c.prepare("SELECT v.date,s.name,r.name,v.portions FROM servings v JOIN schools s ON s.id=v.school JOIN recipes r ON r.id=v.recipe ORDER BY v.id DESC LIMIT 50").map_err(sql)?;
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
    pub fn portions_on(&self, date: &str) -> R<i64> {
        self.c
            .query_row(
                "SELECT COALESCE(SUM(portions),0) FROM servings WHERE date=?",
                [date],
                |r| r.get(0),
            )
            .map_err(sql)
    }
    pub fn backup(&self, path: &std::path::Path) -> R<()> {
        self.c
            .backup(rusqlite::DatabaseName::Main, path, None)
            .map_err(sql)
    }
}
