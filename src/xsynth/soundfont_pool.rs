use crate::elements::sf_list::ForteSFListItem;
use atomic::Atomic;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc, RwLock};
use std::thread;
use tracing::{error, info};
use xsynth_core::soundfont::SampleSoundfont;
use xsynth_core::AudioStreamParams;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SoundfontWorkerStatus {
    Loading,
    Finished,
    Error,
}

#[derive(Clone)]
struct SoundfontThread {
    allow: Arc<AtomicBool>,
    status: Arc<Atomic<SoundfontWorkerStatus>>,
}

impl SoundfontThread {
    pub fn load_new(
        soundfont: ForteSFListItem,
        dest: Arc<RwLock<HashMap<PathBuf, Arc<SampleSoundfont>>>>,
        audio_params: AudioStreamParams,
    ) -> Self {
        let status = Arc::new(Atomic::new(SoundfontWorkerStatus::Loading));
        let statusc = status.clone();
        let allow = Arc::new(AtomicBool::new(true));
        let allowc = allow.clone();
        thread::spawn(move || {
            info!("Loading new soundfont: {:?}", soundfont.path);
            let sf =
                SampleSoundfont::new(soundfont.path.clone(), audio_params, soundfont.pref.init);
            match sf {
                Ok(sf) => {
                    if allowc.load(Ordering::Relaxed) {
                        info!("Finished loading soundfont: {:?}", soundfont.path);
                        dest.write()
                            .unwrap()
                            .insert(soundfont.path.clone(), Arc::new(sf));
                    }
                    statusc.store(SoundfontWorkerStatus::Finished, Ordering::Relaxed);
                }
                Err(err) => {
                    error!("Error loading soundfont: {:?}: {:?}", soundfont.path, err);
                    statusc.store(SoundfontWorkerStatus::Error, Ordering::Relaxed);
                }
            }
        });

        Self { status, allow }
    }

    pub fn status(&self) -> SoundfontWorkerStatus {
        self.status.load(Ordering::Relaxed)
    }

    pub fn cancel(&self) {
        self.allow.store(false, Ordering::Relaxed);
    }
}

pub struct SoundfontPool {
    workers: Vec<SoundfontThread>,
}

impl SoundfontPool {
    pub fn new(
        soundfonts: Vec<ForteSFListItem>,
        dest: Arc<RwLock<HashMap<PathBuf, Arc<SampleSoundfont>>>>,
        audio_params: AudioStreamParams,
    ) -> Self {
        info!("Starting new soundfont thread manager");
        let mut workers = Vec::new();

        for soundfont in soundfonts {
            workers.push(SoundfontThread::load_new(
                soundfont,
                dest.clone(),
                audio_params,
            ));
        }

        Self { workers }
    }

    pub fn cancel(&mut self) {
        for w in &self.workers {
            w.cancel();
        }
    }

    pub fn status(&mut self) -> SoundfontWorkerStatus {
        let mut status = SoundfontWorkerStatus::Finished;

        if self.workers.is_empty() {
            return SoundfontWorkerStatus::Finished;
        }

        self.workers = self
            .workers
            .clone()
            .into_iter()
            .filter(|w| w.status() != SoundfontWorkerStatus::Finished)
            .collect();

        for worker in &self.workers {
            let s = worker.status();
            status = s;
            if s == SoundfontWorkerStatus::Error {
                break;
            }
        }

        if status == SoundfontWorkerStatus::Error {
            self.cancel();
        }

        status
    }
}
