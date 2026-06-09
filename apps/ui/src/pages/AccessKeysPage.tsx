import { api, type AccessKey } from "../lib/api";
import { Layout } from "../components/Layout";
import { PageHeader } from "../components/PageHeader";
import { useEffect, useState, useCallback } from "react";
import * as Dialog from "@radix-ui/react-dialog";

function formatExpiry(ts?: number): string {
  if (!ts) return "Never";
  return new Date(ts * 1000).toLocaleDateString();
}

export default function AccessKeysPage() {
  const [keys, setKeys] = useState<AccessKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [createdKey, setCreatedKey] = useState<{
    key: AccessKey;
    cleartext: string;
  } | null>(null);
  const [form, setForm] = useState({ name: "", expireTime: "" });

  const load = useCallback(async () => {
    try {
      const { keys } = await api.accessKeys.list();
      setKeys(keys);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const handleCreate = async () => {
    const result = await api.accessKeys.create({
      name: form.name,
      expireTime: form.expireTime
        ? Math.floor(new Date(form.expireTime).getTime() / 1000)
        : undefined,
    });
    setCreatedKey(result);
    setForm({ name: "", expireTime: "" });
    setDialogOpen(false);
    load();
  };

  const handleDelete = async (id: string) => {
    await api.accessKeys.delete(id);
    load();
  };

  const handleDisable = async (id: string) => {
    await api.accessKeys.disable(id);
    load();
  };

  return (
    <Layout title="Access Keys">
      <PageHeader
        title="Access Keys"
        description="Create long-lived API keys for backend service authentication"
        actions={
          <>
            <Dialog.Root open={dialogOpen} onOpenChange={setDialogOpen}>
              <Dialog.Trigger asChild>
                <button
                  className="btn btn-primary"
                  id="create-key-btn"
                  data-testid="create-key-btn"
                >
                  + Create Key
                </button>
              </Dialog.Trigger>
              <Dialog.Portal>
                <Dialog.Overlay className="dialog-overlay" />
                <Dialog.Content className="dialog-content">
                  <div className="dialog-header">
                    <Dialog.Title className="dialog-title">
                      Create Access Key
                    </Dialog.Title>
                    <Dialog.Close className="btn btn-ghost btn-icon">
                      ✕
                    </Dialog.Close>
                  </div>
                  <div className="dialog-body">
                    <div className="form-group">
                      <label className="form-label" htmlFor="key-name">
                        Name
                      </label>
                      <input
                        id="key-name"
                        className="input"
                        data-testid="key-name-input"
                        value={form.name}
                        onChange={(e) =>
                          setForm((f) => ({ ...f, name: e.target.value }))
                        }
                        placeholder="My Service Key"
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label" htmlFor="key-expiry">
                        Expiry (optional)
                      </label>
                      <input
                        id="key-expiry"
                        type="date"
                        className="input"
                        data-testid="key-expiry-input"
                        value={form.expireTime}
                        onChange={(e) =>
                          setForm((f) => ({ ...f, expireTime: e.target.value }))
                        }
                      />
                    </div>
                  </div>
                  <div className="dialog-footer">
                    <Dialog.Close asChild>
                      <button className="btn btn-secondary">Cancel</button>
                    </Dialog.Close>
                    <button
                      className="btn btn-primary"
                      onClick={handleCreate}
                      disabled={!form.name}
                      data-testid="create-submit"
                    >
                      Create
                    </button>
                  </div>
                </Dialog.Content>
              </Dialog.Portal>
            </Dialog.Root>
          </>
        }
      />

      {/* Created key cleartext — shown once */}
      {createdKey && (
        <div
          data-testid="cleartext-banner"
          style={{
            padding: "var(--space-4)",
            background: "var(--color-card-bg)",
            border: "1px solid var(--color-success)",
            borderRadius: "var(--radius-lg)",
            marginBottom: "var(--space-6)",
          }}
        >
          <p
            style={{
              fontWeight: "var(--fw-semibold)",
              color: "var(--color-success)",
              marginBottom: 8,
            }}
          >
            ✅ Key created — copy now, it won't be shown again
          </p>
          <code
            data-testid="cleartext-code"
            className="font-mono"
            style={{
              display: "block",
              padding: "var(--space-3)",
              background: "var(--color-content-bg)",
              borderRadius: "var(--radius-md)",
              fontSize: "var(--text-xs)",
              wordBreak: "break-all",
              color: "var(--color-text-primary)",
            }}
          >
            {createdKey.cleartext}
          </code>
          <div style={{ display: "flex", gap: 4, marginTop: 8 }}>
            <button
              className="btn btn-ghost btn-sm"
              onClick={() =>
                navigator.clipboard.writeText(createdKey.cleartext)
              }
            >
              📋 Copy Key
            </button>
            <button
              className="btn btn-ghost btn-sm"
              onClick={() => setCreatedKey(null)}
              data-testid="dismiss-btn"
            >
              Dismiss
            </button>
          </div>

          {/* Curl test example */}
          <div style={{ marginTop: "var(--space-4)" }}>
            <p
              style={{
                fontWeight: "var(--fw-medium)",
                color: "var(--color-text-secondary)",
                marginBottom: 6,
                fontSize: "var(--text-sm)",
              }}
            >
              Test your key:
            </p>
            <code
              data-testid="curl-example"
              className="font-mono"
              style={{
                display: "block",
                padding: "var(--space-3)",
                background: "var(--color-content-bg)",
                borderRadius: "var(--radius-md)",
                fontSize: "var(--text-xs)",
                whiteSpace: "pre-wrap",
                wordBreak: "break-all",
                color: "var(--color-text-primary)",
                lineHeight: 1.6,
              }}
            >
              {`curl -s -w '\\n%{http_code}\\n' ${window.location.origin}/v1/mgmt/user/search \\
  -H 'Content-Type: application/json' \\
  -H 'Authorization: Bearer ${createdKey.cleartext}' \\
  -d '{}' | jq`}
            </code>
            <button
              className="btn btn-ghost btn-sm"
              data-testid="copy-curl-btn"
              style={{ marginTop: 6 }}
              onClick={() =>
                navigator.clipboard.writeText(
                  `curl -s -w '\\n%{http_code}\\n' ${window.location.origin}/v1/mgmt/user/search \\\n  -H 'Content-Type: application/json' \\\n  -H 'Authorization: Bearer ${createdKey.cleartext}' \\\n  -d '{}' | jq`,
                )
              }
            >
              📋 Copy Curl
            </button>
          </div>
        </div>
      )}

      {loading ? (
        <div style={{ display: "flex", justifyContent: "center", padding: 40 }}>
          <div className="loading-spinner" />
        </div>
      ) : keys.length === 0 ? (
        <div className="empty-state">
          <div className="empty-state-icon">🔑</div>
          <div className="empty-state-title">No access keys</div>
          <div className="empty-state-description">
            Access keys allow backend services to authenticate with the Descope
            API.
          </div>
        </div>
      ) : (
        <div className="card">
          <div className="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Status</th>
                  <th>Expires</th>
                  <th>Roles</th>
                  <th style={{ width: 100 }}></th>
                </tr>
              </thead>
              <tbody>
                {keys.map((k) => (
                  <tr key={k.id} data-testid="key-row">
                    <td>
                      <strong>{k.name}</strong>
                    </td>
                    <td>
                      <span
                        className={`badge ${k.status === "active" ? "badge-success" : "badge-neutral"}`}
                      >
                        {k.status}
                      </span>
                    </td>
                    <td style={{ color: "var(--color-text-secondary)" }}>
                      {formatExpiry(k.expiresAt)}
                    </td>
                    <td>
                      <div
                        style={{ display: "flex", flexWrap: "wrap", gap: 4 }}
                      >
                        {k.roleNames.length === 0 ? (
                          <span
                            style={{
                              color: "var(--color-text-tertiary)",
                              fontSize: "var(--text-xs)",
                            }}
                          >
                            —
                          </span>
                        ) : (
                          k.roleNames.map((r) => (
                            <span key={r} className="badge badge-neutral">
                              {r}
                            </span>
                          ))
                        )}
                      </div>
                    </td>
                    <td>
                      <div style={{ display: "flex", gap: 4 }}>
                        {k.status === "active" && (
                          <button
                            className="btn btn-ghost btn-icon btn-sm"
                            onClick={() => handleDisable(k.id)}
                            title="Disable"
                            data-testid="disable-btn"
                          >
                            ⏸️
                          </button>
                        )}
                        <button
                          className="btn btn-ghost btn-icon btn-sm"
                          onClick={() => handleDelete(k.id)}
                          title="Delete"
                          data-testid="delete-btn"
                        >
                          🗑️
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </Layout>
  );
}
