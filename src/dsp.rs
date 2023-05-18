use serde::{Deserialize, Serialize};

mod limiter;

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DSPLimiterSettings {
    pub enabled: bool,
    pub attack_ms: u16,
    pub release_ms: u16,
}

impl Default for DSPLimiterSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            attack_ms: 30,
            release_ms: 80,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DSPSettings {
    pub limiter: DSPLimiterSettings,
}

pub struct ForteAudioDSP {
    channels: u16,
    limiter: Option<Vec<limiter::AudioLimiter>>,
}

impl ForteAudioDSP {
    pub fn new(channels: u16, _sample_rate: u32, settings: DSPSettings) -> Self {
        let limiter = if settings.limiter.enabled {
            let mut v = Vec::new();
            for _ in 0..channels {
                v.push(limiter::AudioLimiter::new(settings.limiter));
            }
            Some(v)
        } else {
            None
        };

        Self { channels, limiter }
    }

    pub fn process(&mut self, vec: &mut [f32]) {
        if let Some(limiter) = self.limiter.as_mut() {
            for (i, s) in vec.iter_mut().enumerate() {
                *s = limiter[i % self.channels as usize].limit(*s);
            }
        }
    }
}
