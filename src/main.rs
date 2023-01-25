mod app;
mod midi_list;
mod state;

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::Vec2::new(900.0, 600.0));
    native_options.min_window_size = Some(egui::Vec2::new(900.0, 500.0));
    eframe::run_native(
        "Forte",
        native_options,
        Box::new(|cc| Box::new(app::ForteApp::new(cc))),
    );
}
