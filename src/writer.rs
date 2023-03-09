use crate::errors::error_types::MIDIRendererError;
use crate::settings::ForteState;
use hound::{WavSpec, WavWriter};
use std::{fs::File, io::BufWriter};
use tracing::{error, info};

pub struct ForteAudioFileWriter {
    writer: WavWriter<BufWriter<File>>,
}

impl ForteAudioFileWriter {
    pub fn new(state: &ForteState, filename: String) -> Result<Self, MIDIRendererError> {
        info!("Creating new audio file writer");
        let spec = WavSpec {
            channels: state.render_settings.audio_channels.count(),
            sample_rate: state.render_settings.sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let writer_result = if let Some(mut dir) = state.render_settings.output_dir.clone() {
            dir.push(filename);
            WavWriter::create(dir, spec)
        } else {
            WavWriter::create(filename, spec)
        };

        match writer_result {
            Ok(writer) => Ok(Self { writer }),
            Err(err) => {
                error!("Writer error: {}", &err.to_string());
                Err(MIDIRendererError::Writer(err.to_string()))
            }
        }
    }

    pub fn write_samples(&mut self, sample: f32) -> Result<(), MIDIRendererError> {
        match self.writer.write_sample(sample) {
            Ok(..) => Ok(()),
            Err(err) => Err(MIDIRendererError::Writer(err.to_string())),
        }
    }

    pub fn finalize(self) -> Result<(), MIDIRendererError> {
        info!("Finalizing audio file");
        match self.writer.finalize() {
            Ok(..) => Ok(()),
            Err(err) => Err(MIDIRendererError::Writer(err.to_string())),
        }
    }
}
