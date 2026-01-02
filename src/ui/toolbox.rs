// ... (imports)
use eframe::egui::{self, RichText, Vec2, Color32};
use crate::app::{App, ModelMode};
use crate::capture;
use super::{BG_DARK, BG_DARKER, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY, popup};

pub fn render(app: &mut App, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(BG_DARK).inner_margin(10.0))
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);
                    
                    // + Nytt (Samma som förut)
                    if ui.add(
                        egui::Button::new(RichText::new("+ Nytt").color(TEXT_PRIMARY).size(13.0))
                            .fill(ACCENT)
                            .rounding(4.0)
                            .min_size(Vec2::new(70.0, 32.0))
                    ).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        app.capture_delay = Some(std::time::Instant::now() + std::time::Duration::from_millis(300));
                        ctx.request_repaint();
                    }
                                    
                    ui.separator();
                    
                    // Lokal / Online (Samma som förut...)
                    let local_selected = app.model_mode == ModelMode::Local;
                    if ui.button("🖥 Lokal").clicked() { app.model_mode = ModelMode::Local; } // (Förenklad kod här för läsbarhet, behåll din styling)
                    if ui.button("☁ Online").clicked() { app.show_online_popup = true; }
                    
                    ui.separator();
                    
                    // --- NYTT: Historik-knappen togglar chatten ---
                    // Vi ändrar färg beroende på om chatten är öppen eller inte
                    let chat_active = app.show_chat && app.screenshot.is_some();
                    let chat_btn_fill = if chat_active { ACCENT } else { BG_DARKER };
                    let chat_btn_text = if chat_active { TEXT_PRIMARY } else { TEXT_SECONDARY };

                    if ui.add(
                        egui::Button::new(RichText::new("📜").color(chat_btn_text).size(16.0))
                            .fill(chat_btn_fill)
                            .rounding(4.0)
                            .min_size(Vec2::new(32.0, 32.0))
                    ).on_hover_text("Visa/Göm Chatt").clicked() {
                        // Toggle logic
                        if app.screenshot.is_some() {
                            app.show_chat = !app.show_chat;
                        } else {
                            // Om ingen bild finns kanske vi vill visa historik ändå? 
                            // För nu gör vi inget om ingen bild finns, eller visar en tom chatt.
                            println!("Ingen bild att chatta om!"); 
                        }
                    }
                    
                    // Inställningar (Samma)
                    if ui.add(egui::Button::new(RichText::new("⚙").size(16.0)).fill(BG_DARKER).min_size(Vec2::new(32.0, 32.0))).clicked() { println!("Inställningar"); }
                });

                // Visa screenshot (Samma)
                if let Some(ref img) = app.screenshot {
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);
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
            });
        });
    
    if app.show_online_popup {
        popup::render_online(app, ctx);
    }
}