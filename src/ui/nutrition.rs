use super::components::*;
use crate::{
    app::App,
    db::{nutrition::Nutrients, purchasing::CATEGORIES},
    validation::{grams, money},
};
use eframe::egui;
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        for (i, n) in ["Fichas técnicas", "Cardápios", "Classificar alimentos"]
            .iter()
            .enumerate()
        {
            if mode_option(ui, n, app.compliance.tab == i) {
                app.compliance.tab = i;
            }
        }
    });
    ui.add_space(8.);
    match app.compliance.tab {
        0=>{
            card(ui,"Aprovar ficha técnica por porção","A aprovação exige login do nutricionista com CRN. Cada versão preserva os ingredientes e a identificação do responsável.",|ui|{
                choose(ui,"Receita com ingredientes cadastrados","technical_recipe",&mut app.compliance.technical_recipe,app.db.names("recipes"));
                ui.columns(2,|c|{field(&mut c[0],"Etapa / faixa etária",&mut app.compliance.card.age_group);field(&mut c[1],"Dieta / restrições atendidas",&mut app.compliance.card.diet);});
                ui.columns(2,|c|{field(&mut c[0],"Rendimento por porção (g)",&mut app.compliance.yield_text);field(&mut c[1],"Custo por porção (R$)",&mut app.compliance.cost_text);});
                field(ui,"Consistência da preparação",&mut app.compliance.card.consistency);
                multiline(ui,"Modo de preparo",&mut app.compliance.card.preparation);multiline(ui,"Padrão de apresentação",&mut app.compliance.card.presentation);
                field(ui,"Fonte e versão dos dados nutricionais / memória de cálculo",&mut app.compliance.card.nutrient_source);
                let labels=["Energia (kcal)","Proteínas (g)","Carboidratos (g)","Lipídios (g)","Fibras (g)","Sódio (mg)","Ferro (mg)","Vitamina A (µg RAE)","Vitamina C (mg)","Cálcio (mg)"];
                for pair in 0..5 {ui.columns(2,|c|{field(&mut c[0],labels[pair*2],&mut app.compliance.nutrients[pair*2]);field(&mut c[1],labels[pair*2+1],&mut app.compliance.nutrients[pair*2+1]);});}
                hint(ui,"Todos os valores nutricionais são por porção preparada. Informe valores reais conferidos pelo nutricionista. O aplicativo não inventa dados nem certifica adequação das metas do cardápio.");
                if primary(ui,"Aprovar nova revisão da ficha"){
                    let r=(||{
                        let f=&mut app.compliance;f.card.yield_grams=grams(&f.yield_text)?;f.card.cost_cents=money(&f.cost_text)?;
                        let values=f.nutrients.iter().map(|s|s.trim().replace(',',".").parse::<f64>().map_err(|_|"Valor nutricional inválido.".to_string())).collect::<Result<Vec<_>,_>>()?;
                        f.card.nutrients=Nutrients{kcal:values[0],protein:values[1],carbs:values[2],fat:values[3],fiber:values[4],sodium:values[5],iron:values[6],vitamin_a:values[7],vitamin_c:values[8],calcium:values[9]};
                        app.db.approve_card(f.technical_recipe,f.card.clone()).map(|_|())
                    })();app.notify(r);
                }
            });ui.add_space(16.);
            card(ui,"Versões aprovadas","Exporte a ficha para conferência, impressão e assinatura pelo profissional responsável.",|ui|{
                choose(ui,"Ficha técnica","card_export",&mut app.compliance.selected_card,app.db.cards().unwrap_or_default());
                if ui.button("Exportar ficha para impressão (HTML)").clicked(){let path=app.dir.join(format!("ficha-{}-{}.html",app.compliance.selected_card,chrono::Utc::now().format("%Y%m%d-%H%M%S%f")));let r=app.db.export_card(app.compliance.selected_card,&path);if app.notify(r){app.notice.as_mut().unwrap().text=format!("Salvo em {}",path.display());}}
            });
        },
        1=>{
            card(ui,"Aprovar cardápio escolar","Planejamento por escola, data, horário/tipo de refeição e versão da ficha técnica. Permite datas futuras.",|ui|{
                ui.columns(2,|c|{choose(&mut c[0],"Escola","menu_school",&mut app.compliance.menu.school,app.db.names("schools"));field(&mut c[1],"Data do cardápio (AAAA-MM-DD)",&mut app.compliance.menu.day);});
                let before=app.compliance.menu.card;
                choose(ui,"Ficha técnica aprovada","menu_card",&mut app.compliance.menu.card,app.db.cards().unwrap_or_default());
                if before!=app.compliance.menu.card {if let Ok(c)=app.db.card(app.compliance.menu.card){app.compliance.menu.age_group=c.age_group;app.compliance.menu.diet=c.diet;}}
                hint(ui,&format!("Etapa/faixa etária: {} • Dieta: {}",app.compliance.menu.age_group,app.compliance.menu.diet));
                ui.columns(2,|c|{field(&mut c[0],"Horário e tipo (ex.: 11h30 — Almoço)",&mut app.compliance.menu.meal);field(&mut c[1],"Porções planejadas",&mut app.compliance.menu_count);});
                field(ui,"ID atual a substituir (somente revisão; deixe vazio no primeiro registro)",&mut app.compliance.menu_replaces);
                if primary(ui,"Aprovar cardápio / revisão"){
                    let r=(||{let f=&mut app.compliance;f.menu.planned=f.menu_count.parse().map_err(|_|"Informe porções inteiras.".to_string())?;f.menu.supersedes=if f.menu_replaces.trim().is_empty(){None}else{Some(f.menu_replaces.parse().map_err(|_|"ID de revisão inválido.".to_string())?)};app.db.approve_menu(&f.menu)})();app.notify(r);
                }
            });ui.add_space(16.);
            card(ui,"Cardápios para conferência e divulgação","O arquivo inclui ingredientes, valores nutricionais, dieta e identificação do responsável. A publicação externa continua sob responsabilidade da prefeitura.",|ui|{
                ui.columns(2,|c|{field(&mut c[0],"Início (AAAA-MM-DD)",&mut app.compliance.menu_start);field(&mut c[1],"Fim (AAAA-MM-DD)",&mut app.compliance.menu_end);});
                match app.db.menus(&app.compliance.menu_start,&app.compliance.menu_end){Ok(rows)=>{for(id,n)in rows{ui.label(format!("#{id} • {n}"));}},Err(e)=>hint(ui,&e)}
                if ui.button("Exportar cardápios da escola selecionada (HTML)").clicked(){let path=app.dir.join(format!("cardapios-{}.html",chrono::Utc::now().format("%Y%m%d-%H%M%S%f")));let f=&app.compliance;let r=app.db.export_menus(f.menu.school,&f.menu_start,&f.menu_end,&path);if app.notify(r){app.notice.as_mut().unwrap().text=format!("Salvo em {}",path.display());}}
            });
        },
        _=>card(ui,"Avaliação técnica dos alimentos","O nutricionista classifica cada alimento e registra se a compra com recursos FNDE é admissível. Conferir proibições e limites do PNAE antes de autorizar.",|ui|{
            choose(ui,"Alimento","assess_food",&mut app.compliance.food,app.db.names("items"));options(ui,"Classificação","assess_category",&mut app.compliance.food_category,&CATEGORIES);
            ui.checkbox(&mut app.compliance.food_allowed,"Conferi as regras vigentes e autorizo a aquisição com recursos FNDE");
            multiline(ui,"Justificativa técnica e referência documental",&mut app.compliance.food_reason);
            if primary(ui,"Registrar avaliação do nutricionista"){let f=&app.compliance;let r=app.db.assess_food(f.food,&f.food_category,f.food_allowed,&f.food_reason);app.notify(r);}
            if ui.button("Exportar avaliações vigentes (CSV)").clicked(){let path=app.dir.join(format!("avaliacao-alimentos-{}.csv",chrono::Utc::now().format("%Y%m%d-%H%M%S%f")));let r=app.db.assess_foods_csv().and_then(|text|std::fs::write(&path,text).map_err(|e|e.to_string()));if app.notify(r){app.notice.as_mut().unwrap().text=format!("Salvo em {}",path.display());}}
            hint(ui,"A lista apresenta a última avaliação por alimento e não substitui conferência da legislação vigente, dos limites por aquisição e da documentação do processo.");
        }),
    }
}
