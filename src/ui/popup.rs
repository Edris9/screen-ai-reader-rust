use eframe::egui::{self, RichText};
use crate::app::{App, ModelMode, OnlineProvider};
use super::{BG_DARK, TEXT_PRIMARY, TEXT_SECONDARY};

pub fn render_online(app: &mut App, ctx: &egui::Context) {
    egui::Window::new("Välj Online Modell")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(egui::Frame::default().fill(BG_DARK).inner_margin(15.0).rounding(8.0))
        .show(ctx, |ui| {
            ui.label(RichText::new("Välj leverantör:").color(TEXT_PRIMARY));
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                if ui.selectable_label(app.online_provider == OnlineProvider::Anthropic, "Anthropic").clicked() {
                    app.online_provider = OnlineProvider::Anthropic;
                }
                if ui.selectable_label(app.online_provider == OnlineProvider::OpenAI, "OpenAI").clicked() {
                    app.online_provider = OnlineProvider::OpenAI;
                }
                if ui.selectable_label(app.online_provider == OnlineProvider::Groq, "Groq").clicked() {
                    app.online_provider = OnlineProvider::Groq;
                }
                if ui.selectable_label(app.online_provider == OnlineProvider::Grok, "Grok").clicked() {
                    app.online_provider = OnlineProvider::Grok;
                }
            });
            
            ui.add_space(10.0);
            ui.label(RichText::new("API Nyckel:").color(TEXT_SECONDARY));
            ui.add(egui::TextEdit::singleline(&mut app.api_key)
                .password(true)
                .desired_width(250.0)
                .hint_text("Skriv din API nyckel här..."));
            
            ui.add_space(15.0);
            ui.horizontal(|ui| {
                if ui.button("Spara").clicked() {
                    app.model_mode = ModelMode::Online;
                    app.show_online_popup = false;
                    println!("Sparade: {:?}", app.online_provider);
                }
                if ui.button("Avbryt").clicked() {
                    app.show_online_popup = false;
                }
            });
        });
}