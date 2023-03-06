use crate::errors::error_types::FileLoadError;
use crate::utils::{bytes_to_filesize_str, f64_to_time_str};
use egui::{containers::scroll_area::ScrollArea, Ui};
use egui_extras::{Column, TableBuilder};
use midi_toolkit::{
    io::{MIDIFile, MIDILoadError},
    pipe,
    sequence::{event::get_channels_array_statistics, to_vec},
};
use num_format::{Locale, ToFormattedString};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ForteListItem {
    pub selected: bool,
    pub path: PathBuf,
    pub filesize: u64,
    pub length: f64,
    pub note_count: u64,
}

pub struct EguiMIDIList {
    list: Vec<ForteListItem>,
    progress: Option<Vec<Option<f32>>>,
}

impl EguiMIDIList {
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
            progress: None,
        }
    }

    pub fn add_item(&mut self, path: PathBuf) -> Result<(), FileLoadError> {
        if !path.exists() {
            return Err(FileLoadError::FileNotFound);
        }

        if let Some(ext) = path.extension() {
            if ext == "mid" {
                let file = MIDIFile::open(path.clone(), None);
                match file {
                    Ok(midi) => {
                        let stats = pipe!(
                            midi.iter_all_tracks()|>to_vec()|>get_channels_array_statistics().unwrap()
                        );

                        let length = stats.calculate_total_duration(midi.ppq()).as_secs_f64();
                        let filesize = std::fs::metadata(path.clone()).unwrap().len();

                        let item = ForteListItem {
                            selected: false,
                            path,
                            filesize,
                            length,
                            note_count: stats.note_count(),
                        };
                        self.list.push(item);
                        Ok(())
                    }
                    Err(error) => match error {
                        MIDILoadError::CorruptChunks => {
                            Err(FileLoadError::Corrupt("Corrupt chunks".to_owned()))
                        }
                        MIDILoadError::FilesystemError(fserr) => {
                            Err(FileLoadError::Corrupt(format!("Filesystem error: {fserr}")))
                        }
                        MIDILoadError::FileTooBig => {
                            Err(FileLoadError::Corrupt("MIDI file too big".to_owned()))
                        }
                    },
                }
            } else {
                Err(FileLoadError::InvalidFormat)
            }
        } else {
            Err(FileLoadError::InvalidFormat)
        }
    }

    pub fn add_folder(&mut self, dir: PathBuf) -> Result<(), FileLoadError> {
        let mut result: Result<(), FileLoadError> = Ok(());
        if let Ok(paths) = std::fs::read_dir(dir) {
            for p in paths {
                if let Ok(p) = p {
                    let p = p.path();
                    if p.is_dir() {
                        result = self.add_folder(p);
                    } else if let Some(ext) = p.extension() {
                        if ext == "mid" {
                            result = self.add_item(p);
                        }
                    }
                }
            }
        } else {
            result = Err(FileLoadError::FileNotFound);
        }
        result
    }

    pub fn select_all(&mut self) {
        self.list = self
            .list
            .clone()
            .into_iter()
            .map(|mut item| {
                item.selected = true;
                item
            })
            .collect();
    }

    pub fn remove_selected_items(&mut self) {
        self.list = self
            .list
            .clone()
            .into_iter()
            .filter(|item| !item.selected)
            .collect();
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn iter_list(&self) -> std::vec::IntoIter<ForteListItem> {
        self.list.clone().into_iter()
    }

    pub fn set_progress(&mut self, progress: Option<Vec<Option<f64>>>) {
        self.progress = match progress {
            Some(progress) => {
                let mut out = Vec::new();
                for (i, p) in progress.iter().enumerate() {
                    match p {
                        Some(p) => out.push(Some((p / self.list[i].length) as f32)),
                        None => out.push(None),
                    }
                }
                Some(out)
            }
            None => None,
        };
    }

    pub fn get_total_progress(&self) -> f32 {
        if let Some(progress) = &self.progress {
            let mut out = 0.0;
            let len = progress.len();
            for p in progress {
                if let Some(p) = p {
                    out += p;
                }
            }
            out / (len as f32)
        } else {
            0.0
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ScrollArea::both().show(ui, |ui| {
            let events = ui.input().events.clone();
            for event in &events {
                if let egui::Event::Key {
                    key,
                    modifiers,
                    pressed,
                } = event
                {
                    match *key {
                        egui::Key::A => {
                            if *pressed && modifiers.ctrl {
                                self.select_all();
                            }
                        }
                        egui::Key::Delete => {
                            self.remove_selected_items();
                        }
                        _ => {}
                    }
                }
            }

            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::centered_and_justified(
                    egui::Direction::LeftToRight,
                ))
                .resizable(true)
                .column(Column::initial(400.0).at_least(50.0).clip(true))
                .columns(Column::auto().at_least(80.0).clip(true), 2)
                .column(Column::auto().at_least(80.0).clip(true).resizable(false))
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Filename");
                    });
                    header.col(|ui| {
                        ui.strong("Size");
                    });
                    header.col(|ui| {
                        ui.strong("Length");
                    });
                    header.col(|ui| {
                        ui.strong("Note Count");
                    });
                })
                .body(|mut body| {
                    let row_height = 24.0;
                    for (idx, item) in self.list.iter_mut().enumerate() {
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                let txt = if let Some(filename) = item.path.file_name() {
                                    if let Some(txt) = filename.to_str() {
                                        txt
                                    } else {
                                        "error"
                                    }
                                } else {
                                    "error"
                                };

                                let mut gen_selectable = || {
                                    let selectable = egui::SelectableLabel::new(item.selected, txt);
                                    if ui.add(selectable).clicked() {
                                        item.selected = !item.selected;
                                    }
                                };

                                if let Some(progress) = &self.progress {
                                    if let Some(progress) = progress.get(idx) {
                                        if let Some(progress) = progress {
                                            ui.horizontal(|ui| {
                                                ui.add(
                                                    egui::widgets::ProgressBar::new(*progress)
                                                        .text(txt),
                                                );
                                            });
                                        } else {
                                            gen_selectable();
                                        }
                                    } else {
                                        gen_selectable();
                                    }
                                } else {
                                    gen_selectable();
                                }
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", bytes_to_filesize_str(item.filesize)));
                            });
                            row.col(|ui| {
                                ui.label(f64_to_time_str(item.length));
                            });
                            row.col(|ui| {
                                ui.label(format!(
                                    "{}",
                                    item.note_count.to_formatted_string(&Locale::en)
                                ));
                            });
                        });
                    }
                });
            ui.allocate_space(ui.available_size());
        });
    }
}
