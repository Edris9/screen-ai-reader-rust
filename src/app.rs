use eframe::egui::{self, Pos2}; // Lade till Pos2
use image::DynamicImage;
use crate::ui;

#[derive(PartialEq, Clone, Default)]
pub enum AppState {
    #[default]
    Toolbox,
    Selecting,
}

#[derive(PartialEq, Clone, Default)]
pub enum ModelMode {
    #[default]
    Local,
    Online,
}

#[derive(PartialEq, Clone, Debug, Default)]
pub enum OnlineProvider {
    #[default]
    Anthropic,
    OpenAI,
    Groq,
    Grok,
}

#[derive(Default)]
pub struct App {
    pub state: AppState,
    pub model_mode: ModelMode,
    pub online_provider: OnlineProvider,
    pub api_key: String,
    pub show_online_popup: bool,
    pub screenshot: Option<DynamicImage>,
    pub screenshot_texture: Option<egui::TextureHandle>,
    
    // Nya fält för selektering
    pub selection_start: Option<Pos2>,
    pub selection_current: Option<Pos2>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.state {
            AppState::Toolbox => ui::toolbox::render(self, ctx),
            AppState::Selecting => ui::selecting::render(self, ctx),
        }
    }
}