use eframe::egui::{self, Color32, CornerRadius, FontId, Stroke, TextStyle};
pub const GREEN: Color32 = Color32::from_rgb(27, 112, 86);
pub const INK: Color32 = Color32::from_rgb(35, 53, 48);
pub const MUTED: Color32 = Color32::from_rgb(112, 127, 120);
pub const BG: Color32 = Color32::from_rgb(244, 247, 244);
pub const LINE: Color32 = Color32::from_rgb(229, 236, 230);
pub const PALE: Color32 = Color32::from_rgb(232, 244, 236);
pub const AMBER: Color32 = Color32::from_rgb(163, 99, 29);
pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::light();
    style.visuals.override_text_color = Some(INK);
    style.visuals.panel_fill = BG;
    style.visuals.window_fill = Color32::WHITE;
    style.visuals.extreme_bg_color = Color32::from_rgb(248, 250, 248);
    style.visuals.selection.bg_fill = PALE;
    style.visuals.selection.stroke = Stroke::new(1.0_f32, INK);
    for w in [
        &mut style.visuals.widgets.noninteractive,
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(9);
        w.fg_stroke = Stroke::new(1.0_f32, INK);
    }
    style.visuals.widgets.inactive.bg_fill = Color32::WHITE;
    style.visuals.widgets.inactive.weak_bg_fill = Color32::WHITE;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, LINE);
    style.visuals.widgets.hovered.bg_fill = PALE;
    style.visuals.widgets.hovered.weak_bg_fill = PALE;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, GREEN);
    style.visuals.widgets.active.bg_fill = PALE;
    style.visuals.widgets.active.weak_bg_fill = PALE;
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, GREEN);
    style.spacing.item_spacing = egui::vec2(12., 12.);
    style.spacing.button_padding = egui::vec2(18., 11.);
    style.spacing.interact_size = egui::vec2(44., 40.);
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(28.));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(15.));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(14.));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(12.));
    ctx.set_style(style);
}
