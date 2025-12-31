mod capture;
mod ai;
mod config;

use capture::{ScreenCapture, CaptureRegion};
use ai::{AiClient, ModelProvider};
use config::{Config, History, HistoryEntry};

use eframe::egui::{self, Color32, RichText, Stroke, Vec2, Pos2, Rect, TextureHandle, ColorImage};
use image::DynamicImage;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::runtime::Runtime;

// Lila färgpalett
const PURPLE_DARK: Color32 = Color32::from_rgb(88, 28, 135);      // purple-900
const PURPLE_MAIN: Color32 = Color32::from_rgb(147, 51, 234);     // purple-600
const PURPLE_LIGHT: Color32 = Color32::from_rgb(192, 132, 252);   // purple-400
const PURPLE_BG: Color32 = Color32::from_rgb(59, 7, 100);         // purple-950

#[derive(PartialEq, Clone)]
enum AppState {
    Idle,
    Capturing,
    SelectingRegion,
    Processing,
    ShowingResult,
}

#[derive(PartialEq, Clone)]
enum ActiveTab {
    LocalModel,
    OnlineModel,
    History,
    Settings,
}

struct ScreenAiApp {
    state: AppState,
    active_tab: ActiveTab,
    
    // Capture
    screen_capture: Option<ScreenCapture>,
    full_screen_image: Option<DynamicImage>,
    screen_texture: Option<TextureHandle>,
    selection_start: Option<Pos2>,
    selection_end: Option<Pos2>,
    captured_region: Option<DynamicImage>,
    captured_base64: Option<String>,
    
    // AI
    ai_client: AiClient,
    response_text: Arc<Mutex<String>>,
    is_generating: bool,
    cancel_tx: Option<mpsc::Sender<()>>,
    response_rx: Option<mpsc::UnboundedReceiver<String>>,
    
    // Config & History  
    config: Config,
    history: History,
    custom_prompt: String,
    
    // Runtime
    runtime: Runtime,
    
    // Settings UI
    show_settings: bool,
    temp_openai_key: String,
    temp_claude_key: String,
}

impl Default for ScreenAiApp {
    fn default() -> Self {
        let config = Config::load();
        let provider = if config.use_local {
            ModelProvider::Local {
                endpoint: config.local_endpoint.clone(),
                model: config.local_model.clone(),
            }
        } else {
            ModelProvider::OpenAI {
                api_key: config.openai_api_key.clone(),
                model: config.openai_model.clone(),
            }
        };
        
        Self {
            state: AppState::Idle,
            active_tab: ActiveTab::LocalModel,
            screen_capture: ScreenCapture::new().ok(),
            full_screen_image: None,
            screen_texture: None,
            selection_start: None,
            selection_end: None,
            captured_region: None,
            captured_base64: None,
            ai_client: AiClient::new(provider),
            response_text: Arc::new(Mutex::new(String::new())),
            is_generating: false,
            cancel_tx: None,
            response_rx: None,
            config: config.clone(),
            history: History::default(),
            custom_prompt: config.default_prompt.clone(),
            runtime: Runtime::new().unwrap(),
            show_settings: false,
            temp_openai_key: config.openai_api_key,
            temp_claude_key: config.claude_api_key,
        }
    }
}

impl ScreenAiApp {
    fn start_capture(&mut self) {
        if let Some(ref capture) = self.screen_capture {
            if let Ok(img) = capture.capture_full() {
                self.full_screen_image = Some(img);
                self.state = AppState::SelectingRegion;
                self.selection_start = None;
                self.selection_end = None;
            }
        }
    }

