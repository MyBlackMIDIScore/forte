use crate::errors::error_types::MIDIRendererError;
use crate::settings::{ForteState, OutputAudioFormat};

pub mod lame;
mod pcm;
pub mod vorbis;

pub const COMMON_SAMPLE_RATES: [u32; 12] = [
    8_000, 11_025, 16_000, 22_050, 44_100, 48_000, 82_200, 96_000, 176_400, 192_000, 352_800,
    384_000,
];
pub const COMMON_BITRATES: [u32; 8] = [
    64_000, 80_000, 96_000, 128_000, 160_000, 192_000, 256_000, 320_000,
];

pub trait AudioWriter {
    fn write_sample(&mut self, sample: f32) -> Result<(), MIDIRendererError>;
    fn finalize(self: Box<Self>) -> Result<(), MIDIRendererError>;
}

pub struct ForteAudioFileWriter {
    writer: Box<dyn AudioWriter>,
}

impl ForteAudioFileWriter {
    pub fn new(state: &ForteState, mut filename: String) -> Result<Self, MIDIRendererError> {
        match state.render_settings.audio_format {
            OutputAudioFormat::Pcm { .. } => filename.push_str(".wav"),
            OutputAudioFormat::Vorbis { .. } => filename.push_str(".ogg"),
            OutputAudioFormat::Lame { .. } => filename.push_str(".mp3"),
        }

        let filepath = match state.render_settings.output_dir.clone() {
            Some(mut dir) => {
                dir.push(filename);
                dir
            }
            None => filename.into(),
        };

        let sample_rate = state.render_settings.sample_rate;
        let channels = state.render_settings.audio_channels.count();

        let writer: Box<dyn AudioWriter> = match state.render_settings.audio_format {
            OutputAudioFormat::Pcm { format } => Box::new(pcm::PCMFileWriter::new(
                channels,
                sample_rate,
                format,
                filepath,
            )?),
            OutputAudioFormat::Vorbis { bitrate } => Box::new(vorbis::VorbisFileWriter::new(
                channels,
                sample_rate,
                bitrate,
                filepath,
            )?),
            OutputAudioFormat::Lame { bitrate } => Box::new(lame::LameFileWriter::new(
                channels,
                sample_rate,
                bitrate,
                filepath,
            )?),
        };

        Ok(Self { writer })
    }

    pub fn write_samples(&mut self, sample: f32) -> Result<(), MIDIRendererError> {
        self.writer.write_sample(sample)
    }

    pub fn finalize(self) -> Result<(), MIDIRendererError> {
        self.writer.finalize()
    }
}
