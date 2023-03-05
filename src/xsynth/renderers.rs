mod standard;
pub use standard::*;

mod buffered;
pub use buffered::*;

use xsynth_core::channel::{ChannelAudioEvent, ChannelConfigEvent};
use xsynth_core::AudioStreamParams;

pub enum SynthEvent {
    Channel(u32, ChannelAudioEvent),
    AllChannels(ChannelAudioEvent),
    ChannelConfig(u32, ChannelConfigEvent),
}

pub trait Renderer: Sync + Send {
    fn stream_params(&self) -> &'_ AudioStreamParams;

    fn send_event(&mut self, event: SynthEvent);

    fn read_samples(&mut self, to: &mut [f32]) {
        assert!(to.len() as u32 % self.stream_params().channels as u32 == 0);
        self.read_samples_unchecked(to);
    }

    fn read_samples_unchecked(&mut self, to: &mut [f32]);

    fn voice_count(&self) -> u64;
}

/*pub fn track_channel_to_index(track: usize, channel: usize) -> usize {
    track * 128 + channel
}*/
