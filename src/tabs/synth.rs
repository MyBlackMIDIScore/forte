use crate::elements::{channel_cfg::EguiChannelConfig, sf_list::EguiSFList};
use crate::settings::ForteState;
use crate::utils::render_in_frame;
use egui::{Context, Ui};

#[derive(PartialEq, Eq, Default)]
enum Panel {
    #[default]
    Soundfonts,
    Settings,
}

#[derive(PartialEq, Eq, Default, Clone)]
pub enum SynthCfgType {
    #[default]
    Global,
    PerChannel,
}

pub struct ForteSynthTab {
    current_panel: Panel,
    sf_load_type: SynthCfgType,
    channel_cfg_type: SynthCfgType,

    sf_global_list: EguiSFList,
    sf_split_lists: Vec<EguiSFList>,
    sf_split_selected: usize,

    channel_cfg_global: EguiChannelConfig,
    channel_cfgs: Vec<EguiChannelConfig>,
    channel_cfg_selected: usize,
}

impl ForteSynthTab {
    pub fn new() -> Self {
        let mut sf_split_lists = Vec::new();
        for _ in 0..16 {
            sf_split_lists.push(EguiSFList::new());
        }

        let mut channel_cfgs = Vec::new();
        for _ in 0..16 {
            channel_cfgs.push(EguiChannelConfig::new());
        }

        Self {
            current_panel: Default::default(),
            sf_load_type: SynthCfgType::Global,
            channel_cfg_type: SynthCfgType::Global,
            sf_global_list: EguiSFList::new(),
            sf_split_lists,
            sf_split_selected: 0,
            channel_cfg_global: EguiChannelConfig::new(),
            channel_cfgs,
            channel_cfg_selected: 0,
        }
    }

    pub fn show<E>(
        &mut self,
        ui: &mut Ui,
        state: &mut ForteState,
        ctx: &Context,
        errors_callback: E,
    ) where
        E: FnMut(String, String),
    {
        ui.horizontal(|ui| {
            ui.heading("Synthesizer Configuration");
            ui.separator();
            ui.selectable_value(
                &mut self.current_panel,
                Panel::Soundfonts,
                "\u{1F50A} SoundFonts",
            );
            ui.selectable_value(
                &mut self.current_panel,
                Panel::Settings,
                "\u{2699} Channel Settings",
            );
        });
        ui.add_space(5.0);

        egui::TopBottomPanel::bottom("soundfonts_bottom_panel")
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                ui.add_space(5.0);
                if ui.button("Apply").clicked() {
                    self.apply_to_state(state);
                }
            });

        match self.current_panel {
            Panel::Soundfonts => {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.sf_load_type,
                        SynthCfgType::Global,
                        "Global",
                    );
                    ui.selectable_value(
                        &mut self.sf_load_type,
                        SynthCfgType::PerChannel,
                        "Per Channel",
                    );
                    if self.sf_load_type == SynthCfgType::PerChannel {
                        ui.separator();
                        let mut chvec: Vec<String> = Vec::new();
                        for i in 1..=16 {
                            chvec.push(format!("Channel {i}"));
                        }

                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_source("sf_split_selector").show_index(
                                ui,
                                &mut self.sf_split_selected,
                                16,
                                |i| chvec[i].clone(),
                            );
                        });
                    }
                });
                ui.add_space(5.0);

                match self.sf_load_type {
                    SynthCfgType::Global => {
                        render_in_frame(ui, |ui| {
                            self.sf_global_list.show(ui, ctx, errors_callback);
                        });
                    }
                    SynthCfgType::PerChannel => {
                        render_in_frame(ui, |ui| {
                            self.sf_split_lists[self.sf_split_selected].show(
                                ui,
                                ctx,
                                errors_callback,
                            );
                        });
                    }
                }
            }
            Panel::Settings => {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.channel_cfg_type,
                        SynthCfgType::Global,
                        "Global",
                    );
                    ui.selectable_value(
                        &mut self.channel_cfg_type,
                        SynthCfgType::PerChannel,
                        "Per Channel",
                    );
                    if self.channel_cfg_type == SynthCfgType::PerChannel {
                        ui.separator();
                        let mut chvec: Vec<String> = Vec::new();
                        for i in 1..=16 {
                            chvec.push(format!("Channel {i}"));
                        }

                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_source("channel_cfg_selector").show_index(
                                ui,
                                &mut self.channel_cfg_selected,
                                16,
                                |i| chvec[i].clone(),
                            );
                        });
                    }
                });
                ui.add_space(5.0);

                match self.channel_cfg_type {
                    SynthCfgType::Global => {
                        render_in_frame(ui, |ui| {
                            self.channel_cfg_global.show(ui);
                            ui.allocate_space(ui.available_size());
                        });
                    }
                    SynthCfgType::PerChannel => {
                        render_in_frame(ui, |ui| {
                            self.channel_cfgs[self.channel_cfg_selected].show(ui);
                            ui.allocate_space(ui.available_size());
                        });
                    }
                }
            }
        }
    }

    fn apply_to_state(&self, state: &mut ForteState) {
        match self.sf_load_type {
            SynthCfgType::Global => {
                for channel in state.synth_settings.channel_settings.iter_mut() {
                    channel.soundfonts = self.sf_global_list.iter_list().collect();
                }
            },
            SynthCfgType::PerChannel => {
                for (idx, list) in self.sf_split_lists.iter().enumerate() {
                    state.synth_settings.channel_settings[idx].soundfonts = list.iter_list().collect();
                }
            },
        }

        match self.channel_cfg_type {
            SynthCfgType::Global => {
                for channel in state.synth_settings.channel_settings.iter_mut() {
                    self.channel_cfg_global.save_to_state_settings(channel);
                }
            },
            SynthCfgType::PerChannel => {
                for (idx, list) in self.channel_cfgs.iter().enumerate() {
                    list.save_to_state_settings(&mut state.synth_settings.channel_settings[idx]);
                }
            },
        }
    }
}
