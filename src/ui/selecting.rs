use eframe::egui::{self, Color32, Rect, Shape, Stroke};
use crate::app::{App, AppState};

pub fn render(app: &mut App, ctx: &egui::Context) {
    // Sätt cursor till crosshair för snipping-känsla
    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(Color32::BLACK).inner_margin(0.0))
        .show(ctx, |ui| {
            let available_rect = ui.available_rect_before_wrap();
            
            // 1. Visa screenshot som bakgrund (fyller hela fönstret)
            if let Some(ref img) = app.screenshot {
                let texture = app.screenshot_texture.get_or_insert_with(|| {
                    let size = [img.width() as usize, img.height() as usize];
                    let pixels: Vec<Color32> = img.to_rgba8().pixels()
                        .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                        .collect();
                    ctx.load_texture("screenshot", egui::ColorImage { size, pixels }, egui::TextureOptions::default())
                });
                
                // Rita bilden över hela ytan
                ui.painter().image(
                    texture.id(),
                    available_rect,
                    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE
                );
            }

            // Hantera input (musdragning)
            let response = ui.interact(available_rect, ui.id(), egui::Sense::drag());
            
            // Starta dragning
            if response.drag_started() {
                app.selection_start = response.hover_pos();
            }
            
            // Uppdatera nuvarande position under dragning
            if response.dragged() {
                app.selection_current = response.hover_pos();
            }

            // Släpp musen -> KLART
            if response.drag_released() {
                if let (Some(start), Some(end)) = (app.selection_start, app.selection_current) {
                    // Skapa rektangel
                    let rect = Rect::from_two_pos(start, end);
                    
                    // Beskär bilden om vi har en screenshot
                    if let Some(img) = &app.screenshot {
                        // VIKTIGT: Hantera DPI scaling (points vs pixels)
                        let pixels_per_point = ctx.pixels_per_point();
                        
                        let crop_x = (rect.min.x * pixels_per_point) as u32;
                        let crop_y = (rect.min.y * pixels_per_point) as u32;
                        let crop_w = (rect.width() * pixels_per_point) as u32;
                        let crop_h = (rect.height() * pixels_per_point) as u32;

                        // Se till att vi inte kraschar om vi är utanför bildens gränser
                        if crop_w > 0 && crop_h > 0 {
                            let cropped = img.crop_imm(crop_x, crop_y, crop_w, crop_h);
                            app.screenshot = Some(cropped); // Spara den beskurna bilden
                            app.screenshot_texture = None;  // Tvinga omladdning av texturen
                            
                            // Här skulle du byta state till "Analysis" eller liknande
                            // För nu går vi tillbaka till toolbox eller visar resultatet
                            println!("Bild beskuren: {}x{}", crop_w, crop_h);
                            
                            // Återställ fönstret till normal storlek (ej fullscreen)
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                            // För demo, gå tillbaka till toolbox (eller en ny 'Result' state)
                            app.state = AppState::Toolbox; 
                        }
                    }
                }
                
                // Nollställ selection
                app.selection_start = None;
                app.selection_current = None;
            }

            // 2. Rita "Dimmer" och Selections-rektangel
            let painter = ui.painter();
            let overlay_color = Color32::from_black_alpha(150); // Mörkare bakgrund

            if let (Some(start), Some(current)) = (app.selection_start, app.selection_current) {
                let selection_rect = Rect::from_two_pos(start, current);
                
                // Rita 4 rektanglar runt selection för att skapa "hålet"
                // Topp
                painter.rect_filled(
                    Rect::from_min_max(available_rect.min, egui::pos2(available_rect.max.x, selection_rect.min.y)),
                    0.0,
                    overlay_color
                );
                // Botten
                painter.rect_filled(
                    Rect::from_min_max(egui::pos2(available_rect.min.x, selection_rect.max.y), available_rect.max),
                    0.0,
                    overlay_color
                );
                // Vänster
                painter.rect_filled(
                    Rect::from_min_max(
                        egui::pos2(available_rect.min.x, selection_rect.min.y),
                        egui::pos2(selection_rect.min.x, selection_rect.max.y)
                    ),
                    0.0,
                    overlay_color
                );
                // Höger
                painter.rect_filled(
                    Rect::from_min_max(
                        egui::pos2(selection_rect.max.x, selection_rect.min.y),
                        egui::pos2(available_rect.max.x, selection_rect.max.y)
                    ),
                    0.0,
                    overlay_color
                );

                // Rita en vit ram runt selekteringen (Accent-färg eller vit)
                painter.rect_stroke(selection_rect, 0.0, Stroke::new(2.0, Color32::WHITE));

                // Valfritt: Visa dimensioner bredvid musen
                let text = format!("{} x {}", selection_rect.width() as i32, selection_rect.height() as i32);
                painter.text(
                    selection_rect.max + egui::vec2(0.0, 10.0),
                    egui::Align2::LEFT_TOP,
                    text,
                    egui::FontId::proportional(14.0),
                    Color32::WHITE,
                );

            } else {
                // Ingen selektering pågår, gör hela skärmen mörk
                painter.rect_filled(available_rect, 0.0, overlay_color);
            }

            // ESC för att avbryta
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                app.state = AppState::Toolbox;
                app.screenshot = None; // Rensa screenshot om man avbryter
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
        });
}