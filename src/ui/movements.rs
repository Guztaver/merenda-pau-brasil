use super::{components::*, theme::*};
use crate::{
    app::App,
    validation::{grams, kg},
};
use eframe::egui::{self, RichText};
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    card(
        ui,
        "Nova movimentação",
        "Recebimentos e perdas podem ser registrados com datas anteriores.",
        |ui| {
            ui.horizontal_wrapped(|ui| {
                if mode_option(ui, "Entrada de alimentos", !app.movements.loss) {
                    app.movements.loss = false;
                }
                if mode_option(ui, "Perda ou ajuste de saída", app.movements.loss) {
                    app.movements.loss = true;
                }
            });
            ui.columns(2, |cols| {
                field(&mut cols[0], "Data (AAAA-MM-DD)", &mut app.movements.day);
                choose(
                    &mut cols[1],
                    "Alimento",
                    "movement_food",
                    &mut app.movements.item,
                    app.db.names("items"),
                );
            });
            ui.columns(2, |cols| {
                field(&mut cols[0], "Peso total (g)", &mut app.movements.weight);
                field(
                    &mut cols[1],
                    "Origem, documento ou justificativa",
                    &mut app.movements.note,
                );
            });
            hint(
                ui,
                "Consumo de refeições deve ser lançado em Refeições servidas.",
            );
            if primary(
                ui,
                if app.movements.loss {
                    "Registrar saída"
                } else {
                    "+  Registrar entrada"
                },
            ) {
                let f = &app.movements;
                let r = grams(&f.weight).and_then(|g| {
                    app.db
                        .movement(&f.day, f.item, if f.loss { -g } else { g }, &f.note)
                });
                if app.notify(r) {
                    app.movements.weight.clear();
                    app.movements.note.clear();
                }
            }
        },
    );
    ui.add_space(18.);
    card(
        ui,
        "Últimos movimentos",
        "Os 50 registros mais recentes, incluindo consumo de refeições.",
        |ui| match app.db.movements() {
            Err(e) => {
                hint(ui, &e);
            }
            Ok(rows) => {
                if rows.is_empty() {
                    empty(
                        ui,
                        "Nenhuma movimentação ainda",
                        "Registre a primeira entrada para começar a usar o estoque.",
                    );
                }
                egui::ScrollArea::horizontal()
                    .id_salt("movement_rows")
                    .show(ui, |ui| {
                        egui::Grid::new("movement_table")
                            .spacing([24., 14.])
                            .show(ui, |ui| {
                                for h in ["DATA", "ALIMENTO", "PESO", "ORIGEM / JUSTIFICATIVA"] {
                                    ui.label(RichText::new(h).size(11.).color(MUTED));
                                }
                                ui.end_row();
                                for r in rows {
                                    ui.label(pretty_date(&r.date));
                                    ui.strong(r.food);
                                    ui.colored_label(
                                        if r.grams > 0 { GREEN } else { AMBER },
                                        format!(
                                            "{}{} kg",
                                            if r.grams > 0 { "+" } else { "" },
                                            kg(r.grams)
                                        ),
                                    );
                                    ui.label(r.note);
                                    ui.end_row();
                                }
                            });
                    });
            }
        },
    );
}
