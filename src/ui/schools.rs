use super::components::*;
use crate::app::App;
use eframe::egui;
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    card(
        ui,
        "Adicionar escola",
        "As unidades cadastradas ficam disponíveis no registro de refeições.",
        |ui| {
            field(ui, "Nome da unidade escolar", &mut app.school_name);
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
            for (_, name) in rows {
                ui.horizontal_wrapped(|ui| {
                    badge(ui, "Escola", false);
                    ui.strong(name);
                });
                ui.separator();
            }
        },
    );
}
