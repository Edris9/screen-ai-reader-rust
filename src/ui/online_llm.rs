use eframe::egui::{self, RichText, Color32};
use crate::app::{App, OnlineProvider};

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new("Online API Inställningar").strong());
            ui.add_space(5.0);

            // 1. Välj Leverantör
            ui.label("Leverantör:");
            let old_provider = app.online_provider.clone();
            
            ui.horizontal(|ui| {
                ui.selectable_value(&mut app.online_provider, OnlineProvider::OpenAI, "OpenAI");
                ui.selectable_value(&mut app.online_provider, OnlineProvider::Anthropic, "Anthropic");
                ui.selectable_value(&mut app.online_provider, OnlineProvider::Groq, "Groq");
                // Lägg till Grok om du vill
            });
            ui.add_space(5.0);

            // 2. Definiera modell-listor
            // (ID, Visningsnamn)
            let models: Vec<(&str, &str)> = match app.online_provider {
                OnlineProvider::OpenAI => vec![
                    ("gpt-4o", "GPT-4o"),
                    ("gpt-4-turbo", "GPT-4 Turbo"),
                    ("gpt-3.5-turbo", "GPT-3.5 Turbo"),
                ],
                OnlineProvider::Anthropic => vec![
                    ("claude-3-5-sonnet-20240620", "Claude 3.5 Sonnet"),
                    ("claude-3-opus-20240229", "Claude 3 Opus"),
                    ("claude-3-haiku-20240307", "Claude 3 Haiku"),
                ],
                OnlineProvider::Groq => vec![
                    ("llama3-70b-8192", "Llama3 70b"),
                    ("mixtral-8x7b-32768", "Mixtral 8x7b"),
                    ("gemma-7b-it", "Gemma 7b"),
                ],
                OnlineProvider::Grok => vec![
                    ("grok-beta", "Grok Beta"),
                ],
            };

            // 3. SMART LOGIK: Automatisk modell-väljare
            // Om den valda modellen är tom, ELLER om den valda modellen inte finns i listan för nuvarande leverantör:
            // -> Byt automatiskt till den första modellen i listan.
            let current_model_valid = models.iter().any(|(id, _)| *id == app.selected_online_model);
            
            if app.selected_online_model.is_empty() || !current_model_valid {
                if let Some(first) = models.first() {
                    app.selected_online_model = first.0.to_string();
                }
            }

            // 4. API Nyckel
            ui.label("API Nyckel:");
            ui.add(egui::TextEdit::singleline(&mut app.api_key).password(true).desired_width(200.0));
            ui.add_space(5.0);

            // 5. Dropdown för Modeller
            ui.label("Modell:");
            egui::ComboBox::from_id_source("online_model_selector")
                .selected_text(
                    // Hitta det "snygga namnet" för den valda modellen
                    models.iter()
                        .find(|(id, _)| *id == app.selected_online_model)
                        .map(|(_, name)| *name)
                        .unwrap_or(&app.selected_online_model)
                )
                .show_ui(ui, |ui| {
                    for (id, name) in models {
                        ui.selectable_value(&mut app.selected_online_model, id.to_string(), name);
                    }
                });
        });
    });
}