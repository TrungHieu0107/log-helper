import type { ReactNode } from "react";
import type { AppTab } from "../App";
import type { Config } from "../types";
import { open } from "@tauri-apps/plugin-dialog";
import { saveConfig } from "../api/commands";

interface LayoutProps {
  activeTab: AppTab;
  onTabChange: (tab: AppTab) => void;
  config: Config;
  updateConfig: (patch: Partial<Config>) => Promise<void>;
  status: string;
  children: ReactNode;
}

const ENCODINGS = [
  "SHIFT_JIS",
  "UTF-8",
  "UTF-16LE",
  "UTF-16BE",
  "EUC-JP",
  "WINDOWS-1252",
];

export default function Layout({
  activeTab,
  onTabChange,
  config,
  updateConfig,
  status,
  children,
}: LayoutProps) {
  const handleBrowse = async () => {
    const selected = await open({
      filters: [
        { name: "Log Files", extensions: ["log", "txt"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });
    if (selected) {
      await updateConfig({ log_file_path: selected as string });
    }
  };

  return (
    <div className="app-layout">
      {/* Toolbar */}
      <div className="toolbar">
        <h1>SQL Log Parser</h1>
        <div className="toolbar-separator" />

        <button
          className={`tab-btn ${activeTab === "LogParser" ? "active" : ""}`}
          onClick={() => onTabChange("LogParser")}
        >
          Log Parser
        </button>
        <button
          className={`tab-btn ${activeTab === "SqlExecutor" ? "active" : ""}`}
          onClick={() => onTabChange("SqlExecutor")}
        >
          SQL Executor
        </button>

        {activeTab === "LogParser" && (
          <>
            <div className="toolbar-separator" />
            <span style={{ fontSize: 12, color: "var(--comment)" }}>Log:</span>
            <input
              type="text"
              value={config.log_file_path}
              onChange={(e) => updateConfig({ log_file_path: e.target.value })}
              onBlur={() => saveConfig(config)}
              placeholder="Path to log file"
              style={{ width: 280 }}
            />
            <button onClick={handleBrowse}>Browse</button>

            <div className="toolbar-separator" />

            <span style={{ fontSize: 12, color: "var(--comment)" }}>
              Encoding:
            </span>
            <select
              value={config.encoding}
              onChange={(e) => updateConfig({ encoding: e.target.value })}
            >
              {ENCODINGS.map((enc) => (
                <option key={enc} value={enc}>
                  {enc}
                </option>
              ))}
            </select>

            <div className="toolbar-separator" />

            <label style={{ fontSize: 12, display: "flex", alignItems: "center", gap: 4 }}>
              <input
                type="checkbox"
                checked={config.auto_copy}
                onChange={(e) => updateConfig({ auto_copy: e.target.checked })}
              />
              Auto Copy
            </label>

            <label style={{ fontSize: 12, display: "flex", alignItems: "center", gap: 4 }}>
              <input
                type="checkbox"
                checked={config.format_sql}
                onChange={(e) => updateConfig({ format_sql: e.target.checked })}
              />
              Format SQL
            </label>
          </>
        )}
      </div>

      {/* Content */}
      {children}

      {/* Status Bar */}
      <div className="statusbar">
        <span>{status}</span>
        <span>v2.1.0</span>
      </div>
    </div>
  );
}
