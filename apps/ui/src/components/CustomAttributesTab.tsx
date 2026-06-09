/**
 * CustomAttributesTab — the "Custom Attributes" sub-tab under /users/attributes.
 * Manages custom attribute definitions (schema): list, create, delete.
 */
import { useEffect, useState, useCallback } from "react";
import { api, type CustomAttribute } from "../lib/api";

const MACHINE_NAME_MAX_LENGTH = 60;
const MACHINE_NAME_PATTERN = /^[a-z][a-z0-9_]*$/;

function validateMachineName(value: string): string | null {
  if (!value) return null; // handled by required check
  if (value.length > MACHINE_NAME_MAX_LENGTH) {
    return `Machine name must be ${MACHINE_NAME_MAX_LENGTH} characters or fewer (currently ${value.length}).`;
  }
  if (!MACHINE_NAME_PATTERN.test(value)) {
    return "Machine name must start with a lowercase letter and contain only lowercase letters, numbers, and underscores.";
  }
  return null;
}

export function CustomAttributesTab() {
  const [attrs, setAttrs] = useState<CustomAttribute[]>([]);
  const [loading, setLoading] = useState(true);
  const [form, setForm] = useState({
    name: "",
    machineName: "",
    attributeType: "text" as CustomAttribute["attributeType"],
    permissions: "admin" as CustomAttribute["permissions"],
  });
  const [error, setError] = useState<string | null>(null);

  const machineNameError = validateMachineName(form.machineName);
  const isFormValid = !!form.name && !!form.machineName && !machineNameError;

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const { attributes } = await api.customAttributes.list();
      setAttrs(attributes ?? []);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const handleCreate = async () => {
    setError(null);
    if (!form.name || !form.machineName) {
      setError("Name and machine name are required.");
      return;
    }
    if (machineNameError) {
      setError(machineNameError);
      return;
    }
    try {
      await api.customAttributes.create({
        name: form.name,
        machineName: form.machineName,
        attributeType: form.attributeType,
        permissions: form.permissions,
      });
      setForm({
        name: "",
        machineName: "",
        attributeType: "text",
        permissions: "admin",
      });
      load();
    } catch (e: unknown) {
      setError(
        e instanceof Error ? e.message : "Failed to create attribute",
      );
    }
  };

  const handleDelete = async (machineName: string) => {
    if (!confirm(`Delete attribute "${machineName}"?`)) return;
    await api.customAttributes.delete(machineName);
    load();
  };

  return (
    <>
      {/* Create form */}
      <div
        className="card"
        style={{ marginBottom: "var(--space-4)", padding: "var(--space-4)" }}
      >
        <div
          style={{
            fontSize: "var(--text-sm)",
            fontWeight: "var(--fw-semibold)",
            marginBottom: "var(--space-3)",
          }}
        >
          Add Custom Attribute
        </div>
        {error && (
          <div
            style={{
              padding: "var(--space-2)",
              background: "var(--color-danger-bg)",
              borderRadius: "var(--radius-md)",
              marginBottom: "var(--space-3)",
              color: "var(--color-danger)",
              fontSize: "var(--text-sm)",
            }}
          >
            {error}
          </div>
        )}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "minmax(0, 1fr) minmax(0, 1fr) auto auto auto",
            gap: "var(--space-2)",
            alignItems: "start",
          }}
        >
          <div className="form-group" style={{ minWidth: 0 }}>
            <label className="form-label">Name</label>
            <input
              className="input"
              data-testid="attr-name-input"
              value={form.name}
              maxLength={MACHINE_NAME_MAX_LENGTH}
              onChange={(e) =>
                setForm((f) => ({
                  ...f,
                  name: e.target.value,
                  machineName: e.target.value
                    .replace(/([a-z])([A-Z])/g, "$1_$2")
                    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1_$2")
                    .toLowerCase()
                    .replace(/[^a-z0-9]+/g, "_")
                    .replace(/^_|_$/g, "")
                    .slice(0, MACHINE_NAME_MAX_LENGTH),
                }))
              }
              placeholder="Attribute name"
            />
            <div
              style={{
                display: "flex",
                justifyContent: "flex-end",
                marginTop: 2,
                fontSize: "var(--text-xs)",
              }}
            >
              <span
                data-testid="attr-name-counter"
                style={{
                  color: form.name.length > MACHINE_NAME_MAX_LENGTH * 0.8
                    ? "var(--color-warning, var(--color-danger))"
                    : "var(--color-text-tertiary)",
                }}
              >
                {form.name.length}/{MACHINE_NAME_MAX_LENGTH}
              </span>
            </div>
          </div>
          <div className="form-group" style={{ minWidth: 0 }}>
            <label className="form-label">Machine Name</label>
            <input
              className="input"
              data-testid="attr-machine-name-input"
              value={form.machineName}
              maxLength={MACHINE_NAME_MAX_LENGTH}
              onChange={(e) =>
                setForm((f) => ({ ...f, machineName: e.target.value }))
              }
              placeholder="machine_name"
              style={{
                ...(machineNameError ? { borderColor: "var(--color-danger)" } : {}),
                overflow: "hidden",
                textOverflow: "ellipsis",
              }}
            />
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                marginTop: 2,
                fontSize: "var(--text-xs)",
                gap: 4,
              }}
            >
              {machineNameError ? (
                <span data-testid="machine-name-error" style={{ color: "var(--color-danger)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} title={machineNameError}>
                  {machineNameError}
                </span>
              ) : (
                <span />
              )}
              <span
                data-testid="machine-name-counter"
                style={{
                  flexShrink: 0,
                  color: form.machineName.length > MACHINE_NAME_MAX_LENGTH * 0.8
                    ? "var(--color-warning, var(--color-danger))"
                    : "var(--color-text-tertiary)",
                }}
              >
                {form.machineName.length}/{MACHINE_NAME_MAX_LENGTH}
              </span>
            </div>
          </div>
          <div className="form-group">
            <label className="form-label">Type</label>
            <select
              className="input"
              data-testid="attr-type-select"
              value={form.attributeType}
              onChange={(e) =>
                setForm((f) => ({
                  ...f,
                  attributeType:
                    e.target.value as CustomAttribute["attributeType"],
                }))
              }
            >
              <option value="text">Text</option>
              <option value="number">Number</option>
              <option value="boolean">Boolean</option>
              <option value="datetime">Datetime</option>
            </select>
          </div>
          <div className="form-group">
            <label className="form-label">Permissions</label>
            <select
              className="input"
              data-testid="attr-perms-select"
              value={form.permissions}
              onChange={(e) =>
                setForm((f) => ({
                  ...f,
                  permissions:
                    e.target.value as CustomAttribute["permissions"],
                }))
              }
            >
              <option value="admin">Admin Only</option>
              <option value="memberRead">Member Read</option>
              <option value="memberWrite">Member Write</option>
              <option value="all">All</option>
            </select>
          </div>
          <button
            className="btn btn-primary"
            data-testid="create-attr-btn"
            onClick={handleCreate}
            disabled={!isFormValid}
          >
            Add
          </button>
        </div>
      </div>

      {/* Attributes table */}
      {loading ? (
        <div style={{ display: "flex", justifyContent: "center", padding: 40 }}>
          <div className="loading-spinner" />
        </div>
      ) : attrs.length === 0 ? (
        <div className="empty-state">
          <div className="empty-state-icon">🏷️</div>
          <div className="empty-state-title">No custom attributes</div>
          <div className="empty-state-description">
            Define custom attributes to extend user profiles.
          </div>
        </div>
      ) : (
        <div className="card">
          <div className="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Machine Name</th>
                  <th>Type</th>
                  <th>Permissions</th>
                  <th style={{ width: 60 }}></th>
                </tr>
              </thead>
              <tbody>
                {attrs.map((attr) => (
                  <tr
                    key={attr.machineName}
                    data-testid={`attr-row-${attr.machineName}`}
                  >
                    <td>{attr.name}</td>
                    <td>
                      <code
                        style={{
                          fontFamily: "var(--font-mono)",
                          fontSize: "var(--text-xs)",
                          padding: "2px 6px",
                          background: "var(--color-neutral-100)",
                          borderRadius: "var(--radius-sm)",
                        }}
                      >
                        {attr.machineName}
                      </code>
                    </td>
                    <td>
                      <span className="badge badge-neutral">
                        {attr.attributeType}
                      </span>
                    </td>
                    <td>
                      <span className="badge badge-neutral">
                        {attr.permissions}
                      </span>
                    </td>
                    <td>
                      <button
                        className="btn btn-ghost btn-icon btn-sm"
                        data-testid={`delete-attr-${attr.machineName}`}
                        onClick={() => handleDelete(attr.machineName)}
                        title="Delete attribute"
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
            {attrs.length} attribute{attrs.length !== 1 ? "s" : ""}
          </div>
        </div>
      )}
    </>
  );
}
