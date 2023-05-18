use egui::{Context, Ui};
use egui_extras::{Size, StripBuilder};

use crate::app::add_gui_error;
use crate::elements::{midi_list::EguiMIDIList, render_settings::show_render_settings};
use crate::settings::ForteState;
use crate::utils::render_in_frame;
use crate::xsynth::{ManagerStatus, RenderThreadManager};
use tracing::{error, info};

use egui_file::FileDialog;
use std::path::Path;

pub struct ForteRenderTab {
    midi_list: EguiMIDIList,
    file_dialog: Option<FileDialog>,
    out_select_dialog: Option<FileDialog>,
    render_manager: Option<RenderThreadManager>,
}

impl ForteRenderTab {
    pub fn new() -> Self {
        Self {
            midi_list: EguiMIDIList::new(),
            file_dialog: None,
            out_select_dialog: None,
            render_manager: None,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, state: &mut ForteState, ctx: &Context) {
        let mut ended = true;
        if state.ui_state.rendering {
            if self.file_dialog.is_some() {
                self.file_dialog = None;
            }

            if let Some(mgr) = self.render_manager.as_mut() {
                let status = mgr.status();
                if status == ManagerStatus::LoadingSoundfonts {
                    egui::Window::new("Loading SoundFonts...")
                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                        .resizable(false)
                        .collapsible(false)
                        .show(ctx, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label("The application is loading the SoundFonts to RAM.")
                            })
                        });
                    ended = false;
                } else if status == ManagerStatus::SFLoadError {
                    state.ui_state.rendering = false;
                    mgr.cancel();
                    error!("Invalid Soundfont chain. Aborting render.");
                    add_gui_error(
                        "Soundfont Loader Error".to_owned(),
                        "Invalid Soundfont chain".to_owned(),
                    );
                } else if status == ManagerStatus::SoundfontsFinished {
                    info!("Starting export");
                    mgr.render(state);
                    ended = false;
                } else if status == ManagerStatus::RenderingMIDIs {
                    mgr.spawn_next();
                    ended = false;
                } else if status == ManagerStatus::RenderFinished && !mgr.spawn_next() {
                    info!("Conversion finished");
                    state.ui_state.rendering = false;
                    mgr.cancel();
                }
            }
        }
        if ended {
            self.render_manager.take();
        }

        let progress = if let Some(mgr) = self.render_manager.as_mut() {
            if mgr.has_finished() {
                None
            } else {
                Some(mgr.get_stats())
            }
        } else {
            None
        };
        self.midi_list.set_stats(progress);

        egui::TopBottomPanel::bottom("render_bottom_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                let width = ui.available_rect_before_wrap().width();
                ui.add_space(5.0);
                StripBuilder::new(ui)
                    .sizes(Size::exact(width / 2.0), 2)
                    .horizontal(|mut strip| {
                        strip.cell(|ui| {
                            let rect = ui.available_rect_before_wrap();
                            ui.heading("List");

                            ui.add_space(8.0);

                            ui.add_enabled_ui(!state.ui_state.rendering, |ui| {
                                ui.horizontal(|ui| {
                                    if ui.add(egui::Button::new("Add MIDI").min_size(egui::Vec2::new(rect.width() / 2.0 - 5.0, 18.0))).clicked() {
                                        let filter = |path: &Path| {
                                            if let Some(path) = path.to_str() {
                                                path.ends_with(".mid")
                                            } else {
                                                false
                                            }
                                        };
                                        let filter = Box::new(filter);

                                        let mut dialog = FileDialog::open_file(None)
                                        .resizable(true)
                                        .show_new_folder(false)
                                        .show_rename(false)
                                        .filter(filter);
                                        dialog.open();
                                        self.file_dialog = Some(dialog);
                                    }
                                    if ui.add(egui::Button::new("Add Folder").min_size(egui::Vec2::new(rect.width() / 2.0 - 5.0, 18.0))).clicked() {
                                        let mut dialog = FileDialog::select_folder(None)
                                        .resizable(true)
                                        .show_new_folder(false)
                                        .show_rename(false);
                                        dialog.open();
                                        self.file_dialog = Some(dialog);
                                    }
                                });

                                ui.horizontal(|ui| {
                                    if ui.add(egui::Button::new("Remove Selected").min_size(egui::Vec2::new(rect.width() / 2.0 - 5.0, 18.0))).clicked() {
                                        self.midi_list.remove_selected_items();
                                    }
                                    if ui.add(egui::Button::new("Clear").min_size(egui::Vec2::new(rect.width() / 2.0 - 5.0, 18.0))).clicked() {
                                        self.midi_list.clear();
                                    }
                                });
                            });

                            if let Some(dialog) = &mut self.file_dialog {
                                if dialog.show(ctx).selected() {
                                    if let Some(path) = dialog.path() {
                                        if path.is_file() {
                                            if let Err(error) = self.midi_list.add_item(path.clone()) {
                                                let title = if let Some(filen) = path.file_name() {
                                                    // Not a safe unwrap but things must be very wrong for it to panic so idc
                                                    format!("There was an error adding \"{}\" to the list.", filen.to_str().unwrap())
                                                } else {
                                                    "There was an error adding the selected MIDI to the list.".to_string()
                                                };
                                                add_gui_error(title, error.to_string());
                                            }
                                        } else if path.is_dir() {
                                            if let Err(error) = self.midi_list.add_folder(path.clone()) {
                                                let title = if let Some(dirn) = path.file_name() {
                                                    // Same as above
                                                    format!("There was an error adding one file from \"{}\" to the list.", dirn.to_str().unwrap())
                                                } else {
                                                    "There was an error adding the selected folder to the list.".to_string()
                                                };
                                                add_gui_error(title, error.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        });

                        strip.cell(|ui| {
                            let rect = ui.available_rect_before_wrap();
                            ui.heading("Actions");

                            ui.horizontal(|ui| {
                                if ui.add_sized([rect.width() / 4.0 - ui.style().spacing.button_padding.x, 40.0], egui::Button::new("Settings").wrap(true)).clicked() {
                                    state.ui_state.render_settings_visible = !state.ui_state.render_settings_visible;
                                }

                                if state.ui_state.rendering {
                                    if ui.add(egui::Button::new("Cancel").min_size(egui::Vec2::new(3.0 * rect.width() / 4.0 - ui.style().spacing.button_padding.x, 40.0))).clicked() {
                                        self.cancel_render();
                                    }
                                } else {
                                    if ui.add_enabled(!self.midi_list.is_empty(), egui::Button::new("Convert!").min_size(egui::Vec2::new(3.0 * rect.width() / 4.0 - ui.style().spacing.button_padding.x, 40.0))).clicked() {
                                        let mut dialog = FileDialog::select_folder(None)
                                            .resizable(true)
                                            .show_new_folder(false)
                                            .show_rename(false);
                                        dialog.open();
                                        self.out_select_dialog = Some(dialog);
                                    }

                                    if let Some(dialog) = &mut self.out_select_dialog {
                                        if dialog.show(ctx).selected() {
                                            if let Some(path) = dialog.path() {
                                                if path.is_dir() {
                                                    state.ui_state.rendering = true;
                                                    state.render_settings.output_dir = Some(path);

                                                    let midis = self.midi_list.iter_list().map(|item| item.path).collect();

                                                    info!("Loading soundfonts");

                                                    match RenderThreadManager::new(state, midis) {
                                                        Ok(m) => self.render_manager = Some(m),
                                                        Err(err) => {
                                                            state.ui_state.rendering = false;
                                                            add_gui_error(
                                                                "Renderer Error".to_owned(),
                                                                err.to_string(),
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            });


                            ui.add_space(5.0);

                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::widgets::ProgressBar::new(self.midi_list.get_total_progress())
                                    .desired_width(rect.width() - 5.0)
                                    .show_percentage()
                                );
                            });
                        });
                    });
            });

        if state.ui_state.render_settings_visible {
            egui::SidePanel::right("render_settings")
                .resizable(false)
                .show_inside(ui, |ui| show_render_settings(ui, state));
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            render_in_frame(ui, |ui| {
                self.midi_list.show(ui);
            });
        });
    }

    pub fn cancel_render(&mut self) {
        info!("Aborting render per user request");
        self.state.ui_state.rendering = false;
        if let Some(mgr) = self.render_manager.as_mut() {
            mgr.cancel();
        }
    }
}
