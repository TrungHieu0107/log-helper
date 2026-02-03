//! Application state and eframe integration.
//!
//! Main application structure implementing eframe::App trait.

use crate::ui::main_window::MainWindow;
use eframe::egui;

/// Main application state.
pub struct App {
    main_window: MainWindow,
}

impl App {
    /// Create a new application instance.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            main_window: MainWindow::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.main_window.render(ctx);
    }
}
