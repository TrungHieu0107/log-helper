//! Main application window.
//!
//! Implements the main UI using egui, matching the C++ ImGui layout.

use crate::config::{Config, ConfigManager, DbConnection};
use crate::core::html_generator::{HtmlGenerator, HtmlOptions};
use crate::core::log_parser::{Execution, IdInfo, LogParser};
use crate::core::query_processor::{ProcessResult, QueryProcessor};
use crate::ui::theme;
use crate::utils::clipboard;
use egui::{Color32, RichText, TextEdit, Ui};

/// Main window state.
pub struct MainWindow {
    // UI State
    search_id: String,
    status_message: String,
    status_is_error: bool,

    // Config
    config_manager: ConfigManager,
    config: Config,

    // Processors
    parser: LogParser,
    processor: QueryProcessor,
    html_generator: HtmlGenerator,

    // Results
    last_result: ProcessResult,
    all_ids: Vec<IdInfo>,
    all_executions: Vec<Execution>,

    // Connection panel
    show_connection_panel: bool,
    editing_connection_index: i32,
    conn_name: String,
    conn_server: String,
    conn_database: String,
    conn_username: String,
    conn_password: String,
    conn_use_windows_auth: bool,

    // Layout
    left_panel_width_ratio: f32,
    theme_applied: bool,
}

impl MainWindow {
    pub fn new() -> Self {
        let config_manager = ConfigManager::new();
        let config = config_manager.load();

        Self {
            search_id: String::new(),
            status_message: String::new(),
            status_is_error: false,

            config_manager,
            config,

            parser: LogParser::new(),
            processor: QueryProcessor::new(),
            html_generator: HtmlGenerator::new(),

            last_result: ProcessResult::default(),
            all_ids: Vec::new(),
            all_executions: Vec::new(),

            show_connection_panel: false,
            editing_connection_index: -1,
            conn_name: String::new(),
            conn_server: String::new(),
            conn_database: String::new(),
            conn_username: String::new(),
            conn_password: String::new(),
            conn_use_windows_auth: true,

            left_panel_width_ratio: 0.55,
            theme_applied: false,
        }
    }

