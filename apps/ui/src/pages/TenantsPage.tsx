import { api, type Tenant } from "../lib/api";
import { Layout } from "../components/Layout";
import { PageHeader } from "../components/PageHeader";
import { useEffect, useState, useCallback } from "react";
import * as Dialog from "@radix-ui/react-dialog";

export default function TenantsPage() {
  const [tenants, setTenants] = useState<Tenant[]>([]);
  const [loading, setLoading] = useState(true);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [form, setForm] = useState({ id: "", name: "" });
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const { tenants } = await api.tenants.list();
      setTenants(tenants);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const handleCreate = async () => {
    setError(null);
    try {
      await api.tenants.create(form.id, form.name);
      setForm({ id: "", name: "" });
      setDialogOpen(false);
      load();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Failed to create tenant");
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm(`Delete tenant ${id}? This cannot be undone.`)) return;
    await api.tenants.delete(id);
    load();
  };

  return (
    <Layout title="Tenants">
      <PageHeader
        title="Tenants"
        description="Manage multi-tenant organizations"
        actions={
          <Dialog.Root open={dialogOpen} onOpenChange={setDialogOpen}>
            <Dialog.Trigger asChild>
              <button
                className="btn btn-primary"
                id="create-tenant-btn"
                data-testid="create-tenant-btn"
              >
                + Create Tenant
              </button>
            </Dialog.Trigger>
            <Dialog.Portal>
              <Dialog.Overlay className="dialog-overlay" />
              <Dialog.Content className="dialog-content">
                <div className="dialog-header">
                  <Dialog.Title className="dialog-title">
                    Create Tenant
                  </Dialog.Title>
                  <Dialog.Close className="btn btn-ghost btn-icon">
                    ✕
                  </Dialog.Close>
                </div>
                <div className="dialog-body">
                  {error && (
                    <div
                      style={{
                        padding: "var(--space-3)",
                        background: "var(--color-danger-bg)",
                        borderRadius: "var(--radius-md)",
                        marginBottom: "var(--space-4)",
                        color: "var(--color-danger)",
                        fontSize: "var(--text-sm)",
                      }}
                    >
                      {error}
                    </div>
                  )}
                  <div className="form-group">
                    <label className="form-label" htmlFor="tenant-id">
                      Tenant ID{" "}
                      <span style={{ color: "var(--color-danger)" }}>*</span>
                    </label>
                    <input
                      id="tenant-id"
                      className="input"
                      data-testid="tenant-id-input"
                      value={form.id}
                      onChange={(e) =>
                        setForm((f) => ({ ...f, id: e.target.value }))
                      }
                      placeholder="acme-corp"
                    />
                    <div
                      style={{
                        fontSize: "var(--text-xs)",
                        color: "var(--color-text-tertiary)",
                        marginTop: 4,
                      }}
                    >
                      Lowercase letters, numbers, and hyphens only
                    </div>
                  </div>
                  <div className="form-group">
                    <label className="form-label" htmlFor="tenant-name">
                      Display Name{" "}
                      <span style={{ color: "var(--color-danger)" }}>*</span>
                    </label>
                    <input
                      id="tenant-name"
                      className="input"
                      data-testid="tenant-name-input"
                      value={form.name}
                      onChange={(e) =>
                        setForm((f) => ({ ...f, name: e.target.value }))
                      }
                      placeholder="Acme Corporation"
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
                    disabled={!form.id || !form.name}
                    data-testid="create-submit"
                  >
                    Create
                  </button>
                </div>
              </Dialog.Content>
            </Dialog.Portal>
          </Dialog.Root>
        }
      />

      {loading ? (
        <div style={{ display: "flex", justifyContent: "center", padding: 40 }}>
          <div className="loading-spinner" />
        </div>
      ) : tenants.length === 0 ? (
        <div className="empty-state">
          <div className="empty-state-icon">🏢</div>
          <div className="empty-state-title">No tenants</div>
          <div className="empty-state-description">
            Create tenants to test multi-tenant authentication flows.
          </div>
        </div>
      ) : (
        <div className="card">
          <div className="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Name</th>
                  <th>Domains</th>
                  <th>Auth Type</th>
                  <th style={{ width: 60 }}></th>
                </tr>
              </thead>
              <tbody>
                {tenants.map((t) => (
                  <tr
                    key={t.id}
                    id={`tenant-row-${t.id}`}
                    data-testid={`tenant-row-${t.id}`}
                  >
                    <td>
                      <code
                        style={{
                          fontSize: "var(--text-sm)",
                          color: "var(--color-accent)",
                        }}
                      >
                        {t.id}
                      </code>
                    </td>
                    <td>
                      <strong>{t.name}</strong>
                    </td>
                    <td>
                      {t.domains.length === 0 ? (
                        <span
                          style={{
                            color: "var(--color-text-tertiary)",
                            fontSize: "var(--text-sm)",
                          }}
                        >
                          —
                        </span>
                      ) : (
                        <div
                          style={{ display: "flex", flexWrap: "wrap", gap: 4 }}
                        >
                          {t.domains.map((d) => (
                            <span key={d} className="badge badge-neutral">
                              {d}
                            </span>
                          ))}
                        </div>
                      )}
                    </td>
                    <td>
                      <span
                        className={`badge ${t.authType === "none" ? "badge-neutral" : "badge-info"}`}
                      >
                        {t.authType}
                      </span>
                    </td>
                    <td>
                      <button
                        className="btn btn-ghost btn-icon btn-sm"
                        onClick={() => handleDelete(t.id)}
                        title="Delete tenant"
                        data-testid="delete-tenant-btn"
                      >
                        🗑️
                      </button>
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
