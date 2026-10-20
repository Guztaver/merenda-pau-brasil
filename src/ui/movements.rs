use super::{components::*, theme::*};
use crate::{
    app::App,
    validation::{grams, kg},
};
use eframe::egui::{self, RichText};
pub fn show(app: &mut App, ui: &mut egui::Ui) {
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
            field(ui,"Data da saída (AAAA-MM-DD)",&mut app.movements.day);
            let lots=app.db.lots(&app.movements.day).unwrap_or_default().into_iter().filter(|l|l.balance>0).map(|l|(l.id,format!("{} • {} • {} kg",l.label,l.location,kg(l.balance)))).collect();
            choose(ui,"Lote","loss_lot",&mut app.movements.lot,lots);
            field(ui,"Peso da perda (g)",&mut app.movements.weight);multiline(ui,"Justificativa e documento de descarte",&mut app.movements.note);
            if primary(ui,"Registrar saída justificada"){
                let f=&app.movements;let r=grams(&f.weight).and_then(|g|app.db.loss(&f.day,f.lot,g,&f.note));if app.notify(r){app.movements.weight.clear();app.movements.note.clear();}
            }
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