    fn process_selection(&mut self) {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let x = start.x.min(end.x) as i32;
            let y = start.y.min(end.y) as i32;
            let width = (end.x - start.x).abs() as u32;
            let height = (end.y - start.y).abs() as u32;
            
            if width > 10 && height > 10 {
                if let Some(ref capture) = self.screen_capture {
                    let region = CaptureRegion { x, y, width, height };
                    if let Ok(img) = capture.capture_region(&region) {
                        if let Ok(base64) = ScreenCapture::image_to_base64(&img) {
                            self.captured_region = Some(img);
                            self.captured_base64 = Some(base64);
                            self.state = AppState::ShowingResult;
                        }
                    }
                }
            }
        }
    }

    fn start_ai_analysis(&mut self) {
        if let Some(ref base64) = self.captured_base64 {
            self.is_generating = true;
            *self.response_text.lock().unwrap() = String::new();
            
            let (tx, rx) = mpsc::unbounded_channel();
            let (cancel_tx, mut cancel_rx) = mpsc::channel(1);
            
            self.response_rx = Some(rx);
            self.cancel_tx = Some(cancel_tx);
            
            let client = self.ai_client.clone();
            let image = base64.clone();
            let prompt = self.custom_prompt.clone();
            let response_text = self.response_text.clone();
            
            self.runtime.spawn(async move {
                let result = client.analyze_image(image, prompt, tx, &mut cancel_rx).await;
                if let Err(e) = result {
                    let mut text = response_text.lock().unwrap();
                    *text = format!("\n\n❌ Fel: {}", e);
                }
            });
            
            self.state = AppState::Processing;
        }
    }

    fn stop_generating(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.try_send(());
        }
        self.is_generating = false;
        self.add_to_history();
    }

    fn cancel_operation(&mut self) {
        self.stop_generating();
        self.state = AppState::Idle;
        self.captured_region = None;
        self.captured_base64 = None;
        self.full_screen_image = None;
        *self.response_text.lock().unwrap() = String::new();
    }

    fn add_to_history(&mut self) {
        let response = self.response_text.lock().unwrap().clone();
        if !response.is_empty() {
            self.history.add(HistoryEntry {
                timestamp: chrono_lite_now(),
                prompt: self.custom_prompt.clone(),
                response,
                image_preview: None,
            });
        }
    }

    fn update_provider(&mut self) {
        match self.active_tab {
            ActiveTab::LocalModel => {
                self.ai_client.set_provider(ModelProvider::Local {
                    endpoint: self.config.local_endpoint.clone(),
                    model: self.config.local_model.clone(),
                });
            }
            ActiveTab::OnlineModel => {
                // Använd Claude som standard för online
                if !self.config.claude_api_key.is_empty() {
                    self.ai_client.set_provider(ModelProvider::Claude {
                        api_key: self.config.claude_api_key.clone(),
                        model: self.config.claude_model.clone(),
                    });
                } else if !self.config.openai_api_key.is_empty() {
                    self.ai_client.set_provider(ModelProvider::OpenAI {
                        api_key: self.config.openai_api_key.clone(),
                        model: self.config.openai_model.clone(),
                    });
                }
            }
            _ => {}
        }
    }
}

impl eframe::App for ScreenAiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Uppdatera response från AI
        if let Some(ref mut rx) = self.response_rx {
            while let Ok(chunk) = rx.try_recv() {
                let mut text = self.response_text.lock().unwrap();
                text.push_str(&chunk);
            }
        }
        
        // Kolla om generering är klar
        if self.is_generating {
            ctx.request_repaint();
        }

        // Fullskärms-selection mode
        if self.state == AppState::SelectingRegion {
            self.render_selection_overlay(ctx);
            return;
        }

        // Huvudfönster
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(PURPLE_BG))
            .show(ctx, |ui| {
                self.render_main_ui(ui, ctx);
            });
    }
}

