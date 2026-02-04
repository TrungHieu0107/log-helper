//! Main application implementing egui UI.

use crate::config::{Config, ConfigManager};
use crate::core::log_parser::{IdInfo, LogParser};
use crate::core::query_processor::{ProcessResult, QueryProcessor};
use crate::utils::clipboard;
use eframe::egui;
use rfd::FileDialog;

/// Main application state.
pub struct SqlLogParserApp {
    // Services
    config_manager: ConfigManager,
    log_parser: LogParser,
    query_processor: QueryProcessor,

    // State
    config: Config,
    ids: Vec<IdInfo>,
    selected_id: String,
    search_input: String,
    result: Option<ProcessResult>,
    status: String,
}

impl SqlLogParserApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config_manager = ConfigManager::new();
        let config = config_manager.load();

        let mut app = Self {
            config_manager,
            log_parser: LogParser::new(config.encoding.clone()),
            query_processor: QueryProcessor::new(),
            config,
            ids: Vec::new(),
            selected_id: String::new(),
            search_input: String::new(),
            result: None,
            status: "Ready".to_string(),
        };

        // Propagate encoding to query processor as well
        app.query_processor.parser_mut().set_encoding(app.config.encoding.clone());

        app
    }

    fn load_ids(&mut self) {
        if self.config.log_file_path.is_empty() {
            self.status = "No log file path set".to_string();
            return;
        }

        self.status = "Loading IDs...".to_string();
        self.ids = self.log_parser.get_all_ids(&self.config.log_file_path);
        self.status = format!("Loaded {} IDs", self.ids.len());
    }

    fn browse_file(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("Log Files", &["log", "txt"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            self.config.log_file_path = path.to_string_lossy().to_string();
            let _ = self.config_manager.save(&self.config);
        }
    }

    fn search(&mut self) {
        if self.search_input.is_empty() {
            self.status = "Enter an ID to search".to_string();
            return;
        }

        self.status = "Searching...".to_string();
        self.result = Some(self.query_processor.process_query(
            &self.search_input,
            &self.config.log_file_path,
            self.config.auto_copy,
        ));

        if let Some(ref result) = self.result {
            if let Some(ref error) = result.error {
                self.status = format!("Error: {}", error);
            } else {
                self.status = "Query found".to_string();
            }
        }
    }

    fn copy_sql(&self) {
        if let Some(ref result) = self.result {
            if !result.filled_sql.is_empty() {
                if clipboard::copy_to_clipboard(&result.filled_sql) {
                    // Status update handled separately
                }
            }
        }
    }

    fn get_last_sql(&mut self) {
        if self.config.log_file_path.is_empty() {
            self.status = "No log file path set".to_string();
            return;
        }

        self.status = "Loading last SQL...".to_string();
        self.result = Some(self.query_processor.process_last_query(
            &self.config.log_file_path,
            self.config.auto_copy,
        ));

        if let Some(ref result) = self.result {
            if let Some(ref error) = result.error {
                 self.status = format!("Error: {}", error);
            } else {
                 self.status = "Last SQL loaded".to_string();
                 self.selected_id = result.query.id.clone();
                 self.search_input = result.query.id.clone();
            }
        }
    }
}

