//! SQL Server connectivity using ODBC.
//!
//! Provides connection management and query execution for SQL Server databases.
//! Uses the `odbc-api` crate for ODBC connectivity.

use odbc_api::{
    buffers::TextRowSet, Connection, ConnectionOptions, Cursor, Environment, ResultSetMetadata,
};
use std::sync::OnceLock;
use serde::Serialize;

/// Global ODBC environment (initialized once).
static ODBC_ENV: OnceLock<Environment> = OnceLock::new();

/// Get or initialize the global ODBC environment.
fn get_environment() -> Result<&'static Environment, String> {
    // Try to get existing environment
    if let Some(env) = ODBC_ENV.get() {
        return Ok(env);
    }
    
    // Create new environment
    match Environment::new() {
        Ok(env) => {
            // Try to set it, if another thread beat us, use theirs
            let _ = ODBC_ENV.set(env);
            Ok(ODBC_ENV.get().unwrap())
        }
        Err(e) => Err(format!("Failed to create ODBC environment: {}", e)),
    }
}

/// SQL column metadata.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SqlColumn {
    pub name: String,
    pub sql_type: i16,
    pub size: usize,
}

/// Result of a SQL query execution.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SqlResult {
    pub success: bool,
    pub error: Option<String>,
    pub columns: Vec<SqlColumn>,
    pub rows: Vec<Vec<String>>,
    pub rows_affected: i32,
}

impl SqlResult {
    /// Create a successful result with no data.
    pub fn success() -> Self {
        Self {
            success: true,
            ..Default::default()
        }
    }

    /// Create an error result.
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            error: Some(msg.into()),
            ..Default::default()
        }
    }
}

/// ODBC drivers to try, in order of preference.
const DRIVERS: &[&str] = &[
    "ODBC Driver 18 for SQL Server",
    "ODBC Driver 17 for SQL Server",
    "SQL Server Native Client 11.0",
    "SQL Server",
];

/// SQL Server connector using ODBC.
pub struct SqlConnector {
    connection: Option<Connection<'static>>,
    last_error: String,
}

impl SqlConnector {
    /// Create a new SQL connector.
    pub fn new() -> Self {
        Self {
            connection: None,
            last_error: String::new(),
        }
    }

    /// Connect to a SQL Server database.
    ///
    /// Tries multiple ODBC drivers until one succeeds.
    pub fn connect(
        &mut self,
        server: &str,
        database: &str,
        username: &str,
        password: &str,
        use_windows_auth: bool,
    ) -> bool {
        // Disconnect any existing connection
        self.disconnect();

        let env = match get_environment() {
            Ok(e) => e,
            Err(e) => {
                self.last_error = e;
                return false;
            }
        };

        // Try each driver
        for driver in DRIVERS {
            let conn_str = if use_windows_auth {
                format!(
                    "DRIVER={{{}}};SERVER={};DATABASE={};Trusted_Connection=yes;TrustServerCertificate=yes;",
                    driver, server, database
                )
            } else {
                format!(
                    "DRIVER={{{}}};SERVER={};DATABASE={};UID={};PWD={};TrustServerCertificate=yes;",
                    driver, server, database, username, password
                )
            };

            match env.connect_with_connection_string(&conn_str, ConnectionOptions::default()) {
                Ok(conn) => {
                    self.connection = Some(conn);
                    self.last_error.clear();
                    return true;
                }
                Err(e) => {
                    self.last_error = format!("Driver '{}' failed: {}", driver, e);
                    // Continue trying other drivers
                }
            }
        }

        // All drivers failed
        if self.last_error.is_empty() {
            self.last_error = "No suitable ODBC driver found".to_string();
        }
        false
    }

    /// Disconnect from the database.
    pub fn disconnect(&mut self) {
        self.connection = None;
    }

    /// Check if connected to a database.
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    /// Get the last error message.
    pub fn get_last_error(&self) -> &str {
        &self.last_error
    }

