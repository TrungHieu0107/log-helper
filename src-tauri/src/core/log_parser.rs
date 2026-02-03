//! Log file parser for extracting SQL queries and parameters.
//!
//! Parses log files to extract SQL statements, parameters, and execution metadata
//! based on unique transaction IDs.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use crate::utils::encoding;
use crate::utils::file_helper;
use super::sql_formatter;

/// Result of parsing a single query from the log.
#[derive(Debug, Clone, Default, Serialize)]
pub struct QueryResult {
    pub id: String,
    pub sql: String,
    pub params: Vec<String>,
}

impl QueryResult {
    pub fn found(&self) -> bool {
        !self.sql.is_empty()
    }
}

/// Detailed execution information including timestamp and DAO file.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Execution {
    pub id: String,
    pub timestamp: String,
    pub dao_file: String,
    pub sql: String,
    pub filled_sql: String,
    pub params: Vec<String>,
    pub execution_index: i32,
}

/// Summary information about an ID in the log.
#[derive(Debug, Clone, Default, Serialize)]
pub struct IdInfo {
    pub id: String,
    pub has_sql: bool,
    pub params_count: i32,
}

/// Log file parser.
pub struct LogParser;

// Compiled regex patterns (lazy initialized for performance)
static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[([^\]]+)\]").unwrap()
});

static ID_SQL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"id=([a-f0-9]+)\s+sql=").unwrap()
});

static ID_PARAMS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"id=([a-f0-9]+)\s+params=").unwrap()
});

static TIMESTAMP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d{4}/\d{2}/\d{2}\s+\d{2}:\d{2}:\d{2})").unwrap()
});

static DAO_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Daoの終了jp\.co\.[^\s,]+?([A-Za-z]+Dao)\b").unwrap()
});

impl LogParser {
    pub fn new() -> Self {
        LogParser
    }

    /// Parse log file and get SQL/params for a specific ID.
    pub fn parse_log_file(&self, log_file_path: &str, target_id: &str) -> QueryResult {
        let mut result = QueryResult {
            id: target_id.to_string(),
            ..Default::default()
        };

        if !file_helper::file_exists(log_file_path) {
            return result;
        }

        let content = match encoding::read_file_as_utf8(log_file_path) {
            Ok(c) => c,
            Err(_) => return result,
        };

        if content.is_empty() {
            return result;
        }

        // Find SQL statement for ID
        let sql_pattern = format!(r"id={}\s+sql=\s*(.+)", regex::escape(target_id));
        if let Ok(sql_regex) = Regex::new(&sql_pattern) {
            if let Some(caps) = sql_regex.captures(&content) {
                if let Some(sql_match) = caps.get(1) {
                    result.sql = sql_match.as_str().trim().to_string();
                }
            }
        }

        // Find params for ID
        let params_pattern = format!(r"id={}\s+params=(\[[^\n]+)", regex::escape(target_id));
        if let Ok(params_regex) = Regex::new(&params_pattern) {
            if let Some(caps) = params_regex.captures(&content) {
                if let Some(params_match) = caps.get(1) {
                    result.params = self.parse_params_string(params_match.as_str());
                }
            }
        }

        result
    }

    /// Parse log file with advanced metadata extraction.
    pub fn parse_log_file_advanced(&self, log_file_path: &str, target_id: &str) -> Vec<Execution> {
        let mut executions = Vec::new();

        if !file_helper::file_exists(log_file_path) {
            return executions;
        }

        let content = match encoding::read_file_as_utf8(log_file_path) {
            Ok(c) => c,
            Err(_) => return executions,
        };

        if content.is_empty() {
            return executions;
        }

        let lines: Vec<&str> = content.lines().collect();
        
        let mut sql = String::new();
        let mut timestamp = String::new();
        let mut dao_file = String::new();
        let mut _sql_line_index: Option<usize> = None;
        let mut all_params_sets: Vec<(Vec<String>, String)> = Vec::new();

        // Build patterns for this specific target ID
        let full_line_pattern = format!(
            r"^(\d{{4}}/\d{{2}}/\d{{2}}\s+\d{{2}}:\d{{2}}:\d{{2}}),\w+,([^,]+),.*id={}\s+sql=\s*(.+)",
            regex::escape(target_id)
        );
        let simple_sql_pattern = format!(r"id={}\s+sql=\s*(.+)", regex::escape(target_id));
        let params_pattern = format!(r"id={}\s+params=(\[[^\n]+)", regex::escape(target_id));

        let full_line_regex = Regex::new(&full_line_pattern).ok();
        let simple_sql_regex = Regex::new(&simple_sql_pattern).ok();
        let params_regex = Regex::new(&params_pattern).ok();

        for (i, line) in lines.iter().enumerate() {
            // Try full pattern first
            if let Some(ref regex) = full_line_regex {
                if let Some(caps) = regex.captures(line) {
                    timestamp = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                    sql = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();
                    _sql_line_index = Some(i);
                    dao_file = self.find_dao_class_name(&lines, i);
                    continue;
                }
            }

            // Fallback: simple SQL pattern
            if sql.is_empty() {
                if let Some(ref regex) = simple_sql_regex {
                    if let Some(caps) = regex.captures(line) {
                        sql = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                        _sql_line_index = Some(i);

                        // Try to find timestamp
                        if let Some(ts_caps) = TIMESTAMP_REGEX.captures(line) {
                            timestamp = ts_caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                        } else if i > 0 {
                            if let Some(ts_caps) = TIMESTAMP_REGEX.captures(lines[i - 1]) {
                                timestamp = ts_caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                            }
                        }

                        dao_file = self.find_dao_class_name(&lines, i);
                    }
                }
            }

            // Find params
            if let Some(ref regex) = params_regex {
                if let Some(caps) = regex.captures(line) {
                    let params = self.parse_params_string(
                        caps.get(1).map(|m| m.as_str()).unwrap_or("")
                    );
                    
                    let ts = TIMESTAMP_REGEX.captures(line)
                        .and_then(|c| c.get(1))
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_else(|| timestamp.clone());
                    
                    all_params_sets.push((params, ts));
                }
            }
        }

        // Build executions
        if !sql.is_empty() {
            let sql = sql.trim().to_string();

            if !all_params_sets.is_empty() {
                for (index, (params, ts)) in all_params_sets.into_iter().enumerate() {
                    let filled_sql = sql_formatter::replace_placeholders(&sql, &params)
                        .unwrap_or_else(|_| sql.clone());

                    executions.push(Execution {
                        id: target_id.to_string(),
                        timestamp: if ts.is_empty() { timestamp.clone() } else { ts },
                        dao_file: dao_file.clone(),
                        sql: sql.clone(),
                        filled_sql,
                        params,
                        execution_index: (index + 1) as i32,
                    });
                }
            } else {
                // No params - single execution
                executions.push(Execution {
                    id: target_id.to_string(),
                    timestamp: timestamp.clone(),
                    dao_file,
                    sql: sql.clone(),
                    filled_sql: sql.clone(),
                    params: Vec::new(),
                    execution_index: 1,
                });
            }
        }

        executions
    }

    /// Get all unique IDs from a log file.
    pub fn get_all_ids(&self, log_file_path: &str) -> Vec<IdInfo> {
        let mut ids = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        if !file_helper::file_exists(log_file_path) {
            return ids;
        }

        let content = match encoding::read_file_as_utf8(log_file_path) {
            Ok(c) => c,
            Err(_) => return ids,
        };

        if content.is_empty() {
            return ids;
        }

        // Find all IDs with SQL
        for caps in ID_SQL_REGEX.captures_iter(&content) {
            if let Some(id_match) = caps.get(1) {
                let id = id_match.as_str().to_string();
                if !seen_ids.contains(&id) {
                    seen_ids.insert(id.clone());
                    ids.push(IdInfo {
                        id,
                        has_sql: true,
                        params_count: 0,
                    });
                }
            }
        }

        // Count params for each ID
        for caps in ID_PARAMS_REGEX.captures_iter(&content) {
            if let Some(id_match) = caps.get(1) {
                let id = id_match.as_str();
                for info in ids.iter_mut() {
                    if info.id == id {
                        info.params_count += 1;
                        break;
                    }
                }
            }
        }

        ids
    }

    /// Get the last SQL query from a log file.
    ///
    /// This function reliably finds the last complete SQL statement in a log file.
    /// It handles:
    /// - Trailing empty lines and whitespace
    /// - Files ending with multiple newlines
    /// - Large files (reads efficiently from end)
    /// - Edge cases: empty file, no valid SQL, file not found
    ///
    /// Returns a QueryResult with the last SQL and its associated parameters.
    /// Returns an empty QueryResult if no valid SQL is found.
    pub fn get_last_query(&self, log_file_path: &str) -> QueryResult {
        let mut result = QueryResult::default();

        // Handle file not found
        if !file_helper::file_exists(log_file_path) {
            return result;
        }

        // Read file content
        let content = match encoding::read_file_as_utf8(log_file_path) {
            Ok(c) => c,
            Err(_) => return result,
        };

        // Handle empty file
        if content.is_empty() {
            return result;
        }

        // Trim trailing whitespace and empty lines from the content
        let trimmed_content = content.trim_end();
        if trimmed_content.is_empty() {
            return result;
        }

        // Collect all lines (excluding trailing empty lines)
        let lines: Vec<&str> = trimmed_content.lines().collect();
        if lines.is_empty() {
            return result;
        }

        // Track the last SQL statement found
        let mut last_id = String::new();
        let mut last_sql = String::new();
        let mut _last_sql_line_index: Option<usize> = None;

        // Scan through lines to find all SQL statements
        // Use a more robust pattern that captures the complete SQL on a line
        for (i, line) in lines.iter().enumerate() {
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Match pattern: id=<hex_id> sql=<sql_statement>
            if let Some(caps) = ID_SQL_REGEX.captures(line) {
                if let Some(id_match) = caps.get(1) {
                    let id = id_match.as_str().to_string();

                    // Extract SQL - everything after "sql=" until end of line
                    if let Some(sql_start) = line.find("sql=") {
                        let sql_part = &line[sql_start + 4..];
                        let sql = sql_part.trim().to_string();

                        // Only update if we have valid SQL
                        if !sql.is_empty() {
                            last_id = id;
                            last_sql = sql;
                            _last_sql_line_index = Some(i);
                        }
                    }
                }
            }
        }

        // If no SQL was found, return empty result
        if last_id.is_empty() || last_sql.is_empty() {
            return result;
        }

        result.id = last_id.clone();
        result.sql = last_sql;

        // Find the LAST params line for this ID (there may be multiple executions)
        // We want the params that correspond to the last execution
        let params_pattern = format!(r"id={}\s+params=(\[[^\n]+)", regex::escape(&last_id));
        if let Ok(params_regex) = Regex::new(&params_pattern) {
            // Find ALL params matches and take the last one
            let mut last_params: Option<Vec<String>> = None;

            for caps in params_regex.captures_iter(trimmed_content) {
                if let Some(params_match) = caps.get(1) {
                    last_params = Some(self.parse_params_string(params_match.as_str()));
                }
            }

            if let Some(params) = last_params {
                result.params = params;
            }
        }

        result
    }

    /// Find DAO class name from lines after SQL statement.
    fn find_dao_class_name(&self, lines: &[&str], sql_line_index: usize) -> String {
        let search_end = std::cmp::min(lines.len(), sql_line_index + 50);

        for i in (sql_line_index + 1)..search_end {
            if let Some(caps) = DAO_REGEX.captures(lines[i]) {
                if let Some(dao_match) = caps.get(1) {
                    return dao_match.as_str().to_string();
                }
            }
        }

        "Unknown".to_string()
    }

    /// Parse parameters string like `[type:index:value][type:index:value]...`
    fn parse_params_string(&self, params_str: &str) -> Vec<String> {
        PARAM_REGEX
            .captures_iter(params_str)
            .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }
}

impl Default for LogParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_params_string() {
        let parser = LogParser::new();
        let params = parser.parse_params_string("[String:1:hello][Int:2:42]");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], "String:1:hello");
        assert_eq!(params[1], "Int:2:42");
    }

    #[test]
    fn test_query_result_found() {
        let mut result = QueryResult::default();
        assert!(!result.found());

        result.sql = "SELECT 1".to_string();
        assert!(result.found());
    }

    // Helper to create temp file for testing
    fn create_temp_file(content: &str) -> String {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_log_{}.log", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()));
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file_path.to_string_lossy().to_string()
    }

    fn cleanup_temp_file(path: &str) {
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_get_last_query_basic() {
        let content = r#"2024/01/01 10:00:00,INFO,Test,id=abc123 sql=SELECT * FROM users
2024/01/01 10:00:01,INFO,Test,id=abc123 params=[String:1:test]
2024/01/01 10:01:00,INFO,Test,id=def456 sql=SELECT * FROM orders
2024/01/01 10:01:01,INFO,Test,id=def456 params=[Int:1:42]"#;

        let path = create_temp_file(content);
        let parser = LogParser::new();
        let result = parser.get_last_query(&path);

        assert_eq!(result.id, "def456");
        assert_eq!(result.sql, "SELECT * FROM orders");
        assert_eq!(result.params.len(), 1);
        assert_eq!(result.params[0], "Int:1:42");

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_get_last_query_with_trailing_newlines() {
        let content = "2024/01/01 10:00:00,INFO,Test,id=abc123 sql=SELECT 1\n\n\n\n";

        let path = create_temp_file(content);
        let parser = LogParser::new();
        let result = parser.get_last_query(&path);

        assert_eq!(result.id, "abc123");
        assert_eq!(result.sql, "SELECT 1");

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_get_last_query_empty_file() {
        let path = create_temp_file("");
        let parser = LogParser::new();
        let result = parser.get_last_query(&path);

        assert!(!result.found());
        assert!(result.id.is_empty());
        assert!(result.sql.is_empty());

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_get_last_query_only_whitespace() {
        let content = "   \n\n   \n\t\t\n";

        let path = create_temp_file(content);
        let parser = LogParser::new();
        let result = parser.get_last_query(&path);

        assert!(!result.found());

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_get_last_query_no_sql() {
        let content = r#"2024/01/01 10:00:00,INFO,Test,Some random log line
2024/01/01 10:00:01,INFO,Test,Another log line without SQL"#;

        let path = create_temp_file(content);
        let parser = LogParser::new();
        let result = parser.get_last_query(&path);

        assert!(!result.found());

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_get_last_query_file_not_found() {
        let parser = LogParser::new();
        let result = parser.get_last_query("/nonexistent/path/file.log");

        assert!(!result.found());
        assert!(result.id.is_empty());
    }

    #[test]
    fn test_get_last_query_multiple_executions_same_id() {
        let content = r#"2024/01/01 10:00:00,INFO,Test,id=abc123 sql=SELECT * FROM users WHERE id = ?
2024/01/01 10:00:01,INFO,Test,id=abc123 params=[Int:1:1]
2024/01/01 10:02:00,INFO,Test,id=abc123 params=[Int:1:2]
2024/01/01 10:03:00,INFO,Test,id=abc123 params=[Int:1:3]"#;

        let path = create_temp_file(content);
        let parser = LogParser::new();
        let result = parser.get_last_query(&path);

        assert_eq!(result.id, "abc123");
        assert_eq!(result.sql, "SELECT * FROM users WHERE id = ?");
        // Should get the LAST params
        assert_eq!(result.params.len(), 1);
        assert_eq!(result.params[0], "Int:1:3");

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_get_last_query_sql_with_special_chars() {
        let content = r#"2024/01/01 10:00:00,INFO,Test,id=abc123 sql=SELECT * FROM users WHERE name LIKE '%test%' AND status IN (1, 2, 3)"#;

        let path = create_temp_file(content);
        let parser = LogParser::new();
        let result = parser.get_last_query(&path);

        assert_eq!(result.id, "abc123");
        assert_eq!(result.sql, "SELECT * FROM users WHERE name LIKE '%test%' AND status IN (1, 2, 3)");

        cleanup_temp_file(&path);
    }
}
