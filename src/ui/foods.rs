use super::components::*;
use crate::{app::App, validation::date};
use eframe::egui;
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    card(
        ui,
        "Novo alimento",
        "Informe o peso mínimo para identificar quando é hora de repor.",
        |ui| {
            ui.columns(2, |cols| {
                field(&mut cols[0], "Nome do alimento", &mut app.foods.name);
                field(&mut cols[1], "Estoque mínimo (g)", &mut app.foods.minimum);
            });
            hint(
                ui,
                "Exemplo: 5.000 gramas = 5 kg. Digite 5000, sem ponto de milhar.",
            );
            if primary(ui, "+  Cadastrar alimento") {
                let r = app.db.add_food(&app.foods.name, &app.foods.minimum);
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
        "O saldo considera todos os movimentos até a data escolhida.",
        |ui| {
            ui.columns(2, |cols| {
                field(
                    &mut cols[0],
                    "Data do saldo (AAAA-MM-DD)",
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
                hint(ui, "Informe uma data válida no formato AAAA-MM-DD.");
            }
        },
    );
}
