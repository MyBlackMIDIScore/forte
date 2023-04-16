use crate::settings::SingleChannelSettings;
use egui::Ui;
use xsynth_core::channel::ChannelInitOptions;

pub struct EguiChannelConfig {
    limit_layers: bool,
    layer_count: usize,
    init: ChannelInitOptions,
    use_threadpool: bool,
}

impl EguiChannelConfig {
    pub fn new(settings: &SingleChannelSettings) -> Self {
        Self {
            limit_layers: settings.layer_limit_enabled,
            layer_count: settings.layer_limit,
            init: settings.channel_init_options,
            use_threadpool: settings.use_threadpool,
        }
    }

    pub fn save_to_state_settings(&self, settings: &mut SingleChannelSettings) {
        settings.channel_init_options = self.init;
        settings.layer_limit = self.layer_count;
        settings.layer_limit_enabled = self.limit_layers;
        settings.use_threadpool = self.use_threadpool;
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.heading("General");
        ui.separator();
        egui::Grid::new("general_synth_settings_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .min_col_width(140.0)
            .show(ui, |ui| {
                ui.label("Use threadpool: ");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.use_threadpool, "");
                    ui.label("\u{2139}").on_hover_text(
                        "Multithreading between thread for every key.\nThis is better for high voice counts.",
                    );
                });
                ui.end_row();

                ui.label("Fade out voice when killing it: ");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.init.fade_out_killing, "");
                    ui.label("\u{2139}").on_hover_text(
                        "Disabling this may cause clicks,\nbut also improve performance.",
                    );
                });
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
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.limit_layers, "");
                    ui.label("\u{2139}").on_hover_text(
                        "Disabling this will cause the channel(s) to render without a voice limit.",
                    );
                });
                ui.end_row();

                ui.label("Synth Layer Count: ");

                ui.horizontal(|ui| {
                    ui.add_enabled_ui(self.limit_layers, |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.layer_count)
                                .speed(1)
                                .clamp_range(1..=100000),
                        );
                    });

                    ui.label("\u{2139}")
                        .on_hover_text("In a channel, 1 layer is 1 voice per key.");
                });
                ui.end_row();
            });
        ui.add_space(5.0);
    }
}
