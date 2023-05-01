#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod elements;
mod errors;
mod settings;
mod tabs;
mod utils;
mod writer;
mod xsynth;

use std::backtrace::Backtrace;
use std::fs::File;
use std::io::prelude::*;
use std::panic;
use std::panic::PanicInfo;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt};

const ICON: &[u8; 92050] = include_bytes!("../assets/forte.png");

fn panic_hook(info: &PanicInfo) {
    let backtrace = Backtrace::force_capture();
    let crash_text = format!("{info}\n\n{backtrace}");

    let mut file = File::create("forte-crash.txt").unwrap();
    file.write_all(crash_text.as_bytes()).unwrap_or_default();
}

fn load_icon() -> eframe::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(ICON).expect("Failed to load icon");
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

fn main() {
    panic::set_hook(Box::new(panic_hook));

    let file_appender = tracing_appender::rolling::hourly("", "forte.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);
    tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .finish()
            .with(fmt::Layer::default().with_writer(file_writer)),
    )
    .expect("Unable to set global tracing subscriber");

    let native_options = eframe::NativeOptions {
        icon_data: Some(load_icon()),
        initial_window_size: Some(egui::Vec2::new(900.0, 600.0)),
        min_window_size: Some(egui::Vec2::new(720.0, 400.0)),
        follow_system_theme: true,
        drag_and_drop_support: true,
        ..Default::default()
    };

    info!("Launching Forte");
    eframe::run_native(
        "Forte",
        native_options,
        Box::new(|cc| Box::new(app::ForteApp::new(cc))),
    )
    .unwrap();
}
