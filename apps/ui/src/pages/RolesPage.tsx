import { api, type Permission, type Role } from "../lib/api";
import { Layout } from "../components/Layout";
import { PageHeader } from "../components/PageHeader";
import { useEffect, useState, useCallback } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import * as Tabs from "@radix-ui/react-tabs";

// ── Shared Dialog ──────────────────────────────────────────────────────────────

function ConfirmDeleteDialog({
  open,
  onClose,
  onConfirm,
  label,
}: {
  open: boolean;
  onClose: () => void;
  onConfirm: () => void;
  label: string;
}) {
  return (
    <Dialog.Root open={open} onOpenChange={(o) => !o && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="dialog-overlay" />
        <Dialog.Content className="dialog-content">
          <div className="dialog-header">
            <Dialog.Title className="dialog-title">
              Delete {label}?
            </Dialog.Title>
            <Dialog.Close className="btn btn-ghost btn-icon">✕</Dialog.Close>
          </div>
          <div className="dialog-body">
            <p
              style={{
                fontSize: "var(--text-sm)",
                color: "var(--color-text-secondary)",
              }}
            >
              This action cannot be undone.
            </p>
          </div>
          <div className="dialog-footer">
            <button className="btn btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button
              className="btn btn-danger"
              onClick={() => {
                onConfirm();
                onClose();
              }}
              data-testid="confirm-delete-btn"
            >
              Delete
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

// ── Permissions Tab ────────────────────────────────────────────────────────────

function PermissionsTab() {
  const [permissions, setPermissions] = useState<Permission[]>([]);
  const [loading, setLoading] = useState(true);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Permission | null>(null);
  const [form, setForm] = useState({ name: "", description: "" });

  const load = useCallback(async () => {
    try {
      const { permissions } = await api.permissions.list();
      setPermissions(permissions);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const handleCreate = async () => {
    await api.permissions.create(form.name, form.description);
    setForm({ name: "", description: "" });
    setDialogOpen(false);
    load();
  };

  const handleDelete = async (p: Permission) => {
    await api.permissions.delete(p.name);
    load();
  };

  return (
    <div>
      <div
        style={{
          display: "flex",
          justifyContent: "flex-end",
          marginBottom: 16,
        }}
      >
        <Dialog.Root open={dialogOpen} onOpenChange={setDialogOpen}>
          <Dialog.Trigger asChild>
            <button
              className="btn btn-primary"
              id="create-permission-btn"
              data-testid="create-permission-btn"
            >
              + Add Permission
            </button>
          </Dialog.Trigger>
          <Dialog.Portal>
            <Dialog.Overlay className="dialog-overlay" />
            <Dialog.Content className="dialog-content">
              <div className="dialog-header">
                <Dialog.Title className="dialog-title">
                  Add Permission
                </Dialog.Title>
                <Dialog.Close className="btn btn-ghost btn-icon">
                  ✕
                </Dialog.Close>
              </div>
              <div className="dialog-body">
                <div className="form-group">
                  <label className="form-label" htmlFor="perm-name">
                    Name
                  </label>
                  <input
                    id="perm-name"
                    className="input"
                    data-testid="perm-name-input"
                    value={form.name}
                    onChange={(e) =>
                      setForm((f) => ({ ...f, name: e.target.value }))
                    }
                    placeholder="read:all"
                  />
                </div>
                <div className="form-group">
                  <label className="form-label" htmlFor="perm-desc">
                    Description
                  </label>
                  <input
                    id="perm-desc"
                    className="input"
                    data-testid="perm-desc-input"
                    value={form.description}
                    onChange={(e) =>
                      setForm((f) => ({ ...f, description: e.target.value }))
                    }
                    placeholder="Can read all resources"
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
                  data-testid="create-submit"
                >
                  Create
                </button>
              </div>
            </Dialog.Content>
          </Dialog.Portal>
        </Dialog.Root>
      </div>

      {loading ? (
        <div style={{ display: "flex", justifyContent: "center", padding: 40 }}>
          <div className="loading-spinner" />
        </div>
      ) : permissions.length === 0 ? (
        <div className="empty-state">
          <div className="empty-state-icon">🔓</div>
          <div className="empty-state-title">No permissions</div>
          <div className="empty-state-description">
            Create a permission to define what actions users can perform.
          </div>
        </div>
      ) : (
        <div className="card">
          <div className="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Description</th>
                  <th style={{ width: 60 }}></th>
                </tr>
              </thead>
              <tbody>
                {permissions.map((p) => (
                  <tr key={p.id}>
                    <td>
                      <span className="badge badge-primary">{p.name}</span>
                    </td>
                    <td style={{ color: "var(--color-text-secondary)" }}>
                      {p.description || "—"}
                    </td>
                    <td>
                      <button
                        className="btn btn-ghost btn-icon btn-sm"
                        onClick={() => setDeleteTarget(p)}
                        title="Delete"
                        data-testid="delete-btn"
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

      <ConfirmDeleteDialog
        open={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        onConfirm={() => deleteTarget && handleDelete(deleteTarget)}
        label={`"${deleteTarget?.name}"`}
      />
    </div>
  );
}

// ── Roles Tab ──────────────────────────────────────────────────────────────────

function RolesTab() {
  const [roles, setRoles] = useState<Role[]>([]);
  const [permissions, setPermissions] = useState<Permission[]>([]);
  const [loading, setLoading] = useState(true);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Role | null>(null);
  const [form, setForm] = useState({
    name: "",
    description: "",
    permissionNames: [] as string[],
  });

  const load = useCallback(async () => {
    try {
      const [r, p] = await Promise.all([
        api.roles.list(),
        api.permissions.list(),
      ]);
      setRoles(r.roles);
      setPermissions(p.permissions);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const togglePerm = (name: string) => {
    setForm((f) => ({
      ...f,
      permissionNames: f.permissionNames.includes(name)
        ? f.permissionNames.filter((p) => p !== name)
        : [...f.permissionNames, name],
    }));
  };

  const handleCreate = async () => {
    await api.roles.create(form.name, form.description, form.permissionNames);
    setForm({ name: "", description: "", permissionNames: [] });
    setDialogOpen(false);
    load();
  };

  return (
    <div>
      <div
        style={{
          display: "flex",
          justifyContent: "flex-end",
          marginBottom: 16,
        }}
      >
        <Dialog.Root open={dialogOpen} onOpenChange={setDialogOpen}>
          <Dialog.Trigger asChild>
            <button
              className="btn btn-primary"
              id="create-role-btn"
              data-testid="create-role-btn"
            >
              + Add Role
            </button>
          </Dialog.Trigger>
          <Dialog.Portal>
            <Dialog.Overlay className="dialog-overlay" />
            <Dialog.Content className="dialog-content">
              <div className="dialog-header">
                <Dialog.Title className="dialog-title">Add Role</Dialog.Title>
                <Dialog.Close className="btn btn-ghost btn-icon">
                  ✕
                </Dialog.Close>
              </div>
              <div className="dialog-body">
                <div className="form-group">
                  <label className="form-label" htmlFor="role-name">
                    Name
                  </label>
                  <input
                    id="role-name"
                    className="input"
                    data-testid="role-name-input"
                    value={form.name}
                    onChange={(e) =>
                      setForm((f) => ({ ...f, name: e.target.value }))
                    }
                    placeholder="Admin"
                  />
                </div>
                <div className="form-group">
                  <label className="form-label" htmlFor="role-desc">
                    Description
                  </label>
                  <input
                    id="role-desc"
                    className="input"
                    data-testid="role-desc-input"
                    value={form.description}
                    onChange={(e) =>
                      setForm((f) => ({ ...f, description: e.target.value }))
                    }
                    placeholder="Full access"
                  />
                </div>
                {permissions.length > 0 && (
                  <div className="form-group">
                    <label className="form-label">Permissions</label>
                    <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
                      {permissions.map((p) => (
                        <label
                          key={p.id}
                          style={{
                            display: "flex",
                            alignItems: "center",
                            gap: 6,
                            cursor: "pointer",
                          }}
                        >
                          <input
                            type="checkbox"
                            checked={form.permissionNames.includes(p.name)}
                            onChange={() => togglePerm(p.name)}
                          />
                          <span style={{ fontSize: "var(--text-sm)" }}>
                            {p.name}
                          </span>
                        </label>
                      ))}
                    </div>
                  </div>
                )}
              </div>
              <div className="dialog-footer">
                <Dialog.Close asChild>
                  <button className="btn btn-secondary">Cancel</button>
                </Dialog.Close>
                <button
                  className="btn btn-primary"
                  onClick={handleCreate}
                  data-testid="create-submit"
                >
                  Create
                </button>
              </div>
            </Dialog.Content>
          </Dialog.Portal>
        </Dialog.Root>
      </div>

      {loading ? (
        <div style={{ display: "flex", justifyContent: "center", padding: 40 }}>
          <div className="loading-spinner" />
        </div>
      ) : roles.length === 0 ? (
        <div className="empty-state">
          <div className="empty-state-icon">🛡️</div>
          <div className="empty-state-title">No roles</div>
          <div className="empty-state-description">
            Create roles to group permissions and assign them to users.
          </div>
        </div>
      ) : (
        <div className="card">
          <div className="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Permissions</th>
                  <th>Flags</th>
                  <th style={{ width: 60 }}></th>
                </tr>
              </thead>
              <tbody>
                {roles.map((r) => (
                  <tr key={r.id}>
                    <td>
                      <strong>{r.name}</strong>
                    </td>
                    <td>
                      <div
                        style={{ display: "flex", flexWrap: "wrap", gap: 4 }}
                      >
                        {r.permissionNames.length === 0 ? (
                          <span
                            style={{
                              color: "var(--color-text-tertiary)",
                              fontSize: "var(--text-xs)",
                            }}
                          >
                            —
                          </span>
                        ) : (
                          r.permissionNames.map((pn) => (
                            <span key={pn} className="badge badge-neutral">
                              {pn}
                            </span>
                          ))
                        )}
                      </div>
                    </td>
                    <td>
                      <div style={{ display: "flex", gap: 4 }}>
                        {r.isDefault && (
                          <span className="badge badge-primary">default</span>
                        )}
                        {r.isHidden && (
                          <span className="badge badge-neutral">hidden</span>
                        )}
                      </div>
                    </td>
                    <td>
                      <button
                        className="btn btn-ghost btn-icon btn-sm"
                        onClick={() => setDeleteTarget(r)}
                        title="Delete"
                        data-testid="delete-btn"
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

      <ConfirmDeleteDialog
        open={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        onConfirm={async () => {
          if (deleteTarget) {
            await api.roles.delete(deleteTarget.name);
            load();
          }
        }}
        label={`role "${deleteTarget?.name}"`}
      />
    </div>
  );
}

// ── Page ───────────────────────────────────────────────────────────────────────

export default function RolesPage() {
  return (
    <Layout title="Authorization">
      <PageHeader
        title="Roles & Permissions"
        description="Manage RBAC permissions and roles for your project"
      />
      <Tabs.Root defaultValue="permissions">
        <div className="tabs-list">
          <Tabs.List style={{ display: "contents" }}>
            <Tabs.Trigger className="tab-trigger" value="permissions">
              Permissions
            </Tabs.Trigger>
            <Tabs.Trigger className="tab-trigger" value="roles">
              Roles
            </Tabs.Trigger>
          </Tabs.List>
        </div>
        <Tabs.Content value="permissions">
          <PermissionsTab />
        </Tabs.Content>
        <Tabs.Content value="roles">
          <RolesTab />
        </Tabs.Content>
      </Tabs.Root>
    </Layout>
  );
}
