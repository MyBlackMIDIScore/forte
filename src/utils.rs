use egui::Ui;

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
