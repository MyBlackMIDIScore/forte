use crate::{app::add_update_message, VERSION};
use egui::Ui;
use reqwest::blocking::ClientBuilder;
use std::{
    collections::HashMap,
    env::consts::{ARCH, OS},
};
use tracing::info;

pub fn set_button_spacing(ui: &mut Ui) {
    ui.spacing_mut().button_padding = (6.0, 3.0).into();
    ui.visuals_mut().widgets.inactive.rounding = egui::Rounding::same(5.0);
    ui.visuals_mut().widgets.active.rounding = egui::Rounding::same(5.0);
    ui.visuals_mut().widgets.hovered.rounding = egui::Rounding::same(5.0);
}

pub fn f64_to_time_str(time: f64) -> String {
    let millis = (time * 10.0) as u64 % 10;
    let secs = time as u64 % 60;
    let mins = time as u64 / 60;

    format!("{:0width$}:{:0width$}.{}", mins, secs, millis, width = 2)
}

pub fn bytes_to_filesize_str(size: u64) -> String {
    if size < 1000 {
        format!("{size}B")
    } else if (1000..1000000).contains(&size) {
        format!("{:.1}KB", size as f32 / 1000.0)
    } else if size >= 1000000 {
        format!("{:.1}MB", size as f32 / 1000000.0)
    } else {
        "error".to_owned()
    }
}

pub fn render_in_frame<E>(ui: &mut Ui, resp: E)
where
    E: FnOnce(&mut Ui),
{
    egui::containers::Frame::none()
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .rounding(5.0)
        .inner_margin(5.0)
        .show(ui, resp);
}

pub fn get_latest_version() -> String {
    let current = crate::VERSION.to_owned();
    let api_url = "https://api.github.com/repos/MyBlackMIDIScore/forte/releases/latest";

    let version = if let Ok(client) = ClientBuilder::new().user_agent("ForteUpdate").build() {
        if let Ok(data) = client.get(api_url).send() {
            let txt = data.text().unwrap_or_default();
            if let Ok(json) =
                serde_json::from_str::<HashMap<String, serde_json::value::Value>>(&txt)
            {
                if let Some(tag) = json.get("tag_name") {
                    tag.as_str().unwrap_or("").to_owned()
                } else {
                    current
                }
            } else {
                current
            }
        } else {
            current
        }
    } else {
        current
    };
    version
}

pub fn get_release_filename() -> String {
    let ext = if OS == "windows" { ".exe" } else { "" };

    format!("forte-{}-{}{}", OS, ARCH, ext)
}

pub fn check_for_updates() {
    let latest = get_latest_version();
    if latest != VERSION {
        info!("New update found: {}", latest);
        let url = format!(
            "https://github.com/MyBlackMIDIScore/forte/releases/latest/download/{}",
            get_release_filename()
        );
        add_update_message(latest, url)
    }
}
