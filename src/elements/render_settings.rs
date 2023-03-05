use crate::settings::{Concurrency, ForteState, RenderMode};
use egui::Ui;
use xsynth_core::ChannelCount;

pub fn show_render_settings(ui: &mut Ui, state: &mut ForteState) {
    ui.heading("Render Settings");
    egui::Grid::new("render_settings_grid")
        .num_columns(2)
        .spacing([5.0, 8.0])
        .show(ui, |ui| {
            let mode = ["Standard", "Realtime Simulation"];
            let mut mode_state = state.render_settings.render_mode.into();

            ui.label("Mode:");
            ui.add_enabled_ui(!state.ui_state.rendering, |ui| {
                egui::ComboBox::from_id_source("mode").show_index(
                    ui,
                    &mut mode_state,
                    mode.len(),
                    |i| mode[i].to_owned(),
                );
            });
            ui.end_row();

            if mode_state != state.render_settings.render_mode.into() {
                match mode_state {
                    0 => state.render_settings.render_mode = RenderMode::Standard,
                    1 => state.render_settings.render_mode = RenderMode::RealtimeSimulation,
                    _ => state.render_settings.render_mode = RenderMode::Standard,
                };
            }

            let conc = ["None", "Items in parallel", "Tracks in parallel (N/A)"];//, "Both"];
            let mut conc_state = state.render_settings.concurrency.into();

            ui.label("Concurrency:");
            ui.add_enabled_ui(!state.ui_state.rendering, |ui| {
                egui::ComboBox::from_id_source("concurrency").show_index(
                    ui,
                    &mut conc_state,
                    conc.len(),
                    |i| conc[i].to_owned(),
                );
            });
            ui.end_row();

            let is_parallel = match state.render_settings.concurrency {
                Concurrency::None | Concurrency::ParallelTracks => false,
                Concurrency::ParallelItems | Concurrency::Both => true,
            };

            ui.label("Max Parallel Items:");
            ui.add_enabled(
                !state.ui_state.rendering && is_parallel,
                egui::DragValue::new(&mut state.render_settings.parallel_midis)
                    .speed(1)
                    .clamp_range(2..=20),
            );
            ui.end_row();

            if conc_state != state.render_settings.concurrency.into() {
                match conc_state {
                    0 => state.render_settings.concurrency = Concurrency::None,
                    1 => state.render_settings.concurrency = Concurrency::ParallelItems,
                    2 => state.render_settings.concurrency = Concurrency::ParallelTracks,
                    3 => state.render_settings.concurrency = Concurrency::Both,
                    _ => state.render_settings.concurrency = Concurrency::None,
                };
            }

            ui.label("Sample Rate: ");
            ui.add_enabled(
                !state.ui_state.rendering,
                egui::DragValue::new(&mut state.render_settings.sample_rate)
                    .speed(100)
                    .clamp_range(8000..=384000),
            );
            ui.end_row();

            let audch = ["Mono", "Stereo"];
            let mut channels_idx = state.render_settings.audio_channels as usize;
            ui.label("Audio Channels: ");
            ui.add_enabled_ui(false, |ui| {
                egui::ComboBox::from_id_source("render_audio_channels_selector").show_index(
                    ui,
                    &mut channels_idx,
                    2,
                    |i| audch[i].to_owned(),
                );
            });

            ui.end_row();
            if channels_idx != state.render_settings.audio_channels as usize {
                state.render_settings.audio_channels = match channels_idx {
                    0 => ChannelCount::Mono,
                    1 => ChannelCount::Stereo,
                    _ => ChannelCount::Stereo,
                };
            }

            ui.label("Apply Audio Limiter: ");
            ui.add_enabled_ui(!state.ui_state.rendering, |ui| {
                ui.checkbox(&mut state.render_settings.use_limiter, "");
            });
            ui.end_row();

            ui.label("Ignore Notes with Velocities Between: ");
            let mut lovel = *state.render_settings.vel_ignore_range.start();
            let mut hivel = *state.render_settings.vel_ignore_range.end();
            ui.add_enabled_ui(!state.ui_state.rendering, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut lovel)
                            .speed(1)
                            .clamp_range(0..=hivel),
                    );
                    ui.label("and");
                    ui.add(
                        egui::DragValue::new(&mut hivel)
                            .speed(1)
                            .clamp_range(0..=127),
                    );
                });
            });
            ui.end_row();
            if lovel != *state.render_settings.vel_ignore_range.start()
                || hivel != *state.render_settings.vel_ignore_range.end()
            {
                state.render_settings.vel_ignore_range = lovel..=hivel;
            }

            ui.label("Realtime Simulation FPS: ");
            let buffer = state.render_settings.realtime_buffer_ms;
            let mut fps = 1000.0 / buffer;
            let fps_prev = fps;
            ui.add_enabled(
                !state.ui_state.rendering,
                egui::DragValue::new(&mut fps)
                    .speed(1.0)
                    .clamp_range(1.0..=100000.0),
            );
            ui.end_row();
            if fps != fps_prev {
                state.render_settings.realtime_buffer_ms = 1000.0 / fps;
            }
        });
}
