use crate::app::add_gui_error;
use crate::elements::sf_cfg::SoundfontConfigWindow;
use crate::errors::error_types::FileLoadError;
use crate::settings::ForteState;
use egui::{containers::scroll_area::ScrollArea, Context, Ui};
use egui_extras::{Column, TableBuilder};
use egui_file::FileDialog;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use tracing::{info, warn};
use xsynth_core::soundfont::{Interpolator, SoundfontInitOptions};
use xsynth_soundfonts::sfz::parse::parse_tokens_resolved;

#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum SFFormat {
    #[default]
    Sfz,
}

#[derive(Clone, PartialEq, Eq, Copy, Debug, Serialize, Deserialize)]
#[serde(remote = "Interpolator")]
pub enum InterpolatorDef {
    Nearest,
    Linear,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(remote = "SoundfontInitOptions")]
pub struct SoundfontInitOptionsDef {
    pub bank: Option<u8>,
    pub preset: Option<u8>,
    pub linear_release: bool,
    pub use_effects: bool,
    #[serde(with = "InterpolatorDef")]
    pub interpolator: Interpolator,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ForteSFListItem {
    pub id: usize,
    pub enabled: bool,
    pub selected: bool,
    pub format: SFFormat,
    pub path: PathBuf,
    #[serde(with = "SoundfontInitOptionsDef")]
    pub init: SoundfontInitOptions,
}

pub struct EguiSFList {
    list: Vec<ForteSFListItem>,
    id_count: usize,

    file_dialog: Option<FileDialog>,

    sf_cfg_win: Vec<SoundfontConfigWindow>,
}

impl EguiSFList {
    pub fn new(list: Vec<ForteSFListItem>) -> Self {
        Self {
            list,
            id_count: 0,
            file_dialog: None,
            sf_cfg_win: Vec::new(),
        }
    }

    pub fn add_item(&mut self, path: PathBuf) -> Result<(), FileLoadError> {
        info!("Adding a soundfont to the list: {:?}", path);
        if !path.exists() {
            warn!("The selected soundfont does not exist");
            return Err(FileLoadError::FileNotFound);
        }

        if let Some(ext) = path.extension() {
            if ext == "sfz" {
                info!("Checking soundfont integrity");
                match parse_tokens_resolved(path.as_path()) {
                    Ok(..) => {
                        let item = ForteSFListItem {
                            id: self.id_count,
                            enabled: true,
                            selected: false,
                            format: SFFormat::Sfz,
                            path,
                            init: SoundfontInitOptions {
                                bank: Some(0),
                                preset: Some(0),
                                interpolator: Interpolator::Linear,
                                ..Default::default()
                            },
                        };
                        self.list.push(item);
                        self.id_count += 1;
                        Ok(())
                    }
                    Err(error) => {
                        warn!("The selected soundfont is corrupt: {error}");
                        Err(FileLoadError::Corrupt(error.to_string()))
                    }
                }
            } else {
                warn!("The selected soundfont does not have the correct format");
                Err(FileLoadError::InvalidFormat)
            }
        } else {
            warn!("The selected soundfont does not have the correct format");
            Err(FileLoadError::InvalidFormat)
        }
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

        // I'm bored to make it close only the windows needed, so instead I'll close all of them
        self.sf_cfg_win.clear();
    }

    pub fn clear(&mut self) {
        self.list.clear();
        self.sf_cfg_win.clear();
    }

