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
        if version > 4 {
            return Err("Banco criado por uma versão mais recente. Atualize o aplicativo.".into());
        }
        if version < 2 {
            let t = c.transaction().map_err(sql)?;
            t.execute_batch(include_str!("schema.sql")).map_err(sql)?;
            migrate_v2(&t)?;
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
        if version < 4 {
            let t = c.transaction().map_err(sql)?;
            t.execute_batch(include_str!("migration_v4.sql"))
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
    pub fn suppliers(&self) -> R<Vec<purchasing::Supplier>> {
        self.require(&[])?;
        let mut stmt = self.c.prepare("SELECT id,name,document,family,caf,verified,COALESCE(verification,'') FROM suppliers ORDER BY name").map_err(sql)?;
        let rows = stmt
            .query_map([], |r| {
                Ok(purchasing::Supplier {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    document: r.get(2)?,
                    family: r.get(3)?,
                    caf: r.get(4)?,
                    verified: r.get(5)?,
                    verification: r.get(6)?,
                })
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }
    pub fn food_choices(&self) -> Vec<(i64, String)> {
        if self.require(&[]).is_err() {
            return Vec::new();
        }
        let Ok(mut stmt) = self
            .c
            .prepare("SELECT id,name,unit FROM items ORDER BY name")
        else {
            return Vec::new();
        };
        let Ok(rows) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                format!("{} • {}", r.get::<_, String>(1)?, r.get::<_, String>(2)?),
            ))
        }) else {
            return Vec::new();
        };
        rows.filter_map(Result::ok).collect()
    }
    pub fn item_measure(&self, item: i64) -> Option<(String, i64)> {
        self.c
            .query_row(
                "SELECT unit,unit_grams FROM items WHERE id=?",
                [item],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok()
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
        let date = crate::validation::iso_date(date)?;
        c.execute(
            "INSERT INTO operations(kind,date,actor,reason) VALUES (?,?,?,?)",
            params![kind, date, actor, reason],
        )
        .map_err(sql)?;
        Ok(c.last_insert_rowid())
    }
}

