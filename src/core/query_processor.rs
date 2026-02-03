//! Query processor that orchestrates parsing, formatting, and clipboard operations.
//!
//! Combines LogParser, SqlFormatter, and ClipboardHelper to process queries.

use crate::core::log_parser::{LogParser, QueryResult};
use crate::core::sql_formatter;
use crate::utils::clipboard;

/// Result of processing a query.
#[derive(Debug, Clone, Default)]
pub struct ProcessResult {
    pub query: QueryResult,
    pub filled_sql: String,
    pub formatted_sql: String,
    pub formatted_params: String,
    pub copied_to_clipboard: bool,
    pub error: Option<String>,
}

impl ProcessResult {
    pub fn success(&self) -> bool {
        self.error.is_none() && self.query.found()
    }
}

/// Processor that combines log parsing with formatting and clipboard operations.
pub struct QueryProcessor {
    parser: LogParser,
}

impl QueryProcessor {
    pub fn new() -> Self {
        Self {
            parser: LogParser::new(),
        }
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

        result.query = self.parser.parse_log_file(log_file_path, target_id);

        if !result.query.found() {
            result.error = Some(format!("ID not found: {}", target_id));
            return result;
        }

        result.formatted_sql = sql_formatter::format_sql(&result.query.sql);
        result.formatted_params = sql_formatter::format_params(&result.query.params);
        result.filled_sql = self.get_filled_query(&result.query);

        if auto_copy && !result.filled_sql.is_empty() {
            result.copied_to_clipboard = clipboard::copy_to_clipboard(&result.filled_sql);
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
