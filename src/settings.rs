use crate::elements::sf_list::ForteSFListItem;
use crate::tabs::ForteTab;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use xsynth_core::channel::ChannelInitOptions;
use xsynth_core::ChannelCount;

#[derive(Default, Copy, Clone)]
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

#[derive(Default, Copy, Clone, PartialEq)]
pub enum Concurrency {
    #[default]
    None,
    ParallelMIDIs,
    ParallelTracks,
    Both,
}

impl From<Concurrency> for usize {
    fn from(val: Concurrency) -> Self {
        match val {
            Concurrency::None => 0,
            Concurrency::ParallelMIDIs => 1,
            Concurrency::ParallelTracks => 2,
            Concurrency::Both => 3,
        }
    }
}

#[derive(Clone)]
pub struct SingleChannelSettings {
    pub channel_init_options: ChannelInitOptions,
    pub layer_limit: Option<usize>,
    pub soundfonts: Vec<ForteSFListItem>,
    pub use_threadpool: bool,
}

impl Default for SingleChannelSettings {
    fn default() -> Self {
        let channel_init_options = ChannelInitOptions {
            fade_out_killing: true,
            //..Default::default()
        };

        Self {
            channel_init_options,
            layer_limit: Some(32),
            soundfonts: Vec::new(),
            use_threadpool: true,
        }
    }
}

#[derive(Clone)]
pub struct SynthSettings {
    pub channel_settings: Vec<SingleChannelSettings>,
}

impl Default for SynthSettings {
    fn default() -> Self {
        Self {
            channel_settings: vec![Default::default(); 16],
        }
    }
}

#[derive(Clone)]
pub struct RenderSettings {
    pub sample_rate: u32,
    pub audio_channels: ChannelCount,
    pub use_limiter: bool,
    pub render_mode: RenderMode,
    pub concurrency: Concurrency,
    pub vel_ignore_range: RangeInclusive<u8>,
    pub realtime_buffer_ms: f32,
    pub output_dir: Option<PathBuf>,
    pub parallel_midis: usize,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            audio_channels: ChannelCount::Stereo,
            use_limiter: true,
            render_mode: RenderMode::Standard,
            concurrency: Concurrency::None,
            vel_ignore_range: 0..=0,
            realtime_buffer_ms: 100.0 / 6.0,
            output_dir: None,
            parallel_midis: 2,
        }
    }
}

#[derive(Clone, Default)]
pub struct UiState {
    pub tab: ForteTab,
    pub rendering: bool,
    pub loading_dialog: Option<(String, f32)>,
    pub render_settings_visible: bool,
}

#[derive(Clone, Default)]
pub struct ForteState {
    pub synth_settings: SynthSettings,
    pub render_settings: RenderSettings,
    pub ui_state: UiState,
}
