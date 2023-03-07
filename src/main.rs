#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod elements;
mod errors;
mod settings;
mod tabs;
mod utils;
mod writer;
mod xsynth;

use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
};
use tracing::info;

fn main() {
    let file_appender = tracing_appender::rolling::hourly("", "forte.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);
    tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .finish()
            .with(fmt::Layer::default().with_writer(file_writer))
    ).expect("Unable to set global tracing subscriber");

    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::Vec2::new(900.0, 600.0));
    native_options.min_window_size = Some(egui::Vec2::new(720.0, 400.0));
    native_options.follow_system_theme = true;
    info!("Launching Forte");
    eframe::run_native(
        "Forte",
        native_options,
        Box::new(|cc| Box::new(app::ForteApp::new(cc))),
    );
}
