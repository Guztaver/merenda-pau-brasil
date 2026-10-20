use super::{components::*, theme::*};
use crate::app::App;
use eframe::egui::{self, RichText};
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let show_errors = app.notice.as_ref().is_some_and(|notice| notice.error);
    card(
        ui,
        "Registrar refeições servidas",
        "Os ingredientes são descontados do estoque em uma única operação.",
        |ui| {
            ui.columns(2, |cols| {
                required_field(
                    &mut cols[0],
                    "Data do consumo (DD/MM/AAAA)",
                    &mut app.servings.day,
                    show_errors,
                );
                required_choose(
                    &mut cols[1],
                    "Escola atendida",
                    "serving_school",
                    &mut app.servings.school,
                    app.db.names("schools"),
                    show_errors,
                );
            });
            ui.columns(2, |cols| {
                required_choose(
                    &mut cols[0],
                    "Cardápio aprovado da escola e data",
                    "serving_recipe",
                    &mut app.servings.recipe,
                    app.db
                        .menu_choices(app.servings.school, &app.servings.day)
                        .unwrap_or_default(),
                    show_errors,
                );
                required_field(
                    &mut cols[1],
                    "Quantidade de porções",
                    &mut app.servings.portions,
                    show_errors,
                );
            });
            if let Ok(n) = crate::validation::integer(&app.servings.portions) {
                if n > 0 && n <= 1_000_000 {
                    if let Ok(card) = app.db.menu_card(app.servings.recipe) {
                        let rows = card.ingredients;
                        if !rows.is_empty() {
                            hint(ui, "Ingredientes que serão retirados do estoque:");
                            ui.horizontal_wrapped(|ui| {
                                for r in rows {
                                    badge(
                                        ui,
                                        &format!(
                                            "{}: {} kg",
                                            r.name,
                                            crate::validation::kg(r.grams * n)
                                        ),
                                        false,
                                    );
                                }
                            });
                        }
                    }
                }
            }
            ui.checkbox(
                &mut app.servings.at_school,
                "Preparação na escola (consumir do estoque transferido para a unidade)",
            );
            hint(ui,"Confira a data e as porções antes de salvar. É permitido registrar dias anteriores.");
            if primary(ui, "Registrar refeições e baixar estoque") {
                let f = &app.servings;
                let r = crate::validation::integer(&f.portions)
                    .map_err(|_| "Informe um número inteiro de porções.".to_string())
                    .and_then(|n| app.db.serve(&f.day, f.school, f.recipe, n, !f.at_school));
                if app.notify(r) {
                    app.servings.portions.clear();
                }
            }
        },
    );
    ui.add_space(18.);
    card(
        ui,
        "Histórico de refeições",
        "Os 50 lançamentos mais recentes da rede municipal.",
        |ui| match app.db.servings() {
            Err(e) => hint(ui, &e),
            Ok(rows) => {
                if rows.is_empty() {
                    empty(
                        ui,
                        "As refeições aparecerão aqui",
                        "Cadastre escolas, fichas e cardápios aprovados para registrar o atendimento.",
                    );
                }
                egui::ScrollArea::horizontal()
                    .id_salt("serving_rows")
                    .show(ui, |ui| {
                        egui::Grid::new("serving_table")
                            .spacing([24., 16.])
                            .show(ui, |ui| {
                                for h in ["DATA", "ESCOLA", "REFEIÇÃO", "PORÇÕES"] {
                                    ui.label(RichText::new(h).size(11.).color(MUTED));
                                }
                                ui.end_row();
                                for r in rows {
                                    ui.label(pretty_date(&r.date));
                                    ui.strong(r.school);
                                    ui.label(r.recipe);
                                    badge(ui, &r.portions.to_string(), false);
                                    ui.end_row();
                                }
                            });
                    });
            }
        },
    );
}