impl ScreenAiApp {
    fn render_selection_overlay(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::from_rgba_unmultiplied(0, 0, 0, 180)))
            .show(ctx, |ui| {
                // Visa instruktioner
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new("🎯 Dra för att välja område")
                        .size(24.0)
                        .color(Color32::WHITE));
                    ui.label(RichText::new("Tryck ESC för att avbryta")
                        .size(16.0)
                        .color(PURPLE_LIGHT));
                });

                // Hantera input
                let response = ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::drag());
                
                if response.drag_started() {
                    self.selection_start = response.interact_pointer_pos();
                }
                
                if response.dragged() {
                    self.selection_end = response.interact_pointer_pos();
                }
                
                if response.drag_stopped() {
                    self.process_selection();
                }

                // Rita selection rektangel
                if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                    let rect = Rect::from_two_pos(start, end);
                    ui.painter().rect_stroke(rect, 0.0, Stroke::new(3.0, PURPLE_MAIN));
                    ui.painter().rect_filled(rect, 0.0, Color32::from_rgba_unmultiplied(147, 51, 234, 50));
                }

                // ESC för att avbryta
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.state = AppState::Idle;
                    self.full_screen_image = None;
                }
            });
    }

    fn render_main_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_space(10.0);
        
        // ═══════════════════════════════════════════════════════
        // LILA MENY-BAR
        // ═══════════════════════════════════════════════════════
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            
            // Tab-knappar
            let tab_style = |active: bool| {
                if active {
                    egui::Button::new(RichText::new("").size(14.0))
                        .fill(PURPLE_MAIN)
                        .stroke(Stroke::new(2.0, PURPLE_LIGHT))
                } else {
                    egui::Button::new(RichText::new("").size(14.0))
                        .fill(PURPLE_DARK)
                        .stroke(Stroke::NONE)
                }
            };

            // Local Model
            let local_btn = ui.add(
                tab_style(self.active_tab == ActiveTab::LocalModel)
                    .min_size(Vec2::new(110.0, 35.0))
            );
            ui.painter().text(
                local_btn.rect.center(),
                egui::Align2::CENTER_CENTER,
                "🖥️ Lokal Modell",
                egui::FontId::proportional(13.0),
                Color32::WHITE,
            );
            if local_btn.clicked() {
                self.active_tab = ActiveTab::LocalModel;
                self.update_provider();
            }

            // Online Model
            let online_btn = ui.add(
                tab_style(self.active_tab == ActiveTab::OnlineModel)
                    .min_size(Vec2::new(110.0, 35.0))
            );
            ui.painter().text(
                online_btn.rect.center(),
                egui::Align2::CENTER_CENTER,
                "☁️ Online Modell",
                egui::FontId::proportional(13.0),
                Color32::WHITE,
            );
            if online_btn.clicked() {
                self.active_tab = ActiveTab::OnlineModel;
                self.update_provider();
            }

            // History
            let history_btn = ui.add(
                tab_style(self.active_tab == ActiveTab::History)
                    .min_size(Vec2::new(90.0, 35.0))
            );
            ui.painter().text(
                history_btn.rect.center(),
                egui::Align2::CENTER_CENTER,
                "📜 Historik",
                egui::FontId::proportional(13.0),
                Color32::WHITE,
            );
            if history_btn.clicked() {
                self.active_tab = ActiveTab::History;
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);

            // Action-knappar
            if self.is_generating {
                // Stop Generating
                if ui.add(egui::Button::new(RichText::new("⏹ Stoppa").color(Color32::WHITE).size(13.0))
                    .fill(Color32::from_rgb(220, 38, 38))
                    .min_size(Vec2::new(90.0, 35.0))).clicked() {
                    self.stop_generating();
                }
            }

            // Cancel
            if self.state != AppState::Idle {
                if ui.add(egui::Button::new(RichText::new("✖ Avbryt").color(Color32::WHITE).size(13.0))
                    .fill(Color32::from_rgb(107, 114, 128))
                    .min_size(Vec2::new(80.0, 35.0))).clicked() {
                    self.cancel_operation();
                }
            }

            // Take Screenshot
            if ui.add(egui::Button::new(RichText::new("📸 Ny Skärmdump").color(Color32::WHITE).size(13.0))
                .fill(PURPLE_MAIN)
                .min_size(Vec2::new(130.0, 35.0))).clicked() {
                self.start_capture();
            }

            // Settings
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                if ui.add(egui::Button::new(RichText::new("⚙").size(18.0).color(Color32::WHITE))
                    .fill(PURPLE_DARK)
                    .min_size(Vec2::new(35.0, 35.0))).clicked() {
                    self.show_settings = !self.show_settings;
                }
            });
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // ═══════════════════════════════════════════════════════
        // HUVUDINNEHÅLL
        // ═══════════════════════════════════════════════════════
        match self.active_tab {
            ActiveTab::LocalModel | ActiveTab::OnlineModel => {
                self.render_analysis_view(ui, ctx);
            }
            ActiveTab::History => {
                self.render_history_view(ui);
            }
            ActiveTab::Settings => {
                self.render_settings_view(ui);
            }
        }

        // Settings popup
        if self.show_settings {
            egui::Window::new("⚙️ Inställningar")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    self.render_settings_view(ui);
                    
                    ui.add_space(10.0);
                    if ui.button("Spara & Stäng").clicked() {
                        self.config.openai_api_key = self.temp_openai_key.clone();
                        self.config.claude_api_key = self.temp_claude_key.clone();
                        let _ = self.config.save();
                        self.show_settings = false;
                        self.update_provider();
                    }
                });
        }
    }

    fn render_analysis_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            // Vänster: Captured image + prompt
            ui.vertical(|ui| {
                ui.set_min_width(300.0);
                
                ui.label(RichText::new("📷 Fångad bild").size(16.0).color(PURPLE_LIGHT));
                ui.add_space(5.0);
                
                // Visa captured image
                if let Some(ref img) = self.captured_region {
                    let size = [img.width() as usize, img.height() as usize];
                    let pixels: Vec<Color32> = img.to_rgba8().pixels()
                        .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                        .collect();
                    
                    let texture = ctx.load_texture(
                        "captured",
                        ColorImage { size, pixels },
                        egui::TextureOptions::default(),
                    );
                    
                    let max_size = Vec2::new(280.0, 200.0);
                    let aspect = img.width() as f32 / img.height() as f32;
                    let display_size = if aspect > 1.0 {
                        Vec2::new(max_size.x, max_size.x / aspect)
                    } else {
                        Vec2::new(max_size.y * aspect, max_size.y)
                    };
                    
                   ui.image(egui::load::SizedTexture::new(&texture, display_size));
                } else {
                    ui.group(|ui| {
                        ui.set_min_size(Vec2::new(280.0, 150.0));
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("Ingen bild fångad\n\nKlicka '📸 Ny Skärmdump'")
                                .size(14.0)
                                .color(Color32::GRAY));
                        });
                    });
                }
                
                ui.add_space(15.0);
                
                // Prompt input
                ui.label(RichText::new("💬 Prompt").size(14.0).color(PURPLE_LIGHT));
                ui.add(egui::TextEdit::multiline(&mut self.custom_prompt)
                    .desired_width(280.0)
                    .desired_rows(3)
                    .hint_text("Beskriv vad AI:n ska göra..."));
                
                ui.add_space(10.0);
                
                // Analyze button
                let can_analyze = self.captured_base64.is_some() && !self.is_generating;
                ui.add_enabled_ui(can_analyze, |ui| {
                    if ui.add(egui::Button::new(RichText::new("🚀 Analysera").size(16.0).color(Color32::WHITE))
                        .fill(PURPLE_MAIN)
                        .min_size(Vec2::new(280.0, 40.0))).clicked() {
                        self.start_ai_analysis();
                    }
                });
                
                // Modell-info
                ui.add_space(10.0);
                let model_info = match &self.ai_client.provider {
                    ModelProvider::Local { model, .. } => format!("🖥️ {}", model),
                    ModelProvider::OpenAI { model, .. } => format!("☁️ OpenAI: {}", model),
                    ModelProvider::Claude { model, .. } => format!("☁️ Claude: {}", model),
                };
                ui.label(RichText::new(model_info).size(12.0).color(Color32::GRAY));
            });
            
            ui.separator();
            
            // Höger: Response
            ui.vertical(|ui| {
                ui.label(RichText::new("🤖 AI-svar").size(16.0).color(PURPLE_LIGHT));
                ui.add_space(5.0);
                
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        let response = self.response_text.lock().unwrap().clone();
                        
                        if self.is_generating {
                            ui.label(RichText::new(format!("{}▌", response))
                                .size(14.0)
                                .color(Color32::WHITE));
                        } else if response.is_empty() {
                            ui.label(RichText::new("Svaret visas här...")
                                .size(14.0)
                                .color(Color32::GRAY));
                        } else {
                            ui.label(RichText::new(&response)
                                .size(14.0)
                                .color(Color32::WHITE));
                        }
                    });
            });
        });
    }

    fn render_history_view(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("📜 Historik").size(18.0).color(PURPLE_LIGHT));
        ui.add_space(10.0);
        
        if self.history.entries.is_empty() {
            ui.label(RichText::new("Ingen historik än. Gör en analys för att börja!")
                .color(Color32::GRAY));
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, entry) in self.history.entries.iter().enumerate().rev() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("#{}", i + 1)).color(PURPLE_LIGHT));
                            ui.label(RichText::new(&entry.timestamp).size(12.0).color(Color32::GRAY));
                        });
                        ui.label(RichText::new(format!("Prompt: {}", &entry.prompt))
                            .size(12.0)
                            .color(Color32::LIGHT_GRAY));
                        ui.add_space(5.0);
                        
                        let preview: String = entry.response.chars().take(200).collect();
                        ui.label(RichText::new(if entry.response.len() > 200 {
                            format!("{}...", preview)
                        } else {
                            preview
                        }).size(13.0).color(Color32::WHITE));
                    });
                    ui.add_space(5.0);
                }
            });
        }
    }

    fn render_settings_view(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("Inställningar").color(PURPLE_LIGHT));
        ui.add_space(15.0);
        
        // Lokal modell
        ui.group(|ui| {
            ui.label(RichText::new("🖥️ Lokal Modell (Ollama)").color(PURPLE_LIGHT));
            ui.horizontal(|ui| {
                ui.label("Endpoint:");
                ui.text_edit_singleline(&mut self.config.local_endpoint);
            });
            ui.horizontal(|ui| {
                ui.label("Modell:");
                ui.text_edit_singleline(&mut self.config.local_model);
            });
        });
        
        ui.add_space(10.0);
        
        // OpenAI
        ui.group(|ui| {
            ui.label(RichText::new("☁️ OpenAI").color(PURPLE_LIGHT));
            ui.horizontal(|ui| {
                ui.label("API Key:");
                ui.add(egui::TextEdit::singleline(&mut self.temp_openai_key).password(true));
            });
            ui.horizontal(|ui| {
                ui.label("Modell:");
                ui.text_edit_singleline(&mut self.config.openai_model);
            });
        });
        
        ui.add_space(10.0);
        
        // Claude
        ui.group(|ui| {
            ui.label(RichText::new("☁️ Claude").color(PURPLE_LIGHT));
            ui.horizontal(|ui| {
                ui.label("API Key:");
                ui.add(egui::TextEdit::singleline(&mut self.temp_claude_key).password(true));
            });
            ui.horizontal(|ui| {
                ui.label("Modell:");
                ui.text_edit_singleline(&mut self.config.claude_model);
            });
        });
        
        ui.add_space(10.0);
        
        // Default prompt
        ui.group(|ui| {
            ui.label(RichText::new("💬 Standard-prompt").color(PURPLE_LIGHT));
            ui.add(egui::TextEdit::multiline(&mut self.config.default_prompt)
                .desired_rows(2)
                .desired_width(f32::INFINITY));
        });
    }
}

// Enkel tidsstämpel utan externa dependencies
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}", secs)
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([700.0, 500.0])
            .with_title("🔮 Screen AI Reader"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Screen AI Reader",
        options,
        Box::new(|_cc| Ok(Box::new(ScreenAiApp::default()))),
    )
}
