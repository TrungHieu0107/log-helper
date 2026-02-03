//! Log file parser for extracting SQL queries and parameters.
//!
//! Parses log files to extract SQL statements, parameters, and execution metadata
//! based on unique transaction IDs.

use once_cell::sync::Lazy;
use regex::Regex;
use crate::utils::encoding;
use crate::utils::file_helper;
use super::sql_formatter;

/// Result of parsing a single query from the log.
#[derive(Debug, Clone, Default)]
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
#[derive(Debug, Clone, Default)]
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
#[derive(Debug, Clone, Default)]
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
        let mut sql_line_index: Option<usize> = None;
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
                    sql_line_index = Some(i);
                    dao_file = self.find_dao_class_name(&lines, i);
                    continue;
                }
            }

            // Fallback: simple SQL pattern
            if sql.is_empty() {
                if let Some(ref regex) = simple_sql_regex {
                    if let Some(caps) = regex.captures(line) {
                        sql = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                        sql_line_index = Some(i);

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
    pub fn get_last_query(&self, log_file_path: &str) -> QueryResult {
        let mut result = QueryResult::default();

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

        // Find all SQL statements and take the last one
        let sql_regex = Regex::new(r"id=([^\s]+)\s+sql=\s*(.+?)(?=\n|id=|$)").ok();
        
        if let Some(regex) = sql_regex {
            for caps in regex.captures_iter(&content) {
                result.id = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                result.sql = caps.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
            }
        }

        // Find params for the last ID
        if !result.id.is_empty() {
            let params_pattern = format!(r"id={}\s+params=(\[[^\n]+)", regex::escape(&result.id));
            if let Ok(params_regex) = Regex::new(&params_pattern) {
                if let Some(caps) = params_regex.captures(&content) {
                    if let Some(params_match) = caps.get(1) {
                        result.params = self.parse_params_string(params_match.as_str());
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
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
