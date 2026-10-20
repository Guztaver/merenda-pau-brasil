use crate::{db::Db, ui, validation::R};
use chrono::Local;
use eframe::egui;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Foods,
    Movements,
    Recipes,
    Servings,
    Schools,
    Reports,
}
impl Page {
    pub const ALL: [Self; 7] = [
        Self::Dashboard,
        Self::Foods,
        Self::Movements,
        Self::Recipes,
        Self::Servings,
        Self::Schools,
        Self::Reports,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Visão geral",
            Self::Foods => "Alimentos e estoque",
            Self::Movements => "Entradas e perdas",
            Self::Recipes => "Receitas",
            Self::Servings => "Refeições servidas",
            Self::Schools => "Escolas",
            Self::Reports => "Relatórios e backup",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Dashboard => "Cuidar da merenda começa aqui.",
            Self::Foods => "Alimentos e estoque",
            Self::Movements => "Cada movimento, registrado.",
            Self::Recipes => "Receitas que alimentam",
            Self::Servings => "Da cozinha para a escola",
            Self::Schools => "Nossa rede de escolas",
            Self::Reports => "Tudo pronto para conferir",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Dashboard => "Um olhar sobre os alimentos e a alimentação da rede municipal.",
            Self::Foods => "Cadastre os alimentos e consulte o saldo na data que precisar.",
            Self::Movements => "Registre recebimentos, perdas e ajustes com a data correta.",
            Self::Recipes => "Organize os ingredientes e o peso necessário para cada porção.",
            Self::Servings => {
                "Registre as porções servidas. O estoque é atualizado automaticamente."
            }
            Self::Schools => {
                "Mantenha as unidades atendidas pela alimentação escolar em um só lugar."
            }
            Self::Reports => {
                "Consulte períodos, exporte relatórios e mantenha seus dados protegidos."
            }
        }
    }
}
#[derive(Default)]
pub struct FoodForm {
    pub name: String,
    pub minimum: String,
    pub day: String,
    pub search: String,
}
#[derive(Default)]
pub struct MovementForm {
    pub day: String,
    pub item: i64,
    pub weight: String,
    pub note: String,
    pub loss: bool,
}
#[derive(Default)]
pub struct RecipeForm {
    pub name: String,
    pub recipe: i64,
    pub item: i64,
    pub weight: String,
}
#[derive(Default)]
pub struct ServingForm {
    pub day: String,
    pub school: i64,
    pub recipe: i64,
    pub portions: String,
}
pub struct ReportForm {
    pub start: String,
    pub end: String,
}
pub struct Notice {
    pub text: String,
    pub error: bool,
}
pub struct App {
    pub(crate) db: Db,
    pub(crate) dir: PathBuf,
    pub(crate) page: Page,
    pub(crate) notice: Option<Notice>,
    pub(crate) foods: FoodForm,
    pub(crate) movements: MovementForm,
    pub(crate) recipes: RecipeForm,
    pub(crate) servings: ServingForm,
    pub(crate) reports: ReportForm,
    pub(crate) school_name: String,
}
impl App {
    pub fn new(db: Db, dir: PathBuf) -> Self {
        let today = Local::now().format("%Y-%m-%d").to_string();
        Self {
            db,
            dir,
            page: Page::Dashboard,
            notice: None,
            foods: FoodForm {
                minimum: "0".into(),
                day: today.clone(),
                ..Default::default()
            },
            movements: MovementForm {
                day: today.clone(),
                ..Default::default()
            },
            recipes: RecipeForm::default(),
            servings: ServingForm {
                day: today.clone(),
                ..Default::default()
            },
            reports: ReportForm {
                start: Local::now().format("%Y-%m-01").to_string(),
                end: today,
            },
            school_name: String::new(),
        }
    }
    pub(crate) fn notify(&mut self, result: R<()>) -> bool {
        let success = result.is_ok();
        self.notice = Some(match result {
            Ok(()) => Notice {
                text: "Registro salvo com sucesso.".into(),
                error: false,
            },
            Err(e) => Notice {
                text: e,
                error: true,
            },
        });
        success
    }
    pub(crate) fn navigate(&mut self, page: Page) {
        self.page = page;
        self.notice = None;
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        ui::shell::show(self, ctx);
    }
}
