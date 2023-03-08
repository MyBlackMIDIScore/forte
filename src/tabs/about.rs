use crate::utils::set_button_spacing;
use egui::Ui;
use crate::{app::add_gui_error, ICON};
use tracing::info;

pub fn show_about(ui: &mut Ui) {
    ui.horizontal(|ui| {
        let mut image = image::load_from_memory(ICON).expect("Failed to load icon");
        if ui.ctx().style().visuals.dark_mode {
            image.invert();
        }
        let image_buffer = image.to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let pixels = image_buffer.to_vec();
        let texture = ui.ctx().load_texture("icon", egui::ColorImage::from_rgba_unmultiplied(size, &pixels), Default::default());

        ui.image(&texture, egui::Vec2::new(40.0, 40.0));

        ui.vertical(|ui| {
            ui.heading("Forte v0.1.0");
            ui.label("Copyright \u{00A9} MBMS 2023");
        })
    });

    ui.separator();

    ui.heading("Build Information");
    egui::Grid::new("synth_settings_grid")
        .num_columns(2)
        .min_col_width(120.0)
        .show(ui, |ui| {
            ui.label("Architecture:");
            ui.label("x64");
            ui.end_row();

            ui.label("Build Number:");
            let build_num = String::from_utf8_lossy(include_bytes!("../../build.number"));
            ui.label(&build_num.trim()[6..]);
            ui.end_row();

            ui.label("XSynth Version:");
            ui.label("0.1.0 (Commit 2ef486b)");
            ui.end_row();

            ui.label("MIDI Toolkit Version:");
            ui.label("0.1.0 (Commit 4482d61)");
            ui.end_row();

            ui.label("Egui Version:");
            ui.label("0.21");
            ui.end_row();
        });

    ui.separator();

    set_button_spacing(ui);
    ui.horizontal(|ui| {
        if ui.button("\u{1F5A5} Check for updates").clicked() {
            info!("No updates found");
            add_gui_error("No Updates Found".to_owned(), "Forte is all up to date!".to_owned());
        }
        if ui.button("\u{1F310} GitHub").clicked() {}
    });
}
