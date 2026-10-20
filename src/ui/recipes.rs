use super::components::*;
use crate::{app::App, validation::grams};
use eframe::egui;
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    card(
        ui,
        "Criar uma receita",
        "Dê um nome à preparação antes de incluir os ingredientes.",
        |ui| {
            field(ui, "Nome da receita", &mut app.recipes.name);
            if primary(ui, "+  Criar receita") {
                let r = app.db.add("recipes", &app.recipes.name);
                if app.notify(r) {
                    app.recipes.name.clear();
                }
            }
        },
    );
    ui.add_space(18.);
    card(
        ui,
        "Ingredientes por porção",
        "Use o peso do ingrediente como ele sai do estoque, antes do preparo.",
        |ui| {
            choose(
                ui,
                "Receita",
                "recipe_editor",
                &mut app.recipes.recipe,
                app.db.names("recipes"),
            );
            ui.columns(2, |cols| {
                choose(
                    &mut cols[0],
                    "Ingrediente",
                    "recipe_food",
                    &mut app.recipes.item,
                    app.db.names("items"),
                );
                field(&mut cols[1], "Peso por porção (g)", &mut app.recipes.weight);
            });
            if primary(ui, "Adicionar / atualizar ingrediente") {
                let f = &app.recipes;
                let r = grams(&f.weight).and_then(|g| app.db.save_ingredient(f.recipe, f.item, g));
                if app.notify(r) {
                    app.recipes.weight.clear();
                }
            }
            ui.add_space(12.);
            match app.db.ingredients(app.recipes.recipe) {
                Err(e) => hint(ui, &e),
                Ok(rows) => {
                    if rows.is_empty() {
                        empty(ui,"Monte sua receita","Selecione uma receita e adicione os alimentos que compõem cada porção.");
                    }
                    for r in rows {
                        ui.horizontal_wrapped(|ui| {
                            ui.strong(r.name);
                            badge(ui, &format!("{} g por porção", r.grams), false);
                            if ui.small_button("Remover").clicked() {
                                let result = app.db.remove_ingredient(app.recipes.recipe, r.id);
                                app.notify(result);
                            }
                        });
                    }
                }
            }
            hint(
                ui,
                "Alterar a receita não modifica o consumo das refeições já registradas.",
            );
        },
    );
}
