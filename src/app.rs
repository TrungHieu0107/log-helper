use crate::config::{Config, ConfigManager};
use crate::core::log_parser::{IdInfo, LogParser};
use crate::core::query_processor::{ProcessResult, QueryProcessor};
use crate::utils::clipboard;
use eframe::egui;
use rfd::FileDialog;
use uuid::Uuid;


use crate::core::db::{ConnectionManager, DbConfig, DbType, DbClient, QueryResult};
use tokio::runtime::Runtime;

#[derive(PartialEq)]
enum AppTab {
    LogParser,
    SqlExecutor,
}

/// Main application state.
pub struct SqlLogParserApp {
    // Services
    config_manager: ConfigManager,
    log_parser: LogParser,
    query_processor: QueryProcessor,
    connection_manager: ConnectionManager,

    // State
    config: Config,
    ids: Vec<IdInfo>,
    selected_id: String,
    
    // Parser State
    search_input: String,
    result: Option<ProcessResult>,
    
    // Feature: SQL Executor
    active_tab: AppTab,
    executor_sql: String,
    
    db_client: DbClient,
    tokio_runtime: Runtime,
    active_connection_id: Option<String>,
    query_result: Option<anyhow::Result<QueryResult>>,
    executing: bool,
    
    // Communication
    query_sender: std::sync::mpsc::Sender<anyhow::Result<QueryResult>>,
    query_receiver: std::sync::mpsc::Receiver<anyhow::Result<QueryResult>>,
    
    test_sender: std::sync::mpsc::Sender<anyhow::Result<()>>,
    test_receiver: std::sync::mpsc::Receiver<anyhow::Result<()>>,
    
    // Connection Editing
    show_connection_modal: bool,
    editing_connection: DbConfig,
    is_new_connection: bool, // true = add new, false = edit existing
    test_conn_status: Option<anyhow::Result<String>>,
    is_testing_conn: bool,
    is_saving: bool,

    editing_mode: ConnectionEditMode,
    editing_fields: crate::core::db::ConnectionFields,
    
    status: String,
}

#[derive(PartialEq)]
enum ConnectionEditMode {
    Url,
    Fields,
}

