use std::sync::Mutex;

use crate::config::{Config, ConfigManager};
use crate::core::db::{ConnectionManager, DbClient};
use crate::core::query_processor::QueryProcessor;

pub struct AppState {
    pub config_manager: Mutex<ConfigManager>,
    pub config: Mutex<Config>,
    pub query_processor: Mutex<QueryProcessor>,
    pub connection_manager: Mutex<ConnectionManager>,
    pub db_client: DbClient,
}

impl AppState {
    pub fn new() -> Self {
        let config_manager = ConfigManager::new();
        let config = config_manager.load();
        let encoding = config.encoding.clone();

        let mut query_processor = QueryProcessor::new();
        query_processor.parser_mut().set_encoding(encoding);

        Self {
            config_manager: Mutex::new(config_manager),
            config: Mutex::new(config),
            query_processor: Mutex::new(query_processor),
            connection_manager: Mutex::new(ConnectionManager::new()),
            db_client: DbClient::new(),
        }
    }
}
