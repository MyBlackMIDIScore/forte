use egui::{Context, Window};
use rand;

#[derive(Clone, PartialEq)]
enum MessageType {
    Error,
    Update,
}

#[derive(Clone)]
pub struct PersistentMessage {
    msg_type: MessageType,
    id: usize,
    string1: String,
    string2: String,
    string3: String,
    is_visible: bool,
}

impl PersistentMessage {
    pub fn error(title: String, body: String) -> Self {
        Self {
            msg_type: MessageType::Error,
            id: rand::random(),
            string1: title,
            string2: body,
            string3: String::new(),
            is_visible: true,
        }
    }

    pub fn update(version: String, url: String, body: String) -> Self {
        Self {
            msg_type: MessageType::Update,
            id: rand::random(),
            string1: version,
            string2: url,
            string3: body,
            is_visible: true,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn show(&mut self, ctx: &Context) {
        match self.msg_type {
            MessageType::Error => {
                Window::new("Error")
                    .id(egui::Id::new(self.id))
                    .resizable(false)
                    .collapsible(false)
                    .open(&mut self.is_visible)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(&self.string1);
                            ui.separator();
                            ui.label(&self.string2);
                        });
                    });
            }
            MessageType::Update => {
                Window::new("New update found!")
                    .id(egui::Id::new(self.id))
                    .resizable(false)
                    .collapsible(false)
                    .open(&mut self.is_visible)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(format!(
                                "A new version of Forte is available: {}",
                                &self.string1
                            ));
                            ui.separator();
                            ui.label(&self.string3);
                            ui.separator();
                            if ui.button("Download").clicked() {
                                open::that(&self.string2).unwrap_or_default();
                            }
                        });
                    });
            }
        }
    }
}