impl SqlLogParserApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config_manager = ConfigManager::new();
        let config = config_manager.load();

        let (query_sender, query_receiver) = std::sync::mpsc::channel();
        let (test_sender, test_receiver) = std::sync::mpsc::channel();

        let mut app = Self {
            config_manager,
            log_parser: LogParser::new(config.encoding.clone()),
            query_processor: QueryProcessor::new(),
            connection_manager: ConnectionManager::new(),
            config,
            ids: Vec::new(),
            selected_id: String::new(),
            search_input: String::new(),
            result: None,
            
            active_tab: AppTab::LogParser,
            executor_sql: String::new(),
            
            db_client: DbClient::new(),
            tokio_runtime: Runtime::new().expect("Failed to create tokio runtime"),
            active_connection_id: None,
            query_result: None,
            executing: false,
            
            query_sender,
            query_receiver,
            test_sender,
            test_receiver,
            
            show_connection_modal: false,
            editing_connection: DbConfig::default(),
            is_new_connection: false,
            test_conn_status: None,
            is_testing_conn: false,
            is_saving: false,

            editing_mode: ConnectionEditMode::Url,
            editing_fields: crate::core::db::ConnectionFields::default(),
            
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

    fn sidebar_parser(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
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
                    
                    // Styling for ID Label
                    let mut text = egui::RichText::new(&label);
                    if is_selected {
                        text = text.color(egui::Color32::from_rgb(80, 250, 123)).strong(); // Green
                    } else {
                        text = text.color(egui::Color32::from_rgb(248, 248, 242)); // Foreground
                    }

                    if ui.add(egui::SelectableLabel::new(is_selected, text)).clicked() {
                        clicked_id = Some(id_info.id.clone());
                    }
                }
                
                if let Some(id) = clicked_id {
                    self.selected_id = id.clone();
                    self.search_input = id;
                    self.search();
                }
            });
        });
    }

    fn ui_parser(&mut self, ui: &mut egui::Ui) {
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
        // Take result out to avoid mutable borrow overlap when calling show_result_panel
        if let Some(mut result) = self.result.take() {
             self.show_result_panel(ui, &mut result);
             self.result = Some(result);
        }
    }
    
    fn show_result_panel(&mut self, ui: &mut egui::Ui, result: &mut ProcessResult) {
        ui.group(|ui| {
             ui.set_min_height(300.0);
             
            if let Some(ref error) = result.error {
                ui.colored_label(egui::Color32::RED, error);
            } else if !result.groups.is_empty() {
                 egui::ScrollArea::vertical()
                    .id_source("executions_list")
                    .max_height(f32::INFINITY)
                    .show(ui, |ui| {
                     
                     for (g_idx, group) in result.groups.iter_mut().enumerate() {
                         ui.push_id(g_idx, |ui| {
                             egui::Frame::none()
                                 .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(98, 114, 164)))
                                 .rounding(8.0)
                                 .inner_margin(10.0)
                                 .show(ui, |ui| {
                                     let dao_name = group.executions.first()
                                         .map(|e| if e.dao_file.is_empty() { "Unknown DAO" } else { &e.dao_file })
                                         .unwrap_or("Unknown DAO");
                                     
                                     ui.add_space(5.0);
                                     ui.label(egui::RichText::new(dao_name).heading().strong().color(egui::Color32::from_rgb(189, 147, 249)));

                                     let template_response = egui::CollapsingHeader::new(egui::RichText::new("Template").color(egui::Color32::from_rgb(139, 233, 253)))
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
                                     
                                     if template_response.header_response.clicked() {
                                         group.is_template_expanded = !group.is_template_expanded;
                                     }
                                     
                                     ui.separator();
                                     
                                     for (e_idx, exec) in group.executions.iter_mut().enumerate() {
                                         ui.push_id(e_idx, |ui| {
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
                                                    egui::Frame::none()
                                                        .fill(egui::Color32::from_rgb(68, 71, 90))
                                                        .rounding(4.0)
                                                        .inner_margin(8.0)
                                                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(98, 114, 164)))
                                                        .show(ui, |ui| {
                                                            ui.horizontal(|ui| {
                                                                if ui.button(egui::RichText::new("📋 Copy SQL").color(egui::Color32::from_rgb(80, 250, 123))).clicked() {
                                                                     let _ = clipboard::copy_to_clipboard(&exec.filled_sql);
                                                                }
                                                                
                                                                if ui.button(egui::RichText::new("▶ Execute").color(egui::Color32::from_rgb(255, 121, 198))).clicked() {
                                                                    self.active_tab = AppTab::SqlExecutor;
                                                                    self.executor_sql = exec.filled_sql.clone();
                                                                }

                                                                ui.label(egui::RichText::new(format!("Index: {}", exec.execution_index)).italics().color(egui::Color32::from_rgb(139, 233, 253)));
                                                            });
                                                            
                                                            ui.add_space(5.0);
                                                            
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
                                                            ui.label(egui::RichText::new("Parameters:").strong().color(egui::Color32::from_rgb(255, 121, 198)));
                                                            let params_text = crate::core::sql_formatter::format_params(&exec.params);
                                                            ui.label(egui::RichText::new(params_text).monospace());
                                                        });
                                                });
                                             
                                             if exec_response.header_response.clicked() {
                                                 exec.is_expanded = !exec.is_expanded;
                                             }
                                         });
                                         ui.add_space(2.0);
                                    }
                                });
                             });
                         ui.add_space(15.0);
                     }
                 });
            } else if !result.executions.is_empty() {
                 ui.label("No groups found (fallback).");
            } else {
                ui.label("No executions found.");
            }
        });
    }

    fn sidebar_executor(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("Connections");
        if ui.button("➕ New Connection").clicked() {
            println!("New Connection button clicked");
            self.editing_connection = DbConfig::default();
            self.editing_connection.id = Uuid::new_v4().to_string();
            self.is_new_connection = true; // Mark as new
            // Default fields
            self.editing_fields = crate::core::db::ConnectionFields::default();
            self.editing_mode = ConnectionEditMode::Url; // Default start
            self.test_conn_status = None; // Clear any previous status

            self.show_connection_modal = true;
        }
        ui.separator();
        
        let mut edit_conn = None;
        let mut delete_id = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
             for conn in &self.connection_manager.connections {
                 ui.horizontal(|ui| {
                     ui.label(&conn.name);
                     
                     if ui.button("✏").on_hover_text("Edit").clicked() {
                         edit_conn = Some(conn.clone());
                     }
                     
                     if ui.button("🗑").on_hover_text("Delete").clicked() {
                         delete_id = Some(conn.id.clone());
                     }
                 });
             }
        });
        
        if let Some(conn) = edit_conn {
            println!("Edit connection clicked: name='{}', id='{}'", conn.name, conn.id);
            self.editing_connection = conn;
            self.is_new_connection = false; // Mark as edit (not new)
            // Parse existing URL to fields if possible
            if let Ok(parsed) = crate::core::db::parse_jdbc_url(&self.editing_connection.url) {
                 self.editing_fields = crate::core::db::ConnectionFields {
                     host: parsed.host,
                     port: parsed.port.to_string(),
                     database: parsed.database.unwrap_or_default(),
                     encrypt: parsed.encrypt,
                     trust_cert: parsed.trust_cert,
                 };
            } else {
                // Fallback / Reset
                self.editing_fields = crate::core::db::ConnectionFields::default();
            }
            // Maybe allow preference?
            self.editing_mode = ConnectionEditMode::Url;
            self.test_conn_status = None; // Clear any previous status

            self.show_connection_modal = true;
        }
        
        if let Some(id) = delete_id {
            self.connection_manager.delete(&id);
        }

        self.connection_edit_modal(ctx);
    }
    
    fn connection_edit_modal(&mut self, ctx: &egui::Context) {
        // NOTE: Test result polling is done at the end of this function
        // to properly handle both test-only and save flows.

        if self.show_connection_modal {
            let mut is_open = true;
            let mut save_triggered = false;
            let mut cancel_triggered = false;
            let mut test_triggered = false;
            
            // Sync state when modal opens logic is handled by caller setting the struct, 
            // but we need to ensure fields <-> url sync on first load? 
            // Ideally we do this when 'New Connection' or 'Edit' is clicked.
            // For now, let's assume `editing_connection` has the source of truth initially.

            egui::Window::new("Connection Details")
                .open(&mut is_open)
                .collapsible(false)
                .resizable(true)
                .min_width(500.0)
                .show(ctx, |ui| {
                    let mut changed = false;
                    
                    // Top: Common Fields
                    egui::Grid::new("common_conn_fields").num_columns(2).spacing([10.0, 10.0]).show(ui, |ui| {
                        ui.label("Name:");
                        if ui.text_edit_singleline(&mut self.editing_connection.name).changed() { changed = true; }
                        ui.end_row();

                        ui.label("Type:");
                        egui::ComboBox::from_id_source("db_type")
                            .selected_text(self.editing_connection.db_type.to_string())
                            .show_ui(ui, |ui| {
                                if ui.selectable_value(&mut self.editing_connection.db_type, DbType::SqlServer, "SQL Server").changed() { changed = true; }
                                // Disable others or handle them simply? Request asked for SQL Server specifics.
                                // Let's keep others available but maybe they don't use the advanced mode logic yet.
                                ui.selectable_value(&mut self.editing_connection.db_type, DbType::Postgres, "Postgres");
                                ui.selectable_value(&mut self.editing_connection.db_type, DbType::Mysql, "MySQL");
                                ui.selectable_value(&mut self.editing_connection.db_type, DbType::Sqlite, "SQLite");
                            });
                        ui.end_row();
                    });

                    ui.separator();
                    
                    if self.editing_connection.db_type == DbType::SqlServer {
                        // Mode Switcher
                         ui.horizontal(|ui| {
                            ui.label("Input Mode:");
                            if ui.radio_value(&mut self.editing_mode, ConnectionEditMode::Url, "URL").changed() {
                                // Sync Fields from URL when switching TO Fields? No, sync when switching FROM Url
                                // Actually, we should sync whenever data changes. 
                                // But if we switch modes, we might want to refresh the view.
                            }
                            if ui.radio_value(&mut self.editing_mode, ConnectionEditMode::Fields, "Host & Port").changed() {
                                // Logic to parse URL into fields if switching to fields?
                                // We'll do it proactively on URL change.
                            }
                        });
                        ui.add_space(10.0);

                        match self.editing_mode {
                            ConnectionEditMode::Url => {
                                ui.label("JDBC URL:");
                                let url_response = ui.add(
                                    egui::TextEdit::multiline(&mut self.editing_connection.url)
                                    .hint_text("jdbc:sqlserver://host:port;databaseName=DB")
                                    .desired_rows(3)
                                    .desired_width(f32::INFINITY)
                                );
                                
                                if url_response.changed() {
                                    changed = true;
                                    // Auto-parse to update fields for "preview" or consistency
                                    if let Ok(parsed) = crate::core::db::parse_jdbc_url(&self.editing_connection.url) {
                                         self.editing_fields = crate::core::db::ConnectionFields {
                                             host: parsed.host,
                                             port: parsed.port.to_string(),
                                             database: parsed.database.unwrap_or_default(),
                                             encrypt: parsed.encrypt,
                                             trust_cert: parsed.trust_cert,
                                         };
                                    }
                                }
                            },
                            ConnectionEditMode::Fields => {
                                egui::Grid::new("mssql_fields").num_columns(2).spacing([10.0, 10.0]).show(ui, |ui| {
                                    ui.label("Host:");
                                    if ui.text_edit_singleline(&mut self.editing_fields.host).changed() { changed = true; }
                                    ui.end_row();
                                    
                                    ui.label("Port:");
                                    if ui.text_edit_singleline(&mut self.editing_fields.port).changed() { changed = true; }
                                    ui.end_row();
                                    
                                    ui.label("Database:");
                                    if ui.text_edit_singleline(&mut self.editing_fields.database).changed() { changed = true; }
                                    ui.end_row();
                                    
                                    ui.label("Encrypted:");
                                    if ui.checkbox(&mut self.editing_fields.encrypt, "").changed() { changed = true; }
                                    ui.end_row();
                                    
                                    ui.label("Trust Server Cert:");
                                    if ui.checkbox(&mut self.editing_fields.trust_cert, "").changed() { changed = true; }
                                    ui.end_row();
                                });
                                
                                if changed {
                                    // Rebuild URL from fields
                                    self.editing_connection.url = crate::core::db::build_jdbc_url(&self.editing_fields);
                                }
                                
                                ui.add_space(5.0);
                                ui.label(egui::RichText::new("Preview URL:").weak());
                                ui.label(egui::RichText::new(&self.editing_connection.url).monospace().small().weak());
                            }
                        }
                    } else {
                        // Fallback for other DB types
                        egui::Grid::new("generic_fields").num_columns(2).spacing([10.0, 10.0]).show(ui, |ui| {
                             ui.label("Connection URL:");
                             if ui.text_edit_singleline(&mut self.editing_connection.url).changed() { changed = true; }
                             ui.end_row();
                        });
                    }

                    ui.separator();
                    
                    // Auth Section (Common-ish)
                    ui.group(|ui| {
                        ui.heading("Authentication");
                        egui::Grid::new("auth_fields").num_columns(2).spacing([10.0, 10.0]).show(ui, |ui| {
                             ui.label("User:");
                             if ui.text_edit_singleline(&mut self.editing_connection.user).changed() { changed = true; }
                             ui.end_row();
    
                             ui.label("Password:");
                             if ui.add(egui::TextEdit::singleline(&mut self.editing_connection.password).password(true)).changed() { changed = true; }
                             ui.end_row();
                        });
                    });
                    
                    if changed {
                        self.test_conn_status = None;
                    }

                    ui.add_space(15.0);
                    
                    // Test Status
                    if self.is_testing_conn {
                         ui.horizontal(|ui| {
                             ui.add(egui::Spinner::new());
                             ui.label("Testing connection...");
                         });
                    } else if let Some(status) = &self.test_conn_status {
                        match status {
                            Ok(msg) => {
                                ui.colored_label(egui::Color32::GREEN, format!("✔ {}", msg));
                            },
                            Err(e) => {
                                ui.colored_label(egui::Color32::RED, format!("❌ Connection failed: {}", e));
                            }
                        }
                    }
                    
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.add_enabled(!self.is_testing_conn, egui::Button::new("Test Connection")).clicked() {
                            test_triggered = true;
                        }
                        
                        // "Save" button now always enabled (validation happens on click), triggers test+save
                         if ui.add_enabled(!self.is_testing_conn, egui::Button::new("Save")).clicked() {
                            println!("Save clicked. Starting validation flow."); 
                            save_triggered = true;
                        }
                        
                        if ui.button("Cancel").clicked() {
                            cancel_triggered = true;
                        }
                    });
                });
            
            // Handle async triggers
            if test_triggered {
                println!("Triggering test connection...");
                self.is_testing_conn = true;
                self.test_conn_status = None;
                
                let conn_config = self.editing_connection.clone();
                let sender = self.test_sender.clone(); 
                
                self.tokio_runtime.spawn(async move {
                    let client = DbClient::new();
                    let result = client.test_connection(&conn_config).await;
                    let _ = sender.send(result);
                });
            } else if save_triggered {
                // Comprehensive validation
                println!("Save triggered. Validating fields...");
                let mut validation_errors: Vec<String> = Vec::new();

                // Validate connection name
                if self.editing_connection.name.trim().is_empty() {
                    validation_errors.push("Connection name is required".to_string());
                }

                // Validate URL
                if self.editing_connection.url.trim().is_empty() {
                    validation_errors.push("Connection URL cannot be empty".to_string());
                } else if self.editing_connection.db_type == DbType::SqlServer {
                    // Validate JDBC URL format for SQL Server
                    if let Err(e) = crate::core::db::parse_jdbc_url(&self.editing_connection.url) {
                        validation_errors.push(format!("Invalid JDBC URL: {}", e));
                    }
                }

                // Validate credentials (optional but warn)
                if self.editing_connection.user.trim().is_empty() {
                    println!("Warning: Username is empty");
                }

                if !validation_errors.is_empty() {
                    println!("Validation failed: {:?}", validation_errors);
                    self.test_conn_status = Some(Err(anyhow::anyhow!("{}", validation_errors.join("\n"))));
                } else {
                     // Start Save Flow: Test -> Then Save
                     println!("Validation passed. Starting save flow: Testing connection first...");
                     println!("Connection details: name='{}', url='{}', user='{}'",
                         self.editing_connection.name,
                         self.editing_connection.url,
                         self.editing_connection.user);
                     self.is_saving = true;
                     self.is_testing_conn = true;
                     self.test_conn_status = None;

                     let conn_config = self.editing_connection.clone();
                     let sender = self.test_sender.clone();

                     self.tokio_runtime.spawn(async move {
                         let client = DbClient::new();
                         let result = client.test_connection(&conn_config).await;
                         let _ = sender.send(result);
                     });
                }
            }

            // Handle window close via X button or Cancel
            if !is_open || cancel_triggered {
                println!("Dialog closed (is_open={}, cancel={}). Resetting state.", is_open, cancel_triggered);
                self.show_connection_modal = false;
                self.test_conn_status = None;
                self.is_saving = false;
                self.is_testing_conn = false;
            }
            
            // Handle async results (polled every frame)
            while let Ok(result) = self.test_receiver.try_recv() {
                println!("Received test result from async task");
                self.is_testing_conn = false;
                match result {
                    Ok(_) => {
                        println!("Test connection successful.");
                        self.test_conn_status = Some(Ok("Connection successful".to_string()));

                        // If we were saving, now proceed to persist
                        if self.is_saving {
                            println!("Proceeding to persist connection...");
                            println!("is_new_connection flag: {}", self.is_new_connection);
                            println!("Connection: name='{}', id='{}'",
                                self.editing_connection.name, self.editing_connection.id);

                            let save_result = if self.is_new_connection {
                                println!("Adding new connection...");
                                self.connection_manager.add(self.editing_connection.clone())
                            } else {
                                println!("Updating existing connection...");
                                self.connection_manager.update(self.editing_connection.clone())
                            };

                            match save_result {
                                Ok(_) => {
                                    println!("Connection {} successfully! Total connections: {}",
                                        if self.is_new_connection { "saved" } else { "updated" },
                                        self.connection_manager.connections.len());
                                    self.test_conn_status = None; // Clear status on success
                                    self.show_connection_modal = false;
                                    self.status = format!("Connection '{}' saved successfully",
                                        self.editing_connection.name);
                                },
                                Err(e) => {
                                    println!("Error {} connection: {}",
                                        if self.is_new_connection { "saving" } else { "updating" }, e);
                                    self.test_conn_status = Some(Err(anyhow::anyhow!("Failed to save: {}", e)));
                                    // Keep dialog open on error
                                }
                            }
                            self.is_saving = false; // Reset flag regardless of outcome
                        }
                    },
                    Err(e) => {
                        println!("Test connection failed: {}", e);
                        self.test_conn_status = Some(Err(e));
                        if self.is_saving {
                            println!("Save aborted due to failed test.");
                            self.is_saving = false;
                        }
                        // Dialog stays open on error
                    }
                }
            }
        }
    }

    fn ui_executor(&mut self, ui: &mut egui::Ui) {
        // Poll for results
        while let Ok(res) = self.query_receiver.try_recv() {
            self.query_result = Some(res);
            self.executing = false;
        }

        ui.heading("SQL Executor");
        ui.separator();
        
        // Connection Selector
        ui.horizontal(|ui| {
            ui.label("Connection:");
            
            let current_name = if let Some(id) = &self.active_connection_id {
                self.connection_manager.connections.iter()
                    .find(|c| c.id == *id)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "Select Connection".to_string())
            } else {
                "Select Connection".to_string()
            };

            egui::ComboBox::from_id_source("active_conn_selector")
                .selected_text(current_name)
                .show_ui(ui, |ui| {
                    let connections = self.connection_manager.connections.clone();
                    for conn in connections {
                        ui.selectable_value(&mut self.active_connection_id, Some(conn.id.clone()), &conn.name);
                    }
                });
            
            ui.add_space(10.0);

            if ui.add_enabled(!self.executing && self.active_connection_id.is_some(), egui::Button::new("▶ Run Query")).clicked() {
                self.run_query();
            }

            if self.executing {
                ui.add(egui::Spinner::new());
                ui.label("Executing...");
            }
        });

        ui.add_space(10.0);

        // SQL Editor
        ui.label("SQL Query:");
        egui::ScrollArea::vertical().id_source("sql_editor_scroll").max_height(200.0).show(ui, |ui| {
            ui.add(egui::TextEdit::multiline(&mut self.executor_sql)
                .font(egui::TextStyle::Monospace)
                .code_editor()
                .desired_width(f32::INFINITY)
                .lock_focus(true));
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Results Section
        ui.heading("Results");
        if let Some(res_raw) = &self.query_result {
            match res_raw {
                Ok(res) => {
                    ui.label(format!("Affected rows: {}, Execution time: {}ms", res.affected_rows, res.execution_time_ms));
                    
                    if !res.columns.is_empty() {
                        egui::ScrollArea::both().id_source("results_table").show(ui, |ui| {
                            egui::Grid::new("result_grid")
                                .striped(true)
                                .max_col_width(300.0)
                                .show(ui, |ui| {
                                    // Header
                                    for col in &res.columns {
                                        ui.strong(col);
                                    }
                                    ui.end_row();

                                    // Rows
                                    for row in &res.rows {
                                        for val in row {
                                            ui.label(val);
                                        }
                                        ui.end_row();
                                    }
                                });
                        });
                    }
                },
                Err(e) => {
                    ui.colored_label(egui::Color32::RED, format!("Error: {}", e));
                }
            }
        } else if !self.executing {
            ui.label("No results yet. Run a query to see results.");
        }
    }

    fn run_query(&mut self) {
        let sql = self.executor_sql.clone();
        let conn_id = match &self.active_connection_id {
            Some(id) => id.clone(),
            None => return,
        };
        let conn = match self.connection_manager.connections.iter().find(|c| c.id == conn_id) {
            Some(c) => c.clone(),
            None => return,
        };

        self.executing = true;
        self.query_result = None;
        
        let sender = self.query_sender.clone();
        
        self.tokio_runtime.spawn(async move {
            let client = DbClient::new();
            let result = client.execute_query(&conn, &sql).await;
            let _ = sender.send(result);
        });
    }
}

impl eframe::App for SqlLogParserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("SQL Log Parser");
                ui.separator();

                // TAB SELECTION
                if ui.selectable_label(self.active_tab == AppTab::LogParser, "📄 Log Parser").clicked() {
                    self.active_tab = AppTab::LogParser;
                }
                if ui.selectable_label(self.active_tab == AppTab::SqlExecutor, "▶ SQL Executor").clicked() {
                    self.active_tab = AppTab::SqlExecutor;
                }

                ui.separator();
                
                // Show Global Settings only if relevant
                if self.active_tab == AppTab::LogParser {
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
                }
            });
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("statusbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("v2.1.0");
                });
            });
        });

        // Left sidebar
        egui::SidePanel::left("sidebar")
            .default_width(200.0)
            .resizable(true)
            .show(ctx, |ui| {
                match self.active_tab {
                    AppTab::LogParser => self.sidebar_parser(ctx, ui),
                    AppTab::SqlExecutor => self.sidebar_executor(ctx, ui),
                }
            });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                AppTab::LogParser => self.ui_parser(ui),
                AppTab::SqlExecutor => self.ui_executor(ui),
            }
        });
    }
}