/// Completes v2 incrementally so databases left partially upgraded by an older
/// build can be opened safely. SQLite DDL is transactional here; this also
/// handles columns that were already created before user_version was advanced.
fn migrate_v2(c: &Connection) -> R<()> {
    // Create referenced tables first: SQLite validates referenced table names
    // while adding a column with a foreign-key constraint.
    c.execute_batch(
        "CREATE TABLE IF NOT EXISTS users(id INTEGER PRIMARY KEY, name TEXT NOT NULL, login TEXT NOT NULL UNIQUE COLLATE NOCASE, password_hash TEXT NOT NULL, role TEXT NOT NULL CHECK(role IN ('Administrador','Estoque','Nutricionista','Consulta')), crn TEXT NOT NULL DEFAULT '', active INTEGER NOT NULL DEFAULT 1);
         CREATE TABLE IF NOT EXISTS audit(id INTEGER PRIMARY KEY, stamp TEXT NOT NULL, actor INTEGER REFERENCES users(id), action TEXT NOT NULL, entity TEXT NOT NULL, detail TEXT NOT NULL, previous TEXT NOT NULL, hash TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS periods(month TEXT PRIMARY KEY, closed INTEGER NOT NULL, reason TEXT NOT NULL, actor INTEGER NOT NULL REFERENCES users(id), stamp TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS suppliers(id INTEGER PRIMARY KEY, name TEXT NOT NULL, document TEXT NOT NULL UNIQUE, family INTEGER NOT NULL CHECK(family IN (0,1)), caf TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS operations(id INTEGER PRIMARY KEY, kind TEXT NOT NULL, date TEXT NOT NULL, actor INTEGER NOT NULL REFERENCES users(id), reason TEXT NOT NULL, reversal_of INTEGER UNIQUE REFERENCES operations(id));
         CREATE TABLE IF NOT EXISTS receipts(id INTEGER PRIMARY KEY, supplier INTEGER NOT NULL REFERENCES suppliers(id), item INTEGER NOT NULL REFERENCES items(id), invoice TEXT NOT NULL, process TEXT NOT NULL, contract TEXT NOT NULL, method TEXT NOT NULL, funding TEXT NOT NULL CHECK(funding IN ('FNDE','Próprios')), category TEXT NOT NULL, grams INTEGER NOT NULL CHECK(grams>0), cents INTEGER NOT NULL CHECK(cents>0), operation_id INTEGER NOT NULL UNIQUE REFERENCES operations(id));
         CREATE TABLE IF NOT EXISTS lots(id INTEGER PRIMARY KEY, item INTEGER NOT NULL REFERENCES items(id), receipt INTEGER REFERENCES receipts(id), batch TEXT NOT NULL, expiry TEXT, location TEXT NOT NULL, legacy INTEGER NOT NULL DEFAULT 0, parent INTEGER REFERENCES lots(id));
         CREATE TABLE IF NOT EXISTS payments(id INTEGER PRIMARY KEY, receipt INTEGER NOT NULL REFERENCES receipts(id), cents INTEGER NOT NULL CHECK(cents>0), document TEXT NOT NULL, operation_id INTEGER NOT NULL UNIQUE REFERENCES operations(id));
         CREATE TABLE IF NOT EXISTS resources(id INTEGER PRIMARY KEY, cents INTEGER NOT NULL CHECK(cents>0), document TEXT NOT NULL, kind TEXT NOT NULL CHECK(kind IN ('Repasse FNDE','Saldo reprogramado')), operation_id INTEGER NOT NULL UNIQUE REFERENCES operations(id));
         CREATE TABLE IF NOT EXISTS technical_cards(id INTEGER PRIMARY KEY, recipe INTEGER NOT NULL REFERENCES recipes(id), revision INTEGER NOT NULL, snapshot TEXT NOT NULL, hash TEXT NOT NULL, actor INTEGER NOT NULL REFERENCES users(id), stamp TEXT NOT NULL, UNIQUE(recipe,revision));
         CREATE TABLE IF NOT EXISTS menus(id INTEGER PRIMARY KEY, school INTEGER NOT NULL REFERENCES schools(id), date TEXT NOT NULL, meal TEXT NOT NULL, age_group TEXT NOT NULL, diet TEXT NOT NULL, card INTEGER NOT NULL REFERENCES technical_cards(id), planned INTEGER NOT NULL CHECK(planned>0), snapshot TEXT NOT NULL, actor INTEGER NOT NULL REFERENCES users(id), stamp TEXT NOT NULL, supersedes INTEGER UNIQUE REFERENCES menus(id));
         CREATE TABLE IF NOT EXISTS documents(id INTEGER PRIMARY KEY, entity TEXT NOT NULL, entity_id INTEGER NOT NULL, name TEXT NOT NULL, stored_name TEXT NOT NULL, hash TEXT NOT NULL, actor INTEGER NOT NULL REFERENCES users(id), stamp TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS food_assessments(id INTEGER PRIMARY KEY,item INTEGER NOT NULL REFERENCES items(id),category TEXT NOT NULL,allowed INTEGER NOT NULL,reason TEXT NOT NULL,actor INTEGER NOT NULL REFERENCES users(id),stamp TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS lot_evidence(lot INTEGER PRIMARY KEY REFERENCES lots(id),supplier INTEGER NOT NULL REFERENCES suppliers(id),invoice TEXT NOT NULL,reason TEXT NOT NULL,actor INTEGER NOT NULL REFERENCES users(id),stamp TEXT NOT NULL);",
    )
    .map_err(sql)?;
    for (table, column, definition) in [
        ("movements", "lot_id", "INTEGER REFERENCES lots(id)"),
        ("movements", "operation_id", "INTEGER REFERENCES operations(id)"),
        ("servings", "operation_id", "INTEGER REFERENCES operations(id)"),
        ("servings", "card_id", "INTEGER REFERENCES technical_cards(id)"),
        ("servings", "menu_id", "INTEGER REFERENCES menus(id)"),
        ("recipes", "approved_card", "INTEGER REFERENCES technical_cards(id)"),
    ] {
        if !has_column(c, table, column)? {
            c.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"))
                .map_err(sql)?;
        }
    }
    c.execute_batch("INSERT INTO lots(item,batch,location,legacy) SELECT id,'LEGADO — conferir documentação e validade','central',1 FROM items WHERE NOT EXISTS (SELECT 1 FROM lots WHERE lots.item=items.id AND legacy=1);")
        .map_err(sql)?;
    c.execute("UPDATE movements SET lot_id=(SELECT id FROM lots WHERE lots.item=movements.item AND legacy=1) WHERE lot_id IS NULL", [])
        .map_err(sql)?;
    c.execute_batch(
        "CREATE INDEX IF NOT EXISTS lot_history ON movements(lot_id,date);
         CREATE INDEX IF NOT EXISTS operation_dates ON operations(date);
         CREATE TRIGGER IF NOT EXISTS immutable_movements_update BEFORE UPDATE ON movements BEGIN SELECT RAISE(ABORT,'Movimentos são imutáveis. Use estorno.'); END;
         CREATE TRIGGER IF NOT EXISTS immutable_movements_delete BEFORE DELETE ON movements BEGIN SELECT RAISE(ABORT,'Movimentos são imutáveis. Use estorno.'); END;
         CREATE TRIGGER IF NOT EXISTS require_operation BEFORE INSERT ON movements WHEN NEW.operation_id IS NULL OR NEW.lot_id IS NULL BEGIN SELECT RAISE(ABORT,'Movimento exige operação e lote.'); END;
         PRAGMA user_version=2;",
    )
    .map_err(sql)
}

fn has_column(c: &Connection, table: &str, column: &str) -> R<bool> {
    let mut stmt = c
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(sql)?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>("name"))
        .map_err(sql)?;
    for name in columns {
        if name.map_err(sql)? == column {
            return Ok(true);
        }
    }
    Ok(false)
}
