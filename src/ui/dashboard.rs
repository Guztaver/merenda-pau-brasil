use super::{components::*, theme::*};
use crate::app::{App, Page};
use eframe::egui::{self, Color32, RichText};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let stock = app.db.stock(&today);
    let alerts = stock.iter().filter(|x| x.1 < x.2).count();
    let portions = match app.db.portions_on(&today) {
        Ok(n) => n.to_string(),
        Err(e) => {
            app.notify(Err(e));
            "—".into()
        }
    };
    ui.columns(3, |columns| {
        for (col, (label, value, detail)) in columns.iter_mut().zip([
            (
                "ALIMENTOS EM ESTOQUE",
                stock.len().to_string(),
                "itens cadastrados no município",
            ),
            ("PORÇÕES HOJE", portions, "refeições registradas nesta data"),
            (
                "PRECISAM DE REPOSIÇÃO",
                alerts.to_string(),
                "alimentos abaixo do mínimo",
            ),
        ]) {
            egui::Frame::new()
                .fill(Color32::WHITE)
                .corner_radius(16)
                .inner_margin(20.)
                .stroke(egui::Stroke::new(1.0_f32, LINE))
                .show(col, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(RichText::new(label).size(10.).color(MUTED).strong());
                    ui.label(RichText::new(value).size(38.).strong().color(GREEN));
                    hint(ui, detail);
                });
        }
    });
    ui.add_space(18.);
    egui::Frame::new()
        .fill(GREEN)
        .corner_radius(18)
        .inner_margin(24.)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                RichText::new("Uma boa refeição começa com organização.")
                    .size(21.)
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.label(
                RichText::new("Registre o que chegou à cozinha e as porções que foram servidas.")
                    .color(Color32::from_rgb(207, 231, 214)),
            );
            ui.add_space(8.);
            ui.horizontal_wrapped(|ui| {
                if ui.button("+  Registrar refeições").clicked() {
                    app.navigate(Page::Servings);
                }
                if ui.button("+  Entrada de alimentos").clicked() {
                    app.navigate(Page::Movements);
                }
            });
        });
    ui.add_space(18.);
    card(
        ui,
        "Estoque de hoje",
        "Saldos em quilogramas • alertas conforme o mínimo de cada alimento",
        |ui| {
            stock_table(ui, stock);
        },
    );
    ui.add_space(18.);
    card(
        ui,
        "Seu fluxo de trabalho",
        "Do recebimento ao acompanhamento",
        |ui| {
            ui.horizontal_wrapped(|ui| {
                for label in [
                    "1  Cadastre escolas e alimentos",
                    "2  Registre entradas",
                    "3  Monte receitas",
                    "4  Lance refeições",
                ] {
                    badge(ui, label, false);
                }
            });
        },
    );
}
