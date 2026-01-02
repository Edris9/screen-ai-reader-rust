use eframe::egui::{self, RichText, Color32};
use crate::app::App;
use crate::ollama;

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new("Lokala Modeller (Ollama)").strong());
            
            // Om vi har ett felmeddelande (t.ex. Ollama är avstängt)
            if let Some(err) = &app.ollama_error {
                ui.label(RichText::new(format!("⚠ {}", err)).color(Color32::RED).size(11.0));
                
                if ui.button("Försök anslut igen").clicked() {
                    refresh_models(app);
                }
            } else {
                // Om listan är tom, försök hämta
                if app.local_models.is_empty() {
                    ui.label("Inga modeller hittades...");
                    if ui.button("Uppdatera lista").clicked() {
                        refresh_models(app);
                    }
                } else {
                    // Visa dropdown
                    ui.horizontal(|ui| {
                        ui.label("Modell:");
                        egui::ComboBox::from_id_source("ollama_selector")
                            .selected_text(&app.selected_local_model)
                            .show_ui(ui, |ui| {
                                for model in &app.local_models {
                                    ui.selectable_value(&mut app.selected_local_model, model.clone(), model);
                                }
                            });
                    });
                }
            }
        });
    });
}

// Hjälpfunktion för att hämta modeller och uppdatera App-state
fn refresh_models(app: &mut App) {
    match ollama::fetch_models() {
        Ok(models) => {
            app.local_models = models;
            app.ollama_error = None;
            // Välj den första modellen automatiskt om ingen är vald
            if !app.local_models.is_empty() && app.selected_local_model.is_empty() {
                app.selected_local_model = app.local_models[0].clone();
            }
        }
        Err(e) => {
            app.ollama_error = Some(e);
            app.local_models.clear();
        }
    }
}