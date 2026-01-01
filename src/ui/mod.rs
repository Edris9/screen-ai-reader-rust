pub mod toolbox;
pub mod selecting;
pub mod popup;
pub mod chatsidebar;

use eframe::egui::Color32;

// Tema färger
pub const BG_DARK: Color32 = Color32::from_rgb(32, 32, 32);
pub const BG_DARKER: Color32 = Color32::from_rgb(24, 24, 24);
pub const ACCENT: Color32 = Color32::from_rgb(0, 120, 212);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(180, 180, 180);