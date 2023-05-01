use crate::errors::error_types::MIDIRendererError;
use crate::writer::{AudioWriter, COMMON_SAMPLE_RATES};
use rand;
use std::fs::File;
use std::num::{NonZeroU32, NonZeroU8};
use std::path::PathBuf;
use vorbis_rs::{VorbisBitrateManagementStrategy, VorbisEncoder};

pub struct VorbisFileWriter {
    encoder: VorbisEncoder<File>,
    cache: Option<f32>,
}

impl VorbisFileWriter {
    pub fn new(
        channels: u16,
        sample_rate: u32,
        bitrate: u32,
        filepath: PathBuf,
    ) -> Result<Self, MIDIRendererError> {
        let file = match File::create(filepath) {
            Ok(file) => file,
            Err(err) => return Err(MIDIRendererError::Writer(err.to_string())),
        };

        let encoder = match VorbisEncoder::new(
            rand::random(),
            [("", ""); 0],
            NonZeroU32::new(sample_rate.max(COMMON_SAMPLE_RATES[0])).unwrap(),
            NonZeroU8::new((channels as u8).max(2)).unwrap(),
            VorbisBitrateManagementStrategy::Vbr {
                target_bitrate: NonZeroU32::new(bitrate).unwrap(),
            },
            None,
            file,
        ) {
            Ok(encoder) => encoder,
            Err(err) => return Err(MIDIRendererError::Writer(err.to_string())),
        };

        Ok(Self {
            encoder,
            cache: None,
        })
    }
}

impl AudioWriter for VorbisFileWriter {
    fn write_sample(&mut self, sample: f32) -> Result<(), MIDIRendererError> {
        // Stereo hardcoded
        if let Some(cache) = self.cache {
            self.encoder
                .encode_audio_block([&[cache], &[sample]])
                .map_err(|e| MIDIRendererError::Writer(e.to_string()))?;
            self.cache = None;
        } else {
            self.cache = Some(sample);
        }
        Ok(())
    }

    fn finalize(self: Box<Self>) -> Result<(), MIDIRendererError> {
        self.encoder
            .finish()
            .map_err(|e| MIDIRendererError::Writer(e.to_string()))?;
        Ok(())
    }
}
