use crate::utils::set_button_spacing;
use egui::Ui;

pub fn show_about(ui: &mut Ui) {
    ui.heading("Forte v0.1.0");
    ui.label("Copyright \u{00A9} MBMS 2023");

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
            let build_num: usize = include!("../../build.number");
            ui.label(build_num.to_string());
            ui.end_row();

            ui.label("XSynth Version:");
            ui.label("0.1.0 (Commit 2ef486b)");
            ui.end_row();

            ui.label("MIDI Toolkit Version:");
            ui.label("0.1.0 (Commit 4482d61)");
            ui.end_row();

            ui.label("Egui Version:");
            ui.label("0.20");
            ui.end_row();
        });

    ui.separator();

    set_button_spacing(ui);
    ui.horizontal(|ui| {
        if ui.button("\u{1F5A5} Check for updates").clicked() {}
        if ui.button("\u{1F310} GitHub").clicked() {}
    });
}