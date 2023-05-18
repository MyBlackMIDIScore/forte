use crate::settings::{ForteState, OutputAudioFormat, PCMSampleFormat, RenderMode};
use crate::writer::{COMMON_BITRATES, COMMON_SAMPLE_RATES};
use egui::Ui;

pub fn show_render_settings(ui: &mut Ui, state: &mut ForteState) {
    let large_label = "Ignore Notes with Velocities Between: ";
    let label_size = ui
        .painter()
        .layout_no_wrap(
            large_label.to_owned(),
            Default::default(),
            egui::Color32::WHITE,
        )
        .size()
        .x;

    ui.heading("Render Settings");
    egui::Grid::new("render_settings_grid")
        .num_columns(2)
        .spacing([5.0, 8.0])
        .min_col_width(label_size)
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

            ui.label("Max Parallel MIDIs:");
            ui.add_enabled(
                !state.ui_state.rendering,
                egui::DragValue::new(&mut state.render_settings.parallel_midis)
                    .speed(1)
                    .clamp_range(1..=20),
            );
            ui.end_row();

            ui.label(large_label);
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

    ui.add_space(5.0);

    ui.heading("Output Settings");
    egui::Grid::new("output_settings_grid")
        .num_columns(2)
        .spacing([5.0, 8.0])
        .min_col_width(label_size)
        .show(ui, |ui| {
            // Not supported by XSynth currently
            /*
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
            */

            ui.label("Sample Rate: ");
            ui.add_enabled_ui(!state.ui_state.rendering, |ui| {
                egui::ComboBox::from_id_source("render_sample_rate_selector")
                    .selected_text(format!("{}", state.render_settings.sample_rate))
                    .show_ui(ui, |ui| {
                        for r in COMMON_SAMPLE_RATES {
                            ui.selectable_value(
                                &mut state.render_settings.sample_rate,
                                r,
                                format!("{r}"),
                            );
                        }
                    })
            });
            ui.end_row();

            #[derive(PartialEq, Debug)]
            enum TemporaryAudioFormat {
                Wav,
                Ogg,
                Mp3,
            }

            let (mut tmpformat, mut pcmformat, mut bitrate) =
                match &state.render_settings.audio_format {
                    OutputAudioFormat::Pcm { format } => {
                        (TemporaryAudioFormat::Wav, *format, 192000u32)
                    }
                    OutputAudioFormat::Vorbis { bitrate } => (
                        TemporaryAudioFormat::Ogg,
                        PCMSampleFormat::Float32,
                        *bitrate,
                    ),
                    OutputAudioFormat::Lame { bitrate } => (
                        TemporaryAudioFormat::Mp3,
                        PCMSampleFormat::Float32,
                        *bitrate,
                    ),
                };

            ui.label("Audio Format: ");
            ui.add_enabled_ui(!state.ui_state.rendering, |ui| {
                egui::ComboBox::from_id_source("render_audio_format_selector")
                    .selected_text(format!("{:?}", tmpformat))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut tmpformat, TemporaryAudioFormat::Wav, "WAV");
                        ui.selectable_value(&mut tmpformat, TemporaryAudioFormat::Ogg, "OGG");
                        ui.selectable_value(&mut tmpformat, TemporaryAudioFormat::Mp3, "MP3");
                    })
            });
            ui.end_row();

            match tmpformat {
                TemporaryAudioFormat::Wav => {
                    ui.label("WAV Sample Format: ");
                    ui.add_enabled_ui(!state.ui_state.rendering, |ui| {
                        egui::ComboBox::from_id_source("render_pcm_format_selector")
                            .selected_text(format!("{}", pcmformat))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut pcmformat,
                                    PCMSampleFormat::Int16,
                                    "16-bit integer",
                                );
                                ui.selectable_value(
                                    &mut pcmformat,
                                    PCMSampleFormat::Float32,
                                    "32-bit float",
                                );
                            })
                    });
                    ui.end_row();

                    state.render_settings.audio_format =
                        OutputAudioFormat::Pcm { format: pcmformat };
                }
                TemporaryAudioFormat::Ogg | TemporaryAudioFormat::Mp3 => {
                    ui.label("Bitrate: ");
                    ui.add_enabled_ui(!state.ui_state.rendering, |ui| {
                        egui::ComboBox::from_id_source("render_bitrate_selector")
                            .selected_text(format!("{}kbps", bitrate / 1000))
                            .show_ui(ui, |ui| {
                                for r in COMMON_BITRATES {
                                    ui.selectable_value(
                                        &mut bitrate,
                                        r,
                                        format!("{}kbps", r / 1000),
                                    );
                                }
                            })
                    });
                    ui.end_row();

                    match tmpformat {
                        TemporaryAudioFormat::Ogg => {
                            state.render_settings.audio_format =
                                OutputAudioFormat::Vorbis { bitrate }
                        }
                        TemporaryAudioFormat::Mp3 => {
                            state.render_settings.audio_format = OutputAudioFormat::Lame { bitrate }
                        }
                        _ => {}
                    }
                }
            }
        });

    ui.add_space(5.0);

    ui.heading("DSP Settings");
    egui::Grid::new("dsp_settings_grid")
        .num_columns(2)
        .spacing([5.0, 8.0])
        .min_col_width(label_size)
        .show(ui, |ui| {
            ui.label("Apply Audio Limiter: ");
            ui.add_enabled_ui(!state.ui_state.rendering, |ui| {
                ui.checkbox(&mut state.render_settings.dsp_settings.limiter.enabled, "");
            });
            ui.end_row();

            ui.label("Limiter Release (ms): ");
            ui.add_enabled(
                !state.ui_state.rendering && state.render_settings.dsp_settings.limiter.enabled,
                egui::DragValue::new(&mut state.render_settings.dsp_settings.limiter.release_ms)
                    .speed(1)
                    .clamp_range(10..=800),
            );
            ui.end_row();

            ui.label("Limiter Attack (ms): ");
            ui.add_enabled(
                !state.ui_state.rendering && state.render_settings.dsp_settings.limiter.enabled,
                egui::DragValue::new(&mut state.render_settings.dsp_settings.limiter.attack_ms)
                    .speed(0.2)
                    .clamp_range(10..=200),
            );
            ui.end_row();
        });
}
