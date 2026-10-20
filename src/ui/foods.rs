use super::components::*;
use crate::{app::App, validation::date};
use eframe::egui;
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let show_errors = app.notice.as_ref().is_some_and(|notice| notice.error);
    card(
        ui,
        "Novo alimento",
        "Escolha como o produto é comprado e informe o peso equivalente de cada unidade.",
        |ui| {
            ui.columns(2, |cols| {
                required_field(
                    &mut cols[0],
                    "Nome do alimento ou material",
                    &mut app.foods.name,
                    show_errors,
                );
                options(
                    &mut cols[1],
                    "Unidade de compra",
                    "food_unit",
                    &mut app.foods.unit,
                    &["g", "kg", "L", "mL", "cx", "un"],
                );
            });
            ui.columns(2, |cols| {
                field(
                    &mut cols[0],
                    "Equivalência em gramas por unidade",
                    &mut app.foods.unit_grams,
                );
                field(
                    &mut cols[1],
                    "Estoque mínimo (gramas equivalentes)",
                    &mut app.foods.minimum,
                );
            });
            hint(ui,"Exemplos: 1 kg = 1.000 g; 1 L de leite ≈ 1.000 g (confira o rótulo); 1 caixa deve usar o peso líquido total indicado na embalagem. A equivalência precisa ser conferida pelo cadastro.");
            if primary(ui, "+  Cadastrar alimento") {
                let r = app.db.add_food(
                    &app.foods.name,
                    &app.foods.minimum,
                    &app.foods.unit,
                    &app.foods.unit_grams,
                );
                if app.notify(r) {
                    app.foods.name.clear();
                }
            }
        },
    );
    ui.add_space(18.);
    card(
        ui,
        "Consultar estoque",
        "O saldo considera movimentos em gramas e mostra também a unidade de compra cadastrada.",
        |ui| {
            ui.columns(2, |cols| {
                field(
                    &mut cols[0],
                    "Data do saldo (DD/MM/AAAA)",
                    &mut app.foods.day,
                );
                field(&mut cols[1], "Buscar alimento", &mut app.foods.search);
            });
            if date(&app.foods.day).is_ok() {
                let search = app.foods.search.to_lowercase();
                let rows = app
                    .db
                    .stock(&app.foods.day)
                    .into_iter()
                    .filter(|r| r.0.to_lowercase().contains(&search))
                    .collect();
                stock_table(ui, rows);
            } else {
                hint(ui, "Informe uma data válida no formato DD/MM/AAAA.");
            }
        },
    );
}