    /// Execute a SQL query and return results.
    pub fn execute_query(&mut self, sql: &str) -> SqlResult {
        let conn = match &self.connection {
            Some(c) => c,
            None => return SqlResult::error("Not connected to database"),
        };

        // Execute the query
        let mut cursor = match conn.execute(sql, ()) {
            Ok(Some(cursor)) => cursor,
            Ok(None) => {
                // Non-SELECT statement (INSERT, UPDATE, DELETE, etc.)
                return SqlResult::success();
            }
            Err(e) => {
                self.last_error = e.to_string();
                return SqlResult::error(&self.last_error);
            }
        };

        // Get column information
        let num_cols = cursor.num_result_cols().unwrap_or(0) as usize;
        let mut columns = Vec::with_capacity(num_cols);

        for i in 1..=num_cols as u16 {
            let name = cursor
                .col_name(i)
                .unwrap_or_else(|_| format!("Column{}", i));
            let data_type = cursor.col_data_type(i).ok();
            
            // Get column size, converting NonZero to usize
            let size = data_type
                .as_ref()
                .and_then(|dt| dt.column_size())
                .map(|nz| nz.get())
                .unwrap_or(0);
            
            columns.push(SqlColumn {
                name,
                sql_type: 0, // Simplified - just use 0 for now
                size,
            });
        }

        // Fetch rows using a text buffer
        let batch_size = 1000;
        let max_str_len = 8192;
        
        let mut result = SqlResult {
            success: true,
            error: None,
            columns,
            rows: Vec::new(),
            rows_affected: 0,
        };

        // Create buffer for fetching rows
        let buffer_desc = (1..=num_cols as u16)
            .map(|_| odbc_api::buffers::BufferDesc::Text { max_str_len })
            .collect::<Vec<_>>();

        let buffer = TextRowSet::from_max_str_lens(batch_size, buffer_desc.iter().map(|_| max_str_len)).unwrap();
        let mut row_set_cursor = cursor.bind_buffer(buffer).unwrap();

        // Fetch all rows
        while let Some(batch) = row_set_cursor.fetch().ok().flatten() {
            for row_idx in 0..batch.num_rows() {
                let mut row = Vec::with_capacity(num_cols);
                for col_idx in 0..num_cols {
                    let value = batch
                        .at(col_idx, row_idx)
                        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
                        .unwrap_or_else(|| "NULL".to_string());
                    row.push(value);
                }
                result.rows.push(row);
            }
        }

        result.rows_affected = result.rows.len() as i32;
        result
    }

    /// Convert a SQL result to CSV format.
    pub fn result_to_csv(result: &SqlResult, separator: &str) -> String {
        let mut output = String::new();

        // Header row
        let header: Vec<String> = result
            .columns
            .iter()
            .map(|col| escape_csv_field(&col.name, separator))
            .collect();
        output.push_str(&header.join(separator));
        output.push('\n');

        // Data rows
        for row in &result.rows {
            let escaped: Vec<String> = row
                .iter()
                .map(|val| escape_csv_field(val, separator))
                .collect();
            output.push_str(&escaped.join(separator));
            output.push('\n');
        }

        output
    }
}

impl Default for SqlConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SqlConnector {
    fn drop(&mut self) {
        self.disconnect();
    }
}

/// Escape a CSV field if it contains special characters.
fn escape_csv_field(value: &str, separator: &str) -> String {
    let needs_quotes = value.contains(separator) || value.contains('"') || value.contains('\n');

    if needs_quotes {
        let escaped = value.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_connector_new() {
        let connector = SqlConnector::new();
        assert!(!connector.is_connected());
    }

    #[test]
    fn test_escape_csv_field() {
        assert_eq!(escape_csv_field("hello", ","), "hello");
        assert_eq!(escape_csv_field("hello,world", ","), "\"hello,world\"");
        assert_eq!(escape_csv_field("say \"hi\"", ","), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_result_to_csv() {
        let result = SqlResult {
            success: true,
            error: None,
            columns: vec![
                SqlColumn { name: "ID".to_string(), sql_type: 0, size: 0 },
                SqlColumn { name: "Name".to_string(), sql_type: 0, size: 0 },
            ],
            rows: vec![
                vec!["1".to_string(), "Alice".to_string()],
                vec!["2".to_string(), "Bob".to_string()],
            ],
            rows_affected: 2,
        };

        let csv = SqlConnector::result_to_csv(&result, ",");
        assert!(csv.contains("ID,Name"));
        assert!(csv.contains("1,Alice"));
        assert!(csv.contains("2,Bob"));
    }
}
