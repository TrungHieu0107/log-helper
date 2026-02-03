//! SQL Log Parser - A Windows desktop application for parsing SQL log files.
//!
//! This application allows you to:
//! - Parse SQL queries and parameters from log files
//! - Format and highlight SQL statements
//! - Generate HTML reports
//! - Execute queries against SQL Server (optional)

// Hide console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod core;
mod ui;
mod utils;

use app::App;
use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    // Configure native window options
    let options = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("SQL Log Parser v2.0")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "SQL Log Parser",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
