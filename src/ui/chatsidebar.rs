use eframe::egui::{self, RichText, Color32, Vec2, ScrollArea, TextEdit, Key, KeyboardShortcut, Modifiers};
use crate::app::{App, ChatMessage, AppState};
use crate::capture;
use super::{BG_DARKER, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY};

pub fn render(app: &mut App, ctx: &egui::Context) {
    egui::SidePanel::right("chat_sidebar")
        .default_width(300.0)
        .resizable(true)
        .frame(egui::Frame::default().fill(BG_DARKER).inner_margin(10.0))
        .show(ctx, |ui| {
            
            // Header
            ui.horizontal(|ui| {
                ui.heading(RichText::new("AI Chatt").color(TEXT_PRIMARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("✕").size(14.0).color(TEXT_SECONDARY)).clicked() {
                        app.show_chat = false;
                    }
                });
            });
            ui.add_space(10.0);

            // --- CHATT HISTORIK ---
            ui.vertical(|ui| {
                ui.set_height(ui.available_height() - 100.0);
                
                ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    // VIKTIGT: Vi itererar med 'mut' så vi kan spara texturen i meddelandet
                    for msg in &mut app.chat_history {
                        let (bg_col, align, txt_col) = if msg.is_user {
                            (ACCENT, egui::Align::Max, TEXT_PRIMARY)
                        } else {
                            (Color32::from_gray(50), egui::Align::Min, TEXT_SECONDARY)
                        };

                        ui.with_layout(egui::Layout::top_down(align), |ui| {
                            egui::Frame::none().fill(bg_col).rounding(8.0).inner_margin(8.0).show(ui, |ui| {
                                ui.vertical(|ui| {
                                    // 1. VISA BILD OM DEN FINNS
                                    if let Some(img) = &msg.image {
                                        // Ladda textur om den inte finns cachad
                                        let texture = msg.texture.get_or_insert_with(|| {
                                            let size = [img.width() as usize, img.height() as usize];
                                            let pixels: Vec<Color32> = img.to_rgba8().pixels()
                                                .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                                                .collect();
                                            ctx.load_texture(
                                                "chat_history_img", 
                                                egui::ColorImage { size, pixels }, 
                                                egui::TextureOptions::default()
                                            )
                                        });

                                        // Räkna ut maxbredd så bilden inte spränger chatten
                                        let max_width = 200.0;
                                        let aspect = texture.aspect_ratio();
                                        let size = Vec2::new(max_width, max_width / aspect);
                                        
                                        ui.image(egui::load::SizedTexture::new(texture.id(), size));
                                        ui.add_space(5.0);
                                    }

                                    // 2. VISA TEXT
                                    if !msg.text.is_empty() {
                                        ui.label(RichText::new(&msg.text).color(txt_col).size(14.0));
                                    }
                                });
                            });
                        });
                        ui.add_space(8.0);
                    }
                    
                    if app.is_loading {
                        ui.spinner();
                        ui.label("Tänker...");
                    }
                });
            });
            ui.separator();

            // --- INPUT ---
            ui.vertical(|ui| {
                let send_shortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Enter);
                let mut send_pressed = ui.input_mut(|i| i.consume_shortcut(&send_shortcut));
                
                let response = ui.add(TextEdit::multiline(&mut app.chat_input)
                    .hint_text("Fråga AI om bilden...")
                    .desired_width(f32::INFINITY)
                    .desired_rows(2));

                if send_pressed && response.has_focus() {
                    if app.chat_input.ends_with('\n') { app.chat_input.pop(); }
                    send_pressed = true;
                    response.request_focus();
                } else {
                    send_pressed = false;
                }
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.button("📷").clicked() {
                        // (Kamera-kod samma som förut...)
                         ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        if let Some(img) = capture::take_screenshot_delayed() {
                            app.screenshot = Some(img);
                            app.screenshot_texture = None;
                            app.selection_start = None;
                            app.selection_current = None;
                            app.state = AppState::Selecting;
                            app.show_chat = true;
                        }
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    }
                    if ui.button("📂").clicked() { }

                    let send_btn = ui.add_enabled(!app.chat_input.trim().is_empty() && !app.is_loading, 
                        egui::Button::new("Skicka").fill(ACCENT).min_size(Vec2::new(60.0, 26.0)));
                    
                    // --- LOGIK FÖR ATT SKICKA ---
                    if (send_btn.clicked() || (send_pressed && !app.chat_input.trim().is_empty())) && !app.is_loading {
                        let text = app.chat_input.clone();
                        app.chat_input.clear();
                        
                        // 1. KLONA BILDEN FÖR HISTORIKEN
                        // Vi kollar om det finns en screenshot just nu och sparar den i meddelandet
                        let image_for_history = app.screenshot.clone();

                        // 2. LÄGG TILL I HISTORIKEN (MED BILD)
                        app.chat_history.push(ChatMessage { 
                            is_user: true, 
                            text: text.clone(),
                            image: image_for_history, // Här sparar vi bilden!
                            texture: None 
                        });
                        
                        app.is_loading = true;

                        // 3. FÖRBERED DATA FÖR TRÅDEN
                        let sender = app.chat_sender.clone();
                        let prompt = text;
                        let model = if app.selected_local_model.is_empty() { "llama3".to_string() } else { app.selected_local_model.clone() };
                        
                        // Vi skickar också en kopia av bilden till AI-funktionen
                        let image_for_ai = app.screenshot.clone();
                        let history = app.chat_history.clone();

                        std::thread::spawn(move || {
                            let result = crate::ollama::send_chat(model, prompt, image_for_ai.as_ref(), &history);
                            match result {
                                Ok(ai_response) => { let _ = sender.send(ai_response); },
                                Err(err) => { let _ = sender.send(format!("Fel: {}", err)); }
                            }
                        });
                    }
                });
            });
        });
}