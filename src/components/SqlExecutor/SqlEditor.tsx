interface SqlEditorProps {
  value: string;
  onChange: (value: string) => void;
}

export default function SqlEditor({ value, onChange }: SqlEditorProps) {
  return (
    <div className="mb-md">
      <label
        style={{
          display: "block",
          marginBottom: 4,
          fontSize: 13,
          color: "var(--comment)",
        }}
      >
        SQL Query:
      </label>
      <textarea
        className="code-editor"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="Enter SQL query..."
        style={{ width: "100%", minHeight: 120 }}
      />
    </div>
  );
}
