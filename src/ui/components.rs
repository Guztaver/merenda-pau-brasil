use super::theme::*;
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
    field_required(ui, label, value, false);
}
pub fn required_field(ui: &mut egui::Ui, label: &str, value: &mut String, show_errors: bool) {
    let error = (show_errors && value.trim().is_empty()).then_some("Preencha este campo.");
    field_error(ui, label, value, error);
}
pub fn field_required(ui: &mut egui::Ui, label: &str, value: &mut String, invalid: bool) {
    field_error(
        ui,
        label,
        value,
        invalid.then_some("Preencha este campo."),
    );
}
pub fn field_error(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    error: Option<&str>,
) {
    let invalid = error.is_some();
    ui.label(RichText::new(label).size(13.).strong().color(if invalid {
        Color32::from_rgb(176, 45, 45)
    } else {
        Color32::from_rgb(44, 53, 60)
    }));
    let response = egui::Frame::new()
        .stroke(if invalid {
            Stroke::new(1.5_f32, Color32::from_rgb(198, 55, 55))
        } else {
            Stroke::new(1.0_f32, Color32::from_rgb(218, 224, 228))
        })
        .corner_radius(8)
        .inner_margin(egui::Margin::same(2))
        .show(ui, |ui| {
            ui.add_sized(
                [(ui.available_width() - 4.).min(420.), 38.],
                egui::TextEdit::singleline(value).margin(egui::vec2(12., 9.)),
            )
        })
        .inner;
    if invalid {
        ui.colored_label(
            Color32::from_rgb(176, 45, 45),
            error.expect("invalid field has a validation message"),
        );
    }
    if response.changed() {
        let previous = value.clone();
        let cursor = egui::text_edit::TextEditState::load(ui.ctx(), response.id)
            .and_then(|state| state.cursor.char_range())
            .map(|range| range.primary.index.0)
            .unwrap_or_else(|| previous.chars().count());
        let digits_before = previous
            .chars()
            .take(cursor)
            .filter(char::is_ascii_digit)
            .count();
        let kind = label.to_lowercase();
        if kind.contains("r$")
            || kind.contains("valor pago")
            || kind.contains("valor creditado")
            || kind.contains("valor desta linha")
            || kind.contains("custo por porção")
        {
            *value = format_money_input(value);
        } else if kind.contains("dd/mm/aaaa")
            || kind.contains("data ")
            || kind.contains("validade")
            || kind.contains("data(")
        {
            *value = format_date_input(value);
        } else if [
            "quantidade",
            "peso",
            "gramas",
            "estoque mínimo",
            "porções",
            "rendimento",
        ]
        .iter()
        .any(|needle| kind.contains(needle))
        {
            *value = format_number_input(value);
        }
        if let Some(mut state) = egui::text_edit::TextEditState::load(ui.ctx(), response.id) {
            let position = if cursor >= previous.chars().count() {
                value.chars().count()
            } else {
                let mut seen = 0;
                value
                    .chars()
                    .enumerate()
                    .find_map(|(index, ch)| {
                        if ch.is_ascii_digit() {
                            seen += 1;
                        }
                        (seen >= digits_before && digits_before > 0).then_some(index + 1)
                    })
                    .unwrap_or_else(|| value.chars().count())
            };
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::one(
                    egui::text::CCursor::new(position),
                )));
            state.store(ui.ctx(), response.id);
        }
    }
}
pub fn choose_required(
    ui: &mut egui::Ui,
    label: &str,
    key: &str,
    id: &mut i64,
    rows: Vec<(i64, String)>,
    invalid: bool,
) {
    ui.label(RichText::new(label).size(13.).strong().color(if invalid {
        Color32::from_rgb(176, 45, 45)
    } else {
        Color32::from_rgb(44, 53, 60)
    }));
    let selected = rows
        .iter()
        .find(|x| x.0 == *id)
        .map(|x| x.1.as_str())
        .unwrap_or("Selecione uma opção");
    egui::Frame::new()
        .stroke(if invalid {
            Stroke::new(1.5_f32, Color32::from_rgb(198, 55, 55))
        } else {
            Stroke::new(1.0_f32, Color32::from_rgb(218, 224, 228))
        })
        .corner_radius(8)
        .inner_margin(egui::Margin::same(2))
        .show(ui, |ui| {
            egui::ComboBox::from_id_salt(key)
                .width((ui.available_width() - 4.).max(0.))
                .height(420.)
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    for (value, name) in &rows {
                        ui.selectable_value(id, *value, name);
                    }
                    if rows.is_empty() {
                        ui.label("Nenhum cadastro disponível.");
                    }
                });
        });
    if invalid {
        ui.colored_label(Color32::from_rgb(176, 45, 45), "Selecione uma opção.");
    }
}
pub fn required_choose(
    ui: &mut egui::Ui,
    label: &str,
    key: &str,
    id: &mut i64,
    rows: Vec<(i64, String)>,
    show_errors: bool,
) {
    let invalid = show_errors && *id == 0;
    choose_required(ui, label, key, id, rows, invalid);
}
fn only_digits(s: &str) -> String {
    s.chars().filter(char::is_ascii_digit).collect()
}
fn group_integer(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push('.')
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}
fn format_money_input(s: &str) -> String {
    let digits = only_digits(s);
    if digits.is_empty() {
        return String::new();
    }
    let padded = format!("{:0>3}", digits);
    let split = padded.len() - 2;
    format!(
        "R$ {},{}",
        group_integer(&padded[..split]),
        &padded[split..]
    )
}
fn format_number_input(s: &str) -> String {
    let mut pieces = s.splitn(2, ',');
    let whole = only_digits(pieces.next().unwrap_or(""));
    let decimals = pieces.next();
    let mut out = group_integer(&whole);
    if let Some(decimals) = decimals {
        out.push(',');
        out.extend(only_digits(decimals).chars().take(3));
    }
    out
}
fn format_date_input(s: &str) -> String {
    let d = only_digits(s).chars().take(8).collect::<String>();
    let mut out = String::new();
    for (i, ch) in d.chars().enumerate() {
        if i == 2 || i == 4 {
            out.push('/')
        }
        out.push(ch);
    }
    out
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
        .height(420.)
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
pub fn stock_table(ui: &mut egui::Ui, rows: Vec<(String, i64, i64, String, i64)>) {
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
                    for h in [
                        "ALIMENTO / MATERIAL",
                        "SALDO ATUAL",
                        "ESTOQUE MÍNIMO",
                        "SITUAÇÃO",
                    ] {
                        ui.label(RichText::new(h).size(11.).color(MUTED).strong());
                    }
                    ui.end_row();
                    for (n, g, m, unit, conversion) in rows {
                        ui.strong(n);
                        ui.label(format!("{} {}", display_quantity(g, conversion), unit));
                        ui.label(format!("{} {}", display_quantity(m, conversion), unit));
                        badge(ui, if g < m { "Repor estoque" } else { "Em dia" }, g < m);
                        ui.end_row();
                    }
                });
        });
}
pub fn display_quantity(grams: i64, conversion: i64) -> String {
    if conversion <= 0 {
        return format!("{grams} g");
    }
    format!("{:.3}", grams as f64 / conversion as f64)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .replace('.', ",")
}
pub fn pretty_date(s: &str) -> String {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .or_else(|_| chrono::NaiveDate::parse_from_str(s, "%d/%m/%Y"))
        .map(|d| d.format("%d/%m/%Y").to_string())
        .unwrap_or_else(|_| s.into())
}
pub fn options(ui: &mut egui::Ui, label: &str, key: &str, value: &mut String, choices: &[&str]) {
    ui.label(RichText::new(label).size(13.).strong());
    egui::ComboBox::from_id_salt(key)
        .width(ui.available_width())
        .selected_text(value.as_str())
        .show_ui(ui, |ui| {
            for choice in choices {
                ui.selectable_value(value, (*choice).to_string(), *choice);
            }
        });
}
pub fn multiline(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.label(RichText::new(label).size(13.).strong());
    ui.add(
        egui::TextEdit::multiline(value)
            .desired_width(ui.available_width())
            .desired_rows(3)
            .margin(egui::vec2(12., 10.)),
    );
}
pub fn required_multiline(ui: &mut egui::Ui, label: &str, value: &mut String, show_errors: bool) {
    let invalid = show_errors && value.trim().is_empty();
    ui.label(RichText::new(label).size(13.).strong().color(if invalid {
        Color32::from_rgb(176, 45, 45)
    } else {
        Color32::from_rgb(44, 53, 60)
    }));
    egui::Frame::new()
        .stroke(if invalid {
            Stroke::new(1.5_f32, Color32::from_rgb(198, 55, 55))
        } else {
            Stroke::new(1.0_f32, Color32::from_rgb(218, 224, 228))
        })
        .corner_radius(8)
        .inner_margin(egui::Margin::same(2))
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(value)
                    .desired_width(ui.available_width())
                    .desired_rows(3)
                    .margin(egui::vec2(12., 10.)),
            );
        });
    if invalid {
        ui.colored_label(Color32::from_rgb(176, 45, 45), "Preencha este campo.");
    }
}
pub fn password(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.label(RichText::new(label).size(13.).strong());
    ui.add(
        egui::TextEdit::singleline(value)
            .password(true)
            .desired_width(ui.available_width())
            .margin(egui::vec2(12., 10.)),
    );
}

#[cfg(test)]
mod input_format_tests {
    #[test]
    fn brazilian_masks_format_currency_quantity_and_date() {
        assert_eq!(super::format_money_input("100000"), "R$ 1.000,00");
        assert_eq!(super::format_number_input("5000000"), "5.000.000");
        assert_eq!(super::format_date_input("20102026"), "20/10/2026");
    }
}