    pub fn iter_list(&self) -> std::vec::IntoIter<ForteSFListItem> {
        self.list.clone().into_iter()
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &Context, state: &mut ForteState) {
        let events = ui.input(|i| i.events.clone());
        for event in &events {
            if let egui::Event::Key {
                key,
                modifiers,
                pressed,
                ..
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

        if !ui.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ui.input(|i| {
                i.raw
                    .dropped_files
                    .clone()
                    .iter()
                    .map(|file| file.path.as_ref().unwrap().clone())
                    .collect::<Vec<PathBuf>>()
            });

            for file in dropped_files {
                if let Err(error) = self.add_item(file.clone()) {
                    let title = if let Some(filen) = file.file_name() {
                        // Not a safe unwrap but things must be very wrong for it to panic so idc
                        format!(
                            "There was an error adding \"{}\" to the list.",
                            filen.to_str().unwrap()
                        )
                    } else {
                        "There was an error adding the selected soundfont to the list.".to_string()
                    };
                    add_gui_error(title, error.to_string());
                }
            }
        }

        self.sf_cfg_win = self
            .sf_cfg_win
            .clone()
            .into_iter()
            .filter(|item| item.visible)
            .collect();

        for cfg in self.sf_cfg_win.iter_mut() {
            let index = self.list.iter().position(|item| item.id == cfg.id());
            if let Some(index) = index {
                cfg.show(ctx, &mut self.list[index]);
            }
        }

        egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.button("Add Soundfont").clicked() {
                    let filter = |path: &Path| {
                        if let Some(path) = path.to_str() {
                            path.ends_with(".sfz")
                        } else {
                            false
                        }
                    };
                    let filter = Box::new(filter);

                    let mut dialog = FileDialog::open_file(state.ui_state.sf_select_last_path.clone())
                    .resizable(true)
                    .show_new_folder(false)
                    .show_rename(false)
                    .filter(filter);
                    dialog.open();
                    self.file_dialog = Some(dialog);
                }
                if ui.button("Remove Selected").clicked() {
                    self.remove_selected_items();
                }
                if ui.button("Clear List").clicked() {
                    self.clear();
                }
                ui.label("Loading order is top to bottom.");
                ui.label("Supported formats: SFZ");

                if let Some(dialog) = &mut self.file_dialog {
                    if dialog.show(ctx).selected() {
                        if let Some(path) = dialog.path() {
                            state.ui_state.sf_select_last_path = Some(path.clone());
                            if path.is_file() {
                                if let Err(error) = self.add_item(path.clone()) {
                                    let title = if let Some(filen) = path.file_name() {
                                        // Not a safe unwrap but things must be very wrong for it to panic so idc
                                        format!("There was an error adding \"{}\" to the list.", filen.to_str().unwrap())
                                    } else {
                                        "There was an error adding the selected soundfont to the list.".to_string()
                                    };
                                    add_gui_error(title, error.to_string());
                                }
                            }
                        }
                    }
                }
            });
        });

        ScrollArea::both().show(ui, |ui| {
            let events = ui.input(|i| i.events.clone());
            for event in &events {
                if let egui::Event::Key {
                    key,
                    modifiers,
                    pressed,
                    ..
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
                .column(Column::exact(20.0).resizable(false))
                .column(Column::initial(400.0).at_least(50.0).clip(true))
                .columns(Column::auto().at_least(40.0).clip(true), 2)
                .column(Column::auto().at_least(40.0).clip(true).resizable(false))
                .header(20.0, |mut header| {
                    header.col(|_ui| {});
                    header.col(|ui| {
                        ui.strong("Filename");
                    });
                    header.col(|ui| {
                        ui.strong("Format");
                    });
                    header.col(|ui| {
                        ui.strong("Bank");
                    });
                    header.col(|ui| {
                        ui.strong("Preset");
                    });
                })
                .body(|mut body| {
                    let row_height = 24.0;
                    for item in self.list.iter_mut() {
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                ui.checkbox(&mut item.enabled, "");
                            });
                            row.col(|ui| {
                                let selectable = if let Some(path) = item.path.to_str() {
                                    ui.selectable_label(item.selected, path)
                                } else {
                                    ui.selectable_label(item.selected, "error")
                                };

                                if selectable.clicked() {
                                    item.selected = !item.selected;
                                }
                                if selectable.double_clicked()
                                    && !self.sf_cfg_win.iter().any(|cfg| cfg.id() == item.id)
                                {
                                    self.sf_cfg_win.push(SoundfontConfigWindow::new(item.id))
                                }
                            });
                            row.col(|ui| {
                                ui.label(match item.format {
                                    SFFormat::Sfz => "SFZ",
                                });
                            });

                            let bank_txt = if let Some(bank) = item.init.bank {
                                format!("{}", bank)
                            } else {
                                "None".to_owned()
                            };
                            row.col(|ui| {
                                ui.label(bank_txt.to_string());
                            });

                            let preset_txt = if let Some(preset) = item.init.preset {
                                format!("{}", preset)
                            } else {
                                "None".to_owned()
                            };
                            row.col(|ui| {
                                ui.label(preset_txt.to_string());
                            });
                        });
                    }
                });
            ui.allocate_space(ui.available_size());
        });
    }
}
