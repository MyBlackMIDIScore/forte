use crate::errors::error_message::ErrorMessage;
use crate::settings::ForteState;
use crate::tabs::{show_about, ForteRenderTab, ForteSynthTab, ForteTab};
use crate::utils::set_button_spacing;
use eframe::glow::Context;
use std::time::Duration;

use tracing::info;

static mut GUI_ERRORS: Vec<ErrorMessage> = Vec::new();

pub struct ForteApp {
    state: ForteState,

    render_tab: ForteRenderTab,
    synth_tab: ForteSynthTab,
}

impl ForteApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        //Self::set_font(&cc.egui_ctx);
        let state = ForteState::load();
        Self {
            render_tab: ForteRenderTab::new(),
            synth_tab: ForteSynthTab::new(&state),
            state,
        }
    }

    /*fn set_font(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "poppins".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/Poppins-Light.ttf")),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "poppins".to_owned());

        ctx.set_fonts(fonts);
    }*/
}

impl eframe::App for ForteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(20));

        unsafe {
            GUI_ERRORS.retain(|er| er.is_visible());
            for error in GUI_ERRORS.iter_mut() {
                error.show(ctx)
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            set_button_spacing(ui);
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.add_space(5.0);

                ui.heading("Forte");
                ui.separator();

                ui.add_enabled_ui(!self.state.ui_state.rendering, |ui| {
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
                });

                ui.allocate_space(egui::Vec2::new(ui.available_width() - 30.0, 0.0));
                egui::widgets::global_dark_light_mode_switch(ui);
            });
            ui.add_space(1.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            set_button_spacing(ui);
            match &self.state.ui_state.tab {
                ForteTab::Render => self.render_tab.show(ui, &mut self.state, ctx),
                ForteTab::Synth => self.synth_tab.show(ui, &mut self.state, ctx),
                ForteTab::About => show_about(ui),
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&Context>) {
        self.state.save().unwrap_or_else(|_| {});
    }
}

pub fn add_gui_error(title: String, body: String) {
    info!("Adding new GUI error message");
    unsafe {
        GUI_ERRORS.push(ErrorMessage::new(title, body));
    }
}
