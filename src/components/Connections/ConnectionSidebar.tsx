import { useState } from "react";
import { deleteConnection } from "../../api/commands";
import type { DbConfig } from "../../types";
import ConnectionModal from "./ConnectionModal";
import { v4 as uuidv4 } from "../../utils/uuid";

interface ConnectionSidebarProps {
  connections: DbConfig[];
  activeConnectionId: string | null;
  onSelectConnection: (id: string) => void;
  onConnectionsChanged: () => Promise<void>;
  setStatus: (status: string) => void;
}

export default function ConnectionSidebar({
  connections,
  activeConnectionId,
  onSelectConnection,
  onConnectionsChanged,
  setStatus,
}: ConnectionSidebarProps) {
  const [showModal, setShowModal] = useState(false);
  const [editingConnection, setEditingConnection] = useState<DbConfig | null>(
    null,
  );

  const handleNew = () => {
    setEditingConnection({
      id: uuidv4(),
      name: "",
      db_type: "SqlServer",
      url: "",
      user: "",
      password: "",
      encoding: null,
    });
    setShowModal(true);
  };

  const handleEdit = (conn: DbConfig) => {
    setEditingConnection({ ...conn });
    setShowModal(true);
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteConnection(id);
      await onConnectionsChanged();
      setStatus("Connection deleted");
    } catch (e) {
      setStatus(`Failed to delete: ${e}`);
    }
  };

  const handleModalClose = async (saved: boolean) => {
    setShowModal(false);
    setEditingConnection(null);
    if (saved) {
      await onConnectionsChanged();
    }
  };

  return (
    <>
      <div className="sidebar">
        <div className="sidebar-header">
          <h2>Connections</h2>
          <button onClick={handleNew}>+ New</button>
        </div>
        <div className="sidebar-body">
          {connections.map((conn) => (
            <div
              key={conn.id}
              className={`sidebar-item ${activeConnectionId === conn.id ? "selected" : ""}`}
              onClick={() => onSelectConnection(conn.id)}
            >
              <span>{conn.name}</span>
              <div className="actions">
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleEdit(conn);
                  }}
                  title="Edit"
                >
                  Edit
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDelete(conn.id);
                  }}
                  title="Delete"
                >
                  Del
                </button>
              </div>
            </div>
          ))}
          {connections.length === 0 && (
            <div
              style={{
                padding: "12px",
                color: "var(--comment)",
                fontSize: 12,
              }}
            >
              No connections. Click "+ New" to add one.
            </div>
          )}
        </div>
      </div>

      {showModal && editingConnection && (
        <ConnectionModal
          connection={editingConnection}
          isNew={!connections.some((c) => c.id === editingConnection.id)}
          onClose={handleModalClose}
          setStatus={setStatus}
        />
      )}
    </>
  );
}
