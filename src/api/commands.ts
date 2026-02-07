import { invoke } from "@tauri-apps/api/core";
import type {
  Config,
  ConnectionFields,
  DbConfig,
  IdInfo,
  ParsedSqlServerUrl,
  ProcessResult,
  QueryResult,
} from "../types";

// ─── Log Parser ─────────────────────────────────────────────────────────────

export async function getAllIds(
  logPath: string,
  encoding: string,
): Promise<IdInfo[]> {
  return invoke<IdInfo[]>("get_all_ids", {
    logPath,
    encoding,
  });
}

export async function processQuery(
  targetId: string,
  logPath: string,
  autoCopy: boolean,
  encoding: string,
): Promise<ProcessResult> {
  return invoke<ProcessResult>("process_query", {
    targetId,
    logPath,
    autoCopy,
    encoding,
  });
}

export async function processLastQuery(
  logPath: string,
  autoCopy: boolean,
  encoding: string,
): Promise<ProcessResult> {
  return invoke<ProcessResult>("process_last_query", {
    logPath,
    autoCopy,
    encoding,
  });
}

// ─── Connections ────────────────────────────────────────────────────────────

export async function listConnections(): Promise<DbConfig[]> {
  return invoke<DbConfig[]>("list_connections");
}

export async function addConnection(config: DbConfig): Promise<void> {
  return invoke<void>("add_connection", { config });
}

export async function updateConnection(config: DbConfig): Promise<void> {
  return invoke<void>("update_connection", { config });
}

export async function deleteConnection(id: string): Promise<void> {
  return invoke<void>("delete_connection", { id });
}

export async function testConnection(config: DbConfig): Promise<string> {
  return invoke<string>("test_connection", { config });
}

export async function executeQuery(
  connectionId: string,
  sql: string,
): Promise<QueryResult> {
  return invoke<QueryResult>("execute_query", { connectionId, sql });
}

// ─── Config ─────────────────────────────────────────────────────────────────

export async function loadConfig(): Promise<Config> {
  return invoke<Config>("load_config");
}

export async function saveConfig(newConfig: Config): Promise<void> {
  return invoke<void>("save_config", { newConfig });
}

// ─── Utilities ──────────────────────────────────────────────────────────────

export async function copyToClipboard(text: string): Promise<boolean> {
  return invoke<boolean>("copy_to_clipboard", { text });
}

export async function parseJdbcUrl(
  url: string,
): Promise<ParsedSqlServerUrl> {
  return invoke<ParsedSqlServerUrl>("parse_jdbc_url_cmd", { url });
}

export async function buildJdbcUrl(fields: ConnectionFields): Promise<string> {
  return invoke<string>("build_jdbc_url_cmd", { fields });
}
