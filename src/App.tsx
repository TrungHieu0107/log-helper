import { useEffect, useState } from "react";
import { api } from "./api/tauri";
import { Config, IdInfo, ProcessResult } from "./api/types";
import { FolderOpen, Search, Copy, RefreshCw, Settings } from "lucide-react";
import { open } from "@tauri-apps/api/dialog";

function App() {
  const [config, setConfig] = useState<Config | null>(null);
  const [ids, setIds] = useState<IdInfo[]>([]);
  const [searchId, setSearchId] = useState("");
  const [status, setStatus] = useState("Ready");
  const [result, setResult] = useState<ProcessResult | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const cfg = await api.loadConfig();
      setConfig(cfg);
    } catch (e) {
      console.error(e);
      setStatus(`Error loading config: ${e}`);
    }
  };

  const loadIds = async () => {
    if (!config || !config.log_file_path) return;
    try {
      setStatus("Loading IDs...");
      const newIds = await api.getAllIds(config.log_file_path);
      setIds(newIds);
      setStatus(`Loaded ${newIds.length} IDs`);
    } catch (e) {
      setStatus(`Error: ${e}`);
    }
  };

  const handleSearch = async () => {
    if (!config || !searchId) return;
    try {
      setStatus("Searching...");
      const res = await api.processQuery(config.log_file_path, searchId, config.auto_copy);
      setResult(res);
      if (res.error) {
        setStatus(`Error: ${res.error}`);
      } else {
        setStatus("Query found");
      }
    } catch (e) {
      setStatus(`Error: ${e}`);
    }
  };

  const handleBrowse = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Log Files", extensions: ["log", "txt"] }],
    });
    if (selected && typeof selected === "string" && config) {
      const newConfig = { ...config, log_file_path: selected };
      setConfig(newConfig);
      await api.saveConfig(newConfig);
      // Auto reload IDs?
    }
  };

  if (!config) return <div className="app-container">Loading...</div>;

  return (
    <div className="app-container">
      {/* Toolbar */}
      <div className="toolbar">
        <div className="flex-row">
          <span style={{ fontWeight: "bold", fontSize: 18 }}>SQL Log Parser</span>
          <div style={{ width: 20 }}></div>
          <span className="text-muted">Log:</span>
          <input
            className="input"
            style={{ width: 300 }}
            value={config.log_file_path}
            onChange={(e) => setConfig({ ...config, log_file_path: e.target.value })}
          />
          <button className="btn flex-row" onClick={handleBrowse}>
            <FolderOpen size={16} /> Browse
          </button>
          <label className="flex-row">
            <input
              type="checkbox"
              checked={config.auto_copy}
              onChange={async (e) => {
                const newConfig = { ...config, auto_copy: e.target.checked };
                setConfig(newConfig);
                await api.saveConfig(newConfig);
              }}
            />
            Auto Copy
          </label>
        </div>
        <div className="spacer"></div>
        <button className="btn-icon">
          <Settings size={20} />
        </button>
      </div>

      {/* Main Area */}
      <div className="main-area">
        {/* Sidebar */}
        <div className="sidebar">
          <div style={{ padding: 10 }} className="flex-row">
            <span className="text-muted">Query IDs</span>
            <div className="spacer"></div>
            <button className="btn-icon" onClick={loadIds} title="Refresh">
              <RefreshCw size={16} />
            </button>
          </div>
          <div style={{ overflowY: "auto", flex: 1, padding: 5 }}>
            {ids.map((idInfo) => (
              <div
                key={idInfo.id}
                className={`list-item ${searchId === idInfo.id ? "selected" : ""}`}
                onClick={() => {
                  setSearchId(idInfo.id);
                  // search immediately?
                }}
                onDoubleClick={handleSearch}
              >
                {idInfo.id}
                {idInfo.params_count > 0 && <span className="text-muted text-sm"> ({idInfo.params_count})</span>}
              </div>
            ))}
          </div>
        </div>

        {/* Content */}
        <div className="content">
          <div className="card">
            <div className="card-header">Search</div>
            <div className="flex-row">
              <span className="text-muted">ID:</span>
              <input
                className="input"
                style={{ width: 200 }}
                value={searchId}
                onChange={(e) => setSearchId(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleSearch()}
              />
              <button className="btn btn-primary flex-row" onClick={handleSearch}>
                <Search size={16} /> Search
              </button>
            </div>
          </div>

          {result && (
            <div className="card">
              <div className="card-header flex-row">
                <span>Result</span>
                <div className="spacer"></div>
                {result.error ? (
                  <span style={{ color: "var(--error)" }}>{result.error}</span>
                ) : (
                  <button
                    className="btn flex-row"
                    onClick={() => navigator.clipboard.writeText(result.filled_sql)}
                  >
                    <Copy size={16} /> Copy SQL
                  </button>
                )}
              </div>

              {!result.error && (
                <>
                  <div className="text-muted" style={{ marginBottom: 5 }}>Formatted SQL:</div>
                  <div className="code-block">{result.formatted_sql}</div>
                  
                  {result.formatted_params && (
                     <>
                        <div className="text-muted" style={{ marginTop: 15, marginBottom: 5 }}>Parameters:</div>
                        <div className="code-block">{result.formatted_params}</div>
                     </>
                  )}
                </>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Status Bar */}
      <div className="status-bar">
        <span>{status}</span>
        <div className="spacer"></div>
        <span>v2.0.0</span>
      </div>
    </div>
  );
}

export default App;
