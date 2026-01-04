use eframe::egui::{self, Pos2, TextureHandle}; // <--- Se till att Color32 är med
use image::DynamicImage;
use std::time::Instant;
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::{ui, capture};

#[derive(Clone)]
pub struct ChatMessage {
    pub is_user: bool,
    pub text: String,
    
    // --- NYA FÄLT ---
    pub image: Option<DynamicImage>, // Sparar själva bilddatan
    pub texture: Option<TextureHandle>, // Sparar texturen för visning (cache)
}

// ... (AppState, ModelMode, OnlineProvider är samma som förut) ...
#[derive(PartialEq, Clone, Default)]
pub enum AppState { #[default] Toolbox, Selecting }
#[derive(PartialEq, Clone, Default)]
pub enum ModelMode { #[default] Local, Online }
#[derive(PartialEq, Clone, Debug, Default)]
pub enum OnlineProvider { #[default] Anthropic, OpenAI, Groq, Grok }

pub struct App {
    // ... (Samma fält som förut) ...
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
    pub local_models: Vec<String>,
    pub selected_local_model: String,
    pub ollama_error: Option<String>,
    pub chat_sender: Sender<String>,
    pub chat_receiver: Receiver<String>,
    pub is_loading: bool,
    pub show_chat: bool,
    pub capture_delay: Option<Instant>, 
}

impl Default for App {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            // ... (Samma defaults) ...
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
            // Uppdatera standardmeddelandet med de nya fälten (None)
            chat_history: vec![
                ChatMessage { 
                    is_user: false, 
                    text: "Hej! Vad vill du göra?".to_string(),
                    image: None,
                    texture: None
                }
            ],
            show_chat: true, 
            capture_delay: None,
            local_models: Vec::new(),
            selected_local_model: String::new(),
            ollama_error: None,
            chat_sender: sender,
            chat_receiver: receiver,
            is_loading: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        // Uppdatera lyssnaren för att hantera de nya fälten
        if let Ok(response_text) = self.chat_receiver.try_recv() {
            self.is_loading = false;
            self.chat_history.push(ChatMessage {
                is_user: false,
                text: response_text,
                image: None,   // AI svarar oftast inte med bild
                texture: None,
            });
        }

        // ... (Resten av update-funktionen är exakt samma som förut) ...
        if let Some(deadline) = self.capture_delay {
            if Instant::now() > deadline {
                if let Some(img) = capture::take_screenshot() {
                    self.screenshot = Some(img);
                    self.screenshot_texture = None; // Reset main texture
                    self.selection_start = None;
                    self.selection_current = None;
                    self.state = AppState::Selecting;
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
            AppState::Selecting => { ui::selecting::render(self, ctx); }
            AppState::Toolbox => {
                if self.screenshot.is_some() && self.show_chat {
                    ui::chatsidebar::render(self, ctx);
                }
                ui::toolbox::render(self, ctx);
            }
        }
    }
}