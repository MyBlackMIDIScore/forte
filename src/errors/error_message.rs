use egui::{Context, Window};
use rand;

#[derive(Clone)]
pub struct ErrorMessage {
    id: usize,
    title: String,
    body: String,
    is_visible: bool,
}

impl ErrorMessage {
    pub fn new(title: String, body: String) -> Self {
        Self {
            id: rand::random(),
            title,
            body,
            is_visible: true,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn show(&mut self, ctx: &Context) {
        Window::new("Error")
            .id(egui::Id::new(self.id))
            .resizable(false)
            .collapsible(false)
            .open(&mut self.is_visible)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(&self.title);
                    ui.separator();
                    ui.label(&self.body);
                });
            });
    }
}
