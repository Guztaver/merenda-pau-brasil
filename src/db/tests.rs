use super::*;
use crate::validation::{date, grams};

#[test]
fn input_validation() {
    assert!(date("2026-02-30").is_err());
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
        3
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
