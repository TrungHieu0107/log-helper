# SQL Log Parser - Rust Version

A desktop application for parsing SQL log files, extracting queries with parameters, and executing them against databases. Built with Rust and egui.

## Project Overview

This application parses log files containing SQL queries and their parameters (commonly from Java/DAO-based applications), fills in placeholder values, and allows direct execution against SQL Server databases.

### Key Features

- **Log Parsing**: Parse log files to extract SQL queries by unique transaction IDs
- **Parameter Replacement**: Automatically fill `?` placeholders with logged parameter values
- **SQL Formatting**: Pretty-print SQL queries for readability
- **SQL Executor**: Execute queries directly against SQL Server (with support for Postgres, MySQL, SQLite)
- **Connection Management**: Save and manage multiple database connections
- **Encoding Support**: SHIFT_JIS, UTF-8, UTF-16, EUC-JP encodings
- **Dracula Theme**: Dark UI with Dracula color scheme

## Architecture

```
src/
├── main.rs              # Entry point, window setup, Dracula theme, CJK fonts
├── app.rs               # Main application state and UI (SqlLogParserApp)
├── config/
│   └── mod.rs           # Configuration management (Config, ConfigManager)
├── core/
│   ├── mod.rs           # Module exports
│   ├── log_parser.rs    # Log file parsing (LogParser, IdInfo, Execution)
│   ├── query_processor.rs # Query orchestration (QueryProcessor, ProcessResult)
│   ├── sql_formatter.rs # SQL formatting and placeholder replacement
│   └── db.rs            # Database connectivity (DbClient, ConnectionManager)
└── utils/
    ├── mod.rs           # Module exports
    ├── file_helper.rs   # File system utilities
    ├── encoding.rs      # SHIFT-JIS and other encoding support
    └── clipboard.rs     # Cross-platform clipboard (arboard)
```

## Core Components

### 1. SqlLogParserApp (`src/app.rs`)
Main application struct implementing `eframe::App`. Contains:
- **Two tabs**: Log Parser and SQL Executor
- **State management**: config, IDs list, search results, database connections
- **Async runtime**: Tokio runtime for database operations
- **MPSC channels**: For async query results and connection testing

### 2. LogParser (`src/core/log_parser.rs`)
Parses log files with patterns:
- `id=<hex_id> sql=<query>` - SQL statement
- `id=<hex_id> params=[type:index:value][...]` - Parameters
- DAO class extraction from `Daoの終了jp.co...` patterns

Key methods:
- `parse_log_file()` - Simple parsing for single ID
- `parse_log_file_advanced()` - Full execution extraction with timestamps
- `get_all_ids()` - List all unique IDs in log
- `get_last_query()` - Get the most recent SQL execution

### 3. QueryProcessor (`src/core/query_processor.rs`)
Orchestrates parsing and formatting:
- Groups executions by SQL template
- Fills placeholders with parameter values
- Auto-copies to clipboard if enabled

### 4. DbClient (`src/core/db.rs`)
Database connectivity using:
- **tiberius**: Native SQL Server TDS protocol
- **sqlx**: For Postgres, MySQL, SQLite

Supports JDBC-style connection strings:
```
jdbc:sqlserver://host:port;databaseName=DB;encrypt=true;trustServerCertificate=true
```

### 5. Configuration (`src/config/mod.rs`)
Stores:
- Log file path
- Encoding preference
- Auto-copy setting
- SQL formatting toggle
- Legacy connection info (migrated to db.rs ConnectionManager)

## Log Format Expected

The parser expects log lines in this format:
```
2024/01/01 10:00:00,INFO,ClassName,id=abc123def sql=SELECT * FROM users WHERE id = ?
2024/01/01 10:00:01,INFO,ClassName,id=abc123def params=[Int:1:42][String:2:test]
```

Parameter format: `[Type:Index:Value]`
- Type: Java type (String, Int, Long, etc.)
- Index: Parameter position (1-based)
- Value: Actual value to substitute

## Dependencies

Key crates:
- `eframe` (0.27) - GUI framework (egui + glow renderer)
- `tiberius` (0.12) - SQL Server TDS client
- `sqlx` (0.7) - Multi-database support
- `tokio` (1.x) - Async runtime
- `serde` / `serde_json` - Config serialization
- `regex` - Log parsing patterns
- `encoding_rs` - Character encoding
- `arboard` (3.x) - Clipboard support
- `rfd` (0.14) - Native file dialogs
- `sqlformat` (0.2) - SQL pretty-printing

## Building

### Prerequisites
```powershell
# Install Rust
winget install Rustlang.Rustup
```

### Build Commands
```powershell
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run
cargo run --release
```

### Output
- Debug: `target/debug/sql-log-parser.exe`
- Release: `target/release/sql-log-parser.exe` (~4-5 MB)

## Current State & TODOs

### Completed
- [x] Log parsing with SHIFT_JIS support
- [x] SQL parameter placeholder replacement
- [x] SQL formatting/pretty-print
- [x] Query grouping by SQL template
- [x] SQL Executor tab with connection management
- [x] SQL Server support via tiberius
- [x] JDBC URL parsing (host:port, databaseName, encrypt, trustServerCertificate)
- [x] Connection test before save
- [x] Dracula theme
- [x] CJK font loading (Meiryo, MS Gothic)

### Known Issues / Potential Improvements
- [ ] Password storage is plain text (consider encryption)
- [ ] Connection pooling (currently creates new connection per query)
- [ ] Export results to CSV/Excel
- [ ] Query history
- [ ] Syntax highlighting in SQL editor
- [ ] Named instance support for SQL Server (partially implemented)
- [ ] Better error messages for connection failures
- [ ] Date/time type handling in result display (shows "NULL/Other" fallback)

## Configuration Files

- `log_parser_config.json` - Main app config (next to exe)
- `db_connections.json` - Database connections (in user config dir or next to exe)

## Testing

```powershell
cargo test
```

Tests cover:
- Parameter string parsing
- JDBC URL parsing/building
- Last query extraction
- SQL placeholder replacement
