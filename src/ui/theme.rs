//! Theme configuration for egui.
//!
//! Dark theme similar to the original C++ ImGui theme.

use egui::{Color32, FontId, Stroke, Style, Visuals};

/// Apply the dark theme to egui context.
pub fn apply_dark_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Dark background colors
    let bg_color = Color32::from_rgb(30, 30, 46);
    let panel_color = Color32::from_rgb(36, 36, 54);
    let widget_color = Color32::from_rgb(45, 45, 68);
    let accent_color = Color32::from_rgb(100, 181, 246);
    let success_color = Color32::from_rgb(129, 199, 132);
    let error_color = Color32::from_rgb(239, 83, 80);
    let text_color = Color32::from_rgb(224, 224, 224);
    let dim_text = Color32::from_rgb(136, 136, 136);

    style.visuals = Visuals::dark();
    
    // Panel colors
    style.visuals.panel_fill = panel_color;
    style.visuals.window_fill = bg_color;
    style.visuals.extreme_bg_color = Color32::from_rgb(20, 20, 30);
    
    // Widget colors
    style.visuals.widgets.noninteractive.bg_fill = widget_color;
    style.visuals.widgets.inactive.bg_fill = widget_color;
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 90);
    style.visuals.widgets.active.bg_fill = accent_color;
    
    // Text colors
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_color);
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_color);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::BLACK);
    
    // Selection
    style.visuals.selection.bg_fill = accent_color.gamma_multiply(0.3);
    style.visuals.selection.stroke = Stroke::new(1.0, accent_color);
    
    // Hyperlinks
    style.visuals.hyperlink_color = accent_color;
    
    // Rounding
    style.visuals.window_rounding = 8.0.into();
    style.visuals.widgets.noninteractive.rounding = 4.0.into();
    style.visuals.widgets.inactive.rounding = 4.0.into();
    style.visuals.widgets.hovered.rounding = 4.0.into();
    style.visuals.widgets.active.rounding = 4.0.into();
    
    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = 12.0.into();
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    
    ctx.set_style(style);
}

/// Get the accent color for UI highlights.
pub fn accent_color() -> Color32 {
    Color32::from_rgb(100, 181, 246)
}

/// Get the success color for positive feedback.
pub fn success_color() -> Color32 {
    Color32::from_rgb(129, 199, 132)
}

/// Get the error color for negative feedback.
pub fn error_color() -> Color32 {
    Color32::from_rgb(239, 83, 80)
}

/// Get the warning color.
pub fn warning_color() -> Color32 {
    Color32::from_rgb(255, 183, 77)
}
