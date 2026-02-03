import { invoke } from "@tauri-apps/api/tauri";
import { Config, DbConnection, IdInfo, ProcessResult, QueryResult, SqlResult } from "./types";

export const api = {
  loadConfig: async (): Promise<Config> => {
    return await invoke("load_config");
  },

  saveConfig: async (config: Config): Promise<void> => {
    return await invoke("save_config", { config });
  },

  getAllIds: async (logFilePath: string): Promise<IdInfo[]> => {
    return await invoke("get_all_ids", { logFilePath });
  },

  parseLogById: async (logFilePath: string, targetId: string): Promise<QueryResult> => {
    return await invoke("parse_log_by_id", { logFilePath, targetId });
  },

  processQuery: async (logFilePath: string, targetId: string, autoCopy: boolean): Promise<ProcessResult> => {
    return await invoke("process_query", { logFilePath, targetId, autoCopy });
  },

  processLastQuery: async (logFilePath: string, autoCopy: boolean): Promise<ProcessResult> => {
    return await invoke("process_last_query", { logFilePath, autoCopy });
  },

  testConnection: async (connectionConfig: DbConnection): Promise<boolean> => {
    return await invoke("test_connection", { connectionConfig });
  },

  executeSql: async (sql: string): Promise<SqlResult> => {
    return await invoke("execute_sql", { sql });
  },
};
