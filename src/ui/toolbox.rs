use eframe::egui::{self, RichText, Vec2};
use crate::app::{App, AppState, ModelMode};
use crate::capture;
use super::{BG_DARK, BG_DARKER, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY, popup};

pub fn render(app: &mut App, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(BG_DARK).inner_margin(10.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);
                
                // + Nytt
                // I src/ui/toolbox.rs inuti knappen för "Nytt"
                if ui.add(
                    egui::Button::new(RichText::new("+ Nytt").color(TEXT_PRIMARY).size(13.0))
                        .fill(ACCENT)
                        .rounding(4.0)
                        .min_size(Vec2::new(70.0, 32.0))
                ).clicked() {
                    // 1. Minimera toolbox
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    
                    // 2. Ta screenshot efter fördröjning
                    if let Some(img) = capture::take_screenshot_delayed() {
                        app.screenshot = Some(img);
                        app.screenshot_texture = None;
                        app.selection_start = None;   // VIKTIGT: Nollställ
                        app.selection_current = None; // VIKTIGT: Nollställ
                        app.state = AppState::Selecting;
                    }
                    
                    // 3. Öppna i Fullskärm för selektering
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    
                    // 4. Fokusera fönstret
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                                
                ui.separator();
                
                // Lokal
                let local_selected = app.model_mode == ModelMode::Local;
                if ui.add(
                    egui::Button::new(RichText::new("🖥 Lokal").color(if local_selected { TEXT_PRIMARY } else { TEXT_SECONDARY }).size(12.0))
                        .fill(if local_selected { ACCENT } else { BG_DARKER })
                        .rounding(4.0)
                        .min_size(Vec2::new(70.0, 32.0))
                ).clicked() {
                    app.model_mode = ModelMode::Local;
                }
                
                // Online
                if ui.add(
                    egui::Button::new(RichText::new("☁ Online").color(if !local_selected { TEXT_PRIMARY } else { TEXT_SECONDARY }).size(12.0))
                        .fill(if !local_selected { ACCENT } else { BG_DARKER })
                        .rounding(4.0)
                        .min_size(Vec2::new(70.0, 32.0))
                ).clicked() {
                    app.show_online_popup = true;
                }
                
                ui.separator();
                
                // Historik
                if ui.add(
                    egui::Button::new(RichText::new("📜").size(16.0))
                        .fill(BG_DARKER)
                        .rounding(4.0)
                        .min_size(Vec2::new(32.0, 32.0))
                ).on_hover_text("Historik").clicked() {
                    println!("Historik klickat!");
                }
                
                // Inställningar
                if ui.add(
                    egui::Button::new(RichText::new("⚙").size(16.0))
                        .fill(BG_DARKER)
                        .rounding(4.0)
                        .min_size(Vec2::new(32.0, 32.0))
                ).on_hover_text("Inställningar").clicked() {
                    println!("Inställningar klickat!");
                }
            });
        });
    
    // Visa popup om aktiv
    if app.show_online_popup {
        popup::render_online(app, ctx);
    }
}