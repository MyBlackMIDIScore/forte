use crate::errors::error_types::MIDIRendererError;
use crate::writer::{split_stereo, AudioWriter};
use mp3lame_encoder::{Birtate, Builder, DualPcm, Encoder, FlushNoGap, MonoPcm};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use tracing::{error, info};

pub struct LameFileWriter {
    channels: u16,
    encoder: Encoder,
    file: File,
}

impl LameFileWriter {
    pub fn new(
        channels: u16,
        sample_rate: u32,
        bitrate: u32,
        filepath: PathBuf,
    ) -> Result<Self, MIDIRendererError> {
        let mut encoder = match Builder::new() {
            Some(e) => e,
            None => {
                error!("Unable to create LAME encoder");
                return Err(MIDIRendererError::Writer(
                    "Unable to create LAME encoder".to_owned(),
                ));
            }
        };
        info!("Creating new LAME encoder");

        encoder
            .set_num_channels(channels as u8)
            .map_err(|e| MIDIRendererError::Writer(e.to_string()))?;
        encoder
            .set_sample_rate(sample_rate)
            .map_err(|e| MIDIRendererError::Writer(e.to_string()))?;
        encoder
            .set_brate(Self::get_bitrate(bitrate))
            .map_err(|e| MIDIRendererError::Writer(e.to_string()))?;
        encoder
            .set_quality(mp3lame_encoder::Quality::Best)
            .map_err(|e| MIDIRendererError::Writer(e.to_string()))?;
        let encoder = encoder
            .build()
            .map_err(|e| MIDIRendererError::Writer(e.to_string()))?;

        let file = File::create(filepath).map_err(|e| MIDIRendererError::Writer(e.to_string()))?;

        Ok(Self {
            channels,
            encoder,
            file,
        })
    }

    fn get_bitrate(bitrate: u32) -> Birtate {
        match bitrate / 1000 {
            64 => Birtate::Kbps64,
            80 => Birtate::Kbps80,
            96 => Birtate::Kbps96,
            128 => Birtate::Kbps128,
            160 => Birtate::Kbps160,
            192 => Birtate::Kbps192,
            256 => Birtate::Kbps256,
            320 => Birtate::Kbps320,
            _ => Birtate::Kbps192,
        }
    }
}

impl AudioWriter for LameFileWriter {
    fn write_samples(&mut self, samples: Vec<f32>) -> Result<(), MIDIRendererError> {
        let mut out = Vec::new();
        let encoded_size = if self.channels == 1 {
            let input = MonoPcm(&samples);
            out.reserve(mp3lame_encoder::max_required_buffer_size(input.0.len()));
            self.encoder
                .encode(input, out.spare_capacity_mut())
                .map_err(|e| MIDIRendererError::Writer(e.to_string()))?
        } else {
            let (left_sgnl, right_sgnl) = split_stereo(samples);
            let input = DualPcm {
                left: &left_sgnl,
                right: &right_sgnl,
            };

            out.reserve(mp3lame_encoder::max_required_buffer_size(input.left.len()));
            self.encoder
                .encode(input, out.spare_capacity_mut())
                .map_err(|e| MIDIRendererError::Writer(e.to_string()))?
        };

        unsafe {
            out.set_len(out.len().wrapping_add(encoded_size));
        }

        self.file
            .write_all(&out)
            .map_err(|e| MIDIRendererError::Writer(e.to_string()))?;

        Ok(())
    }

    fn finalize(mut self: Box<Self>) -> Result<(), MIDIRendererError> {
        info!("Finalizing LAME audio file");
        let mut out = Vec::new();
        out.reserve(mp3lame_encoder::max_required_buffer_size(1));
        let encoded_size = self
            .encoder
            .flush::<FlushNoGap>(out.spare_capacity_mut())
            .map_err(|e| MIDIRendererError::Writer(e.to_string()))?;
        unsafe {
            out.set_len(out.len().wrapping_add(encoded_size));
        }
        self.file
            .write_all(&out)
            .map_err(|e| MIDIRendererError::Writer(e.to_string()))?;
        Ok(())
    }
}
