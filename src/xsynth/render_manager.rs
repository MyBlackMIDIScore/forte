use super::midi_pool::{MIDIPool, MIDIRendererStatus};
use super::soundfont_pool::{SoundfontPool, SoundfontWorkerStatus};
use crate::elements::sf_list::ForteSFListItem;
use crate::errors::error_types::MIDIRendererError;
use crate::settings::ForteState;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use xsynth_core::AudioStreamParams;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ManagerStatus {
    LoadingSoundfonts,
    SoundfontsFinished,
    SFLoadError,
    RenderingMIDIs,
    RenderFinished,
}

pub struct RenderStats {
    pub time: f64,
    pub voice_count: u64,
}

pub struct RenderThreadManager {
    soundfont_pool: SoundfontPool,
    midi_pool: MIDIPool,
}

impl RenderThreadManager {
    pub fn new(state: &ForteState, midis: Vec<PathBuf>) -> Result<Self, MIDIRendererError> {
        let soundfonts = Arc::new(RwLock::new(HashMap::new()));

        let mut soundfonts_paths: Vec<ForteSFListItem> = vec![];
        for channel in state.synth_settings.channel_settings.clone() {
            for sf in channel.soundfonts {
                if soundfonts_paths
                    .clone()
                    .into_iter()
                    .find(|item| item.path == sf.path)
                    .is_none()
                {
                    soundfonts_paths.push(sf);
                }
            }
        }

        let audio_params = AudioStreamParams::new(
            state.render_settings.sample_rate,
            state.render_settings.audio_channels,
        );

        let soundfont_pool = SoundfontPool::new(soundfonts_paths, soundfonts.clone(), audio_params);

        let midi_pool = match MIDIPool::new(state, midis, soundfonts) {
            Ok(pool) => pool,
            Err(err) => return Err(err),
        };

        Ok(Self {
            soundfont_pool,
            midi_pool,
        })
    }

    pub fn status(&mut self) -> ManagerStatus {
        let mut status = match self.soundfont_pool.status() {
            SoundfontWorkerStatus::Loading => ManagerStatus::LoadingSoundfonts,
            SoundfontWorkerStatus::Error => ManagerStatus::SFLoadError,
            SoundfontWorkerStatus::Finished => ManagerStatus::SoundfontsFinished,
        };

        match self.midi_pool.status() {
            MIDIRendererStatus::Rendering => status = ManagerStatus::RenderingMIDIs,
            MIDIRendererStatus::Finished => status = ManagerStatus::RenderFinished,
            _ => {}
        }

        status
    }

    pub fn render(&mut self, state: &ForteState) {
        self.midi_pool.set_soundfonts(state);
        self.midi_pool.run();
    }

    pub fn spawn_next(&mut self) -> bool {
        self.midi_pool.spawn_next()
    }

    pub fn get_stats(&self) -> Vec<Option<RenderStats>> {
        self.midi_pool.get_stats()
    }

    pub fn has_finished(&mut self) -> bool {
        self.midi_pool.has_finished()
    }

    pub fn cancel(&mut self) {
        self.soundfont_pool.cancel();
        self.midi_pool.cancel();
    }
}
