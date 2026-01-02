mod app;
mod capture;
mod ui;
mod ollama;

use app::App;


fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([420.0, 52.0])
            .with_resizable(true) // <--- ÄNDRA HÄR (från false till true)
            .with_min_inner_size([300.0, 52.0]) // <--- LÄGG TILL DENNA (förhindrar att det blir för litet)
            .with_title("Screen AI"),
        ..Default::default()
    };

    
    
    eframe::run_native(
        "Screen AI",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}