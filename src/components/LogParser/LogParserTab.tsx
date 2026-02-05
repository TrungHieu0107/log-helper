import { useState, useCallback } from "react";
import { processQuery } from "../../api/commands";
import type { Config, ProcessResult } from "../../types";
import IdSidebar from "./IdSidebar";
import ExecutionResult from "./ExecutionResult";

interface LogParserTabProps {
  config: Config;
  updateConfig: (patch: Partial<Config>) => Promise<void>;
  setStatus: (status: string) => void;
  onSwitchToExecutor: (sql: string) => void;
}

export default function LogParserTab({
  config,
  updateConfig: _updateConfig,
  setStatus,
  onSwitchToExecutor,
}: LogParserTabProps) {
  const [searchInput, setSearchInput] = useState("");
  const [selectedId, setSelectedId] = useState("");
  const [result, setResult] = useState<ProcessResult | null>(null);

  const doSearch = useCallback(
    async (id: string) => {
      if (!id) {
        setStatus("Enter an ID to search");
        return;
      }
      if (!config.log_file_path) {
        setStatus("No log file path set");
        return;
      }
      setStatus("Searching...");
      try {
        const res = await processQuery(
          id,
          config.log_file_path,
          config.auto_copy,
          config.encoding,
        );
        setResult(res);
        if (res.error) {
          setStatus(`Error: ${res.error}`);
        } else {
          setStatus("Query found");
        }
      } catch (e) {
        setStatus(`Error: ${e}`);
      }
    },
    [config, setStatus],
  );

  const handleSelectId = useCallback(
    (id: string) => {
      setSelectedId(id);
      setSearchInput(id);
      doSearch(id);
    },
    [doSearch],
  );

  const handleLastQuery = useCallback(
    (res: ProcessResult) => {
      setResult(res);
      if (res.query.id) {
        setSelectedId(res.query.id);
        setSearchInput(res.query.id);
      }
    },
    [],
  );

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      doSearch(searchInput);
    }
  };

  return (
    <div className="content-area">
      <IdSidebar
        config={config}
        selectedId={selectedId}
        onSelectId={handleSelectId}
        onLastQuery={handleLastQuery}
        setStatus={setStatus}
      />
      <div className="main-content">
        {/* Search */}
        <div
          className="flex-row mb-lg"
          style={{
            padding: "8px 12px",
            background: "var(--bg-darker)",
            borderRadius: 6,
            border: "1px solid var(--border)",
          }}
        >
          <span style={{ fontSize: 13 }}>ID:</span>
          <input
            type="text"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Enter ID"
            style={{ width: 220 }}
          />
          <button className="btn-primary" onClick={() => doSearch(searchInput)}>
            Search
          </button>
        </div>

        {/* Results */}
        {result && (
          <ExecutionResult
            result={result}
            formatSql={config.format_sql}
            onExecuteSql={onSwitchToExecutor}
          />
        )}

        {!result && (
          <div style={{ color: "var(--comment)", padding: 20 }}>
            Select an ID from the sidebar or search to see results.
          </div>
        )}
      </div>
    </div>
  );
}
