use crate::elements::{channel_cfg::EguiChannelConfig, sf_list::EguiSFList};
use crate::settings::ForteState;
use crate::utils::render_in_frame;
use egui::{Context, Ui};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Default)]
enum Panel {
    #[default]
    Soundfonts,
    Settings,
}

#[derive(PartialEq, Eq, Default, Clone, Copy, Serialize, Deserialize)]
pub enum SynthCfgType {
    #[default]
    Global,
    PerChannel,
}

pub struct ForteSynthTab {
    current_panel: Panel,

    sf_global_list: EguiSFList,
    sf_split_lists: Vec<EguiSFList>,
    sf_split_selected: usize,

    channel_cfg_global: EguiChannelConfig,
    channel_cfgs: Vec<EguiChannelConfig>,
    channel_cfg_selected: usize,
}

impl ForteSynthTab {
    pub fn new(state: &ForteState) -> Self {
        let mut sf_split_lists = Vec::new();
        for i in 0..16 {
            sf_split_lists.push(EguiSFList::new(
                state.synth_settings.individual_settings[i]
                    .soundfonts
                    .clone(),
            ));
        }
        let sf_global_list =
            EguiSFList::new(state.synth_settings.global_settings.soundfonts.clone());

        let mut channel_cfgs = Vec::new();
        for i in 0..16 {
            channel_cfgs.push(EguiChannelConfig::new(
                &state.synth_settings.individual_settings[i],
            ));
        }
        let channel_cfg_global = EguiChannelConfig::new(&state.synth_settings.global_settings);

        Self {
            current_panel: Default::default(),
            sf_global_list,
            sf_split_lists,
            sf_split_selected: 0,
            channel_cfg_global,
            channel_cfgs,
            channel_cfg_selected: 0,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, state: &mut ForteState, ctx: &Context) {
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

        match self.current_panel {
            Panel::Soundfonts => {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut state.synth_settings.sfcfg_type,
                        SynthCfgType::Global,
                        "Global",
                    );
                    ui.selectable_value(
                        &mut state.synth_settings.sfcfg_type,
                        SynthCfgType::PerChannel,
                        "Per Channel",
                    );
                    if state.synth_settings.sfcfg_type == SynthCfgType::PerChannel {
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

                if state.synth_settings.sfcfg_type == SynthCfgType::Global {
                    render_in_frame(ui, |ui| {
                        self.sf_global_list.show(ui, ctx);
                    });
                } else {
                    render_in_frame(ui, |ui| {
                        self.sf_split_lists[self.sf_split_selected].show(ui, ctx);
                    });
                }
            }
            Panel::Settings => {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut state.synth_settings.chcfg_type,
                        SynthCfgType::Global,
                        "Global",
                    );
                    ui.selectable_value(
                        &mut state.synth_settings.chcfg_type,
                        SynthCfgType::PerChannel,
                        "Per Channel",
                    );
                    if state.synth_settings.chcfg_type == SynthCfgType::PerChannel {
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

                if state.synth_settings.chcfg_type == SynthCfgType::Global {
                    render_in_frame(ui, |ui| {
                        self.channel_cfg_global.show(ui);
                        ui.allocate_space(ui.available_size());
                    });
                } else {
                    render_in_frame(ui, |ui| {
                        self.channel_cfgs[self.channel_cfg_selected].show(ui);
                        ui.allocate_space(ui.available_size());
                    });
                }
            }
        }

        // Save the settings on every frame
        self.apply_to_state(state);
    }

    pub fn apply_to_state(&self, state: &mut ForteState) {
        state.synth_settings.global_settings.soundfonts = self.sf_global_list.iter_list().collect();
        for i in 0..16 {
            state.synth_settings.individual_settings[i].soundfonts =
                self.sf_split_lists[i].iter_list().collect();
        }

        self.channel_cfg_global
            .save_to_state_settings(&mut state.synth_settings.global_settings);
        for i in 0..16 {
            self.channel_cfgs[i]
                .save_to_state_settings(&mut state.synth_settings.individual_settings[i]);
        }
    }
}
