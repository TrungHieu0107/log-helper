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
                
                ui.separator();

                if ui.checkbox(&mut self.config.format_sql, "Format SQL").changed() {
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
                        let mut clicked_id = None;
                        for id_info in &self.ids {
                            // Format: "DaoName - ID (count)"
                            let base_label = if !id_info.dao_name.is_empty() && id_info.dao_name != "Unknown" {
                                format!("{} - {}", id_info.dao_name, id_info.id)
                            } else {
                                id_info.id.clone()
                            };

                            let label = if id_info.params_count > 0 {
                                format!("{} ({})", base_label, id_info.params_count)
                            } else {
                                base_label
                            };

                            let is_selected = self.selected_id == id_info.id;
                            if ui.selectable_label(is_selected, &label).clicked() {
                                clicked_id = Some(id_info.id.clone());
                            }
                        }
                        
                        // Handle click outside the loop to avoid mutable borrow conflict
                        if let Some(id) = clicked_id {
                            self.selected_id = id.clone();
                            self.search_input = id;
                            self.search();
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
            if let Some(ref mut result) = self.result {
                ui.group(|ui| {
                     ui.set_min_height(300.0);
                     
                    if let Some(ref error) = result.error {
                        ui.colored_label(egui::Color32::RED, error);
                    } else if !result.groups.is_empty() {
                         egui::ScrollArea::vertical()
                            .id_source("executions_list")
                            .max_height(f32::INFINITY)
                            .show(ui, |ui| {
                             
                             let single_group = result.groups.len() == 1;

                             // Iterate over groups
                             for (g_idx, group) in result.groups.iter_mut().enumerate() {
                                 ui.push_id(g_idx, |ui| {
                                     // Get DAO name from first execution or fallback
                                     let dao_name = group.executions.first()
                                         .map(|e| if e.dao_file.is_empty() { "Unknown DAO" } else { &e.dao_file })
                                         .unwrap_or("Unknown DAO");
                                     
                                     // Display DAO Name as a header (non-collapsible)
                                     ui.add_space(5.0);
                                     ui.label(egui::RichText::new(dao_name).heading().strong().color(egui::Color32::from_rgb(100, 200, 255)));

                                     // Template Section (Collapsible)
                                     let template_response = egui::CollapsingHeader::new("Template")
                                         .id_source(format!("template_header_{}", g_idx))
                                         .open(Some(group.is_template_expanded))
                                         .show(ui, |ui| {
                                             ui.horizontal(|ui| {
                                                 if ui.button("📋 Copy Template").on_hover_text("Copy to clipboard").clicked() {
                                                     let _ = clipboard::copy_to_clipboard(&group.template_sql);
                                                 }
                                             });
                                             
                                             let mut template_display = group.formatted_template_sql.clone();
                                             ui.add(
                                                 egui::TextEdit::multiline(&mut template_display)
                                                     .font(egui::TextStyle::Monospace)
                                                     .desired_width(f32::INFINITY)
                                                     .interactive(false)
                                             );
                                         });
                                     
                                     // Handle toggle manually
                                     if template_response.header_response.clicked() {
                                         group.is_template_expanded = !group.is_template_expanded;
                                     }
                                     
                                     ui.separator();
                                     
                                     // List executions in this group
                                     for (e_idx, exec) in group.executions.iter_mut().enumerate() {
                                                 ui.push_id(e_idx, |ui| {
                                                     // Execution Header (Summary)
                                                     let summary_text = format!(
                                                         "#{} {} - {}", 
                                                         exec.execution_index, 
                                                         exec.timestamp,
                                                         exec.filled_sql.lines().next().unwrap_or("").chars().take(50).collect::<String>()
                                                     );
                                                     
                                                     let exec_response = egui::CollapsingHeader::new(summary_text)
                                                        .id_source(format!("exec_{}", e_idx))
                                                        .open(Some(exec.is_expanded))
                                                        .show(ui, |ui| {
                                                            // Expanded Content
                                                            ui.group(|ui| {
                                                                 ui.horizontal(|ui| {
                                                                     if ui.button("📋 Copy SQL").clicked() {
                                                                          let _ = clipboard::copy_to_clipboard(&exec.filled_sql);
                                                                     }
                                                                 });
                                                                 
                                                                 // Show SQL (Filled or Formatted)
                                                                 let mut display_sql = if self.config.format_sql {
                                                                     exec.formatted_sql.clone()
                                                                 } else {
                                                                     exec.filled_sql.clone()
                                                                 };
                                                                 
                                                                 ui.add(
                                                                     egui::TextEdit::multiline(&mut display_sql)
                                                                         .font(egui::TextStyle::Monospace)
                                                                         .desired_width(f32::INFINITY)
                                                                         .interactive(false) 
                                                                 );
                                                                 
                                                                 ui.separator();
                                                                 ui.label(egui::RichText::new("Parameters:").strong());
                                                                 let params_text = crate::core::sql_formatter::format_params(&exec.params);
                                                                 ui.label(params_text);
                                                             });
                                                        });
                                                     
                                                     if exec_response.header_response.clicked() {
                                                         exec.is_expanded = !exec.is_expanded;
                                                     }
                                                 });
                                                 // Tiny space between items
                                                 ui.add_space(2.0);
                                            }
                                        });
                                 ui.add_space(5.0);
                             }
                         });
                    } else if !result.executions.is_empty() {
                         ui.label("No groups found (fallback).");
                    } else {
                        ui.label("No executions found.");
                    }
                });
            }


        });
    }
}
