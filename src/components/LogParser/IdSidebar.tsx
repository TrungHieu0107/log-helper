import { useState, useEffect, useCallback } from "react";
import { getAllIds, processLastQuery } from "../../api/commands";
import type { Config, IdInfo, ProcessResult } from "../../types";

interface IdSidebarProps {
  config: Config;
  selectedId: string;
  onSelectId: (id: string) => void;
  onLastQuery: (result: ProcessResult) => void;
  setStatus: (status: string) => void;
}

export default function IdSidebar({
  config,
  selectedId,
  onSelectId,
  onLastQuery,
  setStatus,
}: IdSidebarProps) {
  const [ids, setIds] = useState<IdInfo[]>([]);

  const loadIds = useCallback(async () => {
    if (!config.log_file_path) {
      setStatus("No log file path set");
      return;
    }
    setStatus("Loading IDs...");
    try {
      const result = await getAllIds(config.log_file_path, config.encoding);
      setIds(result);
      setStatus(`Loaded ${result.length} IDs`);
    } catch (e) {
      setStatus(`Error loading IDs: ${e}`);
    }
  }, [config.log_file_path, config.encoding, setStatus]);

  useEffect(() => {
    if (config.log_file_path) {
      loadIds();
    }
  }, [config.log_file_path, config.encoding, loadIds]);

  const handleLastQuery = async () => {
    if (!config.log_file_path) {
      setStatus("No log file path set");
      return;
    }
    setStatus("Loading last SQL...");
    try {
      const result = await processLastQuery(
        config.log_file_path,
        config.auto_copy,
        config.encoding,
      );
      if (result.error) {
        setStatus(`Error: ${result.error}`);
      } else {
        setStatus("Last SQL loaded");
        onLastQuery(result);
      }
    } catch (e) {
      setStatus(`Error: ${e}`);
    }
  };

  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <h2>Query IDs</h2>
        <div className="flex-row gap-sm">
          <button onClick={handleLastQuery} title="Load last SQL">
            Last
          </button>
          <button onClick={loadIds} title="Refresh">
            Refresh
          </button>
        </div>
      </div>
      <div className="sidebar-body">
        {ids.map((info) => {
          const label =
            info.dao_name && info.dao_name !== "Unknown"
              ? `${info.dao_name} - ${info.id}`
              : info.id;
          const displayLabel =
            info.params_count > 0
              ? `${label} (${info.params_count})`
              : label;

          return (
            <div
              key={info.id}
              className={`sidebar-item ${selectedId === info.id ? "selected" : ""}`}
              onClick={() => onSelectId(info.id)}
            >
              {displayLabel}
            </div>
          );
        })}
        {ids.length === 0 && (
          <div style={{ padding: "12px", color: "var(--comment)", fontSize: 12 }}>
            No IDs found. Set a log file path and click Refresh.
          </div>
        )}
      </div>
    </div>
  );
}
