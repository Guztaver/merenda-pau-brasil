use super::*;
use crate::validation::grams;
fn db() -> Db {
    let d = Db::init(Connection::open_in_memory().unwrap()).unwrap();
    d.c.execute("INSERT INTO items VALUES(1,'Arroz',1000)", [])
        .unwrap();
    d.add("schools", "Escola A").unwrap();
    d.add("recipes", "Arroz cozido").unwrap();
    d.c.execute("INSERT INTO ingredients VALUES(1,1,100)", [])
        .unwrap();
    d
}
#[test]
fn retroactive_shortage_rolls_back() {
    let mut d = db();
    d.movement("2026-10-01", 1, 1000, "Compra").unwrap();
    d.serve("2026-10-10", 1, 1, 8).unwrap();
    assert!(d.movement("2026-10-05", 1, -300, "Perda").is_err());
    assert_eq!(d.stock("2026-10-20")[0].1, 200);
}
#[test]
fn atomic_serving_and_history() {
    let mut d = db();
    d.movement("2026-10-01", 1, 1000, "Compra").unwrap();
    assert!(d.serve("2026-10-02", 1, 1, 11).is_err());
    assert_eq!(
        d.c.query_row("SELECT COUNT(*) FROM servings", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    d.serve("2026-10-03", 1, 1, 4).unwrap();
    assert_eq!(d.stock("2026-10-02")[0].1, 1000);
    assert_eq!(d.stock("2026-10-03")[0].1, 600);
    assert!(d
        .report("2026-10-01", "2026-10-31")
        .unwrap()
        .contains("0,400"));
}
#[test]
fn input_validation() {
    assert!(date("2026-02-30").is_err());
    assert!(grams("NaN").is_err());
    assert!(grams("0").is_err());
    assert_eq!(grams("100,5").unwrap(), 101);
    assert!(date("2026-1-01").is_err());
}

#[test]
fn recipe_changes_preserve_consumption_history() {
    let mut d = db();
    d.movement("2026-10-01", 1, 5000, "Compra").unwrap();
    d.serve("2026-10-02", 1, 1, 10).unwrap();
    d.save_ingredient(1, 1, 200).unwrap();
    d.serve("2026-10-03", 1, 1, 10).unwrap();
    assert_eq!(d.stock("2026-10-02")[0].1, 4000);
    assert_eq!(d.stock("2026-10-03")[0].1, 2000);
    assert_eq!(d.portions_on("2026-10-02").unwrap(), 10);
    assert_eq!(d.servings().unwrap().len(), 2);
    assert_eq!(d.movements().unwrap()[0].grams, -2000);
    d.remove_ingredient(1, 1).unwrap();
    assert!(d.serve("2026-10-04", 1, 1, 1).is_err());
    assert_eq!(d.stock("2026-10-04")[0].1, 2000);
}

#[test]
fn reopening_existing_database_keeps_history() {
    let mut d = db();
    d.movement("2026-10-01", 1, 1000, "Compra").unwrap();
    d.serve("2026-10-02", 1, 1, 5).unwrap();
    let reopened = Db::init(d.c).unwrap();
    assert_eq!(reopened.stock("2026-10-03")[0].1, 500);
    assert_eq!(reopened.servings().unwrap()[0].portions, 5);
    assert!(reopened
        .report("2026-10-02", "2026-10-02")
        .unwrap()
        .contains("1,000;0,000;0,500;0,500"));
}
