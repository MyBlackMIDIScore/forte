use crate::errors::error_types::MIDIRendererError;
use crate::settings::{Concurrency, ForteState, RenderMode};
use crate::writer::ForteAudioFileWriter;
use crate::xsynth::{
    renderers::{ForteBufferedRenderer, ForteStandardRenderer, Renderer, SynthEvent},
    RenderStats,
};
use atomic::Atomic;
use atomic_float::AtomicF64;
use crossbeam_channel::{Receiver, Sender};
use midi_toolkit::{
    events::Event,
    io::MIDIFile,
    pipe,
    sequence::{
        event::{cancel_tempo_events, scale_event_time, Delta, EventBatch},
        unwrap_items, TimeCaster,
    },
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{
    atomic::{AtomicBool, AtomicU64},
    Arc, RwLock,
};
use std::thread;
use xsynth_core::channel::{ChannelAudioEvent, ChannelConfigEvent, ControlEvent};
use xsynth_core::effects::VolumeLimiter;
use xsynth_core::soundfont::{SampleSoundfont, SoundfontBase};
use xsynth_core::AudioStreamParams;

#[derive(Clone)]
struct RenderStatsAtomic {
    time: Arc<AtomicF64>,
    voices: Arc<AtomicU64>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MIDIRendererStatus {
    Idle,
    Rendering,
    Finished,
    Error,
}

struct MIDIRenderer {
    allow: Arc<AtomicBool>,
    status: Arc<Atomic<MIDIRendererStatus>>,
    soundfonts: Arc<RwLock<HashMap<PathBuf, Arc<SampleSoundfont>>>>,

    receiver: Receiver<Delta<f64, EventBatch<Event>>>,
    renderer: Box<dyn Renderer>,
    writer: Sender<f32>,

    limiter: Option<VolumeLimiter>,
    audio_params: AudioStreamParams,

    output_vec: Vec<f32>,
    missed_samples: f64,
    time: f64,
}

impl MIDIRenderer {
    pub fn load_new(
        state: &ForteState,
        midi_path: PathBuf,
        soundfonts: Arc<RwLock<HashMap<PathBuf, Arc<SampleSoundfont>>>>,
    ) -> Result<Self, MIDIRendererError> {
        let allow = Arc::new(AtomicBool::new(true));

        let audio_params = AudioStreamParams::new(
            state.render_settings.sample_rate,
            state.render_settings.audio_channels,
        );

        let midi = match MIDIFile::open(midi_path.clone(), None) {
            Ok(m) => m,
            Err(err) => return Err(MIDIRendererError::LoadError(err)),
        };

        let (receiver, renderer) = match state.render_settings.concurrency {
            _ => {
                let ppq = midi.ppq();
                let merged = pipe!(
                    midi.iter_all_events_merged_batches()
                    |>TimeCaster::<f64>::cast_event_delta()
                    |>cancel_tempo_events(250000)
                    |>scale_event_time(1.0 / ppq as f64)
                    |>unwrap_items()
                );

                let (midi_snd, midi_rcv) = crossbeam_channel::bounded(100);

                let allow_c1 = allow.clone();
                thread::spawn(move || {
                    for event in merged {
                        if !allow_c1.load(Ordering::Relaxed) {
                            break;
                        }
                        midi_snd.send(event).unwrap_or_default();
                    }
                });

                let mut renderer: Box<dyn Renderer> = match state.render_settings.render_mode {
                    RenderMode::RealtimeSimulation => {
                        Box::new(ForteBufferedRenderer::new(state, 1))
                    }
                    RenderMode::Standard => Box::new(ForteStandardRenderer::new(state, 1)),
                };

                for (i, ch) in state
                    .synth_settings
                    .channel_settings
                    .clone()
                    .into_iter()
                    .enumerate()
                {
                    renderer.send_event(SynthEvent::ChannelConfig(
                        i as u32,
                        ChannelConfigEvent::SetLayerCount(ch.layer_limit),
                    ));
                }

                (midi_rcv, renderer)
            }
        };

        let out_filename = if let Some(filename) = midi_path.file_name() {
            if let Some(filename) = filename.to_str() {
                let mut f = filename.to_owned();
                f += ".wav";
                f
            } else {
                "out.wav".to_owned()
            }
        } else {
            "out.wav".to_owned()
        };

        let mut writer = match ForteAudioFileWriter::new(state, out_filename) {
            Ok(w) => w,
            Err(err) => return Err(err),
        };

        let (writer_snd, writer_rcv) = crossbeam_channel::bounded::<f32>(100);

        let allow_c2 = allow.clone();
        thread::spawn(move || {
            for sample in writer_rcv.clone() {
                if !allow_c2.load(Ordering::Relaxed) {
                    break;
                }
                writer.write_samples(sample).unwrap_or_default();
            }
            writer.finalize().unwrap_or_default();
        });

        Ok(Self {
            allow,
            status: Arc::new(Atomic::new(MIDIRendererStatus::Idle)),
            soundfonts,

            receiver,
            renderer,
            writer: writer_snd,

            limiter: if state.render_settings.use_limiter {
                Some(VolumeLimiter::new(audio_params.channels.count()))
            } else {
                None
            },
            audio_params,

            output_vec: Vec::new(),
            missed_samples: 0.0,
           time: 0.0,
        })
    }

    pub fn get_allow(&self) -> Arc<AtomicBool> {
        self.allow.clone()
    }

    pub fn get_status(&self) -> Arc<Atomic<MIDIRendererStatus>> {
        self.status.clone()
    }

    pub fn set_soundfonts(&mut self, state: &ForteState) {
        let soundfonts = self.soundfonts.read().unwrap();

        for (i, ch) in state
            .synth_settings
            .channel_settings
            .clone()
            .into_iter()
            .enumerate()
        {
            let mut sfs: Vec<Arc<dyn SoundfontBase>> = vec![];
            for sf in ch.soundfonts {
                if let Some(s) = soundfonts.get(&sf.path) {
                    sfs.push(s.clone());
                }
            }
            self.renderer.send_event(SynthEvent::ChannelConfig(
                i as u32,
                ChannelConfigEvent::SetSoundfonts(sfs),
            ));
        }
    }

    fn render_batch(&mut self, event_time: f64, update_stats: impl FnOnce(f64, u64) + Clone) {
        let max_batch_time = 0.01;
        if event_time > max_batch_time {
            let mut remaining_time = event_time;
            loop {
                if remaining_time > max_batch_time {
                    self.render_batch(max_batch_time, update_stats.clone());
                    remaining_time -= max_batch_time;
                } else {
                    self.render_batch(remaining_time, update_stats);
                    break;
                }
            }
        } else {
            let samples = self.audio_params.sample_rate as f64 * event_time + self.missed_samples;
            self.missed_samples = samples % 1.0;
            let samples = samples as usize * self.audio_params.channels.count() as usize;

            self.output_vec.resize(samples, 0.0);
            self.renderer.read_samples(&mut self.output_vec);

            self.time += event_time;
            (update_stats)(self.time, self.renderer.voice_count());

            if let Some(limiter) = &mut self.limiter {
                limiter.limit(&mut self.output_vec);
            }

            for sample in self.output_vec.drain(..) {
                self.writer.send(sample).unwrap_or_default();
            }
        }
    }

    fn finalize(&mut self) {
        loop {
            self.output_vec
                .resize(self.audio_params.sample_rate as usize, 0.0);
            self.renderer.read_samples(&mut self.output_vec);
            let mut is_empty = true;
            for s in &self.output_vec {
                if *s > 0.0001 || *s < -0.0001 {
                    is_empty = false;
                    break;
                }
            }
            if is_empty {
                break;
            }
            for sample in self.output_vec.drain(..) {
                self.writer.send(sample).unwrap_or_default();
            }
        }
        self.status
            .store(MIDIRendererStatus::Finished, Ordering::Relaxed);
    }

    pub fn run(&mut self, stats: Arc<RenderStatsAtomic>) {
        let update_stats = |time: f64, voices: u64| {
            stats.time.store(time, Ordering::Relaxed);
            stats.voices.store(voices, Ordering::Relaxed);
        };

        for batch in self.receiver.clone() {
            if !self.allow.load(Ordering::Relaxed) {
                break;
            }

            if batch.delta > 0.0 {
                self.render_batch(batch.delta, update_stats);
            }

            for event in batch.iter_inner() {
                match event {
                    Event::NoteOn(e) => {
                        self.renderer.send_event(SynthEvent::Channel(
                            e.channel as u32,
                            ChannelAudioEvent::NoteOn {
                                key: e.key,
                                vel: e.velocity,
                            },
                        ));
                    }
                    Event::NoteOff(e) => {
                        self.renderer.send_event(SynthEvent::Channel(
                            e.channel as u32,
                            ChannelAudioEvent::NoteOff { key: e.key },
                        ));
                    }
                    Event::ControlChange(e) => {
                        self.renderer.send_event(SynthEvent::Channel(
                            e.channel as u32,
                            ChannelAudioEvent::Control(ControlEvent::Raw(e.controller, e.value)),
                        ));
                    }
                    Event::PitchWheelChange(e) => {
                        self.renderer.send_event(SynthEvent::Channel(
                            e.channel as u32,
                            ChannelAudioEvent::Control(ControlEvent::PitchBendValue(
                                e.pitch as f32 / 8192.0,
                            )),
                        ));
                    }
                    _ => {}
                }
            }
        }
        self.renderer
            .send_event(SynthEvent::AllChannels(ChannelAudioEvent::AllNotesOff));
        self.renderer
            .send_event(SynthEvent::AllChannels(ChannelAudioEvent::ResetControl));
        self.finalize();
    }
}

struct MIDIRendererContainer {
    renderer: Option<Arc<RwLock<MIDIRenderer>>>,
    stats: Arc<RenderStatsAtomic>,
    status: Arc<Atomic<MIDIRendererStatus>>,
    allow: Arc<AtomicBool>,
}

pub struct MIDIPool {
    max_parallel: usize,
    containers: Vec<MIDIRendererContainer>,
    is_rendering: bool,
}

impl MIDIPool {
    pub fn new(
        state: &ForteState,
        midis: Vec<PathBuf>,
        soundfonts: Arc<RwLock<HashMap<PathBuf, Arc<SampleSoundfont>>>>,
    ) -> Result<Self, MIDIRendererError> {
        if midis.is_empty() {
            return Err(MIDIRendererError::RendererError(
                "Empty MIDI List".to_owned(),
            ));
        }

        let mut containers = Vec::new();

        for midi in midis {
            match MIDIRenderer::load_new(state, midi.clone(), soundfonts.clone()) {
                Ok(r) => {
                    let stats = Arc::new(RenderStatsAtomic {
                        time: Arc::new(AtomicF64::new(0.0)),
                        voices: Arc::new(AtomicU64::new(0)),
                    });

                    containers.push(MIDIRendererContainer {
                        stats,
                        status: r.get_status(),
                        allow: r.get_allow(),
                        renderer: Some(Arc::new(RwLock::new(r))),
                    });
                }
                Err(err) => {
                    for c in containers {
                        c.allow.store(false, Ordering::Relaxed);
                    }
                    return Err(err);
                }
            }
        }

        let max_parallel = match state.render_settings.concurrency {
            Concurrency::None | Concurrency::ParallelTracks => 1,
            Concurrency::ParallelMIDIs | Concurrency::Both => state.render_settings.parallel_midis,
        };

        Ok(Self {
            max_parallel,
            containers,
            is_rendering: false,
        })
    }

    pub fn run(&mut self) {
        for _ in 0..self.max_parallel {
            self.spawn_next();
        }
        self.is_rendering = true;
    }

    pub fn spawn_next(&mut self) -> bool {
        if self.get_active_len() < self.max_parallel {
            for container in &self.containers {
                if container.status.load(Ordering::Relaxed) == MIDIRendererStatus::Idle {
                    let renderer = container.renderer.clone();
                    container
                        .status
                        .store(MIDIRendererStatus::Rendering, Ordering::Relaxed);
                    let cellcallback = container.stats.clone();
                    thread::spawn(move || {
                        if let Some(renderer) = renderer {
                            renderer.write().unwrap().run(cellcallback);
                        }
                    });
                    return true;
                }
            }
            false
        } else {
            false
        }
    }

    fn get_active_len(&self) -> usize {
        let mut active = 0;
        for container in &self.containers {
            if container.status.load(Ordering::Relaxed) == MIDIRendererStatus::Rendering {
                active += 1;
            }
        }
        active
    }

    pub fn status(&mut self) -> MIDIRendererStatus {
        let mut status = MIDIRendererStatus::Finished;

        for container in &mut self.containers {
            status = container.status.load(Ordering::Relaxed);
            if status == MIDIRendererStatus::Finished {
                container.renderer.take();
            }
            if status == MIDIRendererStatus::Error {
                break;
            }
            if status == MIDIRendererStatus::Rendering {
                break;
            }
        }

        if status == MIDIRendererStatus::Error {
            self.cancel();
        } else if status == MIDIRendererStatus::Finished {
            self.is_rendering = false;
        }

        status
    }

    pub fn set_soundfonts(&mut self, state: &ForteState) {
        for container in &self.containers {
            if let Some(renderer) = &container.renderer {
                renderer.write().unwrap().set_soundfonts(state);
            }
        }
    }

    pub fn get_stats(&self) -> Vec<Option<RenderStats>> {
        let mut progress = Vec::new();

        for container in &self.containers {
            let status = container.status.load(Ordering::Relaxed);
            if status != MIDIRendererStatus::Idle {
                progress.push(Some(RenderStats {
                    time: container.stats.time.load(Ordering::Relaxed),
                    voice_count: container.stats.voices.load(Ordering::Relaxed),
                }))
            } else {
                progress.push(None);
            }
        }

        progress
    }

    pub fn has_finished(&mut self) -> bool {
        self.status() == MIDIRendererStatus::Finished
    }

    pub fn cancel(&mut self) {
        for container in &self.containers {
            container.allow.store(false, Ordering::Relaxed);
        }
        self.is_rendering = false;
        self.containers.clear();
    }
}
