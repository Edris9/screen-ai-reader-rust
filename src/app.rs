use eframe::egui::{self, Pos2, TextureHandle};
use image::DynamicImage;
use crate::ui;

// En enkel struktur för ett meddelande
#[derive(Clone)]
pub struct ChatMessage {
    pub is_user: bool,
    pub text: String,
}

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

pub struct App {
    pub state: AppState,
    pub model_mode: ModelMode,
    pub online_provider: OnlineProvider,
    pub api_key: String,
    pub show_online_popup: bool,
    
    // Bildhantering
    pub screenshot: Option<DynamicImage>,
    pub screenshot_texture: Option<TextureHandle>,
    pub selection_start: Option<Pos2>,
    pub selection_current: Option<Pos2>,

    // --- NYTT FÖR CHATTEN ---
    pub chat_input: String,
    pub chat_history: Vec<ChatMessage>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::Toolbox,
            model_mode: ModelMode::Local,
            online_provider: OnlineProvider::Anthropic,
            api_key: String::new(),
            show_online_popup: false,
            screenshot: None,
            screenshot_texture: None,
            selection_start: None,
            selection_current: None,
            // Initiera chatten
            chat_input: String::new(),
            chat_history: vec![
                ChatMessage { is_user: false, text: "Hej! Jag ser bilden. Vad vill du veta?".to_string() }
            ],
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.state {
            AppState::Selecting => {
                // När vi klipper, visa bara selecting UI
                ui::selecting::render(self, ctx);
            }
            AppState::Toolbox => {
                // 1. Om vi har en bild, visa Sidebaren FÖRST (till höger)
                if self.screenshot.is_some() {
                    ui::chatsidebar::render(self, ctx);
                }
                
                // 2. Visa sedan Toolbox/Huvudpanelen
                ui::toolbox::render(self, ctx);
            }
        }
    }
}