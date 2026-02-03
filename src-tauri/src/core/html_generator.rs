//! HTML report generator for SQL executions.
//!
//! Generates styled HTML reports with syntax highlighting for SQL queries.

use crate::core::log_parser::Execution;
use chrono::Local;
use std::fs::File;
use std::io::Write;

/// Options for HTML report generation.
#[derive(Debug, Clone, Default)]
pub struct HtmlOptions {
    pub title: String,
    pub log_file: String,
}

/// HTML report generator.
pub struct HtmlGenerator;

impl HtmlGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate HTML report from executions.
    pub fn generate_report(&self, executions: &[Execution], options: &HtmlOptions) -> String {
        let title = if options.title.is_empty() {
            "SQL Report"
        } else {
            &options.title
        };

        let nav_items: String = executions
            .iter()
            .enumerate()
            .map(|(i, exec)| self.generate_nav_item(exec, i))
            .collect();

        let cards: String = executions
            .iter()
            .enumerate()
            .map(|(i, exec)| self.generate_execution_card(exec, i))
            .collect();

        let datetime = self.get_current_datetime();
        let log_file_display = if options.log_file.is_empty() {
            "N/A".to_string()
        } else {
            self.escape_html(&options.log_file)
        };

        format!(
            r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #e0e0e0;
            min-height: 100vh;
            display: flex;
        }}
        .sidebar {{
            width: 280px;
            background: rgba(0,0,0,0.3);
            padding: 20px;
            overflow-y: auto;
            position: fixed;
            height: 100vh;
            border-right: 1px solid rgba(255,255,255,0.1);
        }}
        .sidebar h2 {{
            color: #64b5f6;
            margin-bottom: 20px;
            font-size: 1.2rem;
        }}
        .nav-item {{
            padding: 12px 15px;
            margin-bottom: 8px;
            background: rgba(255,255,255,0.05);
            border-radius: 8px;
            cursor: pointer;
            transition: all 0.3s;
            border-left: 3px solid transparent;
        }}
        .nav-item:hover {{
            background: rgba(100,181,246,0.2);
            border-left-color: #64b5f6;
        }}
        .nav-item.active {{
            background: rgba(100,181,246,0.3);
            border-left-color: #64b5f6;
        }}
        .nav-item .id {{ font-weight: bold; color: #fff; }}
        .nav-item .dao {{ font-size: 0.85rem; color: #888; margin-top: 4px; }}
        .main {{
            margin-left: 280px;
            flex: 1;
            padding: 30px;
        }}
        .header {{
            margin-bottom: 30px;
            padding-bottom: 20px;
            border-bottom: 1px solid rgba(255,255,255,0.1);
        }}
        .header h1 {{
            color: #64b5f6;
            font-size: 1.8rem;
            margin-bottom: 10px;
        }}
        .header .meta {{
            color: #888;
            font-size: 0.9rem;
        }}
        .card {{
            background: rgba(255,255,255,0.05);
            border-radius: 12px;
            padding: 25px;
            margin-bottom: 25px;
            border: 1px solid rgba(255,255,255,0.1);
            display: none;
        }}
        .card.active {{ display: block; }}
        .card-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 20px;
            padding-bottom: 15px;
            border-bottom: 1px solid rgba(255,255,255,0.1);
        }}
        .card-title {{
            font-size: 1.2rem;
            color: #64b5f6;
        }}
        .card-meta {{
            color: #888;
            font-size: 0.85rem;
        }}
        .section {{
            margin-bottom: 20px;
        }}
        .section-title {{
            color: #81c784;
            font-size: 0.9rem;
            text-transform: uppercase;
            letter-spacing: 1px;
            margin-bottom: 10px;
        }}
        .sql-box {{
            background: #0d1117;
            border-radius: 8px;
            padding: 20px;
            overflow-x: auto;
            font-family: 'Consolas', 'Monaco', monospace;
            font-size: 0.9rem;
            line-height: 1.6;
            border: 1px solid rgba(255,255,255,0.1);
        }}
        .copy-btn {{
            background: #64b5f6;
            color: #000;
            border: none;
            padding: 8px 16px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 0.85rem;
            transition: all 0.3s;
        }}
        .copy-btn:hover {{
            background: #90caf9;
        }}
        .copy-btn.copied {{
            background: #81c784;
        }}
        .keyword {{ color: #569cd6; font-weight: bold; }}
        .string {{ color: #ce9178; }}
        .number {{ color: #b5cea8; }}
        .params-list {{
            background: rgba(0,0,0,0.2);
            border-radius: 8px;
            padding: 15px;
        }}
        .param-item {{
            padding: 8px 12px;
            background: rgba(255,255,255,0.05);
            border-radius: 4px;
            margin-bottom: 6px;
            font-family: monospace;
        }}
        .param-index {{ color: #64b5f6; }}
        .param-type {{ color: #81c784; }}
        .param-value {{ color: #fff; }}
    </style>
</head>
<body>
    <nav class="sidebar">
        <h2>📋 Executions</h2>
        {nav_items}
    </nav>
    <main class="main">
        <div class="header">
            <h1>🔍 {title}</h1>
            <div class="meta">
                <div>Generated: {datetime}</div>
                <div>Log File: {log_file_display}</div>
                <div>Total Executions: {count}</div>
            </div>
        </div>
        {cards}
    </main>
    <script>
        function showCard(index) {{
            document.querySelectorAll('.card').forEach(c => c.classList.remove('active'));
            document.querySelectorAll('.nav-item').forEach(n => n.classList.remove('active'));
            document.getElementById('card-' + index).classList.add('active');
            document.getElementById('nav-' + index).classList.add('active');
        }}
        function copyToClipboard(id, btn) {{
            const text = document.getElementById(id).innerText;
            navigator.clipboard.writeText(text).then(() => {{
                btn.classList.add('copied');
                btn.innerText = '✓ Copied!';
                setTimeout(() => {{
                    btn.classList.remove('copied');
                    btn.innerText = '📋 Copy';
                }}, 2000);
            }});
        }}
        // Show first card by default
        if (document.querySelector('.nav-item')) {{
            showCard(0);
        }}
    </script>
</body>
</html>"#,
            title = self.escape_html(title),
            nav_items = nav_items,
            cards = cards,
            datetime = datetime,
            log_file_display = log_file_display,
            count = executions.len(),
        )
    }

    /// Save HTML report to a file.
    pub fn save_report(&self, html: &str, output_path: &str) -> std::io::Result<()> {
        let mut file = File::create(output_path)?;
        file.write_all(html.as_bytes())
    }

    fn generate_nav_item(&self, exec: &Execution, index: usize) -> String {
        let short_dao = self.get_short_dao_name(&exec.dao_file);
        format!(
            r#"<div class="nav-item" id="nav-{index}" onclick="showCard({index})">
    <div class="id">#{} - {}</div>
    <div class="dao">{}</div>
</div>"#,
            exec.execution_index,
            self.escape_html(&exec.id[..std::cmp::min(8, exec.id.len())]),
            self.escape_html(&short_dao),
            index = index,
        )
    }

    fn generate_execution_card(&self, exec: &Execution, index: usize) -> String {
        let highlighted_sql = self.highlight_sql(&exec.filled_sql);
        let params_html = self.format_params_html(&exec.params);

        format!(
            r#"<div class="card" id="card-{index}">
    <div class="card-header">
        <div>
            <div class="card-title">Execution #{}</div>
            <div class="card-meta">ID: {} | DAO: {}</div>
        </div>
        <button class="copy-btn" onclick="copyToClipboard('sql-{index}', this)">📋 Copy</button>
    </div>
    <div class="section">
        <div class="section-title">Filled SQL</div>
        <div class="sql-box" id="sql-{index}">{}</div>
    </div>
    <div class="section">
        <div class="section-title">Parameters ({})</div>
        <div class="params-list">{}</div>
    </div>
    <div class="section">
        <div class="section-title">Timestamp</div>
        <div class="card-meta">{}</div>
    </div>
</div>"#,
            exec.execution_index,
            self.escape_html(&exec.id),
            self.escape_html(&exec.dao_file),
            highlighted_sql,
            exec.params.len(),
            params_html,
            self.escape_html(&exec.timestamp),
            index = index,
        )
    }

    fn highlight_sql(&self, sql: &str) -> String {
        let keywords = [
            "SELECT", "FROM", "WHERE", "AND", "OR", "INSERT", "UPDATE", "DELETE",
            "INTO", "VALUES", "SET", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER",
            "ON", "AS", "ORDER", "BY", "GROUP", "HAVING", "LIMIT", "OFFSET",
            "CREATE", "ALTER", "DROP", "TABLE", "INDEX", "NULL", "NOT", "IN",
            "BETWEEN", "LIKE", "IS", "EXISTS", "CASE", "WHEN", "THEN", "ELSE", "END",
        ];

        let mut result = self.escape_html(sql);

        // Highlight keywords (case-insensitive)
        for keyword in keywords {
            let pattern = regex::Regex::new(&format!(r"(?i)\b{}\b", keyword)).unwrap();
            result = pattern
                .replace_all(&result, |caps: &regex::Captures| {
                    format!(r#"<span class="keyword">{}</span>"#, &caps[0])
                })
                .to_string();
        }

        // Highlight strings
        let string_pattern = regex::Regex::new(r"'[^']*'").unwrap();
        result = string_pattern
            .replace_all(&result, |caps: &regex::Captures| {
                format!(r#"<span class="string">{}</span>"#, &caps[0])
            })
            .to_string();

        // Highlight numbers
        let number_pattern = regex::Regex::new(r"\b\d+\.?\d*\b").unwrap();
        result = number_pattern
            .replace_all(&result, |caps: &regex::Captures| {
                format!(r#"<span class="number">{}</span>"#, &caps[0])
            })
            .to_string();

        result
    }

    fn format_params_html(&self, params: &[String]) -> String {
        if params.is_empty() {
            return "<div class=\"param-item\">No parameters</div>".to_string();
        }

        params
            .iter()
            .map(|param| {
                let parts: Vec<&str> = param.splitn(3, ':').collect();
                if parts.len() == 3 {
                    format!(
                        r#"<div class="param-item"><span class="param-index">[{}]</span> <span class="param-type">{}</span>: <span class="param-value">{}</span></div>"#,
                        self.escape_html(parts[1]),
                        self.escape_html(parts[0]),
                        self.escape_html(parts[2]),
                    )
                } else {
                    format!(
                        r#"<div class="param-item">{}</div>"#,
                        self.escape_html(param)
                    )
                }
            })
            .collect()
    }

    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    fn get_current_datetime(&self) -> String {
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }

    fn get_short_dao_name(&self, dao_file: &str) -> String {
        // Extract just the class name from full path
        dao_file
            .rsplit('.')
            .next()
            .unwrap_or(dao_file)
            .to_string()
    }
}

impl Default for HtmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        let gen = HtmlGenerator::new();
        assert_eq!(gen.escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(gen.escape_html("a & b"), "a &amp; b");
    }

    #[test]
    fn test_generate_empty_report() {
        let gen = HtmlGenerator::new();
        let html = gen.generate_report(&[], &HtmlOptions::default());
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Total Executions: 0"));
    }
}
