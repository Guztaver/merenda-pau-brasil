use super::theme::*;
use crate::validation::kg;
use eframe::egui::{self, Color32, RichText, Stroke};

pub fn card(
    ui: &mut egui::Ui,
    title: &str,
    description: &str,
    content: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::new()
        .fill(Color32::WHITE)
        .corner_radius(16)
        .stroke(Stroke::new(1.0_f32, LINE))
        .inner_margin(22.)
        .show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 2.).max(0.));
            if !title.is_empty() {
                ui.label(RichText::new(title).size(18.).strong());
            }
            if !description.is_empty() {
                ui.label(RichText::new(description).size(13.).color(MUTED));
            }
            ui.add_space(5.);
            content(ui);
        });
}
pub fn field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.label(RichText::new(label).size(13.).strong());
    ui.add_sized(
        [ui.available_width().min(420.), 40.],
        egui::TextEdit::singleline(value).margin(egui::vec2(12., 10.)),
    );
}
pub fn choose(ui: &mut egui::Ui, label: &str, key: &str, id: &mut i64, rows: Vec<(i64, String)>) {
    ui.label(RichText::new(label).size(13.).strong());
    let selected = rows
        .iter()
        .find(|x| x.0 == *id)
        .map(|x| x.1.as_str())
        .unwrap_or("Selecione uma opção");
    egui::ComboBox::from_id_salt(key)
        .width(ui.available_width())
        .height(240.)
        .selected_text(selected)
        .show_ui(ui, |ui| {
            for (i, n) in &rows {
                ui.selectable_value(id, *i, n);
            }
            if rows.is_empty() {
                ui.label("Nenhum cadastro disponível.");
            }
        });
}
pub fn primary(ui: &mut egui::Ui, text: &str) -> bool {
    ui.add(
        egui::Button::new(RichText::new(text).color(Color32::WHITE).strong())
            .fill(GREEN)
            .stroke(Stroke::NONE)
            .corner_radius(10)
            .min_size(egui::vec2(140., 42.)),
    )
    .clicked()
}
/// Explicit colors keep segmented controls legible regardless of selection visuals.
pub fn mode_option(ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
    let (background, foreground) = if selected {
        (GREEN, Color32::WHITE)
    } else {
        (Color32::WHITE, INK)
    };
    ui.add(
        egui::Button::new(RichText::new(label).color(foreground).strong())
            .fill(background)
            .stroke(Stroke::new(1.0_f32, if selected { GREEN } else { LINE }))
            .corner_radius(10)
            .min_size(egui::vec2(180., 42.)),
    )
    .clicked()
}
pub fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(13.).color(MUTED));
}
pub fn empty(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.add_space(16.);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(title).size(17.).strong());
        hint(ui, description);
    });
    ui.add_space(16.);
}
pub fn badge(ui: &mut egui::Ui, text: &str, warning: bool) {
    egui::Frame::new()
        .fill(if warning {
            Color32::from_rgb(255, 244, 222)
        } else {
            PALE
        })
        .corner_radius(20)
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .size(12.)
                    .color(if warning { AMBER } else { GREEN }),
            );
        });
}
pub fn stock_table(ui: &mut egui::Ui, rows: Vec<(String, i64, i64)>) {
    if rows.is_empty() {
        empty(
            ui,
            "Seu estoque começa aqui",
            "Cadastre alimentos para acompanhar os saldos e receber alertas de reposição.",
        );
        return;
    }
    egui::ScrollArea::horizontal()
        .id_salt("stock_scroll")
        .show(ui, |ui| {
            egui::Grid::new("stock_grid")
                .min_col_width(140.)
                .spacing([30., 12.])
                .show(ui, |ui| {
                    for h in ["ALIMENTO", "SALDO ATUAL", "ESTOQUE MÍNIMO", "SITUAÇÃO"] {
                        ui.label(RichText::new(h).size(11.).color(MUTED).strong());
                    }
                    ui.end_row();
                    for (n, g, m) in rows {
                        ui.strong(n);
                        ui.label(format!("{} kg", kg(g)));
                        ui.label(format!("{} kg", kg(m)));
                        badge(ui, if g < m { "Repor estoque" } else { "Em dia" }, g < m);
                        ui.end_row();
                    }
                });
        });
}
pub fn pretty_date(s: &str) -> String {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(|d| d.format("%d/%m/%Y").to_string())
        .unwrap_or_else(|_| s.into())
}
