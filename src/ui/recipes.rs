use super::components::*;
use crate::{app::App, validation::grams};
use eframe::egui;
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let show_errors = app.notice.as_ref().is_some_and(|notice| notice.error);
    card(
        ui,
        "Criar uma receita",
        "Dê um nome à preparação antes de incluir os ingredientes.",
        |ui| {
            required_field(ui, "Nome da receita", &mut app.recipes.name, show_errors);
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
            required_choose(
                ui,
                "Receita",
                "recipe_editor",
                &mut app.recipes.recipe,
                app.db.names("recipes"),
                show_errors,
            );
            ui.columns(2, |cols| {
                required_choose(
                    &mut cols[0],
                    "Ingrediente",
                    "recipe_food",
                    &mut app.recipes.item,
                    app.db.food_choices(),
                    show_errors,
                );
                required_field(
                    &mut cols[1],
                    "Peso por porção (g)",
                    &mut app.recipes.weight,
                    show_errors,
                );
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
                    if !rows.is_empty() {
                        egui::ScrollArea::horizontal()
                            .id_salt("recipe_ingredients_scroll")
                            .show(ui, |ui| {
                                egui::Grid::new("recipe_ingredients")
                                    .min_col_width(110.)
                                    .spacing([22., 12.])
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for h in ["ALIMENTO", "PESO POR PORÇÃO", "AÇÃO"] {
                                            ui.label(egui::RichText::new(h).size(11.).strong());
                                        }
                                        ui.end_row();
                                        for r in rows {
                                            ui.strong(r.name);
                                            ui.label(format!("{} g", r.grams));
                                            if ui.small_button("Remover").clicked() {
                                                let result = app
                                                    .db
                                                    .remove_ingredient(app.recipes.recipe, r.id);
                                                app.notify(result);
                                            }
                                            ui.end_row();
                                        }
                                    });
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
