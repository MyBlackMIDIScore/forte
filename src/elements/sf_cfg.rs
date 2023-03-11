use crate::elements::sf_list::ForteSFListItem;
use egui::{Context, Window};
use xsynth_core::soundfont::Interpolator;

#[derive(Clone)]
pub struct SoundfontConfigWindow {
    pub visible: bool,
    id: usize,
}

impl SoundfontConfigWindow {
    pub fn new(id: usize) -> Self {
        Self { visible: true, id }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn show(&mut self, ctx: &Context, item: &mut ForteSFListItem) {
        let title = if let Some(path) = item.path.file_name() {
            format!("Config for {path:?}")
        } else {
            format!("Config for {}", self.id)
        };

        Window::new(title)
            .id(egui::Id::new(self.id))
            .open(&mut self.visible)
            .show(ctx, |ui| {
                let col_width = 80.0;

                ui.heading("Instrument");
                ui.separator();
                ui.label("XSynth currently doesn't support instrument settings.");
                egui::Grid::new("sfconfig_window_instr")
                    .num_columns(2)
                    .min_col_width(col_width)
                    .show(ui, |ui| {
                        ui.set_enabled(false);
                        ui.label("Bank: ");
                        ui.add(
                            egui::DragValue::new(&mut item.pref.bank)
                                .speed(1)
                                .clamp_range(0..=127),
                        );
                        ui.end_row();

                        ui.label("Preset: ");
                        ui.add(
                            egui::DragValue::new(&mut item.pref.preset)
                                .speed(1)
                                .clamp_range(0..=127),
                        );
                        ui.end_row();
                    });

                ui.heading("Settings");
                ui.separator();
                egui::Grid::new("sfconfig_window_settings")
                    .num_columns(2)
                    .min_col_width(col_width)
                    .show(ui, |ui| {
                        ui.label("Linear Release Envelope: ");
                        ui.checkbox(&mut item.pref.init.linear_release, "");
                        ui.end_row();

                        let interp = ["Nearest Neighbor", "Linear"];
                        let mut interp_idx = item.pref.init.interpolator as usize;

                        ui.label("Interpolation:");
                        egui::ComboBox::from_id_source("interpolation").show_index(
                            ui,
                            &mut interp_idx,
                            interp.len(),
                            |i| interp[i].to_owned(),
                        );
                        ui.end_row();

                        if interp_idx != item.pref.init.interpolator as usize {
                            match interp_idx {
                                0 => item.pref.init.interpolator = Interpolator::Nearest,
                                1 => item.pref.init.interpolator = Interpolator::Linear,
                                _ => item.pref.init.interpolator = Interpolator::Nearest,
                            };
                        }
                    });
            });
    }
}
