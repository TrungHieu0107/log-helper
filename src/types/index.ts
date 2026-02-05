// ─── Database Types (mirrors src-tauri/src/core/db.rs) ──────────────────────

export type DbType = "Postgres" | "Mysql" | "Sqlite" | "SqlServer";

export type CellValue =
  | "Null"
  | { Text: string }
  | { Int: number }
  | { Float: number }
  | { Bool: boolean }
  | { DateTime: string }
  | { Binary: string };

export interface QueryResult {
  columns: string[];
  rows: CellValue[][];
  affected_rows: number;
  execution_time_ms: number;
}

export interface DbConfig {
  id: string;
  name: string;
  db_type: DbType;
  url: string;
  user: string;
  password: string;
  encoding: string | null;
}

export interface ConnectionFields {
  host: string;
  port: string;
  database: string;
  encrypt: boolean;
  trust_cert: boolean;
}

export interface ParsedSqlServerUrl {
  host: string;
  port: number;
  instance: string | null;
  database: string | null;
  encrypt: boolean;
  trust_cert: boolean;
}

// ─── Log Parser Types (mirrors src-tauri/src/core/log_parser.rs) ────────────

export interface IdInfo {
  id: string;
  dao_name: string;
  has_sql: boolean;
  params_count: number;
}

export interface Execution {
  id: string;
  timestamp: string;
  dao_file: string;
  sql: string;
  filled_sql: string;
  formatted_sql: string;
  params: string[];
  execution_index: number;
}

// ─── Query Processor Types (mirrors src-tauri/src/core/query_processor.rs) ──

export interface QueryGroup {
  template_sql: string;
  formatted_template_sql: string;
  executions: Execution[];
}

export interface ProcessResult {
  query: {
    id: string;
    sql: string;
    params: string[];
  };
  executions: Execution[];
  groups: QueryGroup[];
  filled_sql: string;
  formatted_sql: string;
  formatted_params: string;
  copied_to_clipboard: boolean;
  error: string | null;
}

// ─── Config Types (mirrors src-tauri/src/config/mod.rs) ─────────────────────

export interface Config {
  log_file_path: string;
  html_output_path: string;
  auto_copy: boolean;
  connections: LegacyDbConnection[];
  active_connection_index: number;
  csv_separator: string;
  encoding: string;
  format_sql: boolean;
}

export interface LegacyDbConnection {
  name: string;
  server: string;
  database: string;
  username: string;
  password: string;
  use_windows_auth: boolean;
}
