use super::components::*;
use crate::{
    app::App,
    db::purchasing::{document_valid, CATEGORIES, METHODS},
    validation::{currency, money},
};
use eframe::egui;
pub fn receive_form(app: &mut App, ui: &mut egui::Ui) {
    let show_errors = app.notice.as_ref().is_some_and(|notice| notice.error);
    let measure = app
        .db
        .item_measure(app.compliance.receipt.item)
        .unwrap_or_else(|| ("g".into(), 1));
    let received_day = crate::validation::iso_date(&app.compliance.receipt.day);
    let expiry_day = crate::validation::iso_date(&app.compliance.receipt.expiry);
    let day_error = show_errors.then(|| received_day.as_ref().err().map(String::as_str)).flatten();
    let expiry_error = if show_errors {
        match (&received_day, &expiry_day) {
            (_, Err(error)) => Some(error.as_str()),
            (Ok(day), Ok(expiry)) if expiry < day => {
                Some("A validade não pode ser anterior ao recebimento.")
            }
            _ => None,
        }
    } else {
        None
    };
    let weight_error = if show_errors {
        crate::validation::measured_grams(&app.compliance.receipt_weight, measure.1)
            .err()
    } else {
        None
    };
    let value_error = if show_errors {
        money(&app.compliance.receipt_value).err()
    } else {
        None
    };
    card(
        ui,
        "Receber alimentos com rastreabilidade",
        "Registre a entrega real. Para notas com vários alimentos, lance uma linha por alimento.",
        |ui| {
            ui.columns(2, |c| {
                field_error(
                    &mut c[0],
                    "Data do recebimento (DD/MM/AAAA)",
                    &mut app.compliance.receipt.day,
                    day_error,
                );
                required_choose(
                    &mut c[1],
                    "Fornecedor",
                    "receipt_supplier",
                    &mut app.compliance.receipt.supplier,
                    app.db.supplier_choices().unwrap_or_default(),
                    show_errors,
                );
            });
            ui.columns(2, |c| {
                required_choose(
                    &mut c[0],
                    "Alimento",
                    "receipt_food",
                    &mut app.compliance.receipt.item,
                    app.db.food_choices(),
                    show_errors,
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
                required_field(
                    &mut c[0],
                    "Nota fiscal / parcela de entrega",
                    &mut app.compliance.receipt.invoice,
                    show_errors,
                );
                required_field(
                    &mut c[1],
                    "Número do processo de compra",
                    &mut app.compliance.receipt.process,
                    show_errors,
                );
            });
            ui.columns(2, |c| {
                required_field(
                    &mut c[0],
                    "Contrato / instrumento equivalente",
                    &mut app.compliance.receipt.contract,
                    show_errors,
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
                required_field(
                    &mut c[0],
                    "Identificação do lote",
                    &mut app.compliance.receipt.batch,
                    show_errors,
                );
                field_error(
                    &mut c[1],
                    "Validade (DD/MM/AAAA)",
                    &mut app.compliance.receipt.expiry,
                    expiry_error,
                );
            });
            ui.columns(2, |c| {
                field_error(
                    &mut c[0],
                    &format!("Quantidade recebida ({})", measure.0),
                    &mut app.compliance.receipt_weight,
                    weight_error.as_deref(),
                );
                field_error(
                    &mut c[1],
                    "Valor desta linha da nota (R$)",
                    &mut app.compliance.receipt_value,
                    value_error.as_deref(),
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
                    f.receipt.grams =
                        crate::validation::measured_grams(&f.receipt_weight, measure.1)?;
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
    ui.add_space(12.);
    ui.horizontal_wrapped(|ui| {
        hint(ui, "Faltou um cadastro?");
        if ui.button("+ Cadastrar alimento / material").clicked() {
            app.navigate(crate::app::Page::Foods);
        }
        if ui.button("+ Cadastrar fornecedor").clicked() {
            app.compliance.tab = 0;
            app.navigate(crate::app::Page::Purchases);
        }
        if ui.button("Consultar lotes e validades").clicked() {
            app.navigate(crate::app::Page::Lots);
        }
    });
}
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let show_errors = app.notice.as_ref().is_some_and(|notice| notice.error);
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
                let document_invalid=show_errors&&!document_valid(&app.compliance.supplier_document);
                required_field(ui,"Nome / razão social",&mut app.compliance.supplier_name,show_errors);field_required(ui,"CPF ou CNPJ",&mut app.compliance.supplier_document,document_invalid);
                ui.checkbox(&mut app.compliance.family,"Fornecedor da agricultura familiar");let caf_invalid=show_errors&&app.compliance.family&&app.compliance.caf.trim().is_empty();field_required(ui,"CAF declarado (obrigatório para agricultura familiar)",&mut app.compliance.caf,caf_invalid);
                if primary(ui,"Cadastrar fornecedor"){let f=&app.compliance;let r=app.db.add_supplier(&f.supplier_name,&f.supplier_document,f.family,&f.caf);if app.notify(r){app.compliance.supplier_name.clear();app.compliance.supplier_document.clear();app.compliance.caf.clear();}}
            }); ui.add_space(16.);
            card(ui,"Fornecedores cadastrados","Consulte os dados e selecione um cadastro para editar. Compras e documentos vinculados impedem a exclusão para preservar o histórico.",|ui|{
                match app.db.suppliers() {
                    Err(e) => hint(ui,&e),
                    Ok(rows) if rows.is_empty() => empty(ui,"Nenhum fornecedor cadastrado","Os fornecedores aparecerão aqui após o cadastro."),
                    Ok(rows) => {
                        let mut edit = None;
                        egui::ScrollArea::horizontal().id_salt("suppliers_table_scroll").show(ui,|ui|{
                            egui::Grid::new("suppliers_table").min_col_width(90.).spacing([18.,10.]).striped(true).show(ui,|ui|{
                                for h in ["FORNECEDOR","CPF / CNPJ","PERFIL","CAF","CONFERÊNCIA","AÇÃO"] { ui.label(egui::RichText::new(h).size(11.).strong().color(egui::Color32::from_rgb(100,110,120))); }
                                ui.end_row();
                                for supplier in &rows {
                                    ui.label(egui::RichText::new(&supplier.name).strong());
                                    ui.monospace(format_document(&supplier.document));
                                    ui.label(if supplier.family {"Agricultura familiar"} else {"Demais fornecedores"});
                                    ui.label(if supplier.caf.is_empty() {"—"} else {supplier.caf.as_str()});
                                    ui.label(if supplier.family {if supplier.verified {"Conferido"} else {"Pendente"}} else {"—"}).on_hover_text(&supplier.verification);
                                    if ui.small_button("Editar").clicked() { edit=Some(supplier.clone()); }
                                    ui.end_row();
                                }
                            });
                        });
                        if let Some(supplier)=edit {
                            app.compliance.supplier_edit_id=supplier.id;
                            app.compliance.supplier_edit_name=supplier.name;
                            app.compliance.supplier_edit_document=supplier.document;
                            app.compliance.supplier_edit_family=supplier.family;
                            app.compliance.supplier_edit_caf=supplier.caf;
                        }
                    }
                }
                if app.compliance.supplier_edit_id > 0 {
                    ui.add_space(14.);
                    ui.separator();
                    ui.label(egui::RichText::new(format!("Editar fornecedor #{}",app.compliance.supplier_edit_id)).strong());
                    ui.columns(2,|c|{field(&mut c[0],"Nome / razão social",&mut app.compliance.supplier_edit_name);let document_invalid=show_errors&&!document_valid(&app.compliance.supplier_edit_document);field_required(&mut c[1],"CPF ou CNPJ",&mut app.compliance.supplier_edit_document,document_invalid);});
                    ui.checkbox(&mut app.compliance.supplier_edit_family,"Fornecedor da agricultura familiar");
                    field(ui,"CAF declarado",&mut app.compliance.supplier_edit_caf);
                    ui.horizontal_wrapped(|ui|{
                        if primary(ui,"Salvar alterações") {let f=&app.compliance;let r=app.db.update_supplier(f.supplier_edit_id,&f.supplier_edit_name,&f.supplier_edit_document,f.supplier_edit_family,&f.supplier_edit_caf);app.notify(r);}
                        if ui.button("Excluir fornecedor").clicked(){let r=app.db.delete_supplier(app.compliance.supplier_edit_id);if app.notify(r){app.compliance.supplier_edit_id=0;app.compliance.supplier_edit_name.clear();app.compliance.supplier_edit_document.clear();app.compliance.supplier_edit_caf.clear();}}
                    });
                    hint(ui,"A conferência familiar é invalidada quando CPF/CNPJ, enquadramento ou CAF mudam. As compras já registradas mantêm os dados originais do recebimento.");
                }
            }); ui.add_space(16.);
            card(ui,"Conferência documental da agricultura familiar","Restrinja a função a quem conferiu o documento oficial e a habilitação do fornecedor. O cadastro por si só não comprova elegibilidade.",|ui|{
                choose(ui,"Fornecedor familiar","family_verify_supplier",&mut app.compliance.supplier_verify_id,app.db.supplier_choices().unwrap_or_default());
                field(ui,"Referência do documento conferido (CAF/DAP válida, chamada pública, arquivo)",&mut app.compliance.supplier_verify_reference);
                multiline(ui,"Observação da conferência (não inclua dados pessoais desnecessários)",&mut app.compliance.supplier_verify_reason);
                if primary(ui,"Registrar conferência do administrador"){let f=&app.compliance;let r=app.db.verify_family_supplier(f.supplier_verify_id,&f.supplier_verify_reference,&f.supplier_verify_reason);app.notify(r);}
            });
        },
        1=>{
            card(ui,"Registrar pagamento efetivado","Acesso do administrador. Informe o valor efetivamente pago, sem superar o saldo da nota.",|ui|{
                required_choose(ui,"Recebimento / nota fiscal","payment_receipt",&mut app.compliance.payment_receipt,app.db.receipt_choices().unwrap_or_default(),show_errors);
                ui.columns(2,|c|{required_field(&mut c[0],"Data do pagamento (DD/MM/AAAA)",&mut app.compliance.payment_day,show_errors);required_field(&mut c[1],"Valor pago (R$)",&mut app.compliance.payment_value,show_errors);});
                required_field(ui,"Documento da transação / parcela",&mut app.compliance.payment_document,show_errors);
                if primary(ui,"Registrar pagamento"){let f=&app.compliance;let r=money(&f.payment_value).and_then(|c|app.db.record_payment(&f.payment_day,f.payment_receipt,c,&f.payment_document));if app.notify(r){app.compliance.payment_value.clear();app.compliance.payment_document.clear();}}
            });ui.add_space(16.);
            card(ui,"Repasses e saldos reprogramados","Informe os recursos efetivamente creditados, conforme os extratos oficiais.",|ui|{
                ui.checkbox(&mut app.compliance.carryover,"Saldo reprogramado de exercício anterior (separado dos novos repasses)");
                ui.columns(2,|c|{required_field(&mut c[0],"Data do crédito (DD/MM/AAAA)",&mut app.compliance.resource_day,show_errors);required_field(&mut c[1],"Valor creditado (R$)",&mut app.compliance.resource_value,show_errors);});
                required_field(ui,"Referência / documento do crédito",&mut app.compliance.resource_document,show_errors);
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
fn format_document(document: &str) -> String {
    let d: String = document.chars().filter(char::is_ascii_digit).collect();
    match d.len() {
        11 => format!("{}.{}.{}-{}", &d[0..3], &d[3..6], &d[6..9], &d[9..11]),
        14 => format!(
            "{}.{}.{}/{}-{}",
            &d[0..2],
            &d[2..5],
            &d[5..8],
            &d[8..12],
            &d[12..14]
        ),
        _ => document.to_string(),
    }
}
