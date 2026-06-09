/**
 * UserModal — unified create / edit dialog for users.
 *
 * - Create mode (no `user` prop): empty form, Login ID editable
 * - Edit mode (`user` prop): pre-filled from existing user, Login ID read-only
 * - Status toggle in header (edit mode only)
 * - Custom attribute values (type-appropriate inputs)
 * - Tenant + role assignment
 * - Saves via diff → patch in edit mode
 */
import { useEffect, useState, useCallback, useMemo } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { api, type User, type CustomAttribute, type Tenant, type Role } from "../lib/api";

interface UserForm {
  loginId: string;
  name: string;
  givenName: string;
  familyName: string;
  email: string;
  phone: string;
}

interface TenantAssignment {
  tenantId: string;
  roleNames: string[];
}

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  user?: User | null;
  onSaved: () => void;
}

function computeDiff(
  original: UserForm,
  current: UserForm,
): Partial<UserForm> | null {
  const diff: Partial<UserForm> = {};
  for (const key of Object.keys(original) as (keyof UserForm)[]) {
    if (key === "loginId") continue;
    if (original[key] !== current[key]) {
      diff[key] = current[key];
    }
  }
  return Object.keys(diff).length > 0 ? diff : null;
}

export function UserModal({ open, onOpenChange, user, onSaved }: Props) {
  const isEdit = !!user;

  // ── Basic fields ──────────────────────────────────────────────────────────
  const emptyForm: UserForm = {
    loginId: "",
    name: "",
    givenName: "",
    familyName: "",
    email: "",
    phone: "",
  };

  const [form, setForm] = useState<UserForm>(emptyForm);
  const [originalForm, setOriginalForm] = useState<UserForm>(emptyForm);
  const [status, setStatus] = useState<"enabled" | "disabled">("enabled");
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [password, setPassword] = useState("");

  // ── Custom attributes ────────────────────────────────────────────────────
  const [attrDefs, setAttrDefs] = useState<CustomAttribute[]>([]);
  const [attrValues, setAttrValues] = useState<Record<string, unknown>>({});
  const [originalAttrValues, setOriginalAttrValues] = useState<Record<string, unknown>>({});

  // ── Tenants + roles ──────────────────────────────────────────────────────
  const [allTenants, setAllTenants] = useState<Tenant[]>([]);
  const [allRoles, setAllRoles] = useState<Role[]>([]);
  const [tenantAssignments, setTenantAssignments] = useState<TenantAssignment[]>([]);
  const [originalTenantAssignments, setOriginalTenantAssignments] = useState<TenantAssignment[]>([]);
  const [addingTenant, setAddingTenant] = useState(false);
  const [newTenantId, setNewTenantId] = useState("");
  const [newTenantRoles, setNewTenantRoles] = useState<string[]>([]);
  const [creatingTenant, setCreatingTenant] = useState(false);
  const [newTenantName, setNewTenantName] = useState("");
  const [newRoleName, setNewRoleName] = useState("");

  // ── Load reference data on open ──────────────────────────────────────────
  const loadRefData = useCallback(async () => {
    try {
      const [attrs, tenants, roles] = await Promise.all([
        api.customAttributes.list(),
        api.tenants.list(),
        api.roles.list(),
      ]);
      setAttrDefs(attrs.attributes ?? []);
      setAllTenants(tenants.tenants ?? []);
      setAllRoles(roles.roles ?? []);
    } catch {
      /* reference data unavailable — continue without */
    }
  }, []);

  useEffect(() => {
    if (!open) return;
    loadRefData();

    if (user) {
      const f: UserForm = {
        loginId: user.loginIds[0] ?? "",
        name: user.name ?? "",
        givenName: user.givenName ?? "",
        familyName: user.familyName ?? "",
        email: user.email ?? "",
        phone: user.phone ?? "",
      };
      setForm(f);
      setOriginalForm(f);
      setStatus(user.status === "disabled" ? "disabled" : "enabled");
      setAttrValues({ ...(user.customAttributes ?? {}) });
      setOriginalAttrValues({ ...(user.customAttributes ?? {}) });
      const ta = (user.userTenants ?? []).map((t) => ({
        tenantId: t.tenantId,
        roleNames: [...t.roleNames],
      }));
      setTenantAssignments(ta);
      setOriginalTenantAssignments(ta.map((a) => ({ ...a, roleNames: [...a.roleNames] })));
    } else {
      setForm(emptyForm);
      setOriginalForm(emptyForm);
      setStatus("enabled");
      setAttrValues({});
      setOriginalAttrValues({});
      setTenantAssignments([]);
      setOriginalTenantAssignments([]);
    }
    setError(null);
    setPassword("");
    setAddingTenant(false);
    setNewTenantId("");
    setNewTenantRoles([]);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, user]);

  // ── Status toggle ────────────────────────────────────────────────────────
  const handleToggleStatus = async () => {
    if (!user) return;
    const newStatus = status === "enabled" ? "disabled" : "enabled";
    try {
      await api.users.setStatus(user.loginIds[0], newStatus);
      setStatus(newStatus);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Failed to update status");
    }
  };

  // ── Tenant management ────────────────────────────────────────────────────
  const availableTenants = useMemo(
    () =>
      allTenants.filter(
        (t) => !tenantAssignments.some((a) => a.tenantId === t.id),
      ),
    [allTenants, tenantAssignments],
  );

  const handleAddTenantAssignment = () => {
    if (!newTenantId) return;
    setTenantAssignments((prev) => [
      ...prev,
      { tenantId: newTenantId, roleNames: [...newTenantRoles] },
    ]);
    setAddingTenant(false);
    setNewTenantId("");
    setNewTenantRoles([]);
    setCreatingTenant(false);
    setNewTenantName("");
  };

  const handleCreateTenantInline = async () => {
    if (!newTenantName.trim()) return;
    try {
      const id = newTenantName.trim().toLowerCase().replace(/[^a-z0-9]+/g, "-");
      await api.tenants.create(id, newTenantName.trim());
      const newTenant: Tenant = { id, name: newTenantName.trim(), domains: [], authType: "none" };
      setAllTenants((prev) => [...prev, newTenant]);
      // Auto-assign the newly created tenant
      setTenantAssignments((prev) => [...prev, { tenantId: id, roleNames: [] }]);
      setAddingTenant(false);
      setCreatingTenant(false);
      setNewTenantName("");
      setNewTenantId("");
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Failed to create tenant");
    }
  };

  const handleCreateRoleInline = async () => {
    if (!newRoleName.trim()) return;
    const name = newRoleName.trim();
    if (allRoles.some((r) => r.name === name)) {
      // Role already exists — just auto-assign it to the first tenant that doesn't have it
      setTenantAssignments((prev) =>
        prev.map((ta, i) =>
          i === 0 && !ta.roleNames.includes(name)
            ? { ...ta, roleNames: [...ta.roleNames, name] }
            : ta,
        ),
      );
      setNewRoleName("");
      return;
    }
    try {
      await api.roles.create(name, "", []);
      setAllRoles((prev) => [...prev, { id: name, name, description: "", permissionNames: [], isDefault: false, isHidden: false }]);
      // Auto-assign the new role to the first tenant assignment
      setTenantAssignments((prev) =>
        prev.map((ta, i) =>
          i === 0 ? { ...ta, roleNames: [...ta.roleNames, name] } : ta,
        ),
      );
      setNewRoleName("");
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Failed to create role");
    }
  };

  const handleRemoveTenantAssignment = (tenantId: string) => {
    setTenantAssignments((prev) => prev.filter((a) => a.tenantId !== tenantId));
  };

  const handleTenantRoleChange = (tenantId: string, roleNames: string[]) => {
    setTenantAssignments((prev) =>
      prev.map((a) =>
        a.tenantId === tenantId ? { ...a, roleNames } : a,
      ),
    );
  };

  // ── Save ─────────────────────────────────────────────────────────────────
  const handleSave = async () => {
    setError(null);
    setSaving(true);
    try {
      const loginId = form.loginId;

      if (isEdit) {
        // 1. Diff basic fields and patch
        const diff = computeDiff(originalForm, form);
        const attrDiff: Record<string, unknown> = {};
        for (const key of Object.keys(attrValues)) {
          if (JSON.stringify(attrValues[key]) !== JSON.stringify(originalAttrValues[key])) {
            attrDiff[key] = attrValues[key];
          }
        }
        for (const key of Object.keys(originalAttrValues)) {
          if (!(key in attrValues)) {
            attrDiff[key] = null;
          }
        }

        const hasFieldChanges = diff !== null;
        const hasAttrChanges = Object.keys(attrDiff).length > 0;

        if (hasFieldChanges || hasAttrChanges) {
          const patchBody: Record<string, unknown> = {};
          if (diff) Object.assign(patchBody, diff);
          if (hasAttrChanges) patchBody.customAttributes = { ...attrValues };
          await api.users.update(loginId, patchBody as Partial<User>);
        }

        // 2. Handle tenant changes
        const origTenantIds = new Set(originalTenantAssignments.map((a) => a.tenantId));
        const newTenantIds = new Set(tenantAssignments.map((a) => a.tenantId));

        // Remove old tenants
        for (const orig of originalTenantAssignments) {
          if (!newTenantIds.has(orig.tenantId)) {
            await api.users.removeTenant(loginId, orig.tenantId);
          }
        }
        // Add new tenants
        for (const ta of tenantAssignments) {
          if (!origTenantIds.has(ta.tenantId)) {
            await api.users.addTenant(loginId, ta.tenantId, ta.roleNames);
          } else {
            // Update roles if changed
            const orig = originalTenantAssignments.find((a) => a.tenantId === ta.tenantId);
            if (orig && JSON.stringify(orig.roleNames) !== JSON.stringify(ta.roleNames)) {
              await api.users.setTenantRoles(loginId, ta.tenantId, ta.roleNames);
            }
          }
        }

        // Set password if provided
        if (password) {
          await api.users.setPassword(loginId, password);
        }

        // No diff & no tenant changes & no password → no-op
        if (!hasFieldChanges && !hasAttrChanges && !password &&
            JSON.stringify(originalTenantAssignments) === JSON.stringify(tenantAssignments)) {
          onOpenChange(false);
          return;
        }
      } else {
        // Create mode
        const params: Record<string, unknown> = {
          loginIds: [loginId],
          name: form.name || undefined,
          givenName: form.givenName || undefined,
          familyName: form.familyName || undefined,
          email: form.email || undefined,
          phone: form.phone || undefined,
        };
        if (Object.keys(attrValues).length > 0) {
          params.customAttributes = attrValues;
        }
        await api.users.create(params as Partial<User> & { loginIds: string[] });

        // Set password if provided
        if (password) {
          await api.users.setPassword(loginId, password);
        }

        // Add tenant assignments after create
        for (const ta of tenantAssignments) {
          await api.users.addTenant(loginId, ta.tenantId, ta.roleNames);
        }
      }

      onSaved();
      onOpenChange(false);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Failed to save user");
    } finally {
      setSaving(false);
    }
  };

  // ── Render ───────────────────────────────────────────────────────────────
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="dialog-overlay" />
        <Dialog.Content className="dialog-content" style={{ width: 560 }}>
          <div className="dialog-header">
            <Dialog.Title className="dialog-title" data-testid="user-modal-title">
              {isEdit ? "Edit User" : "Create User"}
            </Dialog.Title>
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              {isEdit && (
                <button
                  className={`badge ${status === "enabled" ? "badge-success" : "badge-neutral"}`}
                  data-testid="status-toggle"
                  onClick={handleToggleStatus}
                  style={{ cursor: "pointer" }}
                  title={`Click to ${status === "enabled" ? "disable" : "enable"}`}
                >
                  {status}
                </button>
              )}
              <Dialog.Close className="btn btn-ghost btn-icon">✕</Dialog.Close>
            </div>
          </div>

          <div className="dialog-body">
            {error && (
              <div
                style={{
                  padding: "var(--space-3)",
                  background: "var(--color-danger-bg)",
                  borderRadius: "var(--radius-md)",
                  color: "var(--color-danger)",
                  fontSize: "var(--text-sm)",
                }}
              >
                {error}
              </div>
            )}

            {/* ── Basic fields ──────────────────────────────────────── */}
            <div className="form-group">
              <label className="form-label" htmlFor="user-login-id">
                Login ID <span style={{ color: "var(--color-danger)" }}>*</span>
              </label>
              <input
                id="user-login-id"
                className="input"
                data-testid="user-login-id"
                value={form.loginId}
                onChange={(e) =>
                  setForm((f) => ({ ...f, loginId: e.target.value }))
                }
                placeholder="email@example.com or +15551234567"
                disabled={isEdit}
              />
            </div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--space-3)" }}>
              <div className="form-group">
                <label className="form-label" htmlFor="user-name">Display Name</label>
                <input
                  id="user-name"
                  className="input"
                  data-testid="user-name-input"
                  value={form.name}
                  onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
                  placeholder="Jane Doe"
                />
              </div>
              <div className="form-group">
                <label className="form-label" htmlFor="user-given-name">Given Name</label>
                <input
                  id="user-given-name"
                  className="input"
                  data-testid="user-given-name-input"
                  value={form.givenName}
                  onChange={(e) => setForm((f) => ({ ...f, givenName: e.target.value }))}
                  placeholder="Jane"
                />
              </div>
            </div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--space-3)" }}>
              <div className="form-group">
                <label className="form-label" htmlFor="user-family-name">Family Name</label>
                <input
                  id="user-family-name"
                  className="input"
                  data-testid="user-family-name-input"
                  value={form.familyName}
                  onChange={(e) => setForm((f) => ({ ...f, familyName: e.target.value }))}
                  placeholder="Doe"
                />
              </div>
              <div className="form-group">
                <label className="form-label" htmlFor="user-email">Email</label>
                <input
                  id="user-email"
                  type="email"
                  className="input"
                  data-testid="user-email-input"
                  value={form.email}
                  onChange={(e) => setForm((f) => ({ ...f, email: e.target.value }))}
                  placeholder="email@example.com"
                />
              </div>
            </div>
            <div className="form-group">
              <label className="form-label" htmlFor="user-phone">Phone</label>
              <input
                id="user-phone"
                type="tel"
                className="input"
                data-testid="user-phone-input"
                value={form.phone}
                onChange={(e) => setForm((f) => ({ ...f, phone: e.target.value }))}
                placeholder="+15551234567"
              />
            </div>
            <div className="form-group">
              <label className="form-label" htmlFor="user-password">
                Password {isEdit && <span style={{ fontSize: "var(--text-xs)", color: "var(--color-text-tertiary)" }}>(leave blank to keep unchanged)</span>}
              </label>
              <input
                id="user-password"
                type="password"
                className="input"
                data-testid="user-password-input"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder={isEdit ? "••••••••" : "Set password"}
                autoComplete="new-password"
              />
            </div>

            {/* ── Custom attributes ──────────────────────────────── */}
            {attrDefs.length > 0 && (
              <div>
                <div style={{
                  fontWeight: "var(--fw-semibold)",
                  color: "var(--color-text-secondary)",
                  marginBottom: "var(--space-2)",
                  textTransform: "uppercase",
                  letterSpacing: "0.08em",
                  fontFamily: "var(--font-mono)",
                  fontSize: "10px",
                }}>
                  Custom Attributes
                </div>
                {attrDefs.map((attr) => (
                  <div key={attr.machineName} className="form-group" style={{ marginBottom: "var(--space-2)" }}>
                    {attr.attributeType === "boolean" ? (
                      <label style={{ display: "flex", alignItems: "center", gap: 8, fontSize: "var(--text-sm)", color: "var(--color-text-primary)" }}>
                        <input
                          type="checkbox"
                          data-testid={`attr-${attr.machineName}`}
                          checked={!!attrValues[attr.machineName]}
                          onChange={(e) =>
                            setAttrValues((v) => ({ ...v, [attr.machineName]: e.target.checked }))
                          }
                        />
                        {attr.name}
                      </label>
                    ) : (
                      <>
                        <label className="form-label" htmlFor={`attr-${attr.machineName}`}>
                          {attr.name}
                        </label>
                        <input
                          id={`attr-${attr.machineName}`}
                          className="input"
                          data-testid={`attr-${attr.machineName}`}
                          type={
                            attr.attributeType === "number"
                              ? "number"
                              : attr.attributeType === "datetime"
                                ? "datetime-local"
                                : "text"
                          }
                          value={(attrValues[attr.machineName] as string) ?? ""}
                          onChange={(e) => {
                            const val =
                              attr.attributeType === "number"
                                ? e.target.value === "" ? "" : Number(e.target.value)
                                : e.target.value;
                            setAttrValues((v) => ({ ...v, [attr.machineName]: val }));
                          }}
                        />
                      </>
                    )}
                  </div>
                ))}
              </div>
            )}
            {attrDefs.length === 0 && isEdit && (
              <div style={{ fontSize: "var(--text-sm)", color: "var(--color-text-tertiary)" }}>
                No custom attributes defined
              </div>
            )}

            {/* ── Authorization (tenant + role assignment) ───────── */}
            <div>
              <div style={{
                fontWeight: "var(--fw-semibold)",
                color: "var(--color-text-secondary)",
                marginBottom: "var(--space-2)",
                textTransform: "uppercase",
                letterSpacing: "0.08em",
                fontFamily: "var(--font-mono)",
                fontSize: "10px",
              }}>
                Authorization
              </div>

              {tenantAssignments.map((ta) => {
                const tenantName =
                  allTenants.find((t) => t.id === ta.tenantId)?.name ?? ta.tenantId;
                return (
                  <div
                    key={ta.tenantId}
                    data-testid={`tenant-assignment-${ta.tenantId}`}
                    style={{
                      padding: "var(--space-2)",
                      background: "var(--color-neutral-50)",
                      borderRadius: "var(--radius-md)",
                      marginBottom: "var(--space-2)",
                    }}
                  >
                    <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: allRoles.length > 0 ? 6 : 0 }}>
                      <span className="badge badge-info" style={{ flexShrink: 0 }}>
                        {tenantName}
                      </span>
                      <div style={{ flex: 1 }} />
                      <button
                        className="btn btn-ghost btn-icon btn-sm"
                        data-testid={`remove-tenant-${ta.tenantId}`}
                        onClick={() => handleRemoveTenantAssignment(ta.tenantId)}
                        title="Remove tenant"
                      >
                        ✕
                      </button>
                    </div>
                    {/* Role assignment: multi-select + inline create */}
                    <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
                      <select
                        multiple
                        data-testid={`tenant-roles-${ta.tenantId}`}
                        value={ta.roleNames}
                        onChange={(e) => {
                          const selected = Array.from(e.target.selectedOptions, (o) => o.value);
                          handleTenantRoleChange(ta.tenantId, selected);
                        }}
                        className="input"
                        style={{ flex: 1, height: "auto", minHeight: 34 }}
                      >
                        {allRoles.map((r) => (
                          <option key={r.name} value={r.name}>{r.name}</option>
                        ))}
                      </select>
                    </div>
                  </div>
                );
              })}

              {/* Inline role creation — available when any tenant is assigned */}
              {tenantAssignments.length > 0 && (
                <div style={{ display: "flex", gap: 6, alignItems: "center", marginBottom: "var(--space-2)" }}>
                  <input
                    className="input"
                    data-testid="new-role-input"
                    autoFocus
                    value={newRoleName}
                    onChange={(e) => setNewRoleName(e.target.value)}
                    placeholder="New role name…"
                    style={{ flex: 1 }}
                    onKeyDown={(e) => { if (e.key === "Enter") handleCreateRoleInline(); }}
                  />
                  <button
                    className="btn btn-secondary btn-sm"
                    data-testid="create-role-inline-btn"
                    onClick={handleCreateRoleInline}
                    disabled={!newRoleName.trim()}
                  >
                    + Role
                  </button>
                </div>
              )}

              {addingTenant ? (
                <div style={{ marginBottom: "var(--space-2)" }}>
                  {creatingTenant ? (
                    <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                      <input
                        className="input"
                        data-testid="new-tenant-name-input"
                        autoFocus
                        value={newTenantName}
                        onChange={(e) => setNewTenantName(e.target.value)}
                        placeholder="Tenant name"
                        style={{ flex: 1 }}
                        onKeyDown={(e) => { if (e.key === "Enter") handleCreateTenantInline(); }}
                      />
                      <button
                        className="btn btn-primary btn-sm"
                        data-testid="create-tenant-inline-btn"
                        onClick={handleCreateTenantInline}
                        disabled={!newTenantName.trim()}
                      >
                        Create
                      </button>
                      <button className="btn btn-ghost btn-sm" onClick={() => { setCreatingTenant(false); setNewTenantName(""); }}>
                        Cancel
                      </button>
                    </div>
                  ) : (
                    <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                      <select
                        className="input"
                        data-testid="new-tenant-select"
                        value={newTenantId}
                        onChange={(e) => {
                          if (e.target.value === "__create__") {
                            setCreatingTenant(true);
                            setNewTenantId("");
                          } else {
                            setNewTenantId(e.target.value);
                          }
                        }}
                        style={{ flex: 1 }}
                      >
                        <option value="">Select tenant…</option>
                        {availableTenants.map((t) => (
                          <option key={t.id} value={t.id}>{t.name || t.id}</option>
                        ))}
                        <option value="__create__">＋ Create new tenant…</option>
                      </select>
                      <button
                        className="btn btn-primary btn-sm"
                        data-testid="confirm-add-tenant"
                        onClick={handleAddTenantAssignment}
                        disabled={!newTenantId}
                      >
                        Add
                      </button>
                      <button className="btn btn-ghost btn-sm" onClick={() => setAddingTenant(false)}>
                        Cancel
                      </button>
                    </div>
                  )}
                </div>
              ) : (
                <button
                  className="btn btn-ghost btn-sm"
                  data-testid="add-tenant-btn"
                  onClick={() => setAddingTenant(true)}
                >
                  + Add Tenant / Role
                </button>
              )}
            </div>
          </div>

          <div className="dialog-footer">
            <Dialog.Close asChild>
              <button className="btn btn-secondary">Cancel</button>
            </Dialog.Close>
            <button
              className="btn btn-primary"
              onClick={handleSave}
              disabled={!form.loginId || saving}
              data-testid="create-submit"
            >
              {saving ? "Saving…" : isEdit ? "Save" : "Create"}
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
