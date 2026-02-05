import { useState } from "react";
import { copyToClipboard } from "../../api/commands";
import type { ProcessResult, QueryGroup, Execution } from "../../types";

interface ExecutionResultProps {
  result: ProcessResult;
  formatSql: boolean;
  onExecuteSql: (sql: string) => void;
}

export default function ExecutionResult({
  result,
  formatSql,
  onExecuteSql,
}: ExecutionResultProps) {
  if (result.error) {
    return (
      <div style={{ color: "var(--red)", padding: 8 }}>{result.error}</div>
    );
  }

  if (result.groups.length === 0 && result.executions.length === 0) {
    return <div style={{ color: "var(--comment)" }}>No executions found.</div>;
  }

  return (
    <div>
      {result.groups.map((group, gIdx) => (
        <GroupView
          key={gIdx}
          group={group}
          groupIndex={gIdx}
          formatSql={formatSql}
          onExecuteSql={onExecuteSql}
        />
      ))}
    </div>
  );
}

function GroupView({
  group,
  groupIndex,
  formatSql,
  onExecuteSql,
}: {
  group: QueryGroup;
  groupIndex: number;
  formatSql: boolean;
  onExecuteSql: (sql: string) => void;
}) {
  const [templateExpanded, setTemplateExpanded] = useState(false);

  const daoName =
    group.executions[0]?.dao_file || "Unknown DAO";

  return (
    <div className="execution-group">
      <div className="dao-name">
        {daoName === "" ? "Unknown DAO" : daoName}
      </div>

      {/* Template SQL */}
      <div
        className="collapsible-header"
        onClick={() => setTemplateExpanded(!templateExpanded)}
      >
        <span className={`arrow ${templateExpanded ? "open" : ""}`}>
          &#9654;
        </span>
        <span style={{ color: "var(--cyan)" }}>Template</span>
      </div>
      {templateExpanded && (
        <div className="collapsible-body mb-md">
          <div className="flex-row mb-sm">
            <button
              onClick={() => copyToClipboard(group.template_sql)}
            >
              Copy Template
            </button>
          </div>
          <div className="execution-item">
            <div className="sql-display">
              {group.formatted_template_sql || group.template_sql}
            </div>
          </div>
        </div>
      )}

      <hr style={{ borderColor: "var(--border)", margin: "8px 0" }} />

      {/* Executions */}
      {group.executions.map((exec, eIdx) => (
        <ExecutionView
          key={eIdx}
          exec={exec}
          defaultExpanded={groupIndex === 0 && eIdx === 0}
          formatSql={formatSql}
          onExecuteSql={onExecuteSql}
        />
      ))}
    </div>
  );
}

function ExecutionView({
  exec,
  defaultExpanded,
  formatSql,
  onExecuteSql,
}: {
  exec: Execution;
  defaultExpanded: boolean;
  formatSql: boolean;
  onExecuteSql: (sql: string) => void;
}) {
  const [expanded, setExpanded] = useState(defaultExpanded);

  const summary = `#${exec.execution_index} ${exec.timestamp} - ${exec.filled_sql.split("\n")[0]?.slice(0, 50) ?? ""}`;

  return (
    <div style={{ margin: "4px 0" }}>
      <div
        className="collapsible-header"
        onClick={() => setExpanded(!expanded)}
      >
        <span className={`arrow ${expanded ? "open" : ""}`}>&#9654;</span>
        <span style={{ fontSize: 12 }}>{summary}</span>
      </div>
      {expanded && (
        <div className="collapsible-body">
          <div className="execution-item">
            <div className="flex-row mb-sm">
              <button
                className="btn-success"
                onClick={() => copyToClipboard(exec.filled_sql)}
              >
                Copy SQL
              </button>
              <button
                className="btn-pink"
                onClick={() => onExecuteSql(exec.filled_sql)}
              >
                Execute
              </button>
              <span
                style={{
                  color: "var(--cyan)",
                  fontStyle: "italic",
                  fontSize: 12,
                }}
              >
                Index: {exec.execution_index}
              </span>
            </div>
            <div className="sql-display">
              {formatSql ? exec.formatted_sql : exec.filled_sql}
            </div>
            <hr
              style={{ borderColor: "var(--border)", margin: "8px 0" }}
            />
            <div style={{ color: "var(--pink)", fontWeight: 600, fontSize: 12 }}>
              Parameters:
            </div>
            <div className="params-display">
              {exec.params.map((p, i) => {
                const parts = p.split(":");
                if (parts.length >= 3) {
                  const type = parts[0];
                  const index = parts[1];
                  const value = parts.slice(2).join(":");
                  return (
                    <div key={i}>
                      [{index}] {type}: {value}
                    </div>
                  );
                }
                return <div key={i}>{p}</div>;
              })}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
