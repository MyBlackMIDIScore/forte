use egui::containers::ComboBox;
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use egui_file::FileDialog;

use std::path::Path;

use crate::midi_list::ForteListItem;
use crate::state::{ForteState, RenderMode, Concurrency};

pub struct ForteApp {
    state: ForteState,
    midi_list: Vec<ForteListItem>,
    file_dialog: Option<FileDialog>,
}

impl Default for ForteApp {
    fn default() -> Self {
        Self {
            state: Default::default(),
            midi_list: Vec::new(),
            file_dialog: None,
        }
    }
}

impl ForteApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for ForteApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("List", |ui| {
                    if ui.button("Add File(s)").clicked() {
                        let filter = |path: &Path| {
                            if path.file_name().unwrap().to_str().unwrap().ends_with(".mid") {true}
                            else {false}
                        };
                        let filter = Box::new(filter);

                        let mut dialog = FileDialog::open_file(None)
                            .resizable(true)
                            .show_new_folder(false)
                            .filter(filter);
                        dialog.open();
                        self.file_dialog = Some(dialog);
                    }

                    if ui.button("Remove Selected").clicked() {
                        let mut to_remove = Vec::new();
                        for (i, item) in self.midi_list.iter().enumerate() {
                            if item.selected {
                                to_remove.push(i)
                            }
                        }
                        for i in to_remove {
                            self.midi_list.remove(i);
                        }
                    }

                    if ui.button("Clear List").clicked() {
                        self.midi_list.clear();
                    }
                });
                ui.menu_button("Misc", |ui| {
                    if ui.button("Preferences").clicked() {}
                    ui.separator();

                    if ui.button("About").clicked() {}
                    ui.separator();

                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });

        if let Some(dialog) = &mut self.file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    let item = ForteListItem::from_path(file);
                    self.midi_list.push(item);
                }
            }
        }

        egui::SidePanel::left("side_panel")
            .resizable(false)
            .exact_width(ctx.available_rect().width() / 5.0)
            .show(ctx, |ui| {
                let height = ui.available_rect_before_wrap().height();
                StripBuilder::new(ui)
                    .sizes(Size::exact(height / 3.0), 3)
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            let rect = ui.available_rect_before_wrap();
                            ui.heading("Setup");

                            ui.label("Mode:");
                            let mode = ["Standard", "Realtime Simulation"];
                            let mut mode_state = self.state.render_mode.into();
                            ComboBox::from_id_source("mode")
                                .width(rect.width() / 1.15)
                                .show_index(ui, &mut mode_state, mode.len(), |i| mode[i].to_owned());
                            if mode_state != self.state.render_mode.into() {
                                match mode_state {
                                    0 => self.state.render_mode = RenderMode::Standard,
                                    1 => self.state.render_mode = RenderMode::RealtimeSimulation,
                                    _ => self.state.render_mode = RenderMode::Standard,
                                };
                            }

                            ui.label("Concurrency:");
                            let conc =
                                ["None", "Items in parallel", "Tracks in parallel", "Both"];
                            let mut conc_state = self.state.concurrency.into();
                            ComboBox::from_id_source("concurrency")
                                .width(rect.width() / 1.15)
                                .show_index(ui, &mut conc_state, conc.len(), |i| conc[i].to_owned());
                            if conc_state != self.state.concurrency.into() {
                                match conc_state {
                                    0 => self.state.concurrency = Concurrency::None,
                                    1 => self.state.concurrency = Concurrency::ParallelItems,
                                    2 => self.state.concurrency = Concurrency::ParallelTracks,
                                    3 => self.state.concurrency = Concurrency::Both,
                                    _ => self.state.concurrency = Concurrency::None,
                                };
                            }
                        });

                        strip.cell(|ui| {
                            ui.separator();
                            let rect = ui.available_rect_before_wrap();
                            ui.heading("Actions");

                            //if ui.button("Synth Configuration").clicked() {}
                            //if ui.button("Render Configuration").clicked() {}

                            ui.add(egui::widgets::Button::new("Render").min_size(
                                egui::Vec2::new(rect.width() / 1.15, rect.height() / 3.0),
                            ));
                        });

                        strip.cell(|ui| {
                            ui.separator();
                            ui.heading("Progress");

                            ui.add(
                                egui::widgets::ProgressBar::new(0.75)
                                    .show_percentage()
                                    .animate(true),
                            );
                        });
                    });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::centered_and_justified(
                    egui::Direction::LeftToRight,
                ))
                .resizable(true)
                .column(Column::remainder())
                .columns(Column::auto().at_least(40.0), 4)
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
                        ui.strong("Track Count");
                    });
                    header.col(|ui| {
                        ui.strong("Note Count");
                    });
                })
                .body(|mut body| {
                    let row_height = 18.0;
                    for item in &mut self.midi_list {
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                let selectable = egui::SelectableLabel::new(item.selected, item.path.file_name().unwrap().to_str().unwrap());
                                if ui.add(selectable).clicked() {
                                    item.selected = !item.selected;
                                }
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", item.filesize));
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", item.length));
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", item.track_count));
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", item.note_count));
                            });
                        });
                    }
                });
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .min_height(0.0)
            .show(ctx, |ui| {
                ui.label("Waiting...");
            });
    }
}
