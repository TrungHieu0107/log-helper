import { useState, useEffect, useCallback } from "react";
import { loadConfig, saveConfig } from "./api/commands";
import type { Config } from "./types";
import Layout from "./components/Layout";
import LogParserTab from "./components/LogParser/LogParserTab";
import SqlExecutorTab from "./components/SqlExecutor/SqlExecutorTab";

export type AppTab = "LogParser" | "SqlExecutor";

function App() {
  const [activeTab, setActiveTab] = useState<AppTab>("LogParser");
  const [config, setConfig] = useState<Config | null>(null);
  const [status, setStatus] = useState("Ready");
  const [executorSql, setExecutorSql] = useState("");

  useEffect(() => {
    loadConfig()
      .then(setConfig)
      .catch((e) => setStatus(`Failed to load config: ${e}`));
  }, []);

  const updateConfig = useCallback(
    async (patch: Partial<Config>) => {
      if (!config) return;
      const updated = { ...config, ...patch };
      setConfig(updated);
      try {
        await saveConfig(updated);
      } catch (e) {
        setStatus(`Failed to save config: ${e}`);
      }
    },
    [config],
  );

  const switchToExecutor = useCallback((sql: string) => {
    setExecutorSql(sql);
    setActiveTab("SqlExecutor");
  }, []);

  if (!config) {
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100vh",
        }}
      >
        <div className="spinner" />
        <span style={{ marginLeft: 8 }}>Loading...</span>
      </div>
    );
  }

  return (
    <Layout
      activeTab={activeTab}
      onTabChange={setActiveTab}
      config={config}
      updateConfig={updateConfig}
      status={status}
    >
      {activeTab === "LogParser" ? (
        <LogParserTab
          config={config}
          updateConfig={updateConfig}
          setStatus={setStatus}
          onSwitchToExecutor={switchToExecutor}
        />
      ) : (
        <SqlExecutorTab
          config={config}
          setStatus={setStatus}
          initialSql={executorSql}
          onSqlConsumed={() => setExecutorSql("")}
        />
      )}
    </Layout>
  );
}

export default App;
