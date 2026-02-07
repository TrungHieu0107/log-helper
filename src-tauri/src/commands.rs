use tauri::State;

use crate::core::db::{
    CellValue, ConnectionFields, DbConfig, ParsedSqlServerUrl,
    QueryResult as DbQueryResult,
};
use crate::core::log_parser::IdInfo;
use crate::core::query_processor::ProcessResult;
use crate::config::Config;
use crate::state::AppState;

// ─── Log Parser Commands ────────────────────────────────────────────────────

#[tauri::command]
pub fn get_all_ids(
    state: State<AppState>,
    log_path: String,
    encoding: String,
) -> Vec<IdInfo> {
    use crate::core::log_parser::LogParser;
    let parser = LogParser::new(encoding);
    parser.get_all_ids(&log_path)
}

#[tauri::command]
pub fn process_query(
    state: State<AppState>,
    target_id: String,
    log_path: String,
    auto_copy: bool,
    encoding: String,
) -> ProcessResult {
    let mut processor = state.query_processor.lock().unwrap();
    processor.parser_mut().set_encoding(encoding);
    processor.process_query(&target_id, &log_path, auto_copy)
}

#[tauri::command]
pub fn process_last_query(
    state: State<AppState>,
    log_path: String,
    auto_copy: bool,
    encoding: String,
) -> ProcessResult {
    let mut processor = state.query_processor.lock().unwrap();
    processor.parser_mut().set_encoding(encoding);
    processor.process_last_query(&log_path, auto_copy)
}

// ─── Database Connection Commands ───────────────────────────────────────────

#[tauri::command]
pub fn list_connections(state: State<AppState>) -> Vec<DbConfig> {
    let mgr = state.connection_manager.lock().unwrap();
    mgr.connections.clone()
}

#[tauri::command]
pub fn add_connection(
    state: State<AppState>,
    config: DbConfig,
) -> Result<(), String> {
    let mut mgr = state.connection_manager.lock().unwrap();
    mgr.add(config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_connection(
    state: State<AppState>,
    config: DbConfig,
) -> Result<(), String> {
    let mut mgr = state.connection_manager.lock().unwrap();
    mgr.update(config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_connection(
    state: State<AppState>,
    id: String,
) -> Result<(), String> {
    let mut mgr = state.connection_manager.lock().unwrap();
    mgr.delete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_connection(
    state: State<'_, AppState>,
    config: DbConfig,
) -> Result<String, String> {
    let client = state.db_client.clone();
    client
        .test_connection(&config)
        .await
        .map_err(|e| e.to_string())?;
    Ok("Connection successful".to_string())
}

#[tauri::command]
pub async fn execute_query(
    state: State<'_, AppState>,
    connection_id: String,
    sql: String,
) -> Result<DbQueryResult, String> {
    let conn = {
        let mgr = state.connection_manager.lock().unwrap();
        mgr.connections
            .iter()
            .find(|c| c.id == connection_id)
            .cloned()
            .ok_or_else(|| "Connection not found".to_string())?
    };

    let client = state.db_client.clone();
    client
        .execute_query(&conn, &sql)
        .await
        .map_err(|e| e.to_string())
}

// ─── Config Commands ────────────────────────────────────────────────────────

#[tauri::command]
pub fn load_config(state: State<AppState>) -> Config {
    let config = state.config.lock().unwrap();
    config.clone()
}

#[tauri::command]
pub fn save_config(
    state: State<AppState>,
    new_config: Config,
) -> Result<(), String> {
    let config_mgr = state.config_manager.lock().unwrap();

    // Update parser encoding if it changed
    {
        let current = state.config.lock().unwrap();
        if current.encoding != new_config.encoding {
            let mut processor = state.query_processor.lock().unwrap();
            processor
                .parser_mut()
                .set_encoding(new_config.encoding.clone());
        }
    }

    let mut config = state.config.lock().unwrap();
    *config = new_config;
    config_mgr.save(&config).map_err(|e| e.to_string())
}

// ─── Utility Commands ───────────────────────────────────────────────────────

#[tauri::command]
pub fn copy_to_clipboard(text: String) -> bool {
    crate::utils::clipboard::copy_to_clipboard(&text)
}

#[tauri::command]
pub fn parse_jdbc_url_cmd(url: String) -> Result<ParsedSqlServerUrl, String> {
    crate::core::db::parse_jdbc_url(&url)
}

#[tauri::command]
pub fn build_jdbc_url_cmd(fields: ConnectionFields) -> String {
    crate::core::db::build_jdbc_url(&fields)
}
