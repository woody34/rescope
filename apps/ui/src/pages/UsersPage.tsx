/**
 * UsersPage — main Users page with sub-tabs for Users list and Custom Attributes.
 * Routes: /users (user list), /users/attributes (attribute definitions)
 */
import { api, type User, type CustomAttribute } from "../lib/api";
import { Layout } from "../components/Layout";
import { PageHeader } from "../components/PageHeader";
import { UserModal } from "../components/UserModal";
import {
  ColumnPicker,
  type ColumnDef,
} from "../components/ColumnPicker";
import { useColumnPreferences } from "../hooks/useColumnPreferences";
import { useEffect, useState, useCallback, useRef } from "react";
import { NavLink, Outlet, useLocation } from "react-router-dom";

function formatDate(ts: number): string {
  if (!ts) return "—";
  return new Date(ts * 1000).toLocaleDateString();
}

// ── Column definitions ────────────────────────────────────────────────────────

const USER_COLUMNS: ColumnDef[] = [
  { key: "loginId", label: "Login ID" },
  { key: "status", label: "Status" },
  { key: "name", label: "Display Name" },
  { key: "email", label: "Email" },
  { key: "phone", label: "Phone" },
  { key: "verified", label: "Verified" },
  { key: "roles", label: "Roles" },
  { key: "tenants", label: "Tenants" },
  { key: "created", label: "Created" },
  { key: "lastLogin", label: "Last Login" },
];

const DEFAULT_VISIBLE = [
  "loginId",
  "status",
  "name",
  "email",
  "phone",
  "created",
];

// ── Sub-tab wrapper (shared layout with tab bar) ──────────────────────────────

export default function UsersPage() {
  const location = useLocation();
  const isAttributes = location.pathname.startsWith("/users/attributes");

  return (
    <Layout title="Users">
      <PageHeader
        title="Users"
        description="Manage test users for your emulator project"
      />

      {/* Tab bar */}
      <div
        data-testid="users-tab-bar"
        style={{
          display: "flex",
          gap: 0,
          borderBottom: "2px solid var(--color-neutral-200)",
          marginBottom: "var(--space-4)",
        }}
      >
        <NavLink
          to="/users"
          end
          data-testid="tab-users"
          className={() => ""}
          style={({ isActive }: { isActive: boolean }) => ({
            padding: "var(--space-2) var(--space-4)",
            fontSize: "var(--text-sm)",
            fontWeight: "var(--fw-medium)",
            color: isActive && !isAttributes
              ? "var(--color-primary)"
              : "var(--color-text-secondary)",
            borderBottom: isActive && !isAttributes
              ? "2px solid var(--color-primary)"
              : "2px solid transparent",
            marginBottom: "-2px",
            cursor: "pointer",
            textDecoration: "none",
          })}
        >
          Users
        </NavLink>
        <NavLink
          to="/users/attributes"
          data-testid="tab-attributes"
          className={() => ""}
          style={({ isActive }: { isActive: boolean }) => ({
            padding: "var(--space-2) var(--space-4)",
            fontSize: "var(--text-sm)",
            fontWeight: "var(--fw-medium)",
            color: isActive
              ? "var(--color-primary)"
              : "var(--color-text-secondary)",
            borderBottom: isActive
              ? "2px solid var(--color-primary)"
              : "2px solid transparent",
            marginBottom: "-2px",
            cursor: "pointer",
            textDecoration: "none",
          })}
        >
          Custom Attributes
        </NavLink>
      </div>

      <Outlet />
    </Layout>
  );
}

// ── Users list tab ────────────────────────────────────────────────────────────

// ── Export helpers ─────────────────────────────────────────────────────────────

function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

function usersToJson(users: User[]): string {
  return JSON.stringify(users, null, 2);
}

function usersToCsv(users: User[], columns: ColumnDef[], visibleKeys: string[]): string {
  const cols = columns.filter((c) => visibleKeys.includes(c.key));
  const header = cols.map((c) => `"${c.label}"`).join(",");
  const rows = users.map((u) =>
    cols
      .map((c) => {
        let val = "";
        switch (c.key) {
          case "loginId": val = u.loginIds.join("; "); break;
          case "status": val = u.status; break;
          case "name": val = u.name ?? ""; break;
          case "email": val = u.email ?? ""; break;
          case "phone": val = u.phone ?? ""; break;
          case "verified":
            val = [u.verifiedEmail && "email", u.verifiedPhone && "phone"].filter(Boolean).join(", ");
            break;
          case "roles": val = (u.roleNames ?? []).join(", "); break;
          case "tenants": val = (u.userTenants ?? []).map((t) => t.tenantName || t.tenantId).join(", "); break;
          case "created": val = formatDate(u.createdTime); break;
          case "lastLogin": val = u.lastLogin ? formatDate(u.lastLogin) : ""; break;
          default:
            if (c.key.startsWith("ca:")) {
              const v = u.customAttributes?.[c.key.slice(3)];
              val = v != null ? String(v) : "";
            }
        }
        return `"${val.replace(/"/g, '""')}"`;
      })
      .join(","),
  );
  return [header, ...rows].join("\n");
}

