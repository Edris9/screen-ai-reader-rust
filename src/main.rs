use eframe::egui::{self, Color32, RichText, Vec2};

// Mörkt tema
const BG_DARK: Color32 = Color32::from_rgb(32, 32, 32);
const BG_DARKER: Color32 = Color32::from_rgb(24, 24, 24);
const ACCENT: Color32 = Color32::from_rgb(0, 120, 212);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255);
const TEXT_SECONDARY: Color32 = Color32::from_rgb(180, 180, 180);

#[derive(PartialEq, Clone)]
enum ModelMode {
    Local,
    Online,
}

struct App {
    model_mode: ModelMode,
}

impl Default for App {
    fn default() -> Self {
        Self {
            model_mode: ModelMode::Local,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(BG_DARK).inner_margin(10.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);
                    
                    // + Nytt
                    if ui.add(
                        egui::Button::new(RichText::new("+ Nytt").color(TEXT_PRIMARY).size(13.0))
                            .fill(ACCENT)
                            .rounding(4.0)
                            .min_size(Vec2::new(70.0, 32.0))
                    ).clicked() {
                        println!("Nytt klickat!");
                    }
                    
                    ui.separator();
                    
                    // Lokal
                    let local_selected = self.model_mode == ModelMode::Local;
                    if ui.add(
                        egui::Button::new(RichText::new("🖥 Lokal").color(if local_selected { TEXT_PRIMARY } else { TEXT_SECONDARY }).size(12.0))
                            .fill(if local_selected { ACCENT } else { BG_DARKER })
                            .rounding(4.0)
                            .min_size(Vec2::new(70.0, 32.0))
                    ).clicked() {
                        self.model_mode = ModelMode::Local;
                    }
                    
                    // Online
                    if ui.add(
                        egui::Button::new(RichText::new("☁ Online").color(if !local_selected { TEXT_PRIMARY } else { TEXT_SECONDARY }).size(12.0))
                            .fill(if !local_selected { ACCENT } else { BG_DARKER })
                            .rounding(4.0)
                            .min_size(Vec2::new(70.0, 32.0))
                    ).clicked() {
                        self.model_mode = ModelMode::Online;
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
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 52.0])
            .with_resizable(false)
            .with_title("Screen AI"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Screen AI",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}