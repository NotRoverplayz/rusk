use eframe::egui::{self, Color32, Stroke};

pub const BACKGROUND: Color32 = Color32::from_rgb(28, 29, 33);
pub const SIDEBAR: Color32 = Color32::from_rgb(34, 35, 40);
pub const PANEL: Color32 = Color32::from_rgb(39, 40, 46);
pub const HOVER: Color32 = Color32::from_rgb(51, 51, 59);
pub const ACCENT: Color32 = Color32::from_rgb(160, 137, 210);
pub const TEXT: Color32 = Color32::from_rgb(229, 229, 234);
pub const MUTED: Color32 = Color32::from_rgb(151, 152, 162);
pub const LINE: Color32 = Color32::from_rgb(55, 56, 63);

pub fn apply(context: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(TEXT);
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = Color32::from_rgb(25, 26, 30);
    visuals.faint_bg_color = PANEL;
    visuals.selection.bg_fill = Color32::from_rgb(69, 60, 88);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.noninteractive.bg_fill = BACKGROUND;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, MUTED);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
    visuals.widgets.inactive.bg_fill = PANEL;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, LINE);
    visuals.widgets.hovered.bg_fill = HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.active.bg_fill = Color32::from_rgb(69, 60, 88);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    context.set_visuals(visuals);

    let mut style = (*context.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(9.0, 5.0);
    context.set_style(style);
}
