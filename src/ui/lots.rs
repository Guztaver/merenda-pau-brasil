use super::components::*;
use crate::{app::App, validation::measured_grams};
use eframe::egui;
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        for (i, n) in ["Lotes e transferências", "Regularizar legado"]
            .iter()
            .enumerate()
        {
            if mode_option(ui, n, app.compliance.tab == i) {
                app.compliance.tab = i;
            }
        }
    });
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if app.compliance.tab == 0 {
        card(ui,"Transferir entre o estoque central e as escolas","A transferência preserva a origem, o lote e a validade. Não é contabilizada como nova compra.",|ui|{
            field(ui,"Data da transferência (DD/MM/AAAA)",&mut app.compliance.transfer_day);
            let transfer_lots=app.db.lots(&app.compliance.transfer_day);
            match &transfer_lots{Err(e)=>hint(ui,e),Ok(lots)=>choose(ui,"Lote de origem","transfer_lot",&mut app.compliance.lot,lots.iter().filter(|l|l.balance>0&&!l.legacy).map(|l|(l.id,format!("{} • {} • {} {} • vence {}",l.label,l.location,display_quantity(l.balance,l.unit_grams),l.unit,pretty_date(&l.expiry)))).collect())}
            let measure=transfer_lots.as_ref().ok().and_then(|lots|lots.iter().find(|l|l.id==app.compliance.lot)).map(|l|(l.unit.clone(),l.unit_grams)).unwrap_or_else(||("g".into(),1));
            let mut schools=vec![(0,"Estoque central".to_string())];schools.extend(app.db.names("schools"));
            choose(ui,"Destino","transfer_to",&mut app.compliance.transfer_school,schools);
            ui.columns(2,|c|{field(&mut c[0],&format!("Quantidade a transferir ({})",measure.0),&mut app.compliance.transfer_weight);field(&mut c[1],"Guia / justificativa de transferência",&mut app.compliance.transfer_note);});
            if primary(ui,"Registrar transferência"){
                let f=&app.compliance;let r=measured_grams(&f.transfer_weight,measure.1).and_then(|g|app.db.transfer(&f.transfer_day,f.lot,g,f.transfer_school,&f.transfer_note));if app.notify(r){app.compliance.transfer_weight.clear();app.compliance.transfer_note.clear();}
            }
        });
        ui.add_space(16.);
        card(ui,"Lotes no estoque de hoje","Saldos físicos por local. Lotes vencidos permanecem visíveis para conferência e descarte justificado.",|ui|{
            match app.db.lots(&today){Err(e)=>hint(ui,&e),Ok(lots)=>{
                if lots.is_empty(){empty(ui,"Nenhum lote registrado","Receba os alimentos em Entradas e perdas.");}
                let rows=lots.into_iter().filter(|l|l.balance!=0).collect::<Vec<_>>();
                if !rows.is_empty(){egui::ScrollArea::horizontal().id_salt("lots_table_scroll").show(ui,|ui|{egui::Grid::new("lots_table").min_col_width(100.).spacing([18.,10.]).striped(true).show(ui,|ui|{
                    for h in ["ALIMENTO / LOTE","SALDO","LOCAL","VALIDADE","SITUAÇÃO"]{ui.label(egui::RichText::new(h).size(11.).strong());}ui.end_row();
                    for l in rows{ui.strong(l.label);ui.label(format!("{} {}",display_quantity(l.balance,l.unit_grams),l.unit));ui.label(l.location);ui.label(if l.expiry.is_empty(){"—".into()}else{pretty_date(&l.expiry)});badge(ui,if l.legacy{"Legado: conferir"}else if l.expiry<today{"Vencido — bloqueado"}else{"Rastreável"},l.legacy||l.expiry<today);ui.end_row();}
                });});}
            }}
            if ui.button("Exportar rastreabilidade CSV").clicked(){let path=app.dir.join(format!("rastreabilidade-{}.csv",chrono::Utc::now().format("%Y%m%d-%H%M%S%f")));let r=app.db.traceability_csv().and_then(|s|std::fs::write(&path,s).map_err(|e|e.to_string()));if app.notify(r){app.notice.as_mut().unwrap().text=format!("Salvo em {}",path.display());}}
        });
    } else {
        card(ui,"Conferência de estoque anterior à atualização","Somente o administrador. Não altera pesos, datas ou valores históricos. Registra os dados realmente conferidos, sem criar pagamento ou aquisição fictícia.",|ui|{
            let rows=app.db.lots(&today).unwrap_or_default().into_iter().filter(|l|l.legacy).map(|l|(l.id,l.label)).collect();
            choose(ui,"Lote legado","legacy_lot",&mut app.compliance.lot,rows);choose(ui,"Fornecedor comprovado","legacy_supplier",&mut app.compliance.legacy_supplier,app.db.names("suppliers"));
            field(ui,"Nota / documento histórico de origem",&mut app.compliance.legacy_invoice);
            ui.columns(2,|c|{field(&mut c[0],"Lote conferido",&mut app.compliance.legacy_batch);field(&mut c[1],"Validade conferida (DD/MM/AAAA)",&mut app.compliance.legacy_expiry);});
            multiline(ui,"Como e com quais documentos o estoque foi conferido",&mut app.compliance.legacy_reason);
            if primary(ui,"Confirmar conferência do legado"){
                let f=&app.compliance;let r=app.db.regularize_legacy(f.lot,f.legacy_supplier,&f.legacy_invoice,&f.legacy_batch,&f.legacy_expiry,&f.legacy_reason);if app.notify(r){app.compliance.legacy_reason.clear();}
            }
        });
    }
}
