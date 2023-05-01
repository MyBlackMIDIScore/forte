use crate::errors::error_types::MIDIRendererError;
use crate::settings::PCMSampleFormat;
use crate::writer::AudioWriter;
use hound::{SampleFormat, WavSpec, WavWriter};
use std::{fs::File, io::BufWriter, path::PathBuf};
use tracing::{error, info};

pub struct PCMFileWriter {
    writer: WavWriter<BufWriter<File>>,
    format: PCMSampleFormat,
}

impl PCMFileWriter {
    pub fn new(
        channels: u16,
        sample_rate: u32,
        format: PCMSampleFormat,
        filepath: PathBuf,
    ) -> Result<Self, MIDIRendererError> {
        info!("Creating new audio file writer");
        let (bits_per_sample, sample_format) = match format {
            PCMSampleFormat::Int16 => (16, SampleFormat::Int),
            PCMSampleFormat::Float32 => (32, SampleFormat::Float),
        };

        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample,
            sample_format,
        };

        match WavWriter::create(filepath, spec) {
            Ok(writer) => Ok(Self { writer, format }),
            Err(err) => {
                error!("Writer error: {}", &err.to_string());
                Err(MIDIRendererError::Writer(err.to_string()))
            }
        }
    }
}

impl AudioWriter for PCMFileWriter {
    fn write_sample(&mut self, sample: f32) -> Result<(), MIDIRendererError> {
        match self.format {
            PCMSampleFormat::Int16 => {
                let sample = sample * std::i16::MAX as f32;
                match self.writer.write_sample(sample as i16) {
                    Ok(..) => Ok(()),
                    Err(err) => Err(MIDIRendererError::Writer(err.to_string())),
                }
            }
            PCMSampleFormat::Float32 => match self.writer.write_sample(sample) {
                Ok(..) => Ok(()),
                Err(err) => Err(MIDIRendererError::Writer(err.to_string())),
            },
        }
    }

    fn finalize(self: Box<Self>) -> Result<(), MIDIRendererError> {
        info!("Finalizing audio file");
        match self.writer.finalize() {
            Ok(..) => Ok(()),
            Err(err) => Err(MIDIRendererError::Writer(err.to_string())),
        }
    }
}
