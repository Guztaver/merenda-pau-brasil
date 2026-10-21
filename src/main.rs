#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod db;
mod forms;
mod ui;
mod validation;
use eframe::egui;
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?)
        .join("MerendaPauBrasil");
    std::fs::create_dir_all(&dir)?;
    let db = db::Db::open(&dir.join("merenda.sqlite")).map_err(std::io::Error::other)?;
    let app = app::App::new(db, dir);
    eframe::run_native(
        "Merenda • Prefeitura de Pau Brasil",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1240., 860.])
                .with_min_inner_size([960., 700.]),
            ..Default::default()
        },
        Box::new(move |cc| {
            ui::theme::apply(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )?;
    Ok(())
}
