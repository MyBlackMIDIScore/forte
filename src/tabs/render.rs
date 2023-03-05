use egui::{Context, Ui};
use egui_extras::{Size, StripBuilder};

use crate::elements::{midi_list::EguiMIDIList, render_settings::show_render_settings};
use crate::settings::ForteState;
use crate::utils::render_in_frame;
use crate::xsynth::{ManagerStatus, RenderThreadManager};

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

    pub fn show<E>(
        &mut self,
        ui: &mut Ui,
        state: &mut ForteState,
        ctx: &Context,
        errors_callback: E,
        //apply_synth_settings: A,
    ) where
        E: FnOnce(String, String) + Clone,
        //A: FnOnce(&mut ForteState),
    {
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
                    (errors_callback).clone()(
                        "Soundfont Loader Error".to_owned(),
                        "Invalid Soundfont Chain".to_owned(),
                    );
                } else if status == ManagerStatus::SoundfontsFinished {
                    mgr.render(state);
                    ended = false;
                } else if status == ManagerStatus::RenderingMIDIs {
                    mgr.spawn_next();
                    ended = false;
                } else if status == ManagerStatus::RenderFinished {
                    if !mgr.spawn_next() {
                        state.ui_state.rendering = false;
                        mgr.cancel();
                    }
                }
            }
        }
        if ended {
            let tmp = self.render_manager.take();
            std::mem::drop(tmp);
        }

        let progress = if let Some(mgr) = self.render_manager.as_mut() {
            if mgr.has_finished() {
                None
            } else {
                Some(mgr.get_progress())
            }
        } else {
            None
        };
        self.midi_list.set_progress(progress);

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
                                                if path.ends_with(".mid") {
                                                    true
                                                } else {
                                                    false
                                                }
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
                                                errors_callback(title, error.to_string());
                                            }
                                        } else if path.is_dir() {
                                            if let Err(error) = self.midi_list.add_folder(path.clone()) {
                                                let title = if let Some(dirn) = path.file_name() {
                                                    // Same as above
                                                    format!("There was an error adding one file from \"{}\" to the list.", dirn.to_str().unwrap())
                                                } else {
                                                    "There was an error adding the selected folder to the list.".to_string()
                                                };
                                                errors_callback(title, error.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        });

                        strip.cell(|ui| {
                            let rect = ui.available_rect_before_wrap();
                            ui.heading("Render");

                            ui.horizontal(|ui| {
                                if ui.add(egui::Button::new("Render Settings").min_size(egui::Vec2::new(rect.width() / 4.0 - 5.0, 40.0))).clicked() {
                                    state.ui_state.render_settings_visible = !state.ui_state.render_settings_visible;
                                }

                                if state.ui_state.rendering {
                                    if ui.add(egui::Button::new("Cancel").min_size(egui::Vec2::new(3.0 * rect.width() / 4.0 - 5.0, 40.0))).clicked() {
                                        state.ui_state.rendering = false;
                                        if let Some(mgr) = self.render_manager.as_mut() {
                                            mgr.cancel();
                                        }
                                    }
                                } else {
                                    if ui.add(egui::Button::new("Render!").min_size(egui::Vec2::new(3.0 * rect.width() / 4.0 - 5.0, 40.0))).clicked() && !self.midi_list.is_empty() {
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
                                                    //apply_synth_settings(state);
                                                    state.render_settings.output_dir = Some(path);

                                                    let midis = self.midi_list.iter_list().map(|item| item.path).collect();

                                                    match RenderThreadManager::new(state, midis) {
                                                        Ok(m) => self.render_manager = Some(m),
                                                        Err(..) => {
                                                            state.ui_state.rendering = false;
                                                            /*(errors_callback).clone()(
                                                                "Renderer Error".to_owned(),
                                                                err.to_string(),
                                                            );*/
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
}
