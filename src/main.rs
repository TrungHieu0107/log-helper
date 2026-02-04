#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod core;
mod utils;

use app::SqlLogParserApp;
use eframe::egui;

fn setup_cjk_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Priority list of CJK fonts to try
    let font_candidates = [
        "meiryo.ttc",
        "msgothic.ttc",
        "yugothr.ttc", // Yu Gothic
    ];

    let fonts_dir = "C:\\Windows\\Fonts";
    
    for font_name in font_candidates {
        let font_path = std::path::Path::new(fonts_dir).join(font_name);
        if font_path.exists() {
            if let Ok(font_data) = std::fs::read(&font_path) {
                // Name the font "cjk"
                fonts.font_data.insert(
                    "cjk".to_owned(),
                    egui::FontData::from_owned(font_data).tweak(
                        egui::FontTweak {
                            scale: 1.0, 
                            ..Default::default()
                        }
                    ),
                );

                // Add "cjk" to Proportional fonts (fallback)
                if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    family.push("cjk".to_owned());
                }

                // Add "cjk" to Monospace fonts (fallback)
                if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                    family.push("cjk".to_owned());
                }
                
                println!("Loaded CJK font: {}", font_name);
                break; // Stop after finding first working font
            }
        }
    }

    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "SQL Log Parser",
        options,
        Box::new(|cc| {
            // Setup fonts
            setup_cjk_fonts(&cc.egui_ctx);
            
            // Enable dark mode by default
            // cc.egui_ctx.set_visuals(egui::Visuals::dark()); // Replaced by Dracula
            setup_dracula_theme(&cc.egui_ctx);
            Box::new(SqlLogParserApp::new(cc))
        }),
    )
}

fn setup_dracula_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    
    // Dracula Color Palette
    let background = egui::Color32::from_rgb(40, 42, 54);     // #282a36
    let current_line = egui::Color32::from_rgb(68, 71, 90);   // #44475a
    let foreground = egui::Color32::from_rgb(248, 248, 242);  // #f8f8f2
    let comment = egui::Color32::from_rgb(98, 114, 164);      // #6272a4
    let cyan = egui::Color32::from_rgb(139, 233, 253);        // #8be9fd
    let green = egui::Color32::from_rgb(80, 250, 123);        // #50fa7b
    // let orange = egui::Color32::from_rgb(255, 184, 108);    // #ffb86c
    let pink = egui::Color32::from_rgb(255, 121, 198);        // #ff79c6
    let purple = egui::Color32::from_rgb(189, 147, 249);      // #bd93f9
    let red = egui::Color32::from_rgb(255, 85, 85);           // #ff5555
    let yellow = egui::Color32::from_rgb(241, 250, 140);      // #f1fa8c

    // Global Widget Visuals
    visuals.widgets.noninteractive.bg_fill = background;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, foreground);
    
    // Inactive (normal state)
    visuals.widgets.inactive.bg_fill = current_line;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, foreground);
    visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
    
    // Hovered
    visuals.widgets.hovered.bg_fill = comment;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, white());
    visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);

    // Active (Pressed)
    visuals.widgets.active.bg_fill = purple;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, background); // Dark text on light accent
    visuals.widgets.active.rounding = egui::Rounding::same(4.0);
    
    // Selection
    visuals.selection.bg_fill = pink;
    visuals.selection.stroke = egui::Stroke::new(1.0, background);

    // Window / Panel backgrounds
    visuals.window_fill = background;
    visuals.panel_fill = background;
    
    // Hyperlinks
    visuals.hyperlink_color = cyan;

    // Apply
    ctx.set_visuals(visuals);
    
    // Typography updates can be done here if needed (e.g. increase size)
}

fn white() -> egui::Color32 {
    egui::Color32::WHITE
}
