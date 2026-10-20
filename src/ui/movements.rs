use super::{components::*, theme::*};
use crate::{app::App, validation::measured_grams};
use eframe::egui::{self, RichText};
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let show_errors = app.notice.as_ref().is_some_and(|notice| notice.error);
    ui.horizontal_wrapped(|ui| {
        if mode_option(ui, "Entrada de alimentos", !app.movements.loss) {
            app.movements.loss = false;
        }
        if mode_option(ui, "Perda ou ajuste de saída", app.movements.loss) {
            app.movements.loss = true;
        }
    });
    if !app.movements.loss {
        super::purchases::receive_form(app, ui);
    } else {
        card(ui,"Registrar perda ou descarte","A saída deve identificar o lote e a justificativa. Lotes vencidos podem ser descartados, mas não consumidos em refeições.",|ui|{
            required_field(ui,"Data da saída (DD/MM/AAAA)",&mut app.movements.day,show_errors);
            let available_lots=app.db.lots(&app.movements.day).unwrap_or_default();
            let lots=available_lots.iter().filter(|l|l.balance>0).map(|l|(l.id,format!("{} • {} • {} {} • vence {}",l.label,l.location,display_quantity(l.balance,l.unit_grams),l.unit,pretty_date(&l.expiry)))).collect();
            required_choose(ui,"Lote","loss_lot",&mut app.movements.lot,lots,show_errors);
            let measure=available_lots.iter().find(|l|l.id==app.movements.lot).map(|l|(l.unit.clone(),l.unit_grams)).unwrap_or_else(||("g".into(),1));
            required_field(ui,&format!("Quantidade da perda ({})",measure.0),&mut app.movements.weight,show_errors);required_multiline(ui,"Justificativa e documento de descarte",&mut app.movements.note,show_errors);
            if primary(ui,"Registrar saída justificada"){
                let f=&app.movements;let r=measured_grams(&f.weight,measure.1).and_then(|g|app.db.loss(&f.day,f.lot,g,&f.note));if app.notify(r){app.movements.weight.clear();app.movements.note.clear();}
            }
            ui.add_space(10.);
            ui.horizontal_wrapped(|ui|{
                hint(ui,"Atalhos de cadastro");
                if ui.button("+ Cadastrar alimento / material").clicked(){app.navigate(crate::app::Page::Foods);}
                else if ui.button("Consultar lotes e validades").clicked(){app.navigate(crate::app::Page::Lots);}
            });
        });
    }
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
                                            "{}{} {}",
                                            if r.grams > 0 { "+" } else { "−" },
                                            display_quantity(r.grams.abs(), r.unit_grams),
                                            r.unit
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
