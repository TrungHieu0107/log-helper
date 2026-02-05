use crate::config::{Config, ConfigManager};
use crate::core::log_parser::{IdInfo, LogParser};
use crate::core::query_processor::{ProcessResult, QueryProcessor};
use crate::utils::clipboard;
use eframe::egui;
use rfd::FileDialog;
use uuid::Uuid;

use crate::core::db::{
    ConnectionManager, DbClient, DbConfig, DbExecutorRegistry, DbType, MsSqlExecutor, QueryResult,
    SqlxExecutor,
};
use std::sync::Arc;
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

    // Connection Editing
    show_connection_modal: bool,
    editing_connection: DbConfig,

    status: String,
}

impl SqlLogParserApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config_manager = ConfigManager::new();
        let config = config_manager.load();

        let (query_sender, query_receiver) = std::sync::mpsc::channel();

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

            db_client: DbClient::new(Arc::new(DbExecutorRegistry::new())),
            tokio_runtime: Runtime::new().expect("Failed to create tokio runtime"),
            active_connection_id: None,
            query_result: None,
            executing: false,

            query_sender,
            query_receiver,

            show_connection_modal: false,
            editing_connection: DbConfig::default(),

            status: "Ready".to_string(),
        };

        let mut registry = DbExecutorRegistry::new();
        registry.register(DbType::Postgres, SqlxExecutor);
        registry.register(DbType::Mysql, SqlxExecutor);
        registry.register(DbType::Sqlite, SqlxExecutor);
        registry.register(DbType::SqlServer, MsSqlExecutor);
        app.db_client = DbClient::new(Arc::new(registry));

        // Propagate encoding to query processor as well
        app.query_processor
            .parser_mut()
            .set_encoding(app.config.encoding.clone());

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
        self.result = Some(
            self.query_processor
                .process_last_query(&self.config.log_file_path, self.config.auto_copy),
        );

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
                    let base_label =
                        if !id_info.dao_name.is_empty() && id_info.dao_name != "Unknown" {
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
                        text = text.color(egui::Color32::from_rgb(80, 250, 123)).strong();
                    // Green
                    } else {
                        text = text.color(egui::Color32::from_rgb(248, 248, 242));
                        // Foreground
                    }

                    if ui
                        .add(egui::SelectableLabel::new(is_selected, text))
                        .clicked()
                    {
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
            self.editing_connection = DbConfig::default();
            self.editing_connection.id = Uuid::new_v4().to_string();
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
            self.editing_connection = conn;
            self.show_connection_modal = true;
        }

        if let Some(id) = delete_id {
            self.connection_manager.delete(&id);
        }

        self.connection_edit_modal(ctx);
    }

    fn connection_edit_modal(&mut self, ctx: &egui::Context) {
        if self.show_connection_modal {
            let mut is_open = true;
            let mut save_triggered = false;
            let mut cancel_triggered = false;

            egui::Window::new("Connection Details")
                .open(&mut is_open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    egui::Grid::new("connection_grid")
                        .num_columns(2)
                        .spacing([10.0, 10.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.editing_connection.name);
                            ui.end_row();

                            ui.label("Type:");
                            egui::ComboBox::from_id_source("db_type")
                                .selected_text(self.editing_connection.db_type.to_string())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.editing_connection.db_type,
                                        DbType::SqlServer,
                                        "SQL Server",
                                    );
                                    ui.selectable_value(
                                        &mut self.editing_connection.db_type,
                                        DbType::Postgres,
                                        "Postgres",
                                    );
                                    ui.selectable_value(
                                        &mut self.editing_connection.db_type,
                                        DbType::Mysql,
                                        "MySQL",
                                    );
                                    ui.selectable_value(
                                        &mut self.editing_connection.db_type,
                                        DbType::Sqlite,
                                        "SQLite",
                                    );
                                });
                            ui.end_row();

                            ui.label("JDBC URL:");
                            ui.text_edit_singleline(&mut self.editing_connection.url)
                                .on_hover_text(
                                    "Example: jdbc:sqlserver://host:port;databaseName=DBNAME",
                                );
                            ui.end_row();

                            ui.label("User:");
                            ui.text_edit_singleline(&mut self.editing_connection.user);
                            ui.end_row();

                            ui.label("Password:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.editing_connection.password)
                                    .password(true),
                            );
                            ui.end_row();
                        });

                    ui.add_space(15.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            save_triggered = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel_triggered = true;
                        }
                    });
                });

            if !is_open || cancel_triggered {
                self.show_connection_modal = false;
            } else if save_triggered {
                if self.editing_connection.id.is_empty() {
                    self.editing_connection.id = Uuid::new_v4().to_string();
                    self.connection_manager.add(self.editing_connection.clone());
                } else {
                    self.connection_manager
                        .update(self.editing_connection.clone());
                }
                self.show_connection_modal = false;
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
                self.connection_manager
                    .connections
                    .iter()
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
                        ui.selectable_value(
                            &mut self.active_connection_id,
                            Some(conn.id.clone()),
                            &conn.name,
                        );
                    }
                });

            ui.add_space(10.0);

            if ui
                .add_enabled(
                    !self.executing && self.active_connection_id.is_some(),
                    egui::Button::new("▶ Run Query"),
                )
                .clicked()
            {
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
        egui::ScrollArea::vertical()
            .id_source("sql_editor_scroll")
            .max_height(200.0)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.executor_sql)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .lock_focus(true),
                );
            });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Results Section
        ui.heading("Results");
        if let Some(res_raw) = &self.query_result {
            match res_raw {
                Ok(res) => {
                    ui.label(format!(
                        "Affected rows: {}, Execution time: {}ms",
                        res.affected_rows, res.execution_time_ms
                    ));

                    if !res.columns.is_empty() {
                        egui::ScrollArea::both()
                            .id_source("results_table")
                            .show(ui, |ui| {
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
                }
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
        let conn = match self
            .connection_manager
            .connections
            .iter()
            .find(|c| c.id == conn_id)
        {
            Some(c) => c.clone(),
            None => return,
        };

        self.executing = true;
        self.query_result = None;

        let sender = self.query_sender.clone();

        let client = self.db_client.clone();
        self.tokio_runtime.spawn(async move {
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
                if ui
                    .selectable_label(self.active_tab == AppTab::LogParser, "📄 Log Parser")
                    .clicked()
                {
                    self.active_tab = AppTab::LogParser;
                }
                if ui
                    .selectable_label(self.active_tab == AppTab::SqlExecutor, "▶ SQL Executor")
                    .clicked()
                {
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

                    if ui
                        .button("⏮ Last SQL")
                        .on_hover_text("Load the last executed SQL query from the log")
                        .clicked()
                    {
                        self.get_last_sql();
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Encoding:");
                        let current_encoding = self.config.encoding.clone();
                        egui::ComboBox::from_id_source("encoding_selector")
                            .selected_text(&current_encoding)
                            .show_ui(ui, |ui| {
                                let encodings = [
                                    "SHIFT_JIS",
                                    "UTF-8",
                                    "UTF-16LE",
                                    "UTF-16BE",
                                    "EUC-JP",
                                    "WINDOWS-1252",
                                ];
                                for &enc in &encodings {
                                    if ui
                                        .selectable_value(
                                            &mut self.config.encoding,
                                            enc.to_string(),
                                            enc,
                                        )
                                        .changed()
                                    {
                                        // Save config and update parsers immediately
                                        let _ = self.config_manager.save(&self.config);
                                        let encoding = self.config.encoding.clone();
                                        self.query_processor
                                            .parser_mut()
                                            .set_encoding(encoding.clone());
                                        self.log_parser.set_encoding(encoding);
                                    }
                                }
                            });
                    });

                    ui.separator();

                    if ui
                        .checkbox(&mut self.config.auto_copy, "Auto Copy")
                        .changed()
                    {
                        let _ = self.config_manager.save(&self.config);
                    }

                    ui.separator();

                    if ui
                        .checkbox(&mut self.config.format_sql, "Format SQL")
                        .changed()
                    {
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
            .show(ctx, |ui| match self.active_tab {
                AppTab::LogParser => self.sidebar_parser(ctx, ui),
                AppTab::SqlExecutor => self.sidebar_executor(ctx, ui),
            });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| match self.active_tab {
            AppTab::LogParser => self.ui_parser(ui),
            AppTab::SqlExecutor => self.ui_executor(ui),
        });
    }
}
