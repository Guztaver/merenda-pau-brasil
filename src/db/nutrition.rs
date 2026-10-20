use super::{
    security::{audit, digest, stamp, User},
    Db,
};
use crate::validation::{sql, R};
use rusqlite::params;
use serde::{Deserialize, Serialize};
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Nutrients {
    pub kcal: f64,
    pub protein: f64,
    pub carbs: f64,
    pub fat: f64,
    pub fiber: f64,
    pub sodium: f64,
    pub iron: f64,
    pub vitamin_a: f64,
    pub vitamin_c: f64,
    pub calcium: f64,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct CardIngredient {
    pub id: i64,
    pub name: String,
    pub grams: i64,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Card {
    pub name: String,
    pub age_group: String,
    pub diet: String,
    pub preparation: String,
    pub presentation: String,
    pub consistency: String,
    pub nutrient_source: String,
    pub nutrients: Nutrients,
    pub yield_grams: i64,
    pub cost_cents: i64,
    pub ingredients: Vec<CardIngredient>,
    pub rt: Option<User>,
}
#[derive(Default)]
pub struct MenuInput {
    pub school: i64,
    pub day: String,
    pub meal: String,
    pub age_group: String,
    pub diet: String,
    pub card: i64,
    pub planned: i64,
    pub supersedes: Option<i64>,
}
pub struct MenuRow {
    pub id: i64,
    pub date: String,
    pub school: String,
    pub meal: String,
    pub portions: i64,
}
impl Db {
    pub fn menu_choices(&self, school: i64, day: &str) -> R<Vec<(i64, String)>> {
        self.require(&[])?;
        let day = crate::validation::iso_date(day)?;
        let mut s=self.c.prepare("SELECT m.id,m.meal,r.name FROM menus m JOIN technical_cards c ON c.id=m.card JOIN recipes r ON r.id=c.recipe WHERE m.school=? AND m.date=? AND NOT EXISTS(SELECT 1 FROM menus n WHERE n.supersedes=m.id) ORDER BY m.meal").map_err(sql)?;
        let rows = s
            .query_map(params![school, day], |r| {
                Ok((
                    r.get(0)?,
                    format!("{} • {}", r.get::<_, String>(1)?, r.get::<_, String>(2)?),
                ))
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }
    pub fn menu_card(&self, id: i64) -> R<Card> {
        self.require(&[])?;
        let snapshot: String = self
            .c
            .query_row("SELECT snapshot FROM menus WHERE id=?", [id], |r| r.get(0))
            .map_err(sql)?;
        serde_json::from_str(&snapshot).map_err(|e| e.to_string())
    }
    pub fn approve_card(&mut self, recipe: i64, mut card: Card) -> R<i64> {
        let rt = self.require_rt()?;
        if [
            &card.age_group,
            &card.diet,
            &card.preparation,
            &card.presentation,
            &card.consistency,
            &card.nutrient_source,
        ]
        .iter()
        .any(|s| s.trim().is_empty())
            || card.yield_grams <= 0
            || card.yield_grams > 1_000_000
            || card.cost_cents <= 0
        {
            return Err("Preencha etapa/faixa etária, dieta, preparo, apresentação, consistência, fonte nutricional, rendimento e custo por porção.".into());
        }
        let n = &card.nutrients;
        if [
            n.kcal,
            n.protein,
            n.carbs,
            n.fat,
            n.fiber,
            n.sodium,
            n.iron,
            n.vitamin_a,
            n.vitamin_c,
            n.calcium,
        ]
        .iter()
        .any(|n| !n.is_finite() || *n < 0. || *n > 1e9)
            || n.kcal <= 0.
        {
            return Err(
                "Informe valores nutricionais por porção, não negativos e energia maior que zero."
                    .into(),
            );
        }
        card.ingredients = self
            .ingredients(recipe)?
            .into_iter()
            .map(|i| CardIngredient {
                id: i.id,
                name: i.name,
                grams: i.grams,
            })
            .collect();
        if card.ingredients.is_empty() {
            return Err("Cadastre os ingredientes da receita antes da ficha.".into());
        }
        let t = self.c.transaction().map_err(sql)?;
        card.name = t
            .query_row("SELECT name FROM recipes WHERE id=?", [recipe], |r| {
                r.get(0)
            })
            .map_err(sql)?;
        card.rt = Some(rt.clone());
        let revision: i64 = t
            .query_row(
                "SELECT COALESCE(MAX(revision),0)+1 FROM technical_cards WHERE recipe=?",
                [recipe],
                |r| r.get(0),
            )
            .map_err(sql)?;
        let snapshot = serde_json::to_string(&card).map_err(|e| e.to_string())?;
        let hash = digest(snapshot.as_bytes());
        t.execute("INSERT INTO technical_cards(recipe,revision,snapshot,hash,actor,stamp) VALUES (?,?,?,?,?,?)",params![recipe,revision,snapshot,hash,rt.id,stamp()]).map_err(sql)?;
        let id = t.last_insert_rowid();
        t.execute(
            "UPDATE recipes SET approved_card=? WHERE id=?",
            params![id, recipe],
        )
        .map_err(sql)?;
        audit(
            &t,
            Some(rt.id),
            "APROVAR_FICHA",
            "technical_cards",
            &format!("ficha={id}; receita={recipe}; revisão={revision}; hash={hash}"),
        )?;
        t.commit().map_err(sql)?;
        Ok(id)
    }
    pub fn cards(&self) -> R<Vec<(i64, String)>> {
        self.require(&[])?;
        let mut s=self.c.prepare("SELECT c.id,r.name,c.revision FROM technical_cards c JOIN recipes r ON r.id=c.recipe ORDER BY c.id DESC").map_err(sql)?;
        let rows = s
            .query_map([], |r| {
                Ok((
                    r.get(0)?,
                    format!(
                        "#{} • {} • revisão {}",
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?
                    ),
                ))
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }
    pub fn card(&self, id: i64) -> R<Card> {
        self.require(&[])?;
        let (snapshot, hash): (String, String) = self
            .c
            .query_row(
                "SELECT snapshot,hash FROM technical_cards WHERE id=?",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(sql)?;
        if digest(snapshot.as_bytes()) != hash {
            return Err("Ficha técnica com falha de integridade.".into());
        }
        serde_json::from_str(&snapshot).map_err(|e| e.to_string())
    }
    pub fn approve_menu(&mut self, f: &MenuInput) -> R<()> {
        let rt = self.require_rt()?;
        let day = crate::validation::iso_date(&f.day)?;
        let card = self.card(f.card)?;
        if f.age_group.trim() != card.age_group.trim()
            || f.diet.trim() != card.diet.trim()
            || f.meal.trim().is_empty()
            || f.planned <= 0
            || f.planned > 1_000_000
        {
            return Err("Etapa/faixa etária e dieta devem coincidir com a ficha; informe horário/tipo de refeição e porções válidas.".into());
        }
        let snapshot = serde_json::to_string(&card).map_err(|e| e.to_string())?;
        let t = self.c.transaction().map_err(sql)?;
        super::governance::unlocked(&t, &f.day)?;
        let current: Option<i64> = {
            let mut s=t.prepare("SELECT m.id FROM menus m WHERE school=? AND date=? AND meal=? AND NOT EXISTS(SELECT 1 FROM menus n WHERE n.supersedes=m.id)").map_err(sql)?;
            let rows = s
                .query_map(params![f.school, day, f.meal.trim()], |r| {
                    r.get::<_, i64>(0)
                })
                .map_err(sql)?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(sql)?
                .into_iter()
                .next()
        };
        if current != f.supersedes {
            return Err("Já existe cardápio para essa escola/data/refeição. Informe o ID atual para substituí-lo mantendo o histórico.".into());
        }
        if let Some(id) = f.supersedes {
            let matches:bool=t.query_row("SELECT EXISTS(SELECT 1 FROM menus WHERE id=? AND school=? AND date=? AND meal=?)",params![id,f.school,day,f.meal.trim()],|r|r.get(0)).map_err(sql)?;
            if !matches {
                return Err("O ID substituído não corresponde ao cardápio informado.".into());
            }
        }
        t.execute("INSERT INTO menus(school,date,meal,age_group,diet,card,planned,snapshot,actor,stamp,supersedes) VALUES (?,?,?,?,?,?,?,?,?,?,?)",params![f.school,day,f.meal.trim(),f.age_group,f.diet,f.card,f.planned,snapshot,rt.id,stamp(),f.supersedes]).map_err(sql)?;
        audit(
            &t,
            Some(rt.id),
            "APROVAR_CARDAPIO",
            "menus",
            &format!(
                "id={}; escola={}; data={}; refeição={}; ficha={}; substitui={:?}",
                t.last_insert_rowid(),
                f.school,
                day,
                f.meal,
                f.card,
                f.supersedes
            ),
        )?;
        t.commit().map_err(sql)
    }
    pub fn menus(&self, start: &str, end: &str) -> R<Vec<MenuRow>> {
        self.require(&[])?;
        let start = crate::validation::iso_date(start)?;
        let end = crate::validation::iso_date(end)?;
        let mut s=self.c.prepare("SELECT m.id,m.date,s.name,m.meal,m.planned FROM menus m JOIN schools s ON s.id=m.school WHERE date BETWEEN ? AND ? AND NOT EXISTS(SELECT 1 FROM menus n WHERE n.supersedes=m.id) ORDER BY m.date,m.meal").map_err(sql)?;
        let rows = s
            .query_map(params![start, end], |r| {
                Ok(MenuRow {
                    id: r.get(0)?,
                    date: r.get(1)?,
                    school: r.get(2)?,
                    meal: r.get(3)?,
                    portions: r.get(4)?,
                })
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }
    pub fn assess_food(&mut self, item: i64, category: &str, allowed: bool, reason: &str) -> R<()> {
        let rt = self.require_rt()?;
        if !super::purchasing::CATEGORIES.contains(&category) || reason.trim().len() < 5 {
            return Err(
                "Informe classificação e justificativa técnica da avaliação do alimento.".into(),
            );
        }
        let t = self.c.transaction().map_err(sql)?;
        t.execute("INSERT INTO food_assessments(item,category,allowed,reason,actor,stamp) VALUES (?,?,?,?,?,?)",params![item,category,allowed,reason.trim(),rt.id,stamp()]).map_err(sql)?;
        audit(
            &t,
            Some(rt.id),
            "AVALIAR_ALIMENTO",
            "food_assessments",
            &format!("item={item}; classe={category}; permitido FNDE={allowed}; {reason}"),
        )?;
        t.commit().map_err(sql)
    }
    pub fn assess_foods_csv(&self) -> R<String> {
        self.require(&[])?;
        let mut out = "Alimento;Categoria;Elegível FNDE conforme avaliação do RT;Justificativa;Nutricionista;CRN;Data da avaliação\n".to_string();
        let mut s=self.c.prepare("SELECT i.name,a.category,a.allowed,a.reason,u.name,u.crn,a.stamp FROM food_assessments a JOIN items i ON i.id=a.item JOIN users u ON u.id=a.actor WHERE a.id IN (SELECT MAX(id) FROM food_assessments GROUP BY item) ORDER BY i.name").map_err(sql)?;
        let rows = s
            .query_map([], |r| {
                Ok(vec![
                    r.get::<_, String>(0)?,
                    r.get(1)?,
                    if r.get::<_, bool>(2)? {
                        "Sim".into()
                    } else {
                        "Não".into()
                    },
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                ])
            })
            .map_err(sql)?;
        for row in rows {
            out += &row
                .map_err(sql)?
                .iter()
                .map(|v| super::exports::csv(v))
                .collect::<Vec<_>>()
                .join(";");
            out.push('\n');
        }
        Ok(out)
    }
}
