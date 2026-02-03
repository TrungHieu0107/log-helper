export interface IdInfo {
  id: string;
  has_sql: boolean;
  params_count: number;
}

export interface QueryResult {
  id: string;
  sql: string;
  params: string[];
}

export interface Execution {
  id: string;
  timestamp: string;
  dao_file: string;
  sql: string;
  filled_sql: string;
  params: string[];
  execution_index: number;
}

export interface ProcessResult {
  query: QueryResult;
  filled_sql: string;
  formatted_sql: string;
  formatted_params: string;
  copied_to_clipboard: boolean;
  error?: string;
}

export interface DbConnection {
  name: string;
  server: string;
  database: string;
  username: string;
  password: string;
  use_windows_auth: boolean;
}

export interface Config {
  log_file_path: string;
  html_output_path: string;
  auto_copy: boolean;
  connections: DbConnection[];
  active_connection_index: number;
  csv_separator: string;
}

export interface SqlColumn {
  name: string;
  sql_type: number;
  size: number;
}

export interface SqlResult {
  success: boolean;
  error?: string;
  columns: SqlColumn[];
  rows: string[][];
  rows_affected: number;
}
