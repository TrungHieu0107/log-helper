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
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(SqlLogParserApp::new(cc))
        }),
    )
}
