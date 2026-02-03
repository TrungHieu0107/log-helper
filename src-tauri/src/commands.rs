use crate::config::{Config, ConfigManager};
use crate::core::log_parser::{IdInfo, LogParser, QueryResult, Execution};
use crate::core::query_processor::{ProcessResult, QueryProcessor};
#[cfg(feature = "sql")]
use crate::utils::sql_connector::{SqlConnector, SqlResult};
use std::sync::Mutex;
use tauri::State;

/// Shared application state.
pub struct AppState {
    pub parser: LogParser, // Stateless
    pub processor: QueryProcessor, // Stateless
    pub config_manager: ConfigManager,
    #[cfg(feature = "sql")]
    pub sql_connector: Mutex<SqlConnector>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            parser: LogParser::new(),
            processor: QueryProcessor::new(),
            config_manager: ConfigManager::new(),
            #[cfg(feature = "sql")]
            sql_connector: Mutex::new(SqlConnector::new()),
        }
    }
}

#[tauri::command]
pub fn load_config(state: State<AppState>) -> Result<Config, String> {
    Ok(state.config_manager.load())
}

#[tauri::command]
pub fn save_config(state: State<AppState>, config: Config) -> Result<(), String> {
    state.config_manager.save(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_ids(state: State<AppState>, log_file_path: String) -> Result<Vec<IdInfo>, String> {
    Ok(state.parser.get_all_ids(&log_file_path))
}

#[tauri::command]
pub fn parse_log_by_id(
    state: State<AppState>,
    log_file_path: String,
    target_id: String
) -> Result<QueryResult, String> {
    Ok(state.parser.parse_log_file(&log_file_path, &target_id))
}

#[tauri::command]
pub fn process_query(
    state: State<AppState>,
    log_file_path: String,
    target_id: String,
    auto_copy: bool
) -> Result<ProcessResult, String> {
    Ok(state.processor.process_query(&target_id, &log_file_path, auto_copy))
}

#[tauri::command]
pub fn process_last_query(
    state: State<AppState>,
    log_file_path: String,
    auto_copy: bool
) -> Result<ProcessResult, String> {
    Ok(state.processor.process_last_query(&log_file_path, auto_copy))
}

#[cfg(feature = "sql")]
#[tauri::command]
pub fn test_connection(
    state: State<AppState>,
    connection_config: crate::config::DbConnection
) -> Result<bool, String> {
    let mut conn = state.sql_connector.lock().map_err(|_| "Failed to lock SQL connector".to_string())?;
    
    let success = conn.connect(
        &connection_config.server,
        &connection_config.database,
        &connection_config.username,
        &connection_config.password,
        connection_config.use_windows_auth
    );

    if !success {
        return Err(conn.get_last_error().to_string());
    }

    Ok(true)
}

#[cfg(feature = "sql")]
#[tauri::command]
pub fn execute_sql(
    state: State<AppState>,
    sql: String
) -> Result<SqlResult, String> {
    let mut conn = state.sql_connector.lock().map_err(|_| "Failed to lock SQL connector".to_string())?;
    
    if !conn.is_connected() {
        return Err("Not connected to database".to_string());
    }

    Ok(conn.execute_query(&sql))
}
