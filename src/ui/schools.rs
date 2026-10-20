use super::components::*;
use crate::app::App;
use eframe::egui;
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let show_errors = app.notice.as_ref().is_some_and(|notice| notice.error);
    card(
        ui,
        "Adicionar escola",
        "As unidades cadastradas ficam disponíveis no registro de refeições.",
        |ui| {
            required_field(
                ui,
                "Nome da unidade escolar",
                &mut app.school_name,
                show_errors,
            );
            if primary(ui, "+  Cadastrar escola") {
                let r = app.db.add("schools", &app.school_name);
                if app.notify(r) {
                    app.school_name.clear();
                }
            }
        },
    );
    ui.add_space(18.);
    card(
        ui,
        "Unidades da rede",
        "Escolas cadastradas para acompanhamento da alimentação escolar.",
        |ui| {
            let rows = app.db.names("schools");
            if rows.is_empty() {
                empty(
                    ui,
                    "Vamos conhecer sua rede?",
                    "Cadastre a primeira escola no formulário acima.",
                );
            }
            if !rows.is_empty() {
                egui::ScrollArea::horizontal()
                    .id_salt("schools_table_scroll")
                    .show(ui, |ui| {
                        egui::Grid::new("schools_table")
                            .min_col_width(100.)
                            .spacing([24., 12.])
                            .striped(true)
                            .show(ui, |ui| {
                                for h in ["CÓDIGO", "UNIDADE ESCOLAR"] {
                                    ui.label(egui::RichText::new(h).size(11.).strong());
                                }
                                ui.end_row();
                                for (id, name) in rows {
                                    ui.label(format!("#{id}"));
                                    ui.strong(name);
                                    ui.end_row();
                                }
                            });
                    });
            }
        },
    );
}
