use eframe::egui::{self, RichText, Color32, Vec2, ScrollArea, TextEdit, Key, KeyboardShortcut, Modifiers};
use crate::app::{App, ChatMessage, AppState};
use crate::capture;
use super::{BG_DARKER, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY};

pub fn render(app: &mut App, ctx: &egui::Context) {
    egui::SidePanel::right("chat_sidebar")
        .default_width(300.0)
        .width_range(250.0..=500.0)
        .resizable(true)
        .frame(egui::Frame::default().fill(BG_DARKER).inner_margin(10.0))
        .show(ctx, |ui| {
            
            // --- HEADER MED STÄNG-KNAPP ---
            ui.horizontal(|ui| {
                ui.heading(RichText::new("AI Chatt").color(TEXT_PRIMARY));
                
                // Skjut allt till höger
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("✕").size(14.0).color(TEXT_SECONDARY))
                        .on_hover_text("Stäng chatt")
                        .clicked() 
                    {
                        app.show_chat = false;
                    }
                });
            });
            
            ui.add_space(10.0);

            // --- CHATT HISTORIK ---
            ui.vertical(|ui| {
                ui.set_height(ui.available_height() - 100.0);
                
                ScrollArea::vertical()
                    .stick_to_bottom(true)
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

            // --- INPUT OCH KNAPPAR ---
            ui.vertical(|ui| {
                
                // Hantera Enter-tryckning INNAN vi ritar textfältet för att kunna "fånga" det
                // Vi vill skicka om Enter trycks (utan Shift)
                let send_shortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Enter);
                let mut send_pressed = ui.input_mut(|i| i.consume_shortcut(&send_shortcut));
                
                // Om man håller Shift så blir det ny rad (standardbeteende), så vi kollar bara "ren" Enter.
                // TextEdit widgeten nedan kommer hantera Shift+Enter automatiskt.

                let response = ui.add(
                    TextEdit::multiline(&mut app.chat_input)
                        .hint_text("Fråga AI om bilden...")
                        .desired_width(f32::INFINITY)
                        .desired_rows(2)
                );

                // Om vi tryckte Enter OCH textfältet hade fokus -> Skicka
                if send_pressed && response.has_focus() {
                    // Ta bort den nya raden som Enter kanske lade till (om det hanns med)
                    if app.chat_input.ends_with('\n') {
                        app.chat_input.pop();
                    }
                    // Trigga sändning
                    send_pressed = true; 
                    
                    // Behåll fokus i rutan
                    response.request_focus();
                } else {
                    send_pressed = false;
                }

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(5.0, 0.0);

                    if ui.button(RichText::new("📷").size(16.0)).clicked() {
                        // (Kamera logik...)
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

                    if ui.button(RichText::new("📂").size(16.0)).clicked() {
                        // (Upload logik...)
                    }

                    let send_btn = ui.add_enabled(
                        !app.chat_input.trim().is_empty(), 
                        egui::Button::new("Skicka").fill(ACCENT).min_size(Vec2::new(60.0, 26.0))
                    );
                    
                    // Skicka om man klickar ELLER trycker Enter
                    if send_btn.clicked() || (send_pressed && !app.chat_input.trim().is_empty()) {
                        let text = app.chat_input.clone();
                        // Rensa input
                        app.chat_input.clear();
                        
                        // Lägg till i historik
                        app.chat_history.push(ChatMessage { is_user: true, text });
                        
                        // Fake AI svar
                        app.chat_history.push(ChatMessage { 
                            is_user: false, 
                            text: "Jag analyserar bilden... (Här kommer AI-svaret)".to_string() 
                        });
                    }
                });
            });
        });
}