    /// Get the current config.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Render the main window.
    pub fn render(&mut self, ctx: &egui::Context) {
        // Apply theme once
        if !self.theme_applied {
            theme::apply_dark_theme(ctx);
            self.theme_applied = true;
        }

        // Top panel with toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.render_toolbar(ui);
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.render_status_bar(ui);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_main_content(ui);
        });

        // Connection panel window
        if self.show_connection_panel {
            self.render_connection_panel(ctx);
        }
    }

    fn render_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("🔍 SQL Log Parser").color(theme::accent_color()));
            ui.separator();

            // Log file path
            ui.label("Log File:");
            let mut log_path = self.config.log_file_path.clone();
            if ui.add(TextEdit::singleline(&mut log_path).desired_width(300.0)).changed() {
                self.config.log_file_path = log_path;
            }

            if ui.button("📂 Browse").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Log files", &["log", "txt"])
                    .pick_file()
                {
                    self.config.log_file_path = path.to_string_lossy().into_owned();
                    let _ = self.config_manager.save(&self.config);
                }
            }

            ui.separator();

            // Auto-copy toggle
            if ui.checkbox(&mut self.config.auto_copy, "Auto Copy").changed() {
                let _ = self.config_manager.save(&self.config);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⚙ Connections").clicked() {
                    self.show_connection_panel = true;
                }
            });
        });
    }

    fn render_status_bar(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let color = if self.status_is_error {
                theme::error_color()
            } else {
                theme::success_color()
            };
            ui.label(RichText::new(&self.status_message).color(color));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new("v2.0.0 Rust").small().color(Color32::GRAY));
            });
        });
    }

    fn render_main_content(&mut self, ui: &mut Ui) {
        let available_width = ui.available_width();
        let left_width = available_width * self.left_panel_width_ratio;
        let right_width = available_width - left_width - 10.0;

        ui.horizontal(|ui| {
            // Left panel - Search and Results
            ui.vertical(|ui| {
                ui.set_width(left_width);
                self.render_left_panel(ui);
            });

            ui.separator();

            // Right panel - IDs list
            ui.vertical(|ui| {
                ui.set_width(right_width);
                self.render_right_panel(ui);
            });
        });
    }

    fn render_left_panel(&mut self, ui: &mut Ui) {
        // Search section
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Search ID:");
                ui.add(TextEdit::singleline(&mut self.search_id).desired_width(150.0));

                if ui.button("🔍 Search").clicked() {
                    self.search_by_id();
                }

                if ui.button("⏮ Last Query").clicked() {
                    self.search_last_query();
                }

                if ui.button("📋 Copy").clicked() {
                    self.copy_to_clipboard();
                }

                if ui.button("📄 Export HTML").clicked() {
                    self.export_html(None);
                }

                if ui.button("📄 Export All").clicked() {
                    self.export_html_all();
                }
            });
        });

        ui.add_space(10.0);

        // Results section
        ui.group(|ui| {
            ui.heading("Results");
            
            if self.last_result.query.found() {
                // SQL display
                ui.collapsing("Filled SQL", |ui| {
                    ui.add(
                        TextEdit::multiline(&mut self.last_result.filled_sql.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(8)
                    );
                });

                ui.collapsing("Formatted SQL", |ui| {
                    ui.add(
                        TextEdit::multiline(&mut self.last_result.formatted_sql.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(8)
                    );
                });

                ui.collapsing("Parameters", |ui| {
                    ui.add(
                        TextEdit::multiline(&mut self.last_result.formatted_params.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(6)
                    );
                });
            } else {
                ui.label(RichText::new("No results. Enter an ID and click Search.").italics());
            }
        });

        // Executions section (if multiple)
        if !self.all_executions.is_empty() && self.all_executions.len() > 1 {
            ui.add_space(10.0);
            ui.group(|ui| {
                ui.heading(format!("Executions ({})", self.all_executions.len()));
                
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for (i, exec) in self.all_executions.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("#{}", exec.execution_index));
                            ui.label(&exec.timestamp);
                            ui.label(format!("{} params", exec.params.len()));
                            if ui.small_button("📋").clicked() {
                                clipboard::copy_to_clipboard(&exec.filled_sql);
                                self.set_status(&format!("Copied execution #{}", exec.execution_index), false);
                            }
                        });
                    }
                });
            });
        }
    }

    fn render_right_panel(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("IDs in Log");
                if ui.button("🔄 Refresh").clicked() {
                    self.load_all_ids();
                }
            });

            ui.add_space(5.0);

            if self.all_ids.is_empty() {
                ui.label("Click Refresh to load IDs from log file.");
            } else {
                ui.label(format!("Found {} IDs", self.all_ids.len()));
                ui.add_space(5.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for id_info in &self.all_ids.clone() {
                        ui.horizontal(|ui| {
                            if ui.selectable_label(false, &id_info.id[..std::cmp::min(12, id_info.id.len())]).clicked() {
                                self.search_id = id_info.id.clone();
                                self.search_by_id();
                            }
                            ui.label(format!("({})", id_info.params_count));
                        });
                    }
                });
            }
        });
    }

    fn render_connection_panel(&mut self, ctx: &egui::Context) {
        egui::Window::new("SQL Server Connections")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                // Connection list
                ui.group(|ui| {
                    ui.heading("Saved Connections");
                    
                    for (i, conn) in self.config.connections.clone().iter().enumerate() {
                        ui.horizontal(|ui| {
                            let is_active = self.config.active_connection_index == i as i32;
                            let label = if is_active {
                                format!("✓ {}", conn.name)
                            } else {
                                conn.name.clone()
                            };

                            if ui.selectable_label(is_active, label).clicked() {
                                self.load_connection_to_form(i);
                            }

                            if ui.small_button("✏").clicked() {
                                self.load_connection_to_form(i);
                            }

                            if ui.small_button("🗑").clicked() {
                                self.delete_connection(i);
                            }
                        });
                    }

                    if ui.button("➕ New Connection").clicked() {
                        self.clear_connection_form();
                        self.editing_connection_index = -1;
                    }
                });

                ui.add_space(10.0);

                // Connection form
                ui.group(|ui| {
                    let title = if self.editing_connection_index >= 0 {
                        "Edit Connection"
                    } else {
                        "New Connection"
                    };
                    ui.heading(title);

                    egui::Grid::new("conn_form").num_columns(2).show(ui, |ui| {
                        ui.label("Name:");
                        ui.add(TextEdit::singleline(&mut self.conn_name).desired_width(200.0));
                        ui.end_row();

                        ui.label("Server:");
                        ui.add(TextEdit::singleline(&mut self.conn_server).desired_width(200.0));
                        ui.end_row();

                        ui.label("Database:");
                        ui.add(TextEdit::singleline(&mut self.conn_database).desired_width(200.0));
                        ui.end_row();

                        ui.checkbox(&mut self.conn_use_windows_auth, "Windows Auth");
                        ui.end_row();

                        if !self.conn_use_windows_auth {
                            ui.label("Username:");
                            ui.add(TextEdit::singleline(&mut self.conn_username).desired_width(200.0));
                            ui.end_row();

                            ui.label("Password:");
                            ui.add(TextEdit::singleline(&mut self.conn_password).password(true).desired_width(200.0));
                            ui.end_row();
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("💾 Save").clicked() {
                            self.save_current_connection();
                        }

                        if ui.button("Close").clicked() {
                            self.show_connection_panel = false;
                        }
                    });
                });
            });
    }

    // Actions

    fn search_by_id(&mut self) {
        if self.search_id.is_empty() {
            self.set_status("Please enter an ID", true);
            return;
        }

        self.last_result = self.processor.process_query(
            &self.search_id,
            &self.config.log_file_path,
            self.config.auto_copy,
        );

        if self.last_result.success() {
            let msg = if self.last_result.copied_to_clipboard {
                "Query found and copied to clipboard"
            } else {
                "Query found"
            };
            self.set_status(msg, false);

            // Also get advanced executions
            self.all_executions = self.parser.parse_log_file_advanced(
                &self.config.log_file_path,
                &self.search_id,
            );
        } else {
            self.set_status(
                self.last_result.error.as_deref().unwrap_or("Not found"),
                true,
            );
            self.all_executions.clear();
        }
    }

    fn search_last_query(&mut self) {
        self.last_result = self.processor.process_last_query(
            &self.config.log_file_path,
            self.config.auto_copy,
        );

        if self.last_result.success() {
            self.search_id = self.last_result.query.id.clone();
            let msg = if self.last_result.copied_to_clipboard {
                "Last query found and copied to clipboard"
            } else {
                "Last query found"
            };
            self.set_status(msg, false);

            self.all_executions = self.parser.parse_log_file_advanced(
                &self.config.log_file_path,
                &self.search_id,
            );
        } else {
            self.set_status(
                self.last_result.error.as_deref().unwrap_or("No queries found"),
                true,
            );
            self.all_executions.clear();
        }
    }

    fn load_all_ids(&mut self) {
        self.all_ids = self.parser.get_all_ids(&self.config.log_file_path);
        
        if self.all_ids.is_empty() {
            self.set_status("No IDs found in log file", true);
        } else {
            self.set_status(&format!("Loaded {} IDs", self.all_ids.len()), false);
        }
    }

    fn export_html(&mut self, target_id: Option<&str>) {
        let id = target_id.unwrap_or(&self.search_id);
        
        if id.is_empty() {
            self.set_status("Please search for an ID first", true);
            return;
        }

        let executions = self.parser.parse_log_file_advanced(&self.config.log_file_path, id);
        
        if executions.is_empty() {
            self.set_status("No executions found for export", true);
            return;
        }

        let options = HtmlOptions {
            title: format!("SQL Report - {}", id),
            log_file: self.config.log_file_path.clone(),
        };

        let html = self.html_generator.generate_report(&executions, &options);
        let output_path = format!("{}/sql_report_{}.html", self.config.html_output_path, id);

        match self.html_generator.save_report(&html, &output_path) {
            Ok(_) => self.set_status(&format!("Exported to {}", output_path), false),
            Err(e) => self.set_status(&format!("Export failed: {}", e), true),
        }
    }

    fn export_html_all(&mut self) {
        if self.all_ids.is_empty() {
            self.load_all_ids();
        }

        if self.all_ids.is_empty() {
            self.set_status("No IDs to export", true);
            return;
        }

        let mut all_executions = Vec::new();
        for id_info in &self.all_ids {
            let executions = self.parser.parse_log_file_advanced(&self.config.log_file_path, &id_info.id);
            all_executions.extend(executions);
        }

        let options = HtmlOptions {
            title: "SQL Report - All Queries".to_string(),
            log_file: self.config.log_file_path.clone(),
        };

        let html = self.html_generator.generate_report(&all_executions, &options);
        let output_path = format!("{}/sql_report_all.html", self.config.html_output_path);

        match self.html_generator.save_report(&html, &output_path) {
            Ok(_) => self.set_status(&format!("Exported {} queries to {}", all_executions.len(), output_path), false),
            Err(e) => self.set_status(&format!("Export failed: {}", e), true),
        }
    }

    fn copy_to_clipboard(&mut self) {
        if self.last_result.filled_sql.is_empty() {
            self.set_status("No SQL to copy", true);
            return;
        }

        if clipboard::copy_to_clipboard(&self.last_result.filled_sql) {
            self.set_status("Copied to clipboard", false);
        } else {
            self.set_status("Failed to copy to clipboard", true);
        }
    }

    // Connection management

    fn load_connection_to_form(&mut self, index: usize) {
        if let Some(conn) = self.config.connections.get(index) {
            self.conn_name = conn.name.clone();
            self.conn_server = conn.server.clone();
            self.conn_database = conn.database.clone();
            self.conn_username = conn.username.clone();
            self.conn_password = conn.password.clone();
            self.conn_use_windows_auth = conn.use_windows_auth;
            self.editing_connection_index = index as i32;
        }
    }

    fn clear_connection_form(&mut self) {
        self.conn_name.clear();
        self.conn_server.clear();
        self.conn_database.clear();
        self.conn_username.clear();
        self.conn_password.clear();
        self.conn_use_windows_auth = true;
        self.editing_connection_index = -1;
    }

    fn save_current_connection(&mut self) {
        let conn = DbConnection {
            name: self.conn_name.clone(),
            server: self.conn_server.clone(),
            database: self.conn_database.clone(),
            username: self.conn_username.clone(),
            password: self.conn_password.clone(),
            use_windows_auth: self.conn_use_windows_auth,
        };

        if self.editing_connection_index >= 0 {
            let idx = self.editing_connection_index as usize;
            if idx < self.config.connections.len() {
                self.config.connections[idx] = conn;
            }
        } else {
            self.config.connections.push(conn);
        }

        if let Err(e) = self.config_manager.save(&self.config) {
            self.set_status(&format!("Failed to save: {}", e), true);
        } else {
            self.set_status("Connection saved", false);
        }
    }

    fn delete_connection(&mut self, index: usize) {
        if index < self.config.connections.len() {
            self.config.connections.remove(index);
            if self.config.active_connection_index >= index as i32 {
                self.config.active_connection_index = -1;
            }
            let _ = self.config_manager.save(&self.config);
        }
    }

    fn set_status(&mut self, msg: &str, is_error: bool) {
        self.status_message = msg.to_string();
        self.status_is_error = is_error;
    }
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::new()
    }
}
