use eframe::egui::{self, RichText, Vec2, Color32}; // Se till att Color32 är med
use crate::app::{App, AppState, ModelMode};
use crate::capture;
use super::{BG_DARK, BG_DARKER, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY, popup};

pub fn render(app: &mut App, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(BG_DARK).inner_margin(10.0))
        .show(ctx, |ui| {
            // Vi använder vertical layout för att lägga bilden under knapparna
            ui.vertical(|ui| {
                // --- Verktygsfältet (Samma som förut) ---
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);
                    
                    // + Nytt
                    if ui.add(
                        egui::Button::new(RichText::new("+ Nytt").color(TEXT_PRIMARY).size(13.0))
                            .fill(ACCENT)
                            .rounding(4.0)
                            .min_size(Vec2::new(70.0, 32.0))
                    ).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        if let Some(img) = capture::take_screenshot_delayed() {
                            app.screenshot = Some(img);
                            app.screenshot_texture = None;
                            app.selection_start = None;
                            app.selection_current = None;
                            app.state = AppState::Selecting;
                        }
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    }
                                    
                    ui.separator();
                    
                    // Lokal / Online knappar (Samma som förut)
                    let local_selected = app.model_mode == ModelMode::Local;
                    if ui.add(
                        egui::Button::new(RichText::new("🖥 Lokal").color(if local_selected { TEXT_PRIMARY } else { TEXT_SECONDARY }).size(12.0))
                            .fill(if local_selected { ACCENT } else { BG_DARKER })
                            .rounding(4.0)
                            .min_size(Vec2::new(70.0, 32.0))
                    ).clicked() {
                        app.model_mode = ModelMode::Local;
                    }
                    
                    if ui.add(
                        egui::Button::new(RichText::new("☁ Online").color(if !local_selected { TEXT_PRIMARY } else { TEXT_SECONDARY }).size(12.0))
                            .fill(if !local_selected { ACCENT } else { BG_DARKER })
                            .rounding(4.0)
                            .min_size(Vec2::new(70.0, 32.0))
                    ).clicked() {
                        app.show_online_popup = true;
                    }
                    
                    ui.separator();
                    
                    // Ikoner
                    if ui.add(egui::Button::new(RichText::new("📜").size(16.0)).fill(BG_DARKER).min_size(Vec2::new(32.0, 32.0))).clicked() { println!("Historik"); }
                    if ui.add(egui::Button::new(RichText::new("⚙").size(16.0)).fill(BG_DARKER).min_size(Vec2::new(32.0, 32.0))).clicked() { println!("Inställningar"); }
                });

                // --- HÄR ÄR NYHETEN: Visa bilden ---
                if let Some(ref img) = app.screenshot {
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Skapa rullningsbart område om bilden är stor
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let texture = app.screenshot_texture.get_or_insert_with(|| {
                            let size = [img.width() as usize, img.height() as usize];
                            let pixels: Vec<Color32> = img.to_rgba8().pixels()
                                .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                                .collect();
                            ctx.load_texture("captured_slice", egui::ColorImage { size, pixels }, egui::TextureOptions::default())
                        });

                        // Räkna ut storlek för att passa fönstrets bredd (responsivt)
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