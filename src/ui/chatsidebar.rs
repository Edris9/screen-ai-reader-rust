use eframe::egui::{self, RichText, Color32, Vec2, ScrollArea, TextEdit};
use crate::app::{App, ChatMessage, AppState};
use crate::capture;
use super::{BG_DARKER, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY};

pub fn render(app: &mut App, ctx: &egui::Context) {
    // SidePanel::right gör att den hamnar till höger om bilden
    egui::SidePanel::right("chat_sidebar")
        .default_width(300.0)
        .width_range(250.0..=500.0) // Användaren kan dra i bredden
        .resizable(true)
        .frame(egui::Frame::default().fill(BG_DARKER).inner_margin(10.0))
        .show(ctx, |ui| {
            
            ui.heading(RichText::new("AI Chatt").color(TEXT_PRIMARY));
            ui.add_space(10.0);

            // --- 1. CHATT HISTORIK (Tar upp all plats som blir över) ---
            ui.vertical(|ui| {
                ui.set_height(ui.available_height() - 100.0); // Spara plats för input längst ner
                
                ScrollArea::vertical()
                    .stick_to_bottom(true) // Scrolla automatiskt ner
                    .show(ui, |ui| {
                    for msg in &app.chat_history {
                        let (bg_col, align, txt_col) = if msg.is_user {
                            (ACCENT, egui::Align::Max, TEXT_PRIMARY)
                        } else {
                            (Color32::from_gray(50), egui::Align::Min, TEXT_SECONDARY)
                        };

                        ui.with_layout(egui::Layout::top_down(align), |ui| {
                            egui::Frame::none()
                                .fill(bg_col)
                                .rounding(8.0)
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.label(RichText::new(&msg.text).color(txt_col).size(14.0));
                                });
                        });
                        ui.add_space(8.0);
                    }
                });
            });

            ui.separator();

            // --- 2. INPUT OCH KNAPPAR (Längst ner) ---
            ui.vertical(|ui| {
                
                // Input fält
                ui.add(
                    TextEdit::multiline(&mut app.chat_input)
                        .hint_text("Fråga AI om bilden...")
                        .desired_width(f32::INFINITY)
                        .desired_rows(2)
                );

                ui.add_space(5.0);

                // Knapprad
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(5.0, 0.0);

                    // Knapp 1: Ta mer bilder (Kameran)
                    if ui.button(RichText::new("📷").size(16.0)).on_hover_text("Lägg till skärmklipp").clicked() {
                        // Minimera och starta capture igen (Samma logik som i toolbox)
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        if let Some(img) = capture::take_screenshot_delayed() {
                            // Här kan du välja om du ska ERSÄTTA bilden eller LÄGGA TILL i en lista
                            // Just nu ersätter vi för enkelhetens skull:
                            app.screenshot = Some(img);
                            app.screenshot_texture = None;
                            app.selection_start = None;
                            app.selection_current = None;
                            app.state = AppState::Selecting;
                        }
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    }

                    // Knapp 2: Ladda upp fil (Mappen)
                    if ui.button(RichText::new("📂").size(16.0)).on_hover_text("Ladda upp fil").clicked() {
                        // TODO: Implementera filväljare (rfd crate)
                        println!("Öppna filväljare här...");
                        app.chat_history.push(ChatMessage {
                            is_user: false,
                            text: "Funktionen 'Ladda upp' kommer snart!".to_string()
                        });
                    }

                    // Knapp 3: Skicka (Pilen)
                    let send_btn = ui.add_enabled(
                        !app.chat_input.trim().is_empty(), 
                        egui::Button::new("Skicka").fill(ACCENT).min_size(Vec2::new(60.0, 26.0))
                    );
                    
                    if send_btn.clicked() {
                        // Lägg till användarens meddelande
                        let text = app.chat_input.clone();
                        app.chat_history.push(ChatMessage { is_user: true, text });
                        app.chat_input.clear();

                        // Simulera AI svar (detta byter vi ut mot riktig AI sen)
                        app.chat_history.push(ChatMessage { 
                            is_user: false, 
                            text: "Jag analyserar bilden... (Här kommer AI-svaret)".to_string() 
                        });
                    }
                });
            });
        });
}