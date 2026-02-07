mod commands;
mod config;
mod core;
mod state;
mod utils;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_all_ids,
            commands::process_query,
            commands::process_last_query,
            commands::list_connections,
            commands::add_connection,
            commands::update_connection,
            commands::delete_connection,
            commands::test_connection,
            commands::execute_query,
            commands::load_config,
            commands::save_config,
            commands::copy_to_clipboard,
            commands::parse_jdbc_url_cmd,
            commands::build_jdbc_url_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
