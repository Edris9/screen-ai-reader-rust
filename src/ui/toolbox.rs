use eframe::egui::{self, RichText, Vec2, Color32};
use crate::app::{App, ModelMode, SavedChat, ChatMessage}; // Importera SavedChat
use super::{BG_DARK, BG_DARKER, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY, popup};
use chrono::Local; // För datum

pub fn render(app: &mut App, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(BG_DARK).inner_margin(10.0))
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // --- VERKTYGSFÄLT ---
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);
                    
                    // --- KNAPP: + Nytt (Med spara-logik) ---
                    if ui.add(egui::Button::new(RichText::new("+ Nytt").color(TEXT_PRIMARY).size(13.0)).fill(ACCENT).rounding(4.0).min_size(Vec2::new(70.0, 32.0))).clicked() {
                        
                        // 1. Spara nuvarande chatt om den innehåller något vettigt
                        // (Mer än bara start-meddelandet)
                        if app.chat_history.len() > 1 {
                            // Hitta första frågan från användaren för sammanfattning
                            let summary = app.chat_history.iter()
                                .find(|m| m.is_user)
                                .map(|m| {
                                    // Ta max 30 tecken och lägg till "..."
                                    let mut s: String = m.text.chars().take(30).collect();
                                    if m.text.len() > 30 { s.push_str("..."); }
                                    s
                                })
                                .unwrap_or_else(|| "Ny chatt".to_string());

                            // Lägg till i listan
                            app.saved_chats.push(SavedChat {
                                timestamp: Local::now(),
                                summary,
                                mode: app.model_mode.clone(),
                                history: app.chat_history.clone(),
                            });
                            
                            // Sortera så nyaste hamnar överst
                            app.saved_chats.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                        }

                        // 2. Rensa för ny session
                        app.chat_history = vec![
                            ChatMessage { 
                                is_user: false, 
                                text: "Hej! Vad vill du göra?".to_string(),
                                image: None,
                                texture: None
                            }
                        ];
                        app.ollama_error = None;
                        
                        // 3. Ta ny bild (som förut)
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        app.capture_delay = Some(std::time::Instant::now() + std::time::Duration::from_millis(300));
                        ctx.request_repaint();
                    }
                                    
                    ui.separator();
                    
                    // Lokal
                    let local_selected = app.model_mode == ModelMode::Local;
                    let local_btn_text = if local_selected { TEXT_PRIMARY } else { TEXT_SECONDARY };
                    let local_btn_fill = if local_selected { ACCENT } else { BG_DARKER };
                    if ui.add(egui::Button::new(RichText::new("🖥 Lokal").color(local_btn_text).size(12.0)).fill(local_btn_fill).rounding(4.0).min_size(Vec2::new(70.0, 32.0))).clicked() {
                        app.model_mode = ModelMode::Local;
                        if app.local_models.is_empty() {
                            if let Ok(models) = crate::ollama::fetch_models() {
                                app.local_models = models;
                                if let Some(first) = app.local_models.first() {
                                    app.selected_local_model = first.clone();
                                }
                            } else {
                                app.ollama_error = Some("Kunde inte ansluta till Ollama".to_string());
                            }
                        }
                    }
                    
                    // Online
                    let online_selected = !local_selected;
                    let online_btn_text = if online_selected { TEXT_PRIMARY } else { TEXT_SECONDARY };
                    let online_btn_fill = if online_selected { ACCENT } else { BG_DARKER };
                    if ui.add(egui::Button::new(RichText::new("☁ Online").color(online_btn_text).size(12.0)).fill(online_btn_fill).rounding(4.0).min_size(Vec2::new(70.0, 32.0))).clicked() {
                        app.show_online_popup = true;
                    }
                    
                    ui.separator();
                    
                    // --- ERSÄTT HELA DET BLOCKET MED DETTA: ---

                    // Chatt / Historik Toggle
                    // ÄNDRING: Vi kollar inte längre om screenshot finns.
                    let chat_active = app.show_chat; 
                    let chat_btn_fill = if chat_active { ACCENT } else { BG_DARKER };
                    let chat_btn_text = if chat_active { TEXT_PRIMARY } else { TEXT_SECONDARY };

                    // Byt ikon: Visa "Pratbubbla" om vi är i historiken (för att gå tillbaka), annars "Klocka" (för historik)
                    let icon = if app.show_history_view { "💬" } else { "🕘" };

                    if ui.add(
                        egui::Button::new(RichText::new(icon).color(chat_btn_text).size(16.0))
                            .fill(chat_btn_fill)
                            .rounding(4.0)
                            .min_size(Vec2::new(32.0, 32.0))
                    ).clicked() {
                        // ÄNDRING: Vi togglar chatten direkt, oavsett om bild finns eller ej
                        app.show_chat = !app.show_chat;
                    }
                    
                    // Inställningar
                    if ui.add(egui::Button::new(RichText::new("⚙").size(16.0)).fill(BG_DARKER).min_size(Vec2::new(32.0, 32.0))).clicked() { println!("Inställningar"); }
                });

                ui.separator();

                // --- OLLAMA INSTÄLLNINGAR ---
                if app.model_mode == ModelMode::Local {
                    crate::ui::local_llm::render(app, ui);
                    ui.separator();
                }

                // --- BILDVISNING & RADERING ---
                let mut should_delete = false;
                if let Some(ref img) = app.screenshot {
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Granskning").strong().color(TEXT_SECONDARY));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(RichText::new("🗑 Ta bort bild").color(Color32::LIGHT_RED).size(12.0))
                                .on_hover_text("Ta bort bilden och fortsätt chatta med bara text")
                                .clicked() 
                            { should_delete = true; }
                        });
                    });

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let texture = app.screenshot_texture.get_or_insert_with(|| {
                            let size = [img.width() as usize, img.height() as usize];
                            let pixels: Vec<Color32> = img.to_rgba8().pixels()
                                .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                                .collect();
                            ctx.load_texture("captured_slice", egui::ColorImage { size, pixels }, egui::TextureOptions::default())
                        });

                        let available_width = ui.available_width();
                        let aspect = texture.aspect_ratio();
                        let display_size = Vec2::new(available_width, available_width / aspect);
                        ui.image(egui::load::SizedTexture::new(texture.id(), display_size));
                    });
                }
                if should_delete {
                    app.screenshot = None;
                    app.screenshot_texture = None;
                }
            });
        });
    
    if app.show_online_popup {
        popup::render_online(app, ctx);
    }
}