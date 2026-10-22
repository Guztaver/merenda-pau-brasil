#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod db;
mod forms;
mod ui;
mod validation;
use eframe::egui;
use std::sync::Arc;
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?)
        .join("MerendaPauBrasil");
    std::fs::create_dir_all(&dir)?;
    let db = db::Db::open(&dir.join("merenda.sqlite")).map_err(std::io::Error::other)?;
    let app = app::App::new(db, dir);
    let mut options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1240., 860.])
            .with_min_inner_size([960., 700.]),
        ..Default::default()
    };
    // WGPU's default request may reject legacy or software adapters. Try every
    // adapter exposed by Windows, preferring one compatible with our window.
    options.wgpu_options.wgpu_setup = egui_wgpu::WgpuSetupCreateNew {
        native_adapter_selector: Some(Arc::new(|adapters, surface| {
            adapters
                .iter()
                .find(|adapter| {
                    surface.is_none_or(|surface| {
                        adapter.is_surface_supported(surface)
                    })
                })
                .cloned()
                .ok_or_else(|| {
                    "Nenhum adaptador gráfico compatível foi encontrado. Atualize o driver de vídeo do Windows e tente novamente.".to_owned()
                })
        })),
        ..Default::default()
    }
    .into();
    eframe::run_native(
        "Merenda • Prefeitura de Pau Brasil",
        options,
        Box::new(move |cc| {
            ui::theme::apply(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )?;
    Ok(())
}
