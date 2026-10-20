use super::{components::*, theme::*, *};
use crate::app::{App, Page};
use eframe::egui::{self, Color32, RichText, Stroke};

pub fn show(app: &mut App, ctx: &egui::Context) {
    egui::SidePanel::left("navigation")
        .resizable(false)
        .exact_width(242.)
        .frame(egui::Frame::new().fill(Color32::WHITE).inner_margin(20.))
        .show(ctx, |ui| {
            ui.add_space(12.);
            ui.horizontal(|ui| {
                egui::Frame::new()
                    .fill(GREEN)
                    .corner_radius(13)
                    .inner_margin(12.)
                    .show(ui, |ui| {
                        ui.label(RichText::new("M").size(25.).strong().color(Color32::WHITE));
                    });
                ui.vertical(|ui| {
                    ui.label(RichText::new("Merenda").size(24.).strong());
                    ui.label(RichText::new("PAU BRASIL • BA").size(10.).color(MUTED));
                });
            });
            ui.add_space(30.);
            ui.label(
                RichText::new("GESTÃO MUNICIPAL")
                    .size(10.)
                    .color(MUTED)
                    .strong(),
            );
            ui.add_space(8.);
            for page in Page::ALL {
                let selected = app.page == page;
                let button = egui::Button::new(
                    RichText::new(page.label())
                        .color(if selected { GREEN } else { MUTED })
                        .strong(),
                )
                .fill(if selected { PALE } else { Color32::TRANSPARENT })
                .stroke(Stroke::NONE)
                .corner_radius(11);
                if ui.add_sized([200., 44.], button).clicked() {
                    app.navigate(page);
                }
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                hint(ui, "Dados neste computador");
                badge(ui, "●  Funcionando offline", false);
                ui.add_space(8.);
            });
        });
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(BG).inner_margin(28.))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("main_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("ALIMENTAÇÃO ESCOLAR")
                                .size(11.)
                                .strong()
                                .color(GREEN),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            hint(ui, &chrono::Local::now().format("%d / %m / %Y").to_string());
                        });
                    });
                    ui.add_space(18.);
                    ui.heading(app.page.title());
                    hint(ui, app.page.description());
                    ui.add_space(16.);
                    if let Some(n) = &app.notice {
                        let mut close = false;
                        egui::Frame::new()
                            .fill(if n.error {
                                Color32::from_rgb(255, 239, 230)
                            } else {
                                PALE
                            })
                            .corner_radius(12)
                            .inner_margin(14.)
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(RichText::new(&n.text).color(if n.error {
                                        AMBER
                                    } else {
                                        GREEN
                                    }));
                                    if ui.small_button("Fechar").clicked() {
                                        close = true;
                                    }
                                });
                            });
                        if close {
                            app.notice = None;
                        }
                        ui.add_space(8.);
                    }
                    match app.page {
                        Page::Dashboard => dashboard::show(app, ui),
                        Page::Foods => foods::show(app, ui),
                        Page::Movements => movements::show(app, ui),
                        Page::Recipes => recipes::show(app, ui),
                        Page::Servings => servings::show(app, ui),
                        Page::Schools => schools::show(app, ui),
                        Page::Reports => reports::show(app, ui),
                    }
                    ui.add_space(24.);
                    hint(
                        ui,
                        "Prefeitura Municipal de Pau Brasil  /  Gestão da merenda escolar",
                    );
                });
        });
}
