mod app;
mod config;
mod events;
mod state;
mod ui;

use app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Ritmo - Library Manager"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Ritmo",
        options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
}
