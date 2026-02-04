//! Query processor that orchestrates parsing, formatting, and clipboard operations.
//!
//! Combines LogParser, SqlFormatter, and ClipboardHelper to process queries.

use crate::core::log_parser::{Execution, LogParser, QueryResult};
use crate::core::sql_formatter;
use crate::utils::clipboard;
use serde::Serialize;

/// Group of executions sharing the same SQL template.
#[derive(Debug, Clone, Default, Serialize)]
pub struct QueryGroup {
    pub template_sql: String,
    pub formatted_template_sql: String,
    pub executions: Vec<Execution>,
    #[serde(skip)]
    pub is_expanded: bool,
    #[serde(skip)]
    pub is_template_expanded: bool,
}

/// Result of processing a query.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ProcessResult {
    /// Single/Legacy query result (e.g. first found)
    pub query: QueryResult,
    /// All executions found for the ID
    pub executions: Vec<Execution>,
    /// Grouped executions
    pub groups: Vec<QueryGroup>,
    
    pub filled_sql: String,
    pub formatted_sql: String,
    pub formatted_params: String,
    pub copied_to_clipboard: bool,
    pub error: Option<String>,
}

impl ProcessResult {
    pub fn success(&self) -> bool {
        self.error.is_none() && (!self.query.sql.is_empty() || !self.executions.is_empty())
    }
}

/// Processor that combines log parsing with formatting and clipboard operations.
pub struct QueryProcessor {
    parser: LogParser,
}

impl QueryProcessor {
    pub fn new() -> Self {
        Self {
            parser: LogParser::default(),
        }
    }

    pub fn parser_mut(&mut self) -> &mut LogParser {
        &mut self.parser
    }

    /// Process a query by ID.
    ///
    /// Parses the log file, formats the SQL and params, optionally copies to clipboard.
    pub fn process_query(
        &self,
        target_id: &str,
        log_file_path: &str,
        auto_copy: bool,
    ) -> ProcessResult {
        let mut result = ProcessResult::default();

        // Use advanced parsing to get all executions
        result.executions = self.parser.parse_log_file_advanced(log_file_path, target_id);

        if result.executions.is_empty() {
             result.error = Some(format!("ID not found: {}", target_id));
             return result;
        }

        // Grouping Logic
        // We preserve order of appearance of templates.
        for exec in &result.executions {
            let mut group_found = false;
            for group in &mut result.groups {
                if group.template_sql == exec.sql {
                    group.executions.push(exec.clone());
                    group_found = true;
                    break;
                }
            }
            
            if !group_found {
                 result.groups.push(QueryGroup {
                     template_sql: exec.sql.clone(),
                     formatted_template_sql: sql_formatter::format_sql(&exec.sql),
                     executions: vec![exec.clone()],
                     is_expanded: false,
                     is_template_expanded: false,
                 });
            }
        }

        // To maintain backward compatibility with UI parts using `result.query`:
        // Populate single `query` field from the LAST execution (most likely what user wants if single view).
        if let Some(last_exec) = result.executions.last() {
             result.query = QueryResult {
                 id: last_exec.id.clone(),
                 sql: last_exec.sql.clone(),
                 params: last_exec.params.clone(),
             };
             
             result.formatted_sql = sql_formatter::format_sql(&last_exec.sql);
             result.formatted_params = sql_formatter::format_params(&last_exec.params);
             result.filled_sql = last_exec.filled_sql.clone();
             
             if auto_copy && !result.filled_sql.is_empty() {
                 result.copied_to_clipboard = clipboard::copy_to_clipboard(&result.filled_sql);
             }
        }

        result
    }

    /// Process the last query in the log file.
    pub fn process_last_query(&self, log_file_path: &str, auto_copy: bool) -> ProcessResult {
        let mut result = ProcessResult::default();

        result.query = self.parser.get_last_query(log_file_path);

        if !result.query.found() {
            result.error = Some("No SQL queries found in log file".to_string());
            return result;
        }

        result.formatted_sql = sql_formatter::format_sql(&result.query.sql);
        result.formatted_params = sql_formatter::format_params(&result.query.params);
        result.filled_sql = self.get_filled_query(&result.query);

        // Synthesize a group for UI consistency
        // Note: Execution struct requires timestamp etc which we don't have fully in QueryResult
        // But we can create a minimal one.
        let exec = Execution {
            id: result.query.id.clone(),
            sql: result.query.sql.clone(),
            params: result.query.params.clone(),
            filled_sql: result.filled_sql.clone(),
            formatted_sql: result.formatted_sql.clone(),
            execution_index: 1,
            timestamp: "Last Execution".to_string(), // Placeholder
            dao_file: "".to_string(),
            is_expanded: false,
        };
        
        result.executions.push(exec.clone());
        result.groups.push(QueryGroup {
            template_sql: exec.sql.clone(),
            formatted_template_sql: exec.formatted_sql.clone(),
            executions: vec![exec],
            is_expanded: false,
            is_template_expanded: false,
        });

        if auto_copy && !result.filled_sql.is_empty() {
            result.copied_to_clipboard = clipboard::copy_to_clipboard(&result.filled_sql);
        }

        result
    }

    /// Get the log parser for direct access.
    pub fn parser(&self) -> &LogParser {
        &self.parser
    }

    /// Fill SQL with parameter values.
    fn get_filled_query(&self, query: &QueryResult) -> String {
        if query.sql.is_empty() {
            return String::new();
        }

        if query.params.is_empty() {
            return query.sql.clone();
        }

        sql_formatter::replace_placeholders(&query.sql, &query.params)
            .unwrap_or_else(|_| query.sql.clone())
    }
}

impl Default for QueryProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_filled_query_no_params() {
        let processor = QueryProcessor::new();
        let query = QueryResult {
            id: "test".to_string(),
            sql: "SELECT 1".to_string(),
            params: Vec::new(),
        };
        assert_eq!(processor.get_filled_query(&query), "SELECT 1");
    }

    #[test]
    fn test_get_filled_query_with_params() {
        let processor = QueryProcessor::new();
        let query = QueryResult {
            id: "test".to_string(),
            sql: "SELECT * FROM t WHERE id = ?".to_string(),
            params: vec!["Int:1:42".to_string()],
        };
        assert_eq!(processor.get_filled_query(&query), "SELECT * FROM t WHERE id = 42");
    }
}
