use crate::errors::error_message::ErrorMessage;
use crate::settings::ForteState;
use crate::tabs::{show_about, ForteRenderTab, ForteSynthTab, ForteTab};
use crate::utils::set_button_spacing;

pub struct ForteApp {
    state: ForteState,
    errors: Vec<ErrorMessage>,

    render_tab: ForteRenderTab,
    synth_tab: ForteSynthTab,
}

impl Default for ForteApp {
    fn default() -> Self {
        Self {
            state: Default::default(),
            errors: Vec::new(),

            render_tab: ForteRenderTab::new(),
            synth_tab: ForteSynthTab::new(),
        }
    }
}

impl ForteApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for ForteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.errors.retain(|er| er.is_visible());
        for error in self.errors.iter_mut() {
            error.show(ctx)
        }

        let add_error = |title: String, error: String| {
            self.errors.push(ErrorMessage::new(title, error));
        };

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            set_button_spacing(ui);
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.add_space(5.0);

                ui.heading("Forte");
                ui.separator();

                ui.selectable_value(
                    &mut self.state.ui_state.tab,
                    ForteTab::Synth,
                    "\u{1f3b9} Synth",
                );
                ui.selectable_value(
                    &mut self.state.ui_state.tab,
                    ForteTab::Render,
                    "\u{1f50a} Render",
                );
                ui.selectable_value(
                    &mut self.state.ui_state.tab,
                    ForteTab::About,
                    "\u{2139} About",
                );

                ui.allocate_space(egui::Vec2::new(ui.available_width() - 30.0, 0.0));
                egui::widgets::global_dark_light_mode_switch(ui);
            });
            ui.add_space(1.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            set_button_spacing(ui);
            match self.state.ui_state.tab {
                ForteTab::Render => self.render_tab.show(ui, &mut self.state, ctx, add_error),
                ForteTab::Synth => self.synth_tab.show(ui, &mut self.state, ctx, add_error),
                ForteTab::About => show_about(ui),
            }
        });
    }
}
