use super::{components::*, theme::*};
use crate::app::App;
use eframe::egui::{self, RichText};
pub fn show(app: &mut App, ctx: &egui::Context) {
    egui::CentralPanel::default().frame(egui::Frame::new().fill(BG).inner_margin(32.)).show(ctx,|ui|{
        egui::ScrollArea::vertical().show(ui,|ui|{
            ui.add_space(25.);
            ui.vertical_centered(|ui|{ui.label(RichText::new("Merenda • Pau Brasil").size(32.).strong().color(GREEN));hint(ui,"Gestão municipal da alimentação escolar • 100% offline");});ui.add_space(24.);
            ui.horizontal(|ui|{
                ui.add_space(((ui.available_width()-480.)/2.).max(0.));
                ui.vertical(|ui|{ui.set_max_width(480.);
                    let setup=app.db.needs_setup();
                    card(ui,if setup{"Configurar o primeiro administrador"}else{"Acesse sua conta local"},if setup{"Crie uma senha de pelo menos 12 caracteres. Não existe senha padrão."}else{"Cada lançamento será vinculado ao usuário responsável."},|ui|{
                        if setup{field(ui,"Nome completo",&mut app.auth.name);}
                        field(ui,"Login",&mut app.auth.login);password(ui,"Senha",&mut app.auth.password);
                        if setup{password(ui,"Confirme a senha",&mut app.auth.confirm);}
                        if let Some(n)=&app.notice{ui.colored_label(AMBER,&n.text);}
                        if primary(ui,if setup{"Criar administrador e entrar"}else{"Entrar"}){
                            let r=if setup{
                                if app.auth.password!=app.auth.confirm{Err("As senhas não coincidem.".into())}else{app.db.setup_admin(&app.auth.name,&app.auth.login,&app.auth.password)}
                            }else{app.db.login(&app.auth.login,&app.auth.password)};
                            app.auth.password.clear();app.auth.confirm.clear();
                            if app.notify(r){app.last_activity=std::time::Instant::now();app.notice=None;}
                        }
                        hint(ui,"As contas são locais. O administrador cadastra os demais usuários e o nutricionista com CRN. Proteja também a conta do Windows e os backups.");
                    });
                });
            });
        });
    });
}
