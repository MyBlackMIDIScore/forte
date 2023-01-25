use std::path::PathBuf;

#[derive(Clone)]
pub struct ForteListItem {
    pub selected: bool,
    pub path: PathBuf,
    pub filesize: usize,
    pub length: f64,
    pub track_count: usize,
    pub note_count: usize,
}

impl ForteListItem {
    pub fn from_path(path: PathBuf) -> Self {
        Self {
            selected: false,
            path,
            filesize: 1000,
            length: 1.,
            track_count: 16,
            note_count: 100,
        }
    }
}
