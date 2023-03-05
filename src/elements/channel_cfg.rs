use crate::settings::SingleChannelSettings;
use egui::Ui;
use xsynth_core::channel::ChannelInitOptions;

pub struct EguiChannelConfig {
    limit_layers: bool,
    layer_count: usize,
    init: ChannelInitOptions,
    use_threadpool: bool,
    volume: f32,
}

impl EguiChannelConfig {
    pub fn new() -> Self {
        Self {
            limit_layers: true,
            layer_count: 10,
            init: SingleChannelSettings::default().channel_init_options,
            use_threadpool: false,
            volume: 1.0,
        }
    }

    pub fn save_to_state_settings(&self, settings: &mut SingleChannelSettings) {
        settings.channel_init_options = self.init;
        settings.layer_limit = match self.limit_layers {
            true => Some(self.layer_count),
            false => None,
        };
        settings.use_threadpool = self.use_threadpool;
        settings.volume = self.volume;
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.heading("General");
        ui.separator();
        egui::Grid::new("general_synth_settings_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .min_col_width(140.0)
            .show(ui, |ui| {
                /*ui.label("Volume: ");
                ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0));
                ui.end_row();*/

                ui.label("Use threadpool: ");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.use_threadpool, "");
                    ui.label("\u{2139}").on_hover_text(
                        "Enabling this will spawn one thread per key (per channel).",
                    );
                });
                ui.end_row();

                ui.label("Fade out voice when killing it: ");
                ui.checkbox(&mut self.init.fade_out_killing, "");
                ui.end_row();
            });
        ui.add_space(5.0);

        ui.heading("Layer Limit");
        ui.separator();
        egui::Grid::new("layerlimit_synth_settings_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .min_col_width(140.0)
            .show(ui, |ui| {
                ui.label("Limit Layers: ");
                ui.checkbox(&mut self.limit_layers, "");
                ui.end_row();

                ui.label("Synth Layer Count: ");
                ui.add_enabled_ui(self.limit_layers, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.layer_count)
                                .speed(1)
                                .clamp_range(1..=100000),
                        );
                        ui.label("\u{2139}")
                            .on_hover_text("In a channel, 1 layer is 1 voice per key.");
                    });
                });
                ui.end_row();
            });
        ui.add_space(5.0);
    }
}
