//! Log file parser for extracting SQL queries and parameters.
//!
//! Parses log files to extract SQL statements, parameters, and execution metadata
//! based on unique transaction IDs.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use crate::utils::encoding::{self, read_file_lines};
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
pub struct LogParser {
    encoding: String,
}

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
    pub fn new(encoding: String) -> Self {
        LogParser { encoding }
    }

    pub fn set_encoding(&mut self, encoding: String) {
        self.encoding = encoding;
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

        let lines = match read_file_lines(log_file_path, &self.encoding) {
            Ok(iter) => iter,
            Err(_) => return result,
        };

        let sql_pattern = format!(r"id={}\s+sql=\s*(.+)", regex::escape(target_id));
        let params_pattern = format!(r"id={}\s+params=(\[[^\n]+)", regex::escape(target_id));
        
        let sql_regex = Regex::new(&sql_pattern).ok();
        let params_regex = Regex::new(&params_pattern).ok();
        
        // Flags to stop early/avoid overwriting if we only want first match (preserves original behavior mostly)
        // Original behavior: `captures` finds first match.
        let mut found_sql = false;
        let mut found_params = false;

        for line in lines {
            if !found_sql {
                 if let Some(ref regex) = sql_regex {
                    if let Some(caps) = regex.captures(&line) {
                        if let Some(sql_match) = caps.get(1) {
                            result.sql = sql_match.as_str().trim().to_string();
                            found_sql = true;
                        }
                    }
                }
            }

            if !found_params {
                if let Some(ref regex) = params_regex {
                    if let Some(caps) = regex.captures(&line) {
                        if let Some(params_match) = caps.get(1) {
                            result.params = self.parse_params_string(params_match.as_str());
                            found_params = true;
                        }
                    }
                }
            }

            if found_sql && found_params {
                break;
            }
        }

        result
    }

    /// Parse log file with advanced metadata extraction.
    /// capturing all executions for a specific ID.
    pub fn parse_log_file_advanced(&self, log_file_path: &str, target_id: &str) -> Vec<Execution> {
        let mut executions = Vec::new();

        if !file_helper::file_exists(log_file_path) {
            return executions;
        }

        let content = match encoding::read_file_as_utf8(log_file_path, &self.encoding) {
            Ok(c) => c,
            Err(_) => return executions,
        };

        if content.is_empty() {
            return executions;
        }

        let lines: Vec<&str> = content.lines().collect();
        
        // State for parsing
        let mut current_sql = String::new();
        let mut current_timestamp = String::new();
        let mut current_dao = String::new();
        let mut execution_count = 0;

        // Patterns
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
            // 1. Check for SQL (Full or Simple)
            let mut found_new_sql = false;
            let mut extracted_sql = String::new();
            let mut extracted_ts = String::new();
            let mut extracted_dao = String::new();

            if let Some(ref regex) = full_line_regex {
                if let Some(caps) = regex.captures(line) {
                    extracted_ts = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                    extracted_sql = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();
                    extracted_dao = self.find_dao_class_name(&lines, i);
                    found_new_sql = true;
                }
            }

            if !found_new_sql {
                if let Some(ref regex) = simple_sql_regex {
                    if let Some(caps) = regex.captures(line) {
                        extracted_sql = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                        
                        // Try to find timestamp in this line or previous
                        if let Some(ts_caps) = TIMESTAMP_REGEX.captures(line) {
                            extracted_ts = ts_caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                        } else if i > 0 {
                            if let Some(ts_caps) = TIMESTAMP_REGEX.captures(lines[i - 1]) {
                                extracted_ts = ts_caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                            }
                        }
                        
                        extracted_dao = self.find_dao_class_name(&lines, i);
                        found_new_sql = true;
                    }
                }
            }

            // Update state if new SQL found
            if found_new_sql {
                 current_sql = extracted_sql;
                 current_timestamp = extracted_ts;
                 current_dao = extracted_dao;
                 continue; 
            }

            // 2. Check for Params
            if let Some(ref regex) = params_regex {
                if let Some(caps) = regex.captures(line) {
                    // Only process params if we have a current SQL context
                    if !current_sql.is_empty() {
                         let params_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                         let params = self.parse_params_string(params_str);
                         
                         // Determine timestamp for this execution
                         // Use line timestamp if available, otherwise fallback to SQL timestamp
                         let ts = TIMESTAMP_REGEX.captures(line)
                            .and_then(|c| c.get(1))
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_else(|| current_timestamp.clone());
                            
                         execution_count += 1;

                         let filled_sql = sql_formatter::replace_placeholders(&current_sql, &params)
                             .unwrap_or_else(|_| current_sql.clone());

                         executions.push(Execution {
                             id: target_id.to_string(),
                             timestamp: ts,
                             dao_file: current_dao.clone(),
                             sql: current_sql.clone(),
                             filled_sql,
                             params,
                             execution_index: execution_count,
                         });
                    }
                }
            }
        }
        
        // Edge case: SQL found but NO params found at all?
        // Logic above only adds execution if params found.
        // If query has no params, we might miss it?
        // If `execution_count` is 0 but `current_sql` is set, check if it has no placeholders?
        // But logging usually logs "params=[]" if empty?
        // "id=... sql=..."
        // "id=... params=[]"
        // If params line is missing entirely, we might want to add it as "Execution without params"?
        // But standard behavior seems to be SQL then Params. 
        // Let's handle the case where we finish parsing and have 0 executions but `current_sql` exists.
        
        if executions.is_empty() && !current_sql.is_empty() {
             executions.push(Execution {
                 id: target_id.to_string(),
                 timestamp: current_timestamp,
                 dao_file: current_dao,
                 sql: current_sql.clone(),
                 filled_sql: current_sql,
                 params: Vec::new(),
                 execution_index: 1,
             });
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

        let lines = match read_file_lines(log_file_path, &self.encoding) {
            Ok(iter) => iter,
            Err(_) => return ids,
        };

        for line in lines {
            // Check for ID + SQL
            if let Some(caps) = ID_SQL_REGEX.captures(&line) {
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

            // Check for ID + Params
            if let Some(caps) = ID_PARAMS_REGEX.captures(&line) {
                 if let Some(id_match) = caps.get(1) {
                    let id = id_match.as_str();
                    // Increment params count for existing ID
                    // Optimization: We could use a map for O(1) lookup, but ids vec order matters for UI?
                    // Original code preserved order of appearance.
                    for info in ids.iter_mut() {
                        if info.id == id {
                            info.params_count += 1;
                            break;
                        }
                    }
                }
            }
        }

        ids
    }

    /// Get the last SQL query from a log file.
    pub fn get_last_query(&self, log_file_path: &str) -> QueryResult {
        let mut result = QueryResult::default();

        if !file_helper::file_exists(log_file_path) {
            return result;
        }

        let lines = match read_file_lines(log_file_path, &self.encoding) {
            Ok(iter) => iter,
            Err(_) => return result,
        };

        for line in lines {
             // Match pattern: id=<hex_id> sql=<sql_statement>
            if let Some(caps) = ID_SQL_REGEX.captures(&line) {
                if let Some(id_match) = caps.get(1) {
                     // Extract SQL - everything after "sql=" until end of line
                    if let Some(sql_start) = line.find("sql=") {
                        let sql_part = &line[sql_start + 4..];
                        let sql = sql_part.trim().to_string();

                        if !sql.is_empty() {
                            result.id = id_match.as_str().to_string();
                            result.sql = sql;
                            result.params.clear(); // Reset params for new query
                        }
                    }
                }
                continue; // Line had sql, so it won't have params typically
            }

            // Match params: id=<hex_id> params=[...]
            // Only update params if ID matches the last found SQL ID
            if !result.id.is_empty() {
                if let Some(caps) = ID_PARAMS_REGEX.captures(&line) {
                     if let Some(id_match) = caps.get(1) {
                         if id_match.as_str() == result.id {
                             if let Some(params_match) = caps.get(1) {
                                // Re-parse params to get just the bracketed part if regex captured full group?
                                // ID_PARAMS_REGEX captures id in group 1.
                                // We need params value.
                                // Let's check regex in file... `id=([a-f0-9]+)\s+params=`
                                // It doesn't capture value in current static regex!
                                // Wait, `get_last_query` original implementation used a local `params_pattern` regex format!
                                // The global `ID_PARAMS_REGEX` doesn't capture the array.
                                // We need to extract it manually or use a new regex.
                                
                                // Let's simplify: find "params=" and take rest?
                                if let Some(params_start) = line.find("params=") {
                                    let params_part = &line[params_start + 7..];
                                    // Parse array from string...
                                    // Or use `parse_params_string` on it? `parse_params_string` uses `PARAM_REGEX` which finds `[...]`.
                                    result.params = self.parse_params_string(params_part);
                                }
                             }
                         }
                     }
                }
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
        Self::new("SHIFT_JIS".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_params_string() {
        let parser = LogParser::default();
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
        let parser = LogParser::default();
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
        let parser = LogParser::default();
        let result = parser.get_last_query(&path);

        assert_eq!(result.id, "abc123");
        assert_eq!(result.sql, "SELECT 1");

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_get_last_query_empty_file() {
        let path = create_temp_file("");
        let parser = LogParser::default();
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
        let parser = LogParser::default();
        let result = parser.get_last_query(&path);

        assert!(!result.found());

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_get_last_query_no_sql() {
        let content = r#"2024/01/01 10:00:00,INFO,Test,Some random log line
2024/01/01 10:00:01,INFO,Test,Another log line without SQL"#;

        let path = create_temp_file(content);
        let parser = LogParser::default();
        let result = parser.get_last_query(&path);

        assert!(!result.found());

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_get_last_query_file_not_found() {
        let parser = LogParser::default();
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
        let parser = LogParser::default();
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
        let parser = LogParser::default();
        let result = parser.get_last_query(&path);

        assert_eq!(result.id, "abc123");
        assert_eq!(result.sql, "SELECT * FROM users WHERE name LIKE '%test%' AND status IN (1, 2, 3)");

        cleanup_temp_file(&path);
    }
}