impl eframe::App for SqlLogParserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("SQL Log Parser");
                ui.separator();

                ui.label("Log:");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.config.log_file_path)
                        .desired_width(300.0)
                        .hint_text("Path to log file"),
                );
                if response.lost_focus() {
                    let _ = self.config_manager.save(&self.config);
                }

                if ui.button("📁 Browse").clicked() {
                    self.browse_file();
                }

                ui.separator();
                
                if ui.button("⏮ Last SQL").on_hover_text("Load the last executed SQL query from the log").clicked() {
                    self.get_last_sql();
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Encoding:");
                    let current_encoding = self.config.encoding.clone();
                    egui::ComboBox::from_id_source("encoding_selector")
                        .selected_text(&current_encoding)
                        .show_ui(ui, |ui| {
                            let encodings = ["SHIFT_JIS", "UTF-8", "UTF-16LE", "UTF-16BE", "EUC-JP", "WINDOWS-1252"];
                            for &enc in &encodings {
                                if ui.selectable_value(&mut self.config.encoding, enc.to_string(), enc).changed() {
                                    // Save config and update parsers immediately
                                    let _ = self.config_manager.save(&self.config);
                                    let encoding = self.config.encoding.clone();
                                    self.query_processor.parser_mut().set_encoding(encoding.clone());
                                    self.log_parser.set_encoding(encoding);
                                }
                            }
                        });
                });

                ui.separator();

                if ui.checkbox(&mut self.config.auto_copy, "Auto Copy").changed() {
                    let _ = self.config_manager.save(&self.config);
                }
            });
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("statusbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("v2.0.0");
                });
            });
        });

        // Left sidebar
        egui::SidePanel::left("sidebar")
            .default_width(200.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Query IDs");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🔄").on_hover_text("Refresh").clicked() {
                            self.load_ids();
                        }
                    });
                });

                ui.separator();

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                        for id_info in &self.ids {
                            let label = if id_info.params_count > 0 {
                                format!("{} ({})", id_info.id, id_info.params_count)
                            } else {
                                id_info.id.clone()
                            };

                            let is_selected = self.selected_id == id_info.id;
                            if ui.selectable_label(is_selected, &label).clicked() {
                                self.selected_id = id_info.id.clone();
                                self.search_input = id_info.id.clone();
                            }
                        }
                    });
                });
            });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            // Search section
            ui.group(|ui| {
                ui.heading("Search");
                ui.horizontal(|ui| {
                    ui.label("ID:");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.search_input)
                            .desired_width(200.0)
                            .hint_text("Enter ID"),
                    );
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.search();
                    }
                    if ui.button("🔍 Search").clicked() {
                        self.search();
                    }
                });
            });

            ui.add_space(10.0);



            ui.add_space(20.0);

            // Result section
            if let Some(ref result) = self.result {
                ui.group(|ui| {
                     ui.set_min_height(300.0);
                     
                    if let Some(ref error) = result.error {
                        ui.colored_label(egui::Color32::RED, error);
                    } else if !result.executions.is_empty() {
                         egui::ScrollArea::vertical()
                            .id_source("executions_list")
                            .max_height(f32::INFINITY)
                            .show(ui, |ui| {
                             let mut current_sql = String::new();
                             
                             for (index, exec) in result.executions.iter().enumerate() {
                                 // Header for new SQL Template (Group valid executions by template)
                                 // Simple grouping: if SQL text changes, show new header.
                                 if exec.sql != current_sql {
                                     current_sql = exec.sql.clone();
                                     if index > 0 { ui.separator(); }
                                     
                                     ui.add_space(5.0);
                                     ui.heading("SQL Template");
                                     
                                     // Format SQL for display
                                     let formatted_template = crate::core::sql_formatter::format_sql(&current_sql);
                                     ui.add(
                                        egui::TextEdit::multiline(&mut formatted_template.as_str())
                                            .font(egui::TextStyle::Monospace)
                                            .desired_width(f32::INFINITY)
                                            .code_editor()
                                            .interactive(false)
                                    );
                                    ui.add_space(5.0);
                                    ui.label(egui::RichText::new("Executions:").strong());
                                 }
                                 
                                 ui.push_id(index, |ui| {
                                     ui.group(|ui| {
                                         ui.horizontal(|ui| {
                                             ui.label(egui::RichText::new(format!("#{} {}", exec.execution_index, exec.timestamp)).strong());
                                             if ui.button("📋 Copy").on_hover_text("Copy this specific execution").clicked() {
                                                  let _ = clipboard::copy_to_clipboard(&exec.filled_sql);
                                             }
                                         });
                                         
                                         // Show filled SQL
                                         let mut filled_sql_text = exec.filled_sql.clone();
                                         let response = ui.add(
                                             egui::TextEdit::multiline(&mut filled_sql_text)
                                                 .font(egui::TextStyle::Monospace)
                                                 .desired_width(f32::INFINITY)
                                                 .interactive(false) 
                                         );
                                                   
                                         if response.hovered() {
                                             let params_text = crate::core::sql_formatter::format_params(&exec.params);
                                             egui::show_tooltip(ui.ctx(), response.id, |ui| {
                                                 ui.label(params_text);
                                             });
                                         }
                                     });
                                 });
                                 ui.add_space(5.0);
                             }
                         });
                    } else {
                        ui.label("No executions found.");
                    }
                });
            }


        });
    }
}
