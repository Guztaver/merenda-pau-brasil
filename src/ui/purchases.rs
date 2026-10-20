use super::components::*;
use crate::{
    app::App,
    db::purchasing::{CATEGORIES, METHODS},
    validation::{currency, money},
};
use eframe::egui;
pub fn receive_form(app: &mut App, ui: &mut egui::Ui) {
    card(
        ui,
        "Receber alimentos com rastreabilidade",
        "Registre a entrega real. Para notas com vários alimentos, lance uma linha por alimento.",
        |ui| {
            ui.columns(2, |c| {
                field(
                    &mut c[0],
                    "Data do recebimento (AAAA-MM-DD)",
                    &mut app.compliance.receipt.day,
                );
                choose(
                    &mut c[1],
                    "Fornecedor",
                    "receipt_supplier",
                    &mut app.compliance.receipt.supplier,
                    app.db.names("suppliers"),
                );
            });
            ui.columns(2, |c| {
                choose(
                    &mut c[0],
                    "Alimento",
                    "receipt_food",
                    &mut app.compliance.receipt.item,
                    app.db.names("items"),
                );
                options(
                    &mut c[1],
                    "Origem do recurso",
                    "funding",
                    &mut app.compliance.receipt.funding,
                    &["FNDE", "Próprios"],
                );
            });
            ui.columns(2, |c| {
                field(
                    &mut c[0],
                    "Nota fiscal / parcela de entrega",
                    &mut app.compliance.receipt.invoice,
                );
                field(
                    &mut c[1],
                    "Número do processo de compra",
                    &mut app.compliance.receipt.process,
                );
            });
            ui.columns(2, |c| {
                field(
                    &mut c[0],
                    "Contrato / instrumento equivalente",
                    &mut app.compliance.receipt.contract,
                );
                options(
                    &mut c[1],
                    "Modalidade da aquisição",
                    "method",
                    &mut app.compliance.receipt.method,
                    &METHODS,
                );
            });
            ui.columns(2, |c| {
                field(
                    &mut c[0],
                    "Identificação do lote",
                    &mut app.compliance.receipt.batch,
                );
                field(
                    &mut c[1],
                    "Validade (AAAA-MM-DD)",
                    &mut app.compliance.receipt.expiry,
                );
            });
            ui.columns(2, |c| {
                field(
                    &mut c[0],
                    "Peso recebido (g)",
                    &mut app.compliance.receipt_weight,
                );
                field(
                    &mut c[1],
                    "Valor desta linha da nota (R$)",
                    &mut app.compliance.receipt_value,
                );
            });
            options(
                ui,
                "Categoria do alimento",
                "receipt_class",
                &mut app.compliance.receipt.category,
                &CATEGORIES,
            );
            hint(ui,"Compras FNDE exigem a avaliação prévia do alimento pelo nutricionista. O recebimento não é contado como pagamento. Anexe o comprovante em Controle e auditoria usando o ID da operação.");
            if primary(ui, "Registrar recebimento e lote") {
                let r = (|| {
                    let f = &mut app.compliance;
                    f.receipt.grams = crate::validation::grams(&f.receipt_weight)?;
                    f.receipt.cents = money(&f.receipt_value)?;
                    app.db.receive(&f.receipt)
                })();
                if app.notify(r) {
                    app.compliance.receipt_weight.clear();
                    app.compliance.receipt_value.clear();
                    app.compliance.receipt.invoice.clear();
                }
            }
        },
    );
}
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        for (i, n) in [
            "Fornecedores",
            "Pagamentos e repasses",
            "Indicadores anuais",
        ]
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
            card(ui,"Cadastrar fornecedor","O CPF/CNPJ é verificado pelos dígitos. O CAF informado deve ser conferido com a documentação oficial.",|ui|{
                field(ui,"Nome / razão social",&mut app.compliance.supplier_name);field(ui,"CPF ou CNPJ",&mut app.compliance.supplier_document);
                ui.checkbox(&mut app.compliance.family,"Fornecedor da agricultura familiar");field(ui,"CAF declarado (obrigatório para agricultura familiar)",&mut app.compliance.caf);
                if primary(ui,"Cadastrar fornecedor"){let f=&app.compliance;let r=app.db.add_supplier(&f.supplier_name,&f.supplier_document,f.family,&f.caf);if app.notify(r){app.compliance.supplier_name.clear();app.compliance.supplier_document.clear();app.compliance.caf.clear();}}
                for(_,n)in app.db.names("suppliers"){ui.label(n);}
            }); ui.add_space(16.);
            card(ui,"Conferência documental da agricultura familiar","Restrinja a função a quem conferiu o documento oficial e a habilitação do fornecedor. O cadastro por si só não comprova elegibilidade.",|ui|{
                choose(ui,"Fornecedor familiar","family_verify_supplier",&mut app.compliance.supplier_verify_id,app.db.names("suppliers"));
                field(ui,"Referência do documento conferido (CAF/DAP válida, chamada pública, arquivo)",&mut app.compliance.supplier_verify_reference);
                multiline(ui,"Observação da conferência (não inclua dados pessoais desnecessários)",&mut app.compliance.supplier_verify_reason);
                if primary(ui,"Registrar conferência do administrador"){let f=&app.compliance;let r=app.db.verify_family_supplier(f.supplier_verify_id,&f.supplier_verify_reference,&f.supplier_verify_reason);app.notify(r);}
            });
        },
        1=>{
            card(ui,"Registrar pagamento efetivado","Acesso do administrador. Informe o valor efetivamente pago, sem superar o saldo da nota.",|ui|{
                choose(ui,"Recebimento / nota fiscal","payment_receipt",&mut app.compliance.payment_receipt,app.db.receipt_choices().unwrap_or_default());
                ui.columns(2,|c|{field(&mut c[0],"Data do pagamento (AAAA-MM-DD)",&mut app.compliance.payment_day);field(&mut c[1],"Valor pago (R$)",&mut app.compliance.payment_value);});
                field(ui,"Documento da transação / parcela",&mut app.compliance.payment_document);
                if primary(ui,"Registrar pagamento"){let f=&app.compliance;let r=money(&f.payment_value).and_then(|c|app.db.record_payment(&f.payment_day,f.payment_receipt,c,&f.payment_document));if app.notify(r){app.compliance.payment_value.clear();app.compliance.payment_document.clear();}}
            });ui.add_space(16.);
            card(ui,"Repasses e saldos reprogramados","Informe os recursos efetivamente creditados, conforme os extratos oficiais.",|ui|{
                ui.checkbox(&mut app.compliance.carryover,"Saldo reprogramado de exercício anterior (separado dos novos repasses)");
                ui.columns(2,|c|{field(&mut c[0],"Data do crédito (AAAA-MM-DD)",&mut app.compliance.resource_day);field(&mut c[1],"Valor creditado (R$)",&mut app.compliance.resource_value);});
                field(ui,"Referência / documento do crédito",&mut app.compliance.resource_document);
                if primary(ui,"Registrar recurso"){let f=&app.compliance;let r=money(&f.resource_value).and_then(|c|app.db.record_resource(&f.resource_day,c,&f.resource_document,if f.carryover{"Saldo reprogramado"}else{"Repasse FNDE"}));if app.notify(r){app.compliance.resource_value.clear();app.compliance.resource_document.clear();}}
            });
        },
        _=>card(ui,"Indicadores sobre os registros do exercício","Os percentuais usam os repasses FNDE do ano como base, nunca o total de compras como denominador.",|ui|{
            field(ui,"Ano de referência",&mut app.compliance.year);
            match app.db.compliance_report(&app.compliance.year){Err(e)=>hint(ui,&e),Ok(v)=>{
                ui.label(format!("Repasses FNDE: R$ {} • Reprogramados (fora da base): R$ {}",currency(v.federal_transfer_cents),currency(v.carryover_cents)));
                ui.label(format!("Pagamentos FNDE registrados: R$ {}",currency(v.federal_paid_cents)));
                if !v.sufficient_base { hint(ui,"Cadastre os repasses FNDE do exercício antes de interpretar os percentuais."); }
                for(label,value,target,min)in [("Agricultura familiar",v.family_percent,45.,true),("In natura / minimamente processados",v.natural_percent,85.,true),("Processados + ultraprocessados",v.processed_percent,10.,false),("Ingredientes culinários",v.culinary_percent,5.,false)]{
                    if let Some(pct)=value{badge(ui,&format!("{label}: {pct:.2}% • referência: {} {target:.0}%",if min{"mín."}else{"máx."}),if min{pct<target}else{pct>target});}else{hint(ui,&format!("{label}: sem base positiva de repasses registrada."));}
                }
                hint(ui,"Referências de 2026: agricultura familiar 45%; in natura 85%; processados/ultraprocessados até 10%; ingredientes culinários até 5%. Indicadores auxiliares, dependentes da classificação do RT e conciliação contábil. Não certificam regularidade de compras ou adequação de cardápios.");
                if ui.button("Exportar conferência de recursos e compras CSV").clicked(){let path=app.dir.join(format!("conferencia-pnae-{}-{}.csv",app.compliance.year,chrono::Utc::now().format("%Y%m%d-%H%M%S%f")));let r=app.db.compliance_csv(&app.compliance.year).and_then(|text|std::fs::write(&path,text).map_err(|e|e.to_string()));if app.notify(r){app.notice.as_mut().unwrap().text=format!("Salvo em {}",path.display());}}
            }}
        }),
    }
}
