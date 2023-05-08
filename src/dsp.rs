use serde::{Deserialize, Serialize};

mod limiter;

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DSPLimiterSettings {
    pub enabled: bool,
    pub attack_ms: u16,
    pub release_ms: u16,
    pub threshold: f32,
    pub lookahead_time_ms: u16,
}

impl Default for DSPLimiterSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            attack_ms: 30,
            release_ms: 100,
            threshold: 0.0,
            lookahead_time_ms: 5,
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
    offset: usize,
    limiter: Option<Vec<limiter::LookaheadLimiter>>,
}

impl ForteAudioDSP {
    pub fn new(channels: u16, sample_rate: u32, settings: DSPSettings) -> Self {
        let mut offset = 0;

        let limiter = if settings.limiter.enabled {
            let mut v = Vec::new();
            for i in 0..channels {
                let l = limiter::LookaheadLimiter::new(sample_rate, settings.limiter);
                if i == 0 {
                    offset += l.offset();
                }
                v.push(l);
            }
            Some(v)
        } else {
            None
        };

        Self {
            channels,
            offset,
            limiter,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn process(&mut self, vec: &mut [f32]) {
        if let Some(limiter) = self.limiter.as_mut() {
            for (i, s) in vec.iter_mut().enumerate() {
                *s = limiter[i % self.channels as usize].limit(*s);
            }
        }
    }
}
