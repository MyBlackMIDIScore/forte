use crate::elements::sf_list::ForteSFListItem;
use crate::tabs::ForteTab;
use std::ops::RangeInclusive;
use xsynth_core::channel::ChannelInitOptions;

#[derive(Default, Copy, Clone)]
pub enum RenderMode {
    #[default]
    Standard = 0,
    RealtimeSimulation = 1,
}

impl Into<usize> for RenderMode {
    fn into(self) -> usize {
        self as usize
    }
}

#[derive(Default, Copy, Clone)]
pub enum Concurrency {
    #[default]
    None = 0,
    ParallelItems = 1,
    ParallelTracks = 2,
    Both = 3,
}

impl Into<usize> for Concurrency {
    fn into(self) -> usize {
        self as usize
    }
}

#[derive(Clone)]
pub struct SingleChannelSettings {
    pub channel_init_options: ChannelInitOptions,
    pub layer_limit: Option<usize>,
    pub soundfonts: Vec<ForteSFListItem>,
    pub use_threadpool: bool,
    pub volume: f32,
}

impl Default for SingleChannelSettings {
    fn default() -> Self {
        Self {
            channel_init_options: Default::default(),
            layer_limit: Some(10),
            soundfonts: Vec::new(),
            use_threadpool: false,
            volume: 1.0,
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
    pub audio_channels: usize,
    pub use_limiter: bool,
    pub render_mode: RenderMode,
    pub concurrency: Concurrency,
    pub vel_ignore_range: RangeInclusive<u8>,
    pub realtime_buffer_ms: f32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            audio_channels: 2,
            use_limiter: true,
            render_mode: RenderMode::Standard,
            concurrency: Concurrency::None,
            vel_ignore_range: 0..=0,
            realtime_buffer_ms: 10.0,
        }
    }
}

#[derive(Clone, Default)]
pub struct UiState {
    pub tab: ForteTab,
    pub render_settings_visible: bool,
}

#[derive(Clone, Default)]
pub struct ForteState {
    pub synth_settings: SynthSettings,
    pub render_settings: RenderSettings,
    pub ui_state: UiState,
}
