use eframe::egui::{self, Pos2, TextureHandle};
use image::DynamicImage;
use std::time::Instant;
use std::sync::mpsc::{channel, Receiver, Sender};
use chrono::{DateTime, Local}; // <--- NYTT: För datum och tid
use crate::{ui, capture};

#[derive(Clone)]
pub struct ChatMessage {
    pub is_user: bool,
    pub text: String,
    pub image: Option<DynamicImage>,
    pub texture: Option<TextureHandle>,
}

// --- NY STRUKTUR: En sparad chatt-session ---
#[derive(Clone)]
pub struct SavedChat {
    pub timestamp: DateTime<Local>, // När sparades den?
    pub summary: String,            // "Kort kort kort..." sammanfattning
    pub mode: ModelMode,            // Var det Lokal eller Online?
    pub history: Vec<ChatMessage>,  // Hela konversationen
}

#[derive(PartialEq, Clone, Default)]
pub enum AppState { #[default] Toolbox, Selecting }

#[derive(PartialEq, Clone, Default)]
pub enum ModelMode { #[default] Local, Online }

#[derive(PartialEq, Clone, Debug, Default)]
pub enum OnlineProvider { #[default] Anthropic, OpenAI, Groq, Grok }

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

    // --- NYA FÄLT FÖR HISTORIK ---
    pub saved_chats: Vec<SavedChat>,     // Lista på alla sparade chattar
    pub show_history_view: bool,         // Visar vi chatten eller historik-listan?
    pub history_filter_local: bool,      // True = Visa Lokal, False = Visa Online

    pub local_models: Vec<String>,
    pub selected_local_model: String,
    pub ollama_error: Option<String>,
    pub chat_sender: Sender<String>,
    pub chat_receiver: Receiver<String>,
    pub is_loading: bool,

    pub show_chat: bool,
    pub capture_delay: Option<Instant>, 

    // online_models: Vec<String>, // Framtida användning
    pub selected_online_model: String,
}

impl Default for App {
    fn default() -> Self {
        let (sender, receiver) = channel();

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
                ChatMessage { 
                    is_user: false, 
                    text: "Hej! Vad vill du göra?".to_string(),
                    image: None,
                    texture: None
                }
            ],
            // Initiera de nya historik-fälten
            saved_chats: Vec::new(),
            show_history_view: false,
            history_filter_local: true,

            show_chat: true, 
            capture_delay: None,
            local_models: Vec::new(),
            selected_local_model: String::new(),
            ollama_error: None,

            chat_sender: sender,
            chat_receiver: receiver,
            is_loading: false,
            selected_online_model: String::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        // 1. Lyssna efter svar (med stöd för den nya "enkla" tråden)
        if let Ok(response_text) = self.chat_receiver.try_recv() {
            self.is_loading = false;
            self.chat_history.push(ChatMessage {
                is_user: false,
                text: response_text,
                image: None,
                texture: None,
            });
        }

        // 2. Timer Logic
        if let Some(deadline) = self.capture_delay {
            if Instant::now() > deadline {
                if let Some(img) = capture::take_screenshot() {
                    self.screenshot = Some(img);
                    self.screenshot_texture = None;
                    self.selection_start = None;
                    self.selection_current = None;
                    self.state = AppState::Selecting;
                    self.show_chat = true; 
                    // När vi tar ny bild, gå ur historik-läget till chatten
                    self.show_history_view = false; 
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

        // 3. Rita UI
        match self.state {
            AppState::Selecting => {
                ui::selecting::render(self, ctx);
            }
            AppState::Toolbox => {
                // ÄNDRING: Vi visar chatten oavsett om vi har en bild eller inte
                if self.show_chat {
                    ui::chatsidebar::render(self, ctx);
                }
                ui::toolbox::render(self, ctx);
            }
        }
    }
}