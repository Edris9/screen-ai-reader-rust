use eframe::egui::{self, RichText, Color32, Vec2, ScrollArea, TextEdit, Key, KeyboardShortcut, Modifiers, Stroke, Rounding};
use crate::app::{App, ChatMessage, AppState, ModelMode}; // Lade till ModelMode
use crate::capture;
use super::{BG_DARKER, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY, BG_DARK};

pub fn render(app: &mut App, ctx: &egui::Context) {
    egui::SidePanel::right("chat_sidebar")
        .default_width(320.0) // Lite bredare för historiken
        .resizable(true)
        .frame(egui::Frame::default().fill(BG_DARKER).inner_margin(10.0))
        .show(ctx, |ui| {
            
            // --- HEADER: Chatt vs Historik ---
            ui.horizontal(|ui| {
                if app.show_history_view {
                    ui.heading(RichText::new("🗂 Historik").color(TEXT_PRIMARY));
                } else {
                    ui.heading(RichText::new("AI Chatt").color(TEXT_PRIMARY));
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Stäng-knapp
                    if ui.button(RichText::new("✕").size(14.0).color(TEXT_SECONDARY)).clicked() {
                        app.show_chat = false;
                    }
                    
                    ui.add_space(8.0);
                    
                    // Knapp för att byta vy (Chatt <-> Historik)
                    let icon = if app.show_history_view { "💬" } else { "🕘" };
                    let tooltip = if app.show_history_view { "Tillbaka till chatt" } else { "Visa historik" };
                    
                    if ui.button(RichText::new(icon).size(16.0).color(TEXT_PRIMARY))
                        .on_hover_text(tooltip)
                        .clicked() 
                    {
                        app.show_history_view = !app.show_history_view;
                    }
                });
            });
            ui.add_space(10.0);
            ui.separator();

            // ============================================================
            // VY 1: HISTORIK-LISTA
            // ============================================================
            if app.show_history_view {
                ui.vertical(|ui| {
                    
                    // --- 1. TOGGLE: Lokal / Online (Samma stil som bild) ---
                    ui.add_space(5.0);
                    let desired_width = ui.available_width();
                    let btn_width = (desired_width / 2.0) - 4.0;
                    
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0); // Ihopklistrade knappar

                        // Knapp: Lokal
                        let bg_local = if app.history_filter_local { ACCENT } else { BG_DARK };
                        let txt_local = if app.history_filter_local { TEXT_PRIMARY } else { TEXT_SECONDARY };
                        if ui.add(egui::Button::new(RichText::new("Lokal").color(txt_local))
                            .min_size(Vec2::new(btn_width, 28.0))
                            .fill(bg_local)
                            .rounding(Rounding { nw: 4.0, sw: 4.0, ne: 0.0, se: 0.0 }) // Runda bara vänsterkant
                        ).clicked() {
                            app.history_filter_local = true;
                        }

                        // Knapp: Online
                        let bg_online = if !app.history_filter_local { ACCENT } else { BG_DARK };
                        let txt_online = if !app.history_filter_local { TEXT_PRIMARY } else { TEXT_SECONDARY };
                        if ui.add(egui::Button::new(RichText::new("Online").color(txt_online))
                            .min_size(Vec2::new(btn_width, 28.0))
                            .fill(bg_online)
                            .rounding(Rounding { nw: 0.0, sw: 0.0, ne: 4.0, se: 4.0 }) // Runda bara högerkant
                        ).clicked() {
                            app.history_filter_local = false;
                        }
                    });
                    ui.add_space(15.0);

                    // --- 2. LISTA MED SPARADE CHATTAR ---
                    ScrollArea::vertical().show(ui, |ui| {
                        // Filtrera listan baserat på valet ovan
                        let filtered_chats: Vec<_> = app.saved_chats.iter().enumerate()
                            .filter(|(_, chat)| {
                                match chat.mode {
                                    ModelMode::Local => app.history_filter_local,
                                    ModelMode::Online => !app.history_filter_local,
                                }
                            })
                            .collect();

                        if filtered_chats.is_empty() {
                            ui.label(RichText::new("Ingen historik än...").color(TEXT_SECONDARY).italics());
                            ui.label(RichText::new("Tryck på '+ Nytt' för att spara en pågående chatt.").size(10.0).color(TEXT_SECONDARY));
                        } else {
                            for (index, chat) in filtered_chats {
                                // Design för varje kort i listan
                                egui::Frame::none()
                                    .fill(Color32::from_gray(40))
                                    .rounding(6.0)
                                    .inner_margin(8.0)
                                    .stroke(Stroke::new(1.0, Color32::from_gray(60)))
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        
                                        // Datum & Tid
                                        let date_str = chat.timestamp.format("%Y-%m-%d  %H:%M").to_string();
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(date_str).size(11.0).color(ACCENT));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.button("Ladda").clicked() {
                                                    // Ladda tillbaka chatten!
                                                    app.chat_history = chat.history.clone();
                                                    app.show_history_view = false; // Gå tillbaka till chatt-vyn
                                                }
                                            });
                                        });
                                        
                                        ui.add_space(2.0);
                                        // Sammanfattningen
                                        ui.label(RichText::new(&chat.summary).strong().color(TEXT_PRIMARY));
                                    });
                                ui.add_space(8.0);
                            }
                        }
                    });
                });

            // ============================================================
            // VY 2: AKTIV CHATT (Din gamla kod)
            // ============================================================
            } else {
                ui.vertical(|ui| {
                    ui.set_height(ui.available_height() - 100.0);
                    
                    ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                        for msg in &mut app.chat_history {
                            let (bg_col, align, txt_col) = if msg.is_user {
                                (ACCENT, egui::Align::Max, TEXT_PRIMARY)
                            } else {
                                (Color32::from_gray(50), egui::Align::Min, TEXT_SECONDARY)
                            };

                            ui.with_layout(egui::Layout::top_down(align), |ui| {
                                egui::Frame::none().fill(bg_col).rounding(8.0).inner_margin(8.0).show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        if let Some(img) = &msg.image {
                                            let texture = msg.texture.get_or_insert_with(|| {
                                                let size = [img.width() as usize, img.height() as usize];
                                                let pixels: Vec<Color32> = img.to_rgba8().pixels()
                                                    .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                                                    .collect();
                                                ctx.load_texture("chat_history_img", egui::ColorImage { size, pixels }, egui::TextureOptions::default())
                                            });
                                            
                                            // Responsiv bild
                                            let max_width = 220.0;
                                            let aspect = texture.aspect_ratio();
                                            let size = Vec2::new(max_width, max_width / aspect);
                                            ui.image(egui::load::SizedTexture::new(texture.id(), size));
                                            ui.add_space(5.0);
                                        }
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

                // --- INPUT (Samma som förut) ---
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
                        
                        if (send_btn.clicked() || (send_pressed && !app.chat_input.trim().is_empty())) && !app.is_loading {
                            let text = app.chat_input.clone();
                            app.chat_input.clear();
                            
                            let image_for_history = app.screenshot.clone();

                            app.chat_history.push(ChatMessage { 
                                is_user: true, 
                                text: text.clone(),
                                image: image_for_history, 
                                texture: None 
                            });
                            
                            app.is_loading = true;

                            let sender = app.chat_sender.clone();
                            let model = if app.selected_local_model.is_empty() { "llava".to_string() } else { app.selected_local_model.clone() };
                            let image_for_ai = app.screenshot.clone();
                            
                            // "Tvätta" historiken (trådsäkert)
                            let safe_history: Vec<(bool, String)> = app.chat_history
                                .iter()
                                .map(|msg| (msg.is_user, msg.text.clone()))
                                .collect();

                            std::thread::spawn(move || {
                                let result = crate::ollama::send_chat(model, image_for_ai.as_ref(), &safe_history);
                                match result {
                                    Ok(ai_response) => { let _ = sender.send(ai_response); },
                                    Err(err) => { let _ = sender.send(format!("Fel: {}", err)); }
                                }
                            });
                        }
                    });
                });
            }
        });
}