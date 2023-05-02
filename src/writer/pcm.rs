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
    fn write_samples(&mut self, samples: Vec<f32>) -> Result<(), MIDIRendererError> {
        match self.format {
            PCMSampleFormat::Int16 => {
                for sample in samples {
                    let sample = (sample * std::i16::MAX as f32) as i16;
                    self.writer
                        .write_sample(sample)
                        .map_err(|err| MIDIRendererError::Writer(err.to_string()))?
                }
            }
            PCMSampleFormat::Float32 => {
                for sample in samples {
                    self.writer
                        .write_sample(sample)
                        .map_err(|err| MIDIRendererError::Writer(err.to_string()))?
                }
            }
        }
        Ok(())
    }

    fn finalize(self: Box<Self>) -> Result<(), MIDIRendererError> {
        info!("Finalizing audio file");
        self.writer
            .finalize()
            .map_err(|err| MIDIRendererError::Writer(err.to_string()))
    }
}
