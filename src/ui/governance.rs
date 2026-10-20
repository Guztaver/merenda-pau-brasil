use super::components::*;
use crate::app::App;
use eframe::egui;
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        for (i, n) in ["Usuários", "Competências e estornos", "Anexos e auditoria"]
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
        0 => {
            card(ui,"Cadastrar usuário local","Somente o administrador. O perfil Nutricionista exige nome e CRN do profissional responsável.",|ui|{
                field(ui,"Nome completo",&mut app.compliance.user_name);field(ui,"Login (letras, números, ponto, hífen ou sublinhado)",&mut app.compliance.user_login);
                password(ui,"Senha inicial (mínimo de 12 caracteres)",&mut app.compliance.user_password);
                options(ui,"Perfil de acesso","new_role",&mut app.compliance.user_role,&crate::db::security::ROLES);field(ui,"CRN (perfil Nutricionista)",&mut app.compliance.user_crn);
                if primary(ui,"Cadastrar usuário"){let f=&app.compliance;let r=app.db.create_user(&f.user_name,&f.user_login,&f.user_password,&f.user_role,&f.user_crn);app.compliance.user_password.clear();app.notify(r);}
                hint(ui,"Administrador: cadastros gerais, finanças, usuários e estornos. Estoque: recebimentos, perdas, transferências e consumo. Nutricionista: avaliações, receitas, fichas e cardápios. Consulta: leitura e relatórios. A aprovação técnica sempre exige o perfil Nutricionista.");
            });
            ui.add_space(16.);
            card(ui,"Gerenciar acesso","A desativação mantém o histórico. Trocas de senha e alterações de acesso ficam na auditoria.",|ui|{
                match app.db.users(){Err(e)=>hint(ui,&e),Ok(users)=>{
                    egui::ScrollArea::horizontal().id_salt("users_table_scroll").show(ui,|ui|{egui::Grid::new("users_table").min_col_width(100.).spacing([22.,10.]).striped(true).show(ui,|ui|{
                        for h in ["USUÁRIO","PERFIL","SITUAÇÃO"]{ui.label(egui::RichText::new(h).size(11.).strong());}ui.end_row();
                        for(_,name,role,active)in &users{ui.strong(name);ui.label(role);badge(ui,if *active{"Ativo"}else{"Inativo"},!*active);ui.end_row();}
                    });});
                    choose(ui,"Usuário","manage_user",&mut app.compliance.user_id,users.into_iter().map(|(id,name,_,_)|(id,name)).collect());
                }}
                field(ui,"Justificativa da alteração",&mut app.compliance.user_reason);
                ui.horizontal_wrapped(|ui|{for(active,label)in [(false,"Desativar acesso"),(true,"Reativar acesso")]{if ui.button(label).clicked(){let f=&app.compliance;let r=app.db.set_user_active(f.user_id,active,&f.user_reason);app.notify(r);}}});
                password(ui,"Nova senha (para redefinição)",&mut app.compliance.reset_password);
                if ui.button("Redefinir senha do usuário selecionado").clicked(){let f=&app.compliance;let r=app.db.reset_user_password(f.user_id,&f.reset_password,&f.user_reason);app.compliance.reset_password.clear();app.notify(r);}
            });
        }
        1 => {
            card(ui,"Fechamento de competências","O administrador pode fechar ou reabrir um mês. Lançamentos em competências fechadas são bloqueados no banco.",|ui|{
                field(ui,"Competência (AAAA-MM)",&mut app.compliance.period);field(ui,"Justificativa do fechamento / reabertura",&mut app.compliance.period_reason);
                ui.horizontal_wrapped(|ui|{for(closed,label)in [(true,"Fechar competência"),(false,"Reabrir com justificativa")]{if ui.button(label).clicked(){let f=&app.compliance;let r=app.db.set_period(&f.period,closed,&f.period_reason);app.notify(r);}}});
                match app.db.periods(){Ok(rows)=>{if !rows.is_empty(){egui::ScrollArea::horizontal().id_salt("periods_table_scroll").show(ui,|ui|{egui::Grid::new("periods_table").min_col_width(100.).spacing([22.,10.]).striped(true).show(ui,|ui|{
                    for h in ["COMPETÊNCIA","SITUAÇÃO","JUSTIFICATIVA"]{ui.label(egui::RichText::new(h).size(11.).strong());}ui.end_row();
                    for(month,closed,reason)in rows{ui.label(month);badge(ui,if closed{"Fechada"}else{"Aberta"},closed);ui.label(reason);ui.end_row();}
                });});}},Err(e)=>hint(ui,&e)}
            });
            ui.add_space(16.);
            card(ui,"Estorno justificado","Não apaga o original. Cria a contrapartida na data escolhida e preserva o vínculo com a operação inicial.",|ui|{
                choose(ui,"Operação ainda não estornada","reversal_op",&mut app.compliance.operation,app.db.operations().unwrap_or_default());
                field(ui,"Data do estorno (DD/MM/AAAA)",&mut app.compliance.reversal_day);multiline(ui,"Motivo do estorno",&mut app.compliance.reversal_reason);
                hint(ui,"O estorno de uma entrada ou transferência será recusado se faltar saldo. Para estornar recebimentos pagos, estorne os pagamentos primeiro. Uma correção de mês fechado pode ser registrada como estorno em mês aberto, sem reescrever o histórico.");
                if primary(ui,"Registrar estorno") {let f=&app.compliance;let r=app.db.reverse(f.operation,&f.reversal_day,&f.reversal_reason);if app.notify(r){app.compliance.reversal_reason.clear();}}
            });
        }
        _ => {
            card(ui,"Anexar documento comprobatório","O arquivo é copiado para a pasta local dos dados, com hash SHA-256 e responsável. O original permanece no lugar.",|ui|{
                options(ui,"Vincular a","document_entity",&mut app.compliance.document_entity,&["Operação","Fornecedor familiar","Ficha técnica","Cardápio"]);
                field(ui,"ID do registro (fornecedor na aba Compras, ou operação listada em Estornos)",&mut app.compliance.document_id);field(ui,"Caminho completo do arquivo (PDF, XML, imagem, TXT, CSV ou DOCX)",&mut app.compliance.document_path);
                if primary(ui,"Copiar e vincular documento"){
                    let f=&app.compliance;let r=f.document_id.parse::<i64>().map_err(|_|"Informe um ID inteiro.".to_string()).and_then(|id|app.db.attach_document(&f.document_entity,id,std::path::Path::new(&f.document_path),&app.dir));if app.notify(r){app.compliance.document_path.clear();}
                }
            });
            ui.add_space(16.);
            card(ui,"Verificação e exportação da auditoria","A trilha encadeia os eventos por hash. Registros não podem ser alterados ou excluídos pelo aplicativo.",|ui|{
                if ui.button("Verificar integridade dos eventos").clicked(){match app.db.verify_audit(){Ok(count)=>{app.notify(Ok(()));app.notice.as_mut().unwrap().text=format!("Cadeia de {count} eventos íntegra.");},Err(e)=>{app.notify(Err(e));}}}
                if ui.button("Exportar trilha CSV").clicked(){let path=app.dir.join(format!("auditoria-{}.csv",chrono::Utc::now().format("%Y%m%d-%H%M%S%f")));let r=app.db.audit_csv().and_then(|s|std::fs::write(&path,s).map_err(|e|e.to_string()));if app.notify(r){app.notice.as_mut().unwrap().text=format!("Salvo em {}",path.display());}}
                hint(ui,"O administrador do computador ainda pode acessar os arquivos SQLite. Os hashes locais não equivalem a assinatura digital nem impedem adulteração por quem controla o computador. Guarde cópias externas com acesso restrito.");
            });
        }
    }
}
