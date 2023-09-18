use crate::{ICON, VERSION};
use egui::Ui;
use std::env::consts::{ARCH, OS};

pub fn show_about(ui: &mut Ui) {
    ui.horizontal(|ui| {
        let image = image::load_from_memory(ICON).expect("Failed to load icon");
        let image_buffer = image.to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let pixels = image_buffer.to_vec();
        let texture = ui.ctx().load_texture(
            "icon",
            egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
            Default::default(),
        );

        let image_size = 100.0;

        let title_size = 32.0;
        let titleid = egui::FontId {
            size: title_size,
            ..Default::default()
        };

        let title_text = format!("Forte v{}", VERSION);
        let title_galley =
            ui.painter()
                .layout_no_wrap(title_text.to_owned(), titleid, egui::Color32::WHITE);

        let cop_text = "Copyright \u{00A9} MBMS 2023";
        let cop_galley = ui.painter().layout_no_wrap(
            cop_text.to_owned(),
            egui::FontId::default(),
            egui::Color32::WHITE,
        );

        let logo_width = image_size
            + ui.spacing().item_spacing.x
            + title_galley.size().x.max(cop_galley.size().x);
        let space = ui.available_width() / 2.0 - logo_width / 2.0;

        ui.add_space(space);
        ui.image(&texture, egui::Vec2::new(image_size, image_size));

        ui.vertical(|ui| {
            let text_height =
                title_galley.size().y + cop_galley.size().y + ui.spacing().item_spacing.y;
            let space = (image_size - text_height) / 2.0;
            ui.add_space(space);

            ui.label(egui::RichText::new(title_text).size(title_size));
            ui.label(cop_text);
        })
    });

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    ui.heading("Build Information");
    egui::Grid::new("synth_settings_grid")
        .num_columns(2)
        .min_col_width(120.0)
        .show(ui, |ui| {
            ui.label("Operating System:");
            ui.label(format!("{}", OS));
            ui.end_row();

            ui.label("Architecture:");
            ui.label(format!("{}", ARCH));
            ui.end_row();

            ui.label("XSynth Version:");
            ui.label("0.1.0 (Commit cc9bf4a)");
            ui.end_row();

            ui.label("MIDI Toolkit Version:");
            ui.label("0.1.0 (Commit cff22ac)");
            ui.end_row();

            ui.label("Egui Version:");
            ui.label("0.22");
            ui.end_row();
        });

    let gh_text = "\u{1F310} GitHub";
    let gh_galley = ui.painter().layout_no_wrap(
        gh_text.to_owned(),
        egui::FontId::default(),
        egui::Color32::WHITE,
    );

    let mut h = ui.available_height();

    let button_height = ui.spacing().button_padding.y * 2.0 + gh_galley.size().y;
    h -= button_height;
    ui.add_space(h);

    ui.horizontal(|ui| {
        let mut w = ui.available_width();

        let button_width = gh_galley.size().x + ui.spacing().button_padding.x;
        w /= 2.0;
        w -= button_width / 2.0;
        ui.add_space(w);

        if ui.button(gh_text).clicked() {
            open::that("https://github.com/MyBlackMIDIScore/forte").unwrap();
        }
    });
}
