#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod core;
mod utils;

use app::SqlLogParserApp;
use eframe::egui;

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
            // Enable dark mode by default
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(SqlLogParserApp::new(cc))
        }),
    )
}
