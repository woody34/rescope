import { useState, useEffect, useCallback } from "react";
import { Layout } from "../components/Layout";
import { PageHeader } from "../components/PageHeader";
import { api, type IdpEmulator } from "../lib/api";

export default function IdentityProvidersPage() {
  const [idps, setIdps] = useState<IdpEmulator[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [tenants, setTenants] = useState<{ id: string; name: string }[]>([]);

  const refresh = useCallback(async () => {
    try {
      const { idps: list } = await api.idps.list();
      setIdps(list);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    api.tenants.list().then(({ tenants }) => setTenants(tenants));
  }, [refresh]);

  const handleDelete = async (id: string) => {
    if (!confirm("Delete this identity provider?")) return;
    await api.idps.delete(id);
    refresh();
  };

  const handleTestSso = (idp: IdpEmulator) => {
    const base = window.location.origin;
    if (idp.protocol === "oidc") {
      window.open(
        `${base}/emulator/idp/${idp.id}/authorize?client_id=test&redirect_uri=${encodeURIComponent(base + "/dashboard")}&response_type=code`,
        "_blank",
      );
    } else {
      window.open(
        `${base}/emulator/idp/${idp.id}/sso?RelayState=${encodeURIComponent(base + "/dashboard")}`,
        "_blank",
      );
    }
  };

  return (
    <Layout title="Identity Providers">
      <PageHeader
        title="Identity Providers"
        description="Emulate external identity providers for SSO testing"
        actions={
          <button
            className="btn btn-primary"
            data-testid="create-idp-btn"
            onClick={() => setShowCreate(true)}
          >
            + Create IdP
          </button>
        }
      />

      {showCreate && (
        <CreateIdpDialog
          tenants={tenants}
          onClose={() => setShowCreate(false)}
          onCreated={refresh}
        />
      )}

      {loading ? (
        <p style={{ color: "var(--text-secondary)" }}>Loading…</p>
      ) : idps.length === 0 ? (
        <div className="empty-state" data-testid="idp-empty-state">
          <div className="empty-state-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            </svg>
          </div>
          <div className="empty-state-title">No identity providers configured</div>
          <div className="empty-state-description">
            Create an OIDC or SAML identity provider to test SSO flows.
          </div>
        </div>
      ) : (
        <table data-testid="idp-table">
          <thead>
            <tr>
              <th>Display Name</th>
              <th>Protocol</th>
              <th>Tenant</th>
              <th>Endpoints</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {idps.map((idp) => (
              <tr key={idp.id} data-testid={`idp-row-${idp.id}`}>
                <td>
                  <strong>{idp.displayName}</strong>
                  <div style={{ fontSize: "0.75rem", color: "var(--text-secondary)" }}>
                    {idp.id}
                  </div>
                </td>
                <td>
                  <span className={`badge badge-${idp.protocol}`}>
                    {idp.protocol.toUpperCase()}
                  </span>
                </td>
                <td>{idp.tenantId}</td>
                <td style={{ fontSize: "0.8rem" }}>
                  {idp.protocol === "oidc" ? (
                    <>
                      <a
                        href={`/emulator/idp/${idp.id}/.well-known/openid-configuration`}
                        target="_blank"
                        className="endpoint-link"
                      >
                        Discovery
                      </a>
                      {" · "}
                      <a
                        href={`/emulator/idp/${idp.id}/jwks`}
                        target="_blank"
                        className="endpoint-link"
                      >
                        JWKS
                      </a>
                    </>
                  ) : (
                    <a
                      href={`/emulator/idp/${idp.id}/metadata`}
                      target="_blank"
                      className="endpoint-link"
                    >
                      Metadata
                    </a>
                  )}
                </td>
                <td>
                  <div style={{ display: "flex", gap: "var(--space-2)" }}>
                    <button
                      className="btn btn-sm btn-secondary"
                      data-testid={`test-sso-${idp.id}`}
                      onClick={() => handleTestSso(idp)}
                    >
                      Test SSO
                    </button>
                    <button
                      className="btn btn-sm btn-danger"
                      data-testid={`delete-idp-${idp.id}`}
                      onClick={() => handleDelete(idp.id)}
                    >
                      Delete
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {idps.length > 0 && (
        <div style={{ marginTop: "var(--space-6)" }}>
          <h3 style={{ marginBottom: "var(--space-3)" }}>Attribute Mappings</h3>
          {idps.map((idp) => (
            <AttributeMappingEditor
              key={idp.id}
              idp={idp}
              onUpdated={refresh}
            />
          ))}
        </div>
      )}
    </Layout>
  );
}

// ─── Create Dialog ───────────────────────────────────────────────────────────

function CreateIdpDialog({
  tenants,
  onClose,
  onCreated,
}: {
  tenants: { id: string; name: string }[];
  onClose: () => void;
  onCreated: () => void;
}) {
  const [displayName, setDisplayName] = useState("");
  const [protocol, setProtocol] = useState<"oidc" | "saml">("oidc");
  const [tenantId, setTenantId] = useState(tenants[0]?.id ?? "");
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    try {
      await api.idps.create({
        protocol,
        displayName,
        tenantId,
        attributeMapping: { email: "user.email", name: "user.name" },
      });
      onCreated();
      onClose();
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog-content" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2 className="dialog-title">Create Identity Provider</h2>
          <button className="btn btn-ghost btn-icon" onClick={onClose}>
            ✕
          </button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="dialog-body">
            <div className="form-group">
              <label className="form-label">
                Display Name{" "}
                <span style={{ color: "var(--color-danger)" }}>*</span>
              </label>
              <input
                className="input"
                type="text"
                data-testid="idp-name-input"
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
                placeholder="e.g., Mock Okta"
                required
              />
            </div>
            <div className="form-group">
              <label className="form-label">Protocol</label>
              <select
                className="input"
                data-testid="idp-protocol-select"
                value={protocol}
                onChange={(e) => setProtocol(e.target.value as "oidc" | "saml")}
              >
                <option value="oidc">OIDC</option>
                <option value="saml">SAML</option>
              </select>
            </div>
            <div className="form-group">
              <label className="form-label">Tenant</label>
              <select
                className="input"
                data-testid="idp-tenant-select"
                value={tenantId}
                onChange={(e) => setTenantId(e.target.value)}
              >
                {tenants.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.name} ({t.id})
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="dialog-footer">
            <button type="button" className="btn btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button
              type="submit"
              className="btn btn-primary"
              data-testid="create-idp-submit"
              disabled={submitting || !displayName || !tenantId}
            >
              {submitting ? "Creating…" : "Create"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

// ─── Attribute Mapping Editor ────────────────────────────────────────────────

function AttributeMappingEditor({
  idp,
  onUpdated,
}: {
  idp: IdpEmulator;
  onUpdated: () => void;
}) {
  const entries = Object.entries(idp.attributeMapping);
  const [newKey, setNewKey] = useState("");
  const [newValue, setNewValue] = useState("");

  const handleAdd = async () => {
    if (!newKey || !newValue) return;
    const updated = { ...idp.attributeMapping, [newKey]: newValue };
    await api.idps.update(idp.id, { attributeMapping: updated });
    setNewKey("");
    setNewValue("");
    onUpdated();
  };

  const handleRemove = async (key: string) => {
    const updated = { ...idp.attributeMapping };
    delete updated[key];
    await api.idps.update(idp.id, { attributeMapping: updated });
    onUpdated();
  };

  return (
    <div
      className="card"
      data-testid={`mapping-editor-${idp.id}`}
      style={{ marginBottom: "var(--space-4)" }}
    >
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "var(--space-3)" }}>
        <strong>{idp.displayName}</strong>
        <span className={`badge badge-${idp.protocol}`}>
          {idp.protocol.toUpperCase()}
        </span>
      </div>
      <table>
        <thead>
          <tr>
            <th>IdP Claim</th>
            <th>User Field</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {entries.map(([key, value]) => (
            <tr key={key}>
              <td><code>{key}</code></td>
              <td><code>{value}</code></td>
              <td>
                <button
                  className="btn btn-sm btn-danger"
                  onClick={() => handleRemove(key)}
                >
                  ×
                </button>
              </td>
            </tr>
          ))}
          <tr>
            <td>
              <input
                className="input"
                type="text"
                data-testid="mapping-key-input"
                placeholder="Claim name"
                value={newKey}
                onChange={(e) => setNewKey(e.target.value)}
                style={{ width: "100%" }}
              />
            </td>
            <td>
              <input
                className="input"
                type="text"
                data-testid="mapping-value-input"
                placeholder="user.field"
                value={newValue}
                onChange={(e) => setNewValue(e.target.value)}
                style={{ width: "100%" }}
              />
            </td>
            <td>
              <button
                className="btn btn-sm btn-primary"
                data-testid="add-mapping-btn"
                onClick={handleAdd}
                disabled={!newKey || !newValue}
              >
                +
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  );
}
