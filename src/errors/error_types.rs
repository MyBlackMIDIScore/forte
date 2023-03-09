use midi_toolkit::io::MIDILoadError;
use std::fmt;

#[derive(Debug)]
pub enum FileLoadError {
    InvalidFormat,
    FileNotFound,
    Corrupt(String),
}

impl fmt::Display for FileLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileLoadError::InvalidFormat => write!(f, "Invalid File Format"),
            FileLoadError::FileNotFound => write!(f, "File Not Found"),
            FileLoadError::Corrupt(string) => write!(f, "File Corrupt: {string}"),
        }
    }
}

#[derive(Debug)]
pub enum MIDIRendererError {
    Load(MIDILoadError),
    Renderer(String),
    Writer(String),
}

impl fmt::Display for MIDIRendererError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MIDIRendererError::Load(e) => match e {
                MIDILoadError::CorruptChunks => write!(f, "MIDI Load Error: Corrupt Chunks"),
                MIDILoadError::FilesystemError(fs) => {
                    write!(f, "MIDI Load Error: Filesystem Error ({fs})")
                }
                MIDILoadError::FileTooBig => write!(f, "MIDI Load Error: File Too Big"),
            },
            MIDIRendererError::Renderer(string) => write!(f, "Renderer Error: {string}"),
            MIDIRendererError::Writer(string) => write!(f, "Writer Error: {string}"),
        }
    }
}