export function UsersListTab() {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [modalOpen, setModalOpen] = useState(false);
  const [editingUser, setEditingUser] = useState<User | null>(null);
  const [customAttrDefs, setCustomAttrDefs] = useState<CustomAttribute[]>([]);

  // Filter state
  const [statusFilter, setStatusFilter] = useState("");
  const [tenantFilter, setTenantFilter] = useState("");
  const [roleFilter, setRoleFilter] = useState("");
  const [tenantOptions, setTenantOptions] = useState<{ id: string; name: string }[]>([]);
  const [roleOptions, setRoleOptions] = useState<string[]>([]);

  // Bulk selection state
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [bulkLoading, setBulkLoading] = useState(false);

  // Debounce ref for text search
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  // Build column list: core columns + dynamic custom attribute columns
  const allColumns: ColumnDef[] = [
    ...USER_COLUMNS,
    ...customAttrDefs.map((a) => ({
      key: `ca:${a.machineName}`,
      label: a.name,
    })),
  ];

  const [visibleColumns, setVisibleColumns] = useColumnPreferences(
    "rescope:users:visibleColumns",
    DEFAULT_VISIBLE,
  );

  const loadUsers = useCallback(async (opts?: {
    text?: string;
    statuses?: string[];
    tenantIds?: string[];
    roleNames?: string[];
  }) => {
    setLoading(true);
    try {
      const params: Record<string, unknown> = { withTestUser: true };
      if (opts?.text) params.text = opts.text;
      if (opts?.statuses?.length) params.statuses = opts.statuses;
      if (opts?.tenantIds?.length) params.tenantIds = opts.tenantIds;
      if (opts?.roleNames?.length) params.roleNames = opts.roleNames;
      const usersRes = await api.users.search(params as Parameters<typeof api.users.search>[0]);
      setUsers(usersRes.users);
    } finally {
      setLoading(false);
    }
  }, []);

  const currentFilters = useCallback(() => ({
    text: search || undefined,
    statuses: statusFilter ? [statusFilter] : undefined,
    tenantIds: tenantFilter ? [tenantFilter] : undefined,
    roleNames: roleFilter ? [roleFilter] : undefined,
  }), [search, statusFilter, tenantFilter, roleFilter]);

  // Initial load: users + tenants + roles + custom attrs
  useEffect(() => {
    loadUsers();
    api.customAttributes.list().then((r) => setCustomAttrDefs(r.attributes));
    api.tenants.list().then((r) =>
      setTenantOptions((r.tenants ?? []).map((t: { id: string; name: string }) => ({ id: t.id, name: t.name }))),
    );
    api.roles.list().then((r) =>
      setRoleOptions((r.roles ?? []).map((rl: { name: string }) => rl.name)),
    );
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Re-fetch when dropdown filters change
  useEffect(() => {
    loadUsers(currentFilters());
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [statusFilter, tenantFilter, roleFilter]);

  // Debounced text search
  const handleSearchChange = (value: string) => {
    setSearch(value);
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      loadUsers({
        text: value || undefined,
        statuses: statusFilter ? [statusFilter] : undefined,
        tenantIds: tenantFilter ? [tenantFilter] : undefined,
        roleNames: roleFilter ? [roleFilter] : undefined,
      });
    }, 300);
  };

  // Reload helper used by modal save, delete, etc.
  const reload = useCallback(() => {
    loadUsers(currentFilters());
  }, [loadUsers, currentFilters]);

  const handleDelete = async (loginId: string) => {
    if (!confirm(`Delete user ${loginId}?`)) return;
    await api.users.delete(loginId);
    reload();
  };

  const openCreate = () => {
    setEditingUser(null);
    setModalOpen(true);
  };

  const openEdit = (user: User) => {
    setEditingUser(user);
    setModalOpen(true);
  };

  // ── Bulk operations ───────────────────────────────────────────────────────

  const toggleSelect = (userId: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(userId)) next.delete(userId);
      else next.add(userId);
      return next;
    });
  };

  const toggleSelectAll = () => {
    if (selectedIds.size === users.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(users.map((u) => u.userId)));
    }
  };

  const clearSelection = () => setSelectedIds(new Set());

  const bulkSetStatus = async (status: "enabled" | "disabled") => {
    if (!confirm(`${status === "enabled" ? "Enable" : "Disable"} ${selectedIds.size} user(s)?`)) return;
    setBulkLoading(true);
    try {
      const selected = users.filter((u) => selectedIds.has(u.userId));
      await Promise.all(selected.map((u) => api.users.setStatus(u.loginIds[0], status)));
      clearSelection();
      reload();
    } finally {
      setBulkLoading(false);
    }
  };

  const bulkDelete = async () => {
    if (!confirm(`Delete ${selectedIds.size} user(s)? This cannot be undone.`)) return;
    setBulkLoading(true);
    try {
      const selected = users.filter((u) => selectedIds.has(u.userId));
      await Promise.all(selected.map((u) => api.users.delete(u.loginIds[0])));
      clearSelection();
      reload();
    } finally {
      setBulkLoading(false);
    }
  };

  // ── Export ─────────────────────────────────────────────────────────────────

  const exportJson = () => {
    const blob = new Blob([usersToJson(users)], { type: "application/json" });
    downloadBlob(blob, "rescope-users.json");
  };

  const exportCsv = () => {
    const blob = new Blob([usersToCsv(users, allColumns, visibleColumns)], { type: "text/csv" });
    downloadBlob(blob, "rescope-users.csv");
  };

  const isVisible = (key: string) => visibleColumns.includes(key);

  // Render a cell value
  const renderCell = (u: User, key: string) => {
    if (!key) return null;
    switch (key) {
      case "loginId":
        return (
          <div>
            <div
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "var(--text-sm)",
              }}
            >
              {u.loginIds[0]}
            </div>
            {u.loginIds.length > 1 && (
              <div
                style={{
                  color: "var(--color-text-tertiary)",
                  fontSize: "var(--text-xs)",
                }}
              >
                +{u.loginIds.length - 1} more
              </div>
            )}
          </div>
        );
      case "name":
        return (
          u.name || (
            <span style={{ color: "var(--color-text-tertiary)" }}>—</span>
          )
        );
      case "status":
        return (
          <span
            className={`badge ${u.status === "enabled" ? "badge-success" : "badge-neutral"}`}
          >
            {u.status}
          </span>
        );
      case "email":
        return (
          u.email || (
            <span style={{ color: "var(--color-text-tertiary)" }}>—</span>
          )
        );
      case "phone":
        return (
          u.phone || (
            <span style={{ color: "var(--color-text-tertiary)" }}>—</span>
          )
        );
      case "verified":
        return (
          <div style={{ display: "flex", gap: 4, flexWrap: "wrap" }}>
            {u.verifiedEmail && (
              <span className="badge badge-info">email</span>
            )}
            {u.verifiedPhone && (
              <span className="badge badge-info">phone</span>
            )}
            {!u.verifiedEmail && !u.verifiedPhone && (
              <span
                style={{
                  color: "var(--color-text-tertiary)",
                  fontSize: "var(--text-xs)",
                }}
              >
                —
              </span>
            )}
          </div>
        );
      case "roles":
        return (
          <div style={{ display: "flex", gap: 4, flexWrap: "wrap" }}>
            {(u.roleNames ?? []).map((r) => (
              <span key={r} className="badge badge-primary">
                {r}
              </span>
            ))}
            {(!u.roleNames || u.roleNames.length === 0) && (
              <span style={{ color: "var(--color-text-tertiary)", fontSize: "var(--text-xs)" }}>—</span>
            )}
          </div>
        );
      case "tenants":
        return (
          <div style={{ display: "flex", gap: 4, flexWrap: "wrap" }}>
            {(u.userTenants ?? []).map((t) => (
              <span key={t.tenantId} className="badge badge-info">
                {t.tenantName || t.tenantId}
              </span>
            ))}
            {(!u.userTenants || u.userTenants.length === 0) && (
              <span style={{ color: "var(--color-text-tertiary)", fontSize: "var(--text-xs)" }}>—</span>
            )}
          </div>
        );
      case "created":
        return (
          <span
            style={{
              color: "var(--color-text-secondary)",
              fontSize: "var(--text-sm)",
            }}
          >
            {formatDate(u.createdTime)}
          </span>
        );
      case "lastLogin":
        return (
          <span
            style={{
              color: "var(--color-text-secondary)",
              fontSize: "var(--text-sm)",
            }}
          >
            {u.lastLogin ? formatDate(u.lastLogin) : "—"}
          </span>
        );
      default: {
        // Custom attribute columns use "ca:machine_name" key format
        if (key.startsWith("ca:")) {
          const machineName = key.slice(3);
          const val = u.customAttributes?.[machineName];
          if (val === undefined || val === null) {
            return <span style={{ color: "var(--color-text-tertiary)" }}>—</span>;
          }
          if (typeof val === "boolean") {
            return <span className={`badge ${val ? "badge-success" : "badge-neutral"}`}>{val ? "Yes" : "No"}</span>;
          }
          return <span style={{ fontSize: "var(--text-sm)" }}>{String(val)}</span>;
        }
        return null;
      }
    }
  };

  return (
    <>
      <UserModal
        open={modalOpen}
        onOpenChange={setModalOpen}
        user={editingUser}
        onSaved={reload}
      />

      {/* Search + filter bar */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "var(--space-3)",
          marginBottom: "var(--space-3)",
          flexWrap: "wrap",
        }}
      >
        <input
          id="user-search"
          className="input"
          data-testid="user-search"
          style={{ maxWidth: 280, minWidth: 180 }}
          value={search}
          onChange={(e) => handleSearchChange(e.target.value)}
          placeholder="Search users…"
        />
        <select
          className="input"
          data-testid="status-filter"
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
          style={{ maxWidth: 140, fontSize: "var(--text-sm)" }}
        >
          <option value="">All Statuses</option>
          <option value="enabled">Enabled</option>
          <option value="disabled">Disabled</option>
          <option value="invited">Invited</option>
        </select>
        {tenantOptions.length > 0 && (
          <select
            className="input"
            data-testid="tenant-filter"
            value={tenantFilter}
            onChange={(e) => setTenantFilter(e.target.value)}
            style={{ maxWidth: 160, fontSize: "var(--text-sm)" }}
          >
            <option value="">All Tenants</option>
            {tenantOptions.map((t) => (
              <option key={t.id} value={t.id}>
                {t.name || t.id}
              </option>
            ))}
          </select>
        )}
        {roleOptions.length > 0 && (
          <select
            className="input"
            data-testid="role-filter"
            value={roleFilter}
            onChange={(e) => setRoleFilter(e.target.value)}
            style={{ maxWidth: 140, fontSize: "var(--text-sm)" }}
          >
            <option value="">All Roles</option>
            {roleOptions.map((r) => (
              <option key={r} value={r}>
                {r}
              </option>
            ))}
          </select>
        )}
        <div style={{ flex: 1 }} />

        {/* Export dropdown */}
        <div style={{ position: "relative", display: "inline-block" }}>
          <button
            className="btn btn-ghost"
            data-testid="export-btn"
            onClick={(e) => {
              const menu = (e.currentTarget.nextElementSibling as HTMLElement);
              menu.style.display = menu.style.display === "block" ? "none" : "block";
            }}
          >
            ↓ Export
          </button>
          <div
            data-testid="export-menu"
            style={{
              display: "none",
              position: "absolute",
              right: 0,
              top: "100%",
              zIndex: 10,
              background: "var(--color-bg-elevated)",
              border: "1px solid var(--color-border)",
              borderRadius: "var(--radius-md)",
              boxShadow: "var(--shadow-md)",
              minWidth: 120,
              padding: "var(--space-1) 0",
            }}
          >
            <button
              className="btn btn-ghost"
              data-testid="export-json-btn"
              style={{ width: "100%", justifyContent: "flex-start", borderRadius: 0 }}
              onClick={() => exportJson()}
            >
              JSON
            </button>
            <button
              className="btn btn-ghost"
              data-testid="export-csv-btn"
              style={{ width: "100%", justifyContent: "flex-start", borderRadius: 0 }}
              onClick={() => exportCsv()}
            >
              CSV
            </button>
          </div>
        </div>

        <ColumnPicker
          columns={allColumns}
          visibleKeys={visibleColumns}
          onChange={setVisibleColumns}
        />
        <button
          className="btn btn-primary"
          id="create-user-btn"
          data-testid="create-user-btn"
          onClick={openCreate}
        >
          + Create User
        </button>
      </div>

      {/* Bulk action toolbar */}
      {selectedIds.size > 0 && (
        <div
          data-testid="bulk-toolbar"
          style={{
            display: "flex",
            alignItems: "center",
            gap: "var(--space-3)",
            marginBottom: "var(--space-3)",
            padding: "var(--space-2) var(--space-4)",
            background: "var(--color-bg-elevated)",
            border: "1px solid var(--color-primary)",
            borderRadius: "var(--radius-md)",
            fontSize: "var(--text-sm)",
          }}
        >
          <span data-testid="bulk-count" style={{ fontWeight: "var(--fw-medium)" }}>
            {selectedIds.size} selected
          </span>
          <button
            className="btn btn-ghost btn-sm"
            data-testid="bulk-enable-btn"
            disabled={bulkLoading}
            onClick={() => bulkSetStatus("enabled")}
          >
            Enable
          </button>
          <button
            className="btn btn-ghost btn-sm"
            data-testid="bulk-disable-btn"
            disabled={bulkLoading}
            onClick={() => bulkSetStatus("disabled")}
          >
            Disable
          </button>
          <button
            className="btn btn-ghost btn-sm"
            data-testid="bulk-delete-btn"
            disabled={bulkLoading}
            style={{ color: "var(--color-danger)" }}
            onClick={bulkDelete}
          >
            Delete
          </button>
          <div style={{ flex: 1 }} />
          <button
            className="btn btn-ghost btn-sm"
            data-testid="bulk-deselect-btn"
            onClick={clearSelection}
          >
            Deselect All
          </button>
        </div>
      )}

      {loading ? (
        <div style={{ display: "flex", justifyContent: "center", padding: 40 }}>
          <div className="loading-spinner" />
        </div>
      ) : users.length === 0 ? (
        <div className="empty-state">
          <div className="empty-state-icon">👤</div>
          <div className="empty-state-title">
            {search || statusFilter || tenantFilter || roleFilter
              ? "No matching users"
              : "No users yet"}
          </div>
          <div className="empty-state-description">
            {search || statusFilter || tenantFilter || roleFilter
              ? "Try adjusting your filters."
              : "Create a user to get started."}
          </div>
        </div>
      ) : (
        <div className="card">
          <div className="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th style={{ width: 36, textAlign: "center" }}>
                    <input
                      type="checkbox"
                      data-testid="select-all-checkbox"
                      checked={users.length > 0 && selectedIds.size === users.length}
                      onChange={toggleSelectAll}
                      style={{ cursor: "pointer" }}
                    />
                  </th>
                  {allColumns.filter((c) => isVisible(c.key)).map((c) => (
                    <th key={c.key}>{c.label}</th>
                  ))}
                  <th style={{ width: 60 }}></th>
                </tr>
              </thead>
              <tbody>
                {users.map((u: User) => (
                  <tr
                    key={u.userId}
                    id={`user-row-${u.loginIds[0]}`}
                    data-testid={`user-row-${u.loginIds[0]}`}
                    onClick={() => openEdit(u)}
                    style={{
                      cursor: "pointer",
                      background: selectedIds.has(u.userId) ? "var(--color-primary-alpha-10, rgba(0,255,136,0.07))" : undefined,
                    }}
                  >
                    <td style={{ textAlign: "center" }}>
                      <input
                        type="checkbox"
                        data-testid={`select-user-${u.loginIds[0]}`}
                        checked={selectedIds.has(u.userId)}
                        onClick={(e) => e.stopPropagation()}
                        onChange={() => toggleSelect(u.userId)}
                        style={{ cursor: "pointer" }}
                      />
                    </td>
                    {allColumns.filter((c) => isVisible(c.key)).map((c) => (
                      <td key={c.key}>{renderCell(u, c.key)}</td>
                    ))}
                    <td>
                      <button
                        className="btn btn-ghost btn-icon btn-sm"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDelete(u.loginIds[0]);
                        }}
                        title="Delete user"
                        data-testid="delete-user-btn"
                      >
                        <svg
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="1.75"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          style={{ width: 14, height: 14 }}
                        >
                          <polyline points="3 6 5 6 21 6" />
                          <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
                          <path d="M10 11v6M14 11v6" />
                          <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
                        </svg>
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div
            style={{
              padding: "var(--space-3) var(--space-4)",
              borderTop: "1px solid var(--color-border)",
              color: "var(--color-text-secondary)",
              fontSize: "var(--text-sm)",
            }}
          >
            {users.length} user{users.length !== 1 ? "s" : ""}
          </div>
        </div>
      )}
    </>
  );
}
