use std::fmt;

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
