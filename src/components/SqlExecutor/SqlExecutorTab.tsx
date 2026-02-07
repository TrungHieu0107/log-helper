import { useState, useEffect, useCallback } from "react";
import { executeQuery, listConnections } from "../../api/commands";
import type { Config, DbConfig, QueryResult } from "../../types";
import ConnectionSidebar from "../Connections/ConnectionSidebar";
import SqlEditor from "./SqlEditor";
import ResultTable from "./ResultTable";

interface SqlExecutorTabProps {
  config: Config;
  setStatus: (status: string) => void;
  initialSql: string;
  onSqlConsumed: () => void;
}

export default function SqlExecutorTab({
  config: _config,
  setStatus,
  initialSql,
  onSqlConsumed,
}: SqlExecutorTabProps) {
  const [sql, setSql] = useState("");
  const [connections, setConnections] = useState<DbConfig[]>([]);
  const [activeConnectionId, setActiveConnectionId] = useState<string | null>(
    null,
  );
  const [queryResult, setQueryResult] = useState<QueryResult | null>(null);
  const [queryError, setQueryError] = useState<string | null>(null);
  const [executing, setExecuting] = useState(false);

  // Load connections on mount
  const refreshConnections = useCallback(async () => {
    try {
      const conns = await listConnections();
      setConnections(conns);
    } catch (e) {
      setStatus(`Failed to load connections: ${e}`);
    }
  }, [setStatus]);

  useEffect(() => {
    refreshConnections();
  }, [refreshConnections]);

  // Consume initial SQL from LogParser "Execute" button
  useEffect(() => {
    if (initialSql) {
      setSql(initialSql);
      onSqlConsumed();
    }
  }, [initialSql, onSqlConsumed]);

  const handleRunQuery = async () => {
    if (!activeConnectionId) {
      setStatus("Select a connection first");
      return;
    }
    if (!sql.trim()) {
      setStatus("Enter a SQL query");
      return;
    }

    setExecuting(true);
    setQueryResult(null);
    setQueryError(null);
    setStatus("Executing query...");

    try {
      const result = await executeQuery(activeConnectionId, sql);
      setQueryResult(result);
      setStatus(
        `Query completed in ${result.execution_time_ms}ms. Affected rows: ${result.affected_rows}`,
      );
    } catch (e) {
      setQueryError(String(e));
      setStatus(`Query failed: ${e}`);
    } finally {
      setExecuting(false);
    }
  };

  return (
    <div className="content-area">
      <ConnectionSidebar
        connections={connections}
        activeConnectionId={activeConnectionId}
        onSelectConnection={setActiveConnectionId}
        onConnectionsChanged={refreshConnections}
        setStatus={setStatus}
      />
      <div className="main-content">
        <h2 style={{ marginBottom: 12, fontSize: 16 }}>SQL Executor</h2>

        {/* Connection selector + Run */}
        <div className="flex-row mb-md">
          <span style={{ fontSize: 13, color: "var(--comment)" }}>
            Connection:
          </span>
          <select
            value={activeConnectionId || ""}
            onChange={(e) =>
              setActiveConnectionId(e.target.value || null)
            }
            style={{ minWidth: 200 }}
          >
            <option value="">Select Connection</option>
            {connections.map((conn) => (
              <option key={conn.id} value={conn.id}>
                {conn.name}
              </option>
            ))}
          </select>
          <button
            className="btn-primary"
            disabled={executing || !activeConnectionId}
            onClick={handleRunQuery}
          >
            {executing ? "Executing..." : "Run Query"}
          </button>
          {executing && <span className="spinner" />}
        </div>

        {/* SQL Editor */}
        <SqlEditor value={sql} onChange={setSql} />

        <hr style={{ borderColor: "var(--border)", margin: "12px 0" }} />

        {/* Results */}
        <h3 style={{ fontSize: 14, marginBottom: 8 }}>Results</h3>

        {queryResult && (
          <>
            <div className="meta-info">
              Affected rows: {queryResult.affected_rows}, Execution time:{" "}
              {queryResult.execution_time_ms}ms
            </div>
            {queryResult.columns.length > 0 ? (
              <ResultTable result={queryResult} />
            ) : (
              <div style={{ color: "var(--comment)" }}>
                Query executed successfully. No result set returned.
              </div>
            )}
          </>
        )}

        {queryError && (
          <div style={{ color: "var(--red)", padding: 8 }}>
            Error: {queryError}
          </div>
        )}

        {!queryResult && !queryError && !executing && (
          <div style={{ color: "var(--comment)" }}>
            No results yet. Run a query to see results.
          </div>
        )}
      </div>
    </div>
  );
}
