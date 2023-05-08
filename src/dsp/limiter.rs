use super::DSPLimiterSettings;
use std::collections::VecDeque;

pub struct LookaheadLimiter {
    attack_coeff: f32,
    release_coeff: f32,
    threshold: f32,
    lookahead_samples: usize,
    lookahead: VecDeque<f32>,
    peak: f32,
}

impl LookaheadLimiter {
    pub fn new(sample_rate: u32, settings: DSPLimiterSettings) -> Self {
        let sample_rate = sample_rate as f32;
        let lookahead_samples = (sample_rate * settings.lookahead_time_ms as f32 / 1000.0) as usize;
        let attack_coeff = (-1.0 / (settings.attack_ms as f32 / 1000.0 * sample_rate)).exp();
        let release_coeff = (-1.0 / (settings.release_ms as f32 / 1000.0 * sample_rate)).exp();
        let threshold = 10.0f32.powf(settings.threshold / 20.0);
        LookaheadLimiter {
            attack_coeff,
            release_coeff,
            threshold,
            lookahead_samples,
            lookahead: VecDeque::with_capacity(lookahead_samples),
            peak: 0.0,
        }
    }

    pub fn offset(&self) -> usize {
        self.lookahead_samples
    }

    pub fn limit(&mut self, input: f32) -> f32 {
        self.lookahead.push_back(input);
        if self.lookahead.len() < self.lookahead_samples {
            0.0
        } else {
            let x = self.lookahead.pop_front().unwrap();

            let lookahead_peak = self
                .lookahead
                .iter()
                .map(|x| x.abs())
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);

            self.peak = self.peak.max(lookahead_peak);

            let gain = if self.peak > self.threshold {
                self.threshold / self.peak
            } else {
                1.0
            };

            if self.peak > x {
                self.peak *= self.release_coeff;
            } else {
                self.peak *= self.attack_coeff;
            }

            x * gain
        }
    }
}
