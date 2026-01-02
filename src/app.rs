// ... (imports)
use eframe::egui::{self, Pos2, TextureHandle};
use image::DynamicImage;
use std::time::Instant;
use crate::{ui, capture};

// ... (ChatMessage, AppState, ModelMode, OnlineProvider structs/enums är samma som förut) ...
// (Kopiera in dem om du ersätter hela filen, men här visar jag ändringarna i App structen)

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
    
    pub screenshot: Option<DynamicImage>,
    pub screenshot_texture: Option<TextureHandle>,
    pub selection_start: Option<Pos2>,
    pub selection_current: Option<Pos2>,

    pub chat_input: String,
    pub chat_history: Vec<ChatMessage>,
    
    // --- NYTT FÄLT ---
    pub show_chat: bool, // Håller reda på om chatten är öppen eller stängd
    
    pub capture_delay: Option<Instant>, 
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
            chat_input: String::new(),
            chat_history: vec![
                ChatMessage { is_user: false, text: "Hej! Vad vill du göra?".to_string() }
            ],
            // Vi visar chatten som standard när appen startar (eller sätt false om du vill)
            show_chat: true, 
            capture_delay: None, 
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        // Timer Logic (Samma som förut)
        if let Some(deadline) = self.capture_delay {
            if Instant::now() > deadline {
                if let Some(img) = capture::take_screenshot() {
                    self.screenshot = Some(img);
                    self.screenshot_texture = None;
                    self.selection_start = None;
                    self.selection_current = None;
                    self.state = AppState::Selecting;
                    
                    // --- NYTT: Öppna alltid chatten automatiskt när vi tar en ny bild ---
                    self.show_chat = true; 
                }
                self.capture_delay = None;
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                ctx.request_repaint();
            } else {
                ctx.request_repaint(); 
                return;
            }
        }

        match self.state {
            AppState::Selecting => {
                ui::selecting::render(self, ctx);
            }
            AppState::Toolbox => {
                // --- NYTT: Visa bara sidebar om vi har bild OCH show_chat är true ---
                if self.screenshot.is_some() && self.show_chat {
                    ui::chatsidebar::render(self, ctx);
                }
                ui::toolbox::render(self, ctx);
            }
        }
    }
}