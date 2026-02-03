#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod core;
mod utils;

use commands::AppState;
use tauri::Manager;

fn main() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::load_config,
            commands::save_config,
            commands::get_all_ids,
            commands::parse_log_by_id,
            commands::process_query,
            commands::process_last_query,
            commands::test_connection,
            commands::execute_sql,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
