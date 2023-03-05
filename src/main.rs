mod app;
mod elements;
mod errors;
mod settings;
mod tabs;
mod utils;
mod writer;
mod xsynth;

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::Vec2::new(900.0, 600.0));
    native_options.min_window_size = Some(egui::Vec2::new(720.0, 400.0));
    native_options.follow_system_theme = true;
    eframe::run_native(
        "Forte",
        native_options,
        Box::new(|cc| Box::new(app::ForteApp::new(cc))),
    );
}
