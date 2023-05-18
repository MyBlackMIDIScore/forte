use super::DSPLimiterSettings;
use fundsp::hacker32::*;

pub struct AudioLimiter {
    limiter: An<Limiter<f32, U1, (f32, f32)>>,
}

impl AudioLimiter {
    pub fn new(settings: DSPLimiterSettings) -> Self {
        Self {
            limiter: limiter((
                settings.attack_ms as f32 / 1000.0,
                settings.release_ms as f32 / 1000.0,
            )),
        }
    }

    pub fn limit(&mut self, input: f32) -> f32 {
        self.limiter.tick(&Frame::from([input]))[0]
    }
}
