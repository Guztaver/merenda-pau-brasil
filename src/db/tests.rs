use super::*;
use crate::validation::{date, grams};

#[test]
fn input_validation() {
    assert!(date("2026-02-30").is_err());
    assert!(date("31/02/2026").is_err());
    assert_eq!(
        crate::validation::iso_date("20/10/2026").unwrap(),
        "2026-10-20"
    );
    assert_eq!(
        crate::validation::measured_grams("5.000,5", 2).unwrap(),
        10_001
    );
    assert_eq!(grams("5.000.000").unwrap(), 5_000_000);
    assert_eq!(crate::validation::integer("5.000").unwrap(), 5_000);
    assert_eq!(crate::validation::money("R$ 1.000,00").unwrap(), 100_000);
    assert!(super::purchasing::document_valid("529.982.247-25"));
    assert!(grams("NaN").is_err());
    assert!(grams("0").is_err());
    assert_eq!(grams("100,5").unwrap(), 101);
    assert!(date("2026-1-01").is_err());
}

#[test]
fn utf8_csv_fields_are_quoted_and_formula_protected() {
    assert_eq!(
        super::exports::csv("Pau Brasil;\"teste\""),
        "\"Pau Brasil;\"\"teste\"\"\""
    );
    assert_eq!(super::exports::csv("=1+1"), "\"'=1+1\"");
}

#[test]
fn categories_and_financial_indicators_use_federal_transfers_as_base() {
    let mut db = Db::init(Connection::open_in_memory().unwrap()).unwrap();
    db.setup_admin("Administrador", "admin", "UmaSenhaMuitoSegura123")
        .unwrap();
    let report = db.compliance_report("2026").unwrap();
    assert!(!report.sufficient_base);
    assert_eq!(report.family_percent, None);
    assert_eq!(super::purchasing::CATEGORIES.len(), 4);
    assert_eq!(super::purchasing::METHODS.len(), 4);
}

#[test]
fn legacy_database_migrates_without_dropping_existing_inventory() {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch(include_str!("schema.sql")).unwrap();
    c.execute(
        "INSERT INTO items(id,name,minimum) VALUES(1,'Arroz',500)",
        [],
    )
    .unwrap();
    c.execute(
        "INSERT INTO movements(date,item,grams,note) VALUES('2026-01-01',1,2000,'Saldo legado')",
        [],
    )
    .unwrap();
    c.pragma_update(None, "user_version", 1).unwrap();
    let db = Db::init(c).unwrap();
    assert_eq!(
        db.c.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        4
    );
    assert_eq!(
        db.c.query_row("SELECT grams FROM movements WHERE item=1", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        2000
    );
    assert_eq!(
        db.c.query_row("SELECT COUNT(*) FROM lots WHERE legacy=1", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn partially_applied_v2_migration_is_recovered_without_duplicate_column() {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch(include_str!("schema.sql")).unwrap();
    c.execute_batch("CREATE TABLE lots(id INTEGER PRIMARY KEY, item INTEGER NOT NULL REFERENCES items(id), receipt INTEGER, batch TEXT NOT NULL, expiry TEXT, location TEXT NOT NULL, legacy INTEGER NOT NULL DEFAULT 0, parent INTEGER);")
        .unwrap();
    c.execute_batch("ALTER TABLE movements ADD COLUMN lot_id INTEGER REFERENCES lots(id);")
        .unwrap();
    c.execute("INSERT INTO items(id,name,minimum) VALUES(1,'Arroz',0)", [])
        .unwrap();
    c.execute(
        "INSERT INTO movements(date,item,grams,note) VALUES('2026-01-01',1,2500,'Saldo legado')",
        [],
    )
    .unwrap();
    c.pragma_update(None, "user_version", 1).unwrap();

    let db = Db::init(c).unwrap();

    assert_eq!(
        db.c.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        4
    );
    assert_eq!(
        db.c.query_row("SELECT grams FROM movements WHERE item=1", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        2500
    );
    assert_eq!(
        db.c.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('movements') WHERE name='lot_id'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
}

#[test]
fn v3_migration_preserves_supplier_snapshot_and_adds_item_units() {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch(include_str!("schema.sql")).unwrap();
    c.execute_batch(include_str!("migration_v2.sql")).unwrap();
    c.execute_batch("CREATE TRIGGER immutable_receipts_update BEFORE UPDATE ON receipts BEGIN SELECT RAISE(ABORT,'Registro imutável.'); END;").unwrap();
    c.execute_batch(include_str!("migration_v3.sql")).unwrap();
    c.execute("INSERT INTO users(id,name,login,password_hash,role,crn) VALUES(1,'Admin','admin','x','Administrador','')",[]).unwrap();
    c.execute("INSERT INTO items(id,name,minimum) VALUES(1,'Arroz',0)", [])
        .unwrap();
    c.execute("INSERT INTO suppliers(id,name,document,family,caf,verified) VALUES(1,'Fornecedor antigo','52998224725',1,'CAF-1',1)",[]).unwrap();
    c.execute("INSERT INTO operations(id,kind,date,actor,reason) VALUES(1,'RECEBIMENTO','2026-01-10',1,'NF-1')",[]).unwrap();
    c.execute("INSERT INTO receipts(id,supplier,item,invoice,process,contract,method,funding,category,grams,cents,operation_id) VALUES(1,1,1,'NF-1','PROC','CONT','Pregão','Próprios','In natura/minimamente processado',1000,100,1)",[]).unwrap();
    let db = Db::init(c).unwrap();
    assert!(db
        .c
        .query_row("SELECT supplier_family FROM receipts WHERE id=1", [], |r| r
            .get::<_, bool>(0))
        .unwrap());
    assert_eq!(db.c.query_row("SELECT supplier_name||':'||supplier_document||':'||supplier_caf FROM receipts WHERE id=1",[],|r|r.get::<_,String>(0)).unwrap(),"Fornecedor antigo:52998224725:CAF-1");
    assert_eq!(
        db.c.query_row(
            "SELECT unit||':'||unit_grams FROM items WHERE id=1",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "g:1"
    );
    assert!(db
        .c
        .execute(
            "UPDATE receipts SET supplier_name='alterado' WHERE id=1",
            []
        )
        .is_err());
}

#[test]
fn role_checks_deny_administrative_actions_to_read_only_accounts() {
    let mut db = Db::init(Connection::open_in_memory().unwrap()).unwrap();
    db.setup_admin("Admin", "admin", "UmaSenhaMuitoSegura123")
        .unwrap();
    db.create_user(
        "Consulta",
        "leitura",
        "OutraSenhaMuitoSegura123",
        "Consulta",
        "",
    )
    .unwrap();
    db.logout();
    db.login("leitura", "OutraSenhaMuitoSegura123").unwrap();
    assert!(db
        .add_supplier("Fornecedor", "529.982.247-25", false, "")
        .is_err());
    assert!(db.verify_audit().unwrap() >= 3);
}
