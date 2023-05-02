use crate::errors::error_types::MIDIRendererError;
use crate::writer::{split_stereo, AudioWriter, COMMON_SAMPLE_RATES};
use rand;
use std::fs::File;
use std::num::{NonZeroU32, NonZeroU8};
use std::path::PathBuf;
use vorbis_rs::{VorbisBitrateManagementStrategy, VorbisEncoder};

pub struct VorbisFileWriter {
    channels: u16,
    encoder: VorbisEncoder<File>,
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

        Ok(Self { channels, encoder })
    }
}

impl AudioWriter for VorbisFileWriter {
    fn write_samples(&mut self, samples: Vec<f32>) -> Result<(), MIDIRendererError> {
        if self.channels == 1 {
            self.encoder
                .encode_audio_block([&samples])
                .map_err(|e| MIDIRendererError::Writer(e.to_string()))?;
        } else {
            let (left_sgnl, right_sgnl) = split_stereo(samples);
            self.encoder
                .encode_audio_block([&left_sgnl, &right_sgnl])
                .map_err(|e| MIDIRendererError::Writer(e.to_string()))?;
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
