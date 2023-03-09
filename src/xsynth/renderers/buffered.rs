use super::{Renderer, SynthEvent};
use crate::settings::ForteState;
use crossbeam_channel::Sender;
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::thread;
use tracing::info;
use xsynth_core::channel::{ChannelEvent, VoiceChannel};
use xsynth_core::helpers::{prepapre_cache_vec, sum_simd};
use xsynth_core::{AudioPipe, AudioStreamParams, BufferedRenderer, FunctionAudioPipe};

pub struct ForteBufferedRenderer {
    buffered: BufferedRenderer,
    senders: Vec<Sender<ChannelEvent>>,
    voice_count: Arc<AtomicU64>,
    audio_params: AudioStreamParams,
}

impl ForteBufferedRenderer {
    pub fn new(state: &ForteState, instances: usize) -> Self {
        info!("Creating new buffered renderer with {instances} instance(s)");
        let mut channel_stats = Vec::new();
        let mut senders = Vec::new();
        let mut command_senders = Vec::new();

        let audio_params = AudioStreamParams::new(
            state.render_settings.sample_rate,
            state.render_settings.audio_channels,
        );

        let (output_sender, output_receiver) = crossbeam_channel::bounded::<Vec<f32>>(16);

        for _ in 0..instances {
            for ch in state.synth_settings.channel_settings.clone() {
                let pool = if ch.use_threadpool {
                    Some(Arc::new(rayon::ThreadPoolBuilder::new().build().unwrap()))
                } else {
                    None
                };

                let mut channel = VoiceChannel::new(ch.channel_init_options, audio_params, pool);
                let stats = channel.get_channel_stats();
                channel_stats.push(stats);

                let (event_sender, event_receiver) = crossbeam_channel::unbounded();
                senders.push(event_sender);

                let (command_sender, command_receiver) = crossbeam_channel::bounded::<Vec<f32>>(1);

                command_senders.push(command_sender);

                let output_sender = output_sender.clone();
                thread::spawn(move || loop {
                    channel.push_events_iter(event_receiver.try_iter());
                    let mut vec = match command_receiver.recv() {
                        Ok(vec) => vec,
                        Err(_) => break,
                    };
                    channel.push_events_iter(event_receiver.try_iter());
                    channel.read_samples(&mut vec);
                    output_sender.send(vec).unwrap();
                });
            }
        }

        let mut vec_cache: VecDeque<Vec<f32>> = VecDeque::new();
        for _ in 0..16 {
            vec_cache.push_front(Vec::new());
        }

        let voice_count = Arc::new(AtomicU64::new(0));
        let voice_countc = voice_count.clone();

        let fnpipe = FunctionAudioPipe::new(audio_params, move |out| {
            for sender in command_senders.iter() {
                let mut buf = vec_cache.pop_front().unwrap();
                prepapre_cache_vec(&mut buf, out.len(), 0.0);

                sender.send(buf).unwrap();
            }

            for _ in 0..16 {
                let buf = output_receiver.recv().unwrap();
                sum_simd(&buf, out);
                vec_cache.push_front(buf);
            }

            let total_voices = channel_stats.iter().map(|c| c.voice_count()).sum();
            voice_countc.store(total_voices, Ordering::SeqCst);
        });

        let buffered = BufferedRenderer::new(
            fnpipe,
            audio_params,
            (audio_params.sample_rate as f32 * state.render_settings.realtime_buffer_ms / 1000.0)
                as usize,
        );

        Self {
            buffered,
            senders,
            voice_count,
            audio_params,
        }
    }
}

impl Renderer for ForteBufferedRenderer {
    fn stream_params(&self) -> &AudioStreamParams {
        &self.audio_params
    }

    fn send_event(&mut self, event: SynthEvent) {
        match event {
            SynthEvent::Channel(channel, event) => {
                self.senders[channel as usize]
                    .send(ChannelEvent::Audio(event))
                    .unwrap_or_default();
            }
            SynthEvent::AllChannels(event) => {
                for sender in self.senders.iter_mut() {
                    sender
                        .send(ChannelEvent::Audio(event.clone()))
                        .unwrap_or_default();
                }
            }
            SynthEvent::ChannelConfig(channel, config) => {
                self.senders[channel as usize]
                    .send(ChannelEvent::Config(config))
                    .unwrap_or_default();
            }
        }
    }

    fn read_samples_unchecked(&mut self, to: &mut [f32]) {
        self.buffered.read(to);
    }

    fn voice_count(&self) -> u64 {
        self.voice_count.load(Ordering::Relaxed)
    }
}
