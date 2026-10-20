use super::components::*;
use crate::{
    app::App,
    validation::{grams, kg},
};
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
            field(ui,"Data da transferência (AAAA-MM-DD)",&mut app.compliance.transfer_day);
            match app.db.lots(&app.compliance.transfer_day){Err(e)=>hint(ui,&e),Ok(lots)=>choose(ui,"Lote de origem","transfer_lot",&mut app.compliance.lot,lots.iter().filter(|l|l.balance>0&&!l.legacy).map(|l|(l.id,format!("{} • {} • {} kg • vence {}",l.label,l.location,kg(l.balance),l.expiry))).collect())}
            let mut schools=vec![(0,"Estoque central".to_string())];schools.extend(app.db.names("schools"));
            choose(ui,"Destino","transfer_to",&mut app.compliance.transfer_school,schools);
            ui.columns(2,|c|{field(&mut c[0],"Peso a transferir (g)",&mut app.compliance.transfer_weight);field(&mut c[1],"Guia / justificativa de transferência",&mut app.compliance.transfer_note);});
            if primary(ui,"Registrar transferência"){
                let f=&app.compliance;let r=grams(&f.transfer_weight).and_then(|g|app.db.transfer(&f.transfer_day,f.lot,g,f.transfer_school,&f.transfer_note));if app.notify(r){app.compliance.transfer_weight.clear();app.compliance.transfer_note.clear();}
            }
        });
        ui.add_space(16.);
        card(ui,"Lotes no estoque de hoje","Saldos físicos por local. Lotes vencidos permanecem visíveis para conferência e descarte justificado.",|ui|{
            match app.db.lots(&today){Err(e)=>hint(ui,&e),Ok(lots)=>{
                if lots.is_empty(){empty(ui,"Nenhum lote registrado","Receba os alimentos em Entradas e perdas.");}
                for l in lots.into_iter().filter(|l|l.balance!=0){
                    ui.strong(l.label);ui.label(format!("{} kg • Local: {} • Validade: {}",kg(l.balance),l.location,if l.expiry.is_empty(){"não informada"}else{&l.expiry}));
                    badge(ui,if l.legacy{"Legado: conferência obrigatória"}else if l.expiry<today{"Vencido — bloqueado para refeições"}else{"Rastreável — conferir condições sanitárias"},l.legacy||l.expiry<today);ui.separator();
                }
            }}
            if ui.button("Exportar rastreabilidade CSV").clicked(){let path=app.dir.join(format!("rastreabilidade-{}.csv",chrono::Utc::now().format("%Y%m%d-%H%M%S%f")));let r=app.db.traceability_csv().and_then(|s|std::fs::write(&path,s).map_err(|e|e.to_string()));if app.notify(r){app.notice.as_mut().unwrap().text=format!("Salvo em {}",path.display());}}
        });
    } else {
        card(ui,"Conferência de estoque anterior à atualização","Somente o administrador. Não altera pesos, datas ou valores históricos. Registra os dados realmente conferidos, sem criar pagamento ou aquisição fictícia.",|ui|{
            let rows=app.db.lots(&today).unwrap_or_default().into_iter().filter(|l|l.legacy).map(|l|(l.id,l.label)).collect();
            choose(ui,"Lote legado","legacy_lot",&mut app.compliance.lot,rows);choose(ui,"Fornecedor comprovado","legacy_supplier",&mut app.compliance.legacy_supplier,app.db.names("suppliers"));
            field(ui,"Nota / documento histórico de origem",&mut app.compliance.legacy_invoice);
            ui.columns(2,|c|{field(&mut c[0],"Lote conferido",&mut app.compliance.legacy_batch);field(&mut c[1],"Validade conferida (AAAA-MM-DD)",&mut app.compliance.legacy_expiry);});
            multiline(ui,"Como e com quais documentos o estoque foi conferido",&mut app.compliance.legacy_reason);
            if primary(ui,"Confirmar conferência do legado"){
                let f=&app.compliance;let r=app.db.regularize_legacy(f.lot,f.legacy_supplier,&f.legacy_invoice,&f.legacy_batch,&f.legacy_expiry,&f.legacy_reason);if app.notify(r){app.compliance.legacy_reason.clear();}
            }
        });
    }
}
