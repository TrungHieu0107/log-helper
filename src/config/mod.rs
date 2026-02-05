//! Configuration management module.
//!
//! Handles loading and saving application configuration from JSON file.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Single database connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DbConnection {
    pub name: String,
    pub server: String,
    pub database: String,
    pub username: String,
    pub password: String,
    #[serde(default = "default_true")]
    pub use_windows_auth: bool,
}

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub log_file_path: String,
    #[serde(default)]
    pub html_output_path: String,
    #[serde(skip)]
    pub config_file: String,
    #[serde(default = "default_true")]
    pub auto_copy: bool,
    #[serde(default)]
    pub connections: Vec<DbConnection>,
    #[serde(default = "default_neg_one")]
    pub active_connection_index: i32,
    #[serde(default = "default_comma")]
    pub csv_separator: String,
    #[serde(default = "default_encoding")]
    pub encoding: String,
    #[serde(default = "default_true")]
    pub format_sql: bool,
}

fn default_true() -> bool {
    true
}

fn default_neg_one() -> i32 {
    -1
}

fn default_comma() -> String {
    ",".to_string()
}

fn default_encoding() -> String {
    "SHIFT_JIS".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_file_path: String::new(),
            html_output_path: String::new(),
            config_file: String::new(),
            auto_copy: true,
            connections: Vec::new(),
            active_connection_index: -1,
            csv_separator: ",".to_string(),
            encoding: "SHIFT_JIS".to_string(),
            format_sql: true,
        }
    }
}

/// Configuration manager for loading/saving config.
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        let config_path = Self::get_exe_directory().join("log_parser_config.json");
        Self { config_path }
    }

    /// Get the directory containing the executable.
    fn get_exe_directory() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Get the default log file path.
    fn get_default_log_path() -> String {
        Self::get_exe_directory()
            .join("stcApp.log")
            .to_string_lossy()
            .into_owned()
    }

    /// Get the config file path.
    pub fn get_config_file_path(&self) -> &Path {
        &self.config_path
    }

    /// Load configuration from file.
    pub fn load(&self) -> Config {
        let mut config = self.try_load().unwrap_or_default();
        
        config.config_file = self.config_path.to_string_lossy().into_owned();

        // Set defaults if empty
        if config.log_file_path.is_empty() {
            config.log_file_path = Self::get_default_log_path();
        }
        if config.html_output_path.is_empty() {
            config.html_output_path = Self::get_exe_directory().to_string_lossy().into_owned();
        }
        if config.csv_separator.is_empty() {
            config.csv_separator = ",".to_string();
        }
        if config.encoding.is_empty() {
            config.encoding = "SHIFT_JIS".to_string();
        }

        config
    }

    fn try_load(&self) -> Option<Config> {
        if !self.config_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&self.config_path).ok()?;
        let mut config: Config = serde_json::from_str(&content).ok()?;

        // Migration: convert old single connection format to new format
        if config.connections.is_empty() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(server) = json.get("sqlServer").and_then(|v| v.as_str()) {
                    if !server.is_empty() {
                        let conn = DbConnection {
                            name: "Default".to_string(),
                            server: server.to_string(),
                            database: json.get("sqlDatabase")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            username: json.get("sqlUsername")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            password: json.get("sqlPassword")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            use_windows_auth: json.get("sqlUseWindowsAuth")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(true),
                        };
                        config.connections.push(conn);
                        config.active_connection_index = 0;
                    }
                }
            }
        }

        Some(config)
    }

    /// Save configuration to file.
    pub fn save(&self, config: &Config) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&self.config_path, json)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.auto_copy);
        assert_eq!(config.active_connection_index, -1);
        assert_eq!(config.csv_separator, ",");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.auto_copy, config.auto_copy);
    }
}
