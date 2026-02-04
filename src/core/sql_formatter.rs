//! SQL formatting utilities.
//!
//! Provides functions to format SQL queries with line breaks at keywords,
//! format parameter lists, and replace placeholders with actual values.

use std::collections::HashMap;

/// Format SQL with line breaks at major keywords for readability.
pub fn format_sql(sql: &str) -> String {
    if sql.is_empty() {
        return "Not found".to_string();
    }

    let keywords = ["SELECT", "FROM", "WHERE", "AND", "OR", "ORDER BY", "GROUP BY"];
    let mut formatted = sql.to_string();

    for keyword in keywords {
        let pattern = format!(" {} ", keyword);
        let replacement = format!("\n{} ", keyword);

        // Case-insensitive replacement
        let upper_formatted = formatted.to_uppercase();
        let mut result = String::new();
        let mut last_end = 0;

        for (start, _) in upper_formatted.match_indices(&pattern) {
            result.push_str(&formatted[last_end..start]);
            result.push_str(&replacement);
            last_end = start + pattern.len();
        }
        result.push_str(&formatted[last_end..]);
        formatted = result;
    }

    formatted.trim().to_string()
}

/// Format a parameter list for display.
///
/// Parses parameters in the format `TYPE:INDEX:VALUE` and formats them nicely.
pub fn format_params(params: &[String]) -> String {
    if params.is_empty() {
        return "Not found".to_string();
    }

    let mut result = String::new();

    for param in params {
        // Parse format: TYPE:INDEX:VALUE
        let parts: Vec<&str> = param.splitn(3, ':').collect();
        
        if parts.len() == 3 {
            let param_type = parts[0];
            let param_index = parts[1];
            let param_value = parts[2];
            result.push_str(&format!("  [{}] {}: {}\n", param_index, param_type, param_value));
        } else {
            result.push_str(&format!("  {}\n", param));
        }
    }

    result
}

/// Replace `?` placeholders in SQL with actual parameter values.
///
/// Parameters must be in the format `TYPE:INDEX:VALUE`.
/// String values are quoted and escaped, numeric values are used as-is.
pub fn replace_placeholders(query: &str, params: &[String]) -> Result<String, String> {
    // Build map of position -> value
    let mut values_by_pos: HashMap<i32, String> = HashMap::new();

    for param in params {
        let parts: Vec<&str> = param.splitn(3, ':').collect();
        
        if parts.len() != 3 {
            continue;
        }

        let param_type = parts[0].to_lowercase();
        let pos: i32 = parts[1].parse().map_err(|_| "Invalid parameter index")?;
        let value = parts[2];

        // Handle NULL explicitly
        if value == "null" {
            values_by_pos.insert(pos, "NULL".to_string());
            continue;
        }

        let parsed_value = match param_type.as_str() {
            "string" | "timestamp" | "date" => {
                // Escape single quotes for SQL
                let escaped = value.replace('\'', "''");
                format!("'{}'", escaped)
            }
            "boolean" => {
                 value.to_uppercase()
            }
            "bigdecimal" | "number" | "int" | "long" | "float" | "double" => {
                value.to_string()
            }
            _ => {
                // Default fallback: treat as string (safer) or error? 
                // Let's treat unknown as string to be safe against new types, or allow raw value?
                // User requirement: "Handling different parameter types correctly".
                // If it's unknown, maybe it's safest to quote it if it looks like a string?
                // Let's err on side of quoting unless it looks numeric.
                if value.chars().all(|c| c.is_numeric() || c == '.') {
                     value.to_string()
                } else {
                    let escaped = value.replace('\'', "''");
                    format!("'{}'", escaped)
                }
            }
        };

        values_by_pos.insert(pos, parsed_value);
    }

    // Replace ? placeholders in order
    let mut result = String::new();
    let mut index = 1;

    for ch in query.chars() {
        if ch == '?' {
            match values_by_pos.get(&index) {
                Some(value) => result.push_str(value),
                None => {
                    // If param missing, keep ? to indicate issue/allow debugging
                    result.push('?'); 
                    // Or return Err? User wants "debuggability". 
                    // Keeping ? with a warning might be better than failing hard?
                    // Previous logic returned Err. Let's stick to Err for strictness.
                    return Err(format!("Missing value for position {}", index));
                },
            }
            index += 1;
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_sql() {
        let sql = "SELECT * FROM users WHERE id = 1 AND active = true";
        let formatted = format_sql(sql);
        assert!(formatted.contains("\nWHERE"));
        assert!(formatted.contains("\nAND"));
    }

    #[test]
    fn test_format_params() {
        let params = vec![
            "String:1:hello".to_string(),
            "Int:2:42".to_string(),
        ];
        let formatted = format_params(&params);
        assert!(formatted.contains("[1] String: hello"));
        assert!(formatted.contains("[2] Int: 42"));
    }

    #[test]
    fn test_replace_placeholders() {
        let query = "SELECT * FROM users WHERE name = ? AND id = ?";
        let params = vec![
            "String:1:John".to_string(),
            "Int:2:42".to_string(),
        ];
        let result = replace_placeholders(query, &params).unwrap();
        assert_eq!(result, "SELECT * FROM users WHERE name = 'John' AND id = 42");
    }

    #[test]
    fn test_replace_placeholders_types() {
        let query = "SELECT * FROM t WHERE a = ? AND b = ? AND c = ?";
        let params = vec![
            "String:1:null".to_string(),
            "Timestamp:2:2024-01-01 10:00:00".to_string(),
            "Boolean:3:true".to_string(),
        ];
        let result = replace_placeholders(query, &params).unwrap();
        assert_eq!(result, "SELECT * FROM t WHERE a = NULL AND b = '2024-01-01 10:00:00' AND c = TRUE");
    }

    #[test]
    fn test_escape_quotes() {
        let query = "INSERT INTO t (name) VALUES (?)";
        let params = vec!["String:1:O'Brien".to_string()];
        let result = replace_placeholders(query, &params).unwrap();
        assert_eq!(result, "INSERT INTO t (name) VALUES ('O''Brien')");
    }
}
