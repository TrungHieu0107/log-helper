import { useState } from "react";
import {
  addConnection,
  updateConnection,
  testConnection,
  parseJdbcUrl,
  buildJdbcUrl,
} from "../../api/commands";
import type { ConnectionFields, DbConfig, DbType } from "../../types";

interface ConnectionModalProps {
  connection: DbConfig;
  isNew: boolean;
  onClose: (saved: boolean) => void;
  setStatus: (status: string) => void;
}

type InputMode = "url" | "fields";

const DB_TYPES: { value: DbType; label: string }[] = [
  { value: "SqlServer", label: "SQL Server" },
  { value: "Postgres", label: "Postgres" },
  { value: "Mysql", label: "MySQL" },
  { value: "Sqlite", label: "SQLite" },
];

export default function ConnectionModal({
  connection,
  isNew,
  onClose,
  setStatus,
}: ConnectionModalProps) {
  const [conn, setConn] = useState<DbConfig>({ ...connection });
  const [inputMode, setInputMode] = useState<InputMode>("url");
  const [fields, setFields] = useState<ConnectionFields>({
    host: "localhost",
    port: "1433",
    database: "",
    encrypt: false,
    trust_cert: true,
  });
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{
    ok: boolean;
    message: string;
  } | null>(null);
  const [saving, setSaving] = useState(false);

  const updateField = (key: keyof DbConfig, value: string | DbType) => {
    setConn((prev) => ({ ...prev, [key]: value }));
    setTestResult(null);
  };

  const handleUrlChange = async (url: string) => {
    updateField("url", url);
    // Try to parse into fields for preview
    try {
      const parsed = await parseJdbcUrl(url);
      setFields({
        host: parsed.host,
        port: parsed.port.toString(),
        database: parsed.database || "",
        encrypt: parsed.encrypt,
        trust_cert: parsed.trust_cert,
      });
    } catch {
      // Ignore parse errors while typing
    }
  };

  const handleFieldChange = async (
    key: keyof ConnectionFields,
    value: string | boolean,
  ) => {
    const updated = { ...fields, [key]: value };
    setFields(updated);
    setTestResult(null);
    try {
      const url = await buildJdbcUrl(updated);
      setConn((prev) => ({ ...prev, url }));
    } catch {
      // Ignore
    }
  };

  const handleTest = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const msg = await testConnection(conn);
      setTestResult({ ok: true, message: msg });
    } catch (e) {
      setTestResult({ ok: false, message: String(e) });
    } finally {
      setTesting(false);
    }
  };

  const handleSave = async () => {
    // Validate
    const errors: string[] = [];
    if (!conn.name.trim()) errors.push("Connection name is required");
    if (!conn.url.trim()) errors.push("Connection URL is required");

    if (errors.length > 0) {
      setTestResult({ ok: false, message: errors.join("\n") });
      return;
    }

    // Test then save
    setSaving(true);
    setTesting(true);
    setTestResult(null);

    try {
      await testConnection(conn);
      setTestResult({ ok: true, message: "Connection successful" });
    } catch (e) {
      setTestResult({ ok: false, message: `Connection failed: ${e}` });
      setSaving(false);
      setTesting(false);
      return;
    }

    setTesting(false);

    try {
      if (isNew) {
        await addConnection(conn);
      } else {
        await updateConnection(conn);
      }
      setStatus(`Connection '${conn.name}' saved successfully`);
      onClose(true);
    } catch (e) {
      setTestResult({ ok: false, message: `Failed to save: ${e}` });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={() => onClose(false)}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h2>Connection Details</h2>

        {/* Name & Type */}
        <div className="form-row">
          <label>Name:</label>
          <input
            type="text"
            value={conn.name}
            onChange={(e) => updateField("name", e.target.value)}
          />
        </div>
        <div className="form-row">
          <label>Type:</label>
          <select
            value={conn.db_type}
            onChange={(e) =>
              updateField("db_type", e.target.value as DbType)
            }
          >
            {DB_TYPES.map((dt) => (
              <option key={dt.value} value={dt.value}>
                {dt.label}
              </option>
            ))}
          </select>
        </div>

        <hr style={{ borderColor: "var(--border)", margin: "12px 0" }} />

        {/* URL / Fields mode for SQL Server */}
        {conn.db_type === "SqlServer" && (
          <>
            <div className="mode-toggle">
              <label>
                <input
                  type="radio"
                  name="inputMode"
                  checked={inputMode === "url"}
                  onChange={() => setInputMode("url")}
                />
                URL
              </label>
              <label>
                <input
                  type="radio"
                  name="inputMode"
                  checked={inputMode === "fields"}
                  onChange={() => setInputMode("fields")}
                />
                Host & Port
              </label>
            </div>

            {inputMode === "url" ? (
              <div className="form-row">
                <label>JDBC URL:</label>
                <textarea
                  value={conn.url}
                  onChange={(e) => handleUrlChange(e.target.value)}
                  placeholder="jdbc:sqlserver://host:port;databaseName=DB"
                  rows={3}
                  style={{ flex: 1 }}
                />
              </div>
            ) : (
              <>
                <div className="form-row">
                  <label>Host:</label>
                  <input
                    type="text"
                    value={fields.host}
                    onChange={(e) =>
                      handleFieldChange("host", e.target.value)
                    }
                  />
                </div>
                <div className="form-row">
                  <label>Port:</label>
                  <input
                    type="text"
                    value={fields.port}
                    onChange={(e) =>
                      handleFieldChange("port", e.target.value)
                    }
                  />
                </div>
                <div className="form-row">
                  <label>Database:</label>
                  <input
                    type="text"
                    value={fields.database}
                    onChange={(e) =>
                      handleFieldChange("database", e.target.value)
                    }
                  />
                </div>
                <div className="form-row">
                  <label>Encrypted:</label>
                  <input
                    type="checkbox"
                    checked={fields.encrypt}
                    onChange={(e) =>
                      handleFieldChange("encrypt", e.target.checked)
                    }
                  />
                </div>
                <div className="form-row">
                  <label>Trust Server Cert:</label>
                  <input
                    type="checkbox"
                    checked={fields.trust_cert}
                    onChange={(e) =>
                      handleFieldChange("trust_cert", e.target.checked)
                    }
                  />
                </div>
                <div
                  style={{
                    fontSize: 11,
                    color: "var(--comment)",
                    marginTop: 4,
                    fontFamily: "monospace",
                  }}
                >
                  Preview: {conn.url}
                </div>
              </>
            )}
          </>
        )}

        {/* Generic URL for other DB types */}
        {conn.db_type !== "SqlServer" && (
          <div className="form-row">
            <label>Connection URL:</label>
            <input
              type="text"
              value={conn.url}
              onChange={(e) => updateField("url", e.target.value)}
            />
          </div>
        )}

        <hr style={{ borderColor: "var(--border)", margin: "12px 0" }} />

        {/* Auth */}
        <h3 style={{ fontSize: 14, marginBottom: 8 }}>Authentication</h3>
        <div className="form-row">
          <label>User:</label>
          <input
            type="text"
            value={conn.user}
            onChange={(e) => updateField("user", e.target.value)}
          />
        </div>
        <div className="form-row">
          <label>Password:</label>
          <input
            type="password"
            value={conn.password}
            onChange={(e) => updateField("password", e.target.value)}
          />
        </div>

        {/* Test Status */}
        {testing && (
          <div className="flex-row mt-md">
            <span className="spinner" />
            <span>Testing connection...</span>
          </div>
        )}
        {testResult && !testing && (
          <div
            className={`mt-md ${testResult.ok ? "status-success" : "status-error"}`}
          >
            {testResult.ok ? "✔ " : "✖ "}
            {testResult.message}
          </div>
        )}

        {/* Actions */}
        <div className="form-actions">
          <button disabled={testing || saving} onClick={handleTest}>
            Test Connection
          </button>
          <button
            className="btn-primary"
            disabled={testing || saving}
            onClick={handleSave}
          >
            {saving ? "Saving..." : "Save"}
          </button>
          <button onClick={() => onClose(false)}>Cancel</button>
        </div>
      </div>
    </div>
  );
}
