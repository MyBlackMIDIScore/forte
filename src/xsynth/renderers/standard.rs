use super::{Renderer, SynthEvent};
use crate::settings::ForteState;
use rayon::prelude::*;
use std::sync::Arc;
use tracing::info;
use xsynth_core::channel::{ChannelAudioEvent, ChannelEvent, VoiceChannel};
use xsynth_core::helpers::sum_simd;
use xsynth_core::{AudioPipe, AudioStreamParams};

const MAX_EVENT_CACHE_SIZE: u32 = 1024 * 1024;

pub struct ForteStandardRenderer {
    thread_pool: rayon::ThreadPool,
    cached_event_count: u32,
    channel_events_cache: Box<[Vec<ChannelAudioEvent>]>,
    sample_cache_vecs: Box<[Vec<f32>]>,
    channels: Box<[VoiceChannel]>,
    audio_params: AudioStreamParams,
}

impl ForteStandardRenderer {
    pub fn new(state: &ForteState, instances: usize) -> Self {
        info!("Creating new renderer with {instances} instance(s)");
        let mut channels = Vec::new();
        let mut channel_events_cache = Vec::new();
        let mut sample_cache_vecs = Vec::new();

        let audio_params = AudioStreamParams::new(
            state.render_settings.sample_rate,
            state.render_settings.audio_channels,
        );

        for _ in 0..instances {
            for ch in state.synth_settings.channel_settings.clone() {
                let pool = if ch.use_threadpool {
                    Some(Arc::new(rayon::ThreadPoolBuilder::new().build().unwrap()))
                } else {
                    None
                };

                channels.push(VoiceChannel::new(
                    ch.channel_init_options,
                    audio_params,
                    pool.clone(),
                ));
                channel_events_cache.push(Vec::new());
                sample_cache_vecs.push(Vec::new());
            }
        }

        let thread_pool = rayon::ThreadPoolBuilder::new().build().unwrap();

        Self {
            thread_pool,
            cached_event_count: 0,
            channel_events_cache: channel_events_cache.into_boxed_slice(),
            channels: channels.into_boxed_slice(),
            sample_cache_vecs: sample_cache_vecs.into_boxed_slice(),
            audio_params,
        }
    }

    fn flush_events(&mut self) {
        if self.cached_event_count == 0 {
            return;
        }

        let thread_pool = &mut self.thread_pool;
        let channels = &mut self.channels;
        let channel_events_cache = &mut self.channel_events_cache;

        thread_pool.install(move || {
            channels
                .par_iter_mut()
                .zip(channel_events_cache.par_iter_mut())
                .for_each(|(channel, events)| {
                    channel.push_events_iter(events.drain(..).map(ChannelEvent::Audio));
                });
        });

        self.cached_event_count = 0;
    }

    fn render_to(&mut self, buffer: &mut [f32]) {
        self.flush_events();

        let thread_pool = &mut self.thread_pool;
        let channels = &mut self.channels;
        let sample_cache_vecs = &mut self.sample_cache_vecs;

        thread_pool.install(move || {
            channels
                .par_iter_mut()
                .zip(sample_cache_vecs.par_iter_mut())
                .for_each(|(channel, samples)| {
                    samples.resize(buffer.len(), 0.0);
                    channel.read_samples(samples.as_mut_slice());
                });

            for vec in sample_cache_vecs.iter_mut() {
                sum_simd(vec, buffer);
                vec.clear();
            }
        });
    }
}

impl Renderer for ForteStandardRenderer {
    fn stream_params(&self) -> &AudioStreamParams {
        &self.audio_params
    }

    fn send_event(&mut self, event: SynthEvent) {
        match event {
            SynthEvent::Channel(channel, event) => {
                self.channel_events_cache[channel as usize].push(event);
                self.cached_event_count += 1;
                if self.cached_event_count > MAX_EVENT_CACHE_SIZE {
                    self.flush_events();
                }
            }
            SynthEvent::AllChannels(event) => {
                for channel in self.channel_events_cache.iter_mut() {
                    channel.push(event.clone());
                }
                self.cached_event_count += self.channel_events_cache.len() as u32;
                if self.cached_event_count > MAX_EVENT_CACHE_SIZE {
                    self.flush_events();
                }
            }
            SynthEvent::ChannelConfig(channel, config) => {
                self.channels[channel as usize].process_event(ChannelEvent::Config(config));
            }
        }
    }

    fn read_samples_unchecked(&mut self, to: &mut [f32]) {
        self.render_to(to);
    }

    fn voice_count(&self) -> u64 {
        self.channels
            .iter()
            .map(|c| c.get_channel_stats().voice_count())
            .sum()
    }
}
