use crate::dsp::DSPSettings;
use crate::elements::sf_list::ForteSFListItem;
use crate::tabs::ForteTab;
use crate::tabs::SynthCfgType;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use tracing::{info, warn};
use xsynth_core::channel::ChannelInitOptions;
use xsynth_core::ChannelCount;

#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub enum RenderMode {
    #[default]
    Standard = 0,
    RealtimeSimulation = 1,
}

impl From<RenderMode> for usize {
    fn from(val: RenderMode) -> Self {
        val as usize
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(remote = "ChannelInitOptions", default)]
pub struct ChannelInitOptionsDef {
    pub fade_out_killing: bool,
    pub drums_only: bool
}

impl Default for ChannelInitOptionsDef {
    fn default() -> Self {
        Self {
            fade_out_killing: true,
            drums_only: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SingleChannelSettings {
    #[serde(with = "ChannelInitOptionsDef")]
    pub channel_init_options: ChannelInitOptions,
    pub layer_limit: usize,
    pub layer_limit_enabled: bool,
    pub soundfonts: Vec<ForteSFListItem>,
    pub use_threadpool: bool,
}

impl Default for SingleChannelSettings {
    fn default() -> Self {
        let channel_init_options = ChannelInitOptions {
            fade_out_killing: true,
            drums_only: false,
        };

        Self {
            channel_init_options,
            layer_limit: 16,
            layer_limit_enabled: true,
            soundfonts: Vec::new(),
            use_threadpool: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SynthSettings {
    pub sfcfg_type: SynthCfgType,
    pub chcfg_type: SynthCfgType,
    pub global_settings: SingleChannelSettings,
    pub individual_settings: Vec<SingleChannelSettings>,
}

impl Default for SynthSettings {
    fn default() -> Self {
        Self {
            sfcfg_type: SynthCfgType::Global,
            chcfg_type: SynthCfgType::Global,
            global_settings: Default::default(),
            individual_settings: vec![Default::default(); 16],
        }
    }
}

impl SynthSettings {
    pub fn unify(&self) -> Vec<SingleChannelSettings> {
        let mut vec = vec![SingleChannelSettings::default(); 16];

        // Save the channel settings first because the config type might be different
        // for the soundfonts and it may override the first values

        match self.chcfg_type {
            SynthCfgType::Global => {
                for c in vec.iter_mut() {
                    *c = self.global_settings.clone();
                }
            }
            SynthCfgType::PerChannel => {
                vec[..16].clone_from_slice(&self.individual_settings[..16]);
            }
        }

        match self.sfcfg_type {
            SynthCfgType::Global => {
                for c in vec.iter_mut() {
                    c.soundfonts = self.global_settings.soundfonts.clone();
                }
            }
            SynthCfgType::PerChannel => {
                for (idx, c) in vec.iter_mut().enumerate() {
                    c.soundfonts = self.individual_settings[idx].soundfonts.clone();
                }
            }
        }

        vec
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "ChannelCount")]
pub enum ChannelCountDef {
    Mono,
    Stereo,
}

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Debug)]
pub enum PCMSampleFormat {
    Int16,
    Float32,
}

impl std::fmt::Display for PCMSampleFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PCMSampleFormat::Int16 => write!(f, "16-bit integer"),
            PCMSampleFormat::Float32 => write!(f, "32-bit float"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "args")]
pub enum OutputAudioFormat {
    Pcm { format: PCMSampleFormat },
    Vorbis { bitrate: u32 },
    Lame { bitrate: u32 },
}

impl std::fmt::Display for OutputAudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OutputAudioFormat::Pcm { .. } => write!(f, "WAV"),
            OutputAudioFormat::Vorbis { .. } => write!(f, "OGG"),
            OutputAudioFormat::Lame { .. } => write!(f, "MP3"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderSettings {
    pub sample_rate: u32,
    #[serde(with = "ChannelCountDef")]
    pub audio_channels: ChannelCount,
    pub dsp_settings: DSPSettings,
    pub render_mode: RenderMode,
    pub vel_ignore_range: RangeInclusive<u8>,
    pub realtime_buffer_ms: f32,
    pub parallel_midis: usize,
    pub output_dir: Option<PathBuf>,
    pub audio_format: OutputAudioFormat,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            audio_channels: ChannelCount::Stereo,
            dsp_settings: Default::default(),
            render_mode: RenderMode::Standard,
            vel_ignore_range: 0..=0,
            realtime_buffer_ms: 100.0 / 6.0,
            output_dir: None,
            parallel_midis: 1,
            audio_format: OutputAudioFormat::Pcm {
                format: PCMSampleFormat::Float32,
            },
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UiState {
    pub tab: ForteTab,
    pub rendering: bool,
    pub render_settings_visible: bool,
    pub midi_select_last_path: Option<PathBuf>,
    pub output_select_last_path: Option<PathBuf>,
    pub sf_select_last_path: Option<PathBuf>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ForteState {
    pub synth_settings: SynthSettings,
    pub render_settings: RenderSettings,
    pub ui_state: UiState,
}

impl ForteState {
    fn get_config_path() -> Result<PathBuf, ()> {
        let mut path = match dirs::config_dir() {
            Some(dir) => dir,
            None => {
                warn!("No config directory found. Cannot save config");
                return Err(());
            }
        };
        path.push("forte");
        std::fs::create_dir_all(&path).unwrap_or_default();
        path.push("config.toml");
        Ok(path)
    }

    pub fn save(&self) -> std::io::Result<()> {
        let string = toml::to_string(self).unwrap();
        //.map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, ""))?;

        let path = Self::get_config_path()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, ""))?;

        let mut file = File::create(path)?;
        file.write_all(string.as_bytes())?;
        info!("Saved state");
        Ok(())
    }

    pub fn load() -> ForteState {
        let warn = || {
            warn!("Could not load config file. Using defaults.");
        };

        let path = match Self::get_config_path() {
            Ok(path) => path,
            Err(..) => {
                warn();
                return Default::default();
            }
        };

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(..) => {
                warn();
                return Default::default();
            }
        };

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(file) => file,
            Err(..) => {
                warn();
                return Default::default();
            }
        };

        toml::from_str(&contents).unwrap_or_default()
    }
}
