use super::components::*;
use crate::app::{App, Notice};
use chrono::Local;
use eframe::egui;
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    card(
        ui,
        "Relatório por período",
        "Exporte saldos, entradas, saídas e refeições por escola em formato CSV.",
        |ui| {
            ui.columns(2, |cols| {
                field(
                    &mut cols[0],
                    "Data inicial (AAAA-MM-DD)",
                    &mut app.reports.start,
                );
                field(
                    &mut cols[1],
                    "Data final (AAAA-MM-DD)",
                    &mut app.reports.end,
                );
            });
            if primary(ui, "Exportar relatório CSV") {
                let r = app
                    .db
                    .report(&app.reports.start, &app.reports.end)
                    .and_then(|text| {
                        let p = app.dir.join(format!(
                            "relatorio-{}-{}-{}.csv",
                            app.reports.start,
                            app.reports.end,
                            Local::now().format("%Y%m%d-%H%M%S%3f")
                        ));
                        std::fs::write(&p, text).map_err(|e| e.to_string())?;
                        Ok(p)
                    });
                app.notice = Some(match r {
                    Ok(p) => Notice {
                        text: format!("Relatório salvo em {}", p.display()),
                        error: false,
                    },
                    Err(e) => Notice {
                        text: e,
                        error: true,
                    },
                });
            }
            hint(ui,"As datas inicial e final são incluídas. O relatório auxilia a conferência e não substitui formulários oficiais.");
        },
    );
    ui.add_space(18.);
    card(
        ui,
        "Seus dados, bem guardados",
        "Crie cópias regularmente e guarde também em outro dispositivo.",
        |ui| {
            ui.horizontal_wrapped(|ui| {
                if primary(ui, "Criar cópia de segurança") {
                    let p = app.dir.join(format!(
                        "backup-{}.sqlite",
                        Local::now().format("%Y%m%d-%H%M%S%3f")
                    ));
                    app.notice = Some(match app.db.backup(&p) {
                        Ok(()) => Notice {
                            text: format!("Backup criado em {}", p.display()),
                            error: false,
                        },
                        Err(e) => Notice {
                            text: e,
                            error: true,
                        },
                    });
                }
                if ui.button("Abrir pasta dos dados").clicked() {
                    if let Err(e) = std::process::Command::new("explorer.exe")
                        .arg(&app.dir)
                        .spawn()
                    {
                        app.notify(Err(e.to_string()));
                    }
                }
            });
            hint(ui, &format!("Pasta dos dados: {}", app.dir.display()));
            hint(
                ui,
                "Use o banco em um computador. Evite pastas de rede ou sincronização.",
            );
        },
    );
}
