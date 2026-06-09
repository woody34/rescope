import { api } from "../lib/api";
import { Layout } from "../components/Layout";
import { PageHeader } from "../components/PageHeader";
import { useState } from "react";

export default function SnapshotPage() {
  const [exporting, setExporting] = useState(false);
  const [importing, setImporting] = useState(false);
  const [resetting, setResetting] = useState(false);
  const [message, setMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);

  const showMessage = (type: "success" | "error", text: string) => {
    setMessage({ type, text });
    setTimeout(() => setMessage(null), 4000);
  };

  const handleExport = async () => {
    setExporting(true);
    try {
      const snap = await api.snapshot.export();
      const blob = new Blob([JSON.stringify(snap, null, 2)], {
        type: "application/json",
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `emulator-snapshot-${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(url);
      showMessage("success", "Snapshot exported successfully");
    } catch (e: unknown) {
      showMessage(
        "error",
        `Export failed: ${e instanceof Error ? e.message : String(e)}`,
      );
    } finally {
      setExporting(false);
    }
  };

  const handleImport = async (file: File) => {
    setImporting(true);
    try {
      const text = await file.text();
      const snap = JSON.parse(text);
      await api.snapshot.import(snap);
      showMessage(
        "success",
        "Snapshot imported successfully — emulator state restored",
      );
    } catch (e: unknown) {
      showMessage(
        "error",
        `Import failed: ${e instanceof Error ? e.message : String(e)}`,
      );
    } finally {
      setImporting(false);
    }
  };

  const handleReset = async () => {
    if (
      !confirm(
        "Reset all runtime state? Users, tokens, and OTPs will be cleared. Config stores (permissions, roles, etc.) are preserved.",
      )
    )
      return;
    setResetting(true);
    try {
      await api.emulator.reset();
      showMessage("success", "Emulator runtime state reset");
    } catch (e: unknown) {
      showMessage(
        "error",
        `Reset failed: ${e instanceof Error ? e.message : String(e)}`,
      );
    } finally {
      setResetting(false);
    }
  };

  return (
    <Layout title="Emulator">
      <PageHeader
        title="Snapshot"
        description="Export and import the complete emulator state for reproducible test environments"
      />

      {message && (
        <div
          style={{
            padding: "var(--space-3) var(--space-4)",
            background:
              message.type === "success"
                ? "var(--color-success-bg)"
                : "var(--color-error-bg)",
            border: `1px solid ${message.type === "success" ? "var(--color-success)" : "var(--color-error)"}`,
            borderRadius: "var(--radius-md)",
            color:
              message.type === "success"
                ? "var(--color-success-text)"
                : "var(--color-error-text)",
            fontSize: "var(--text-sm)",
            fontWeight: "var(--fw-medium)",
            marginBottom: "var(--space-5)",
          }}
        >
          {message.type === "success" ? "✅" : "❌"} {message.text}
        </div>
      )}

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr 1fr",
          gap: "var(--space-5)",
        }}
      >
        {/* Export */}
        <div className="card">
          <div className="card-header">
            <div>
              <div className="card-title">Export</div>
              <div className="card-description">Download state as JSON</div>
            </div>
            <span style={{ fontSize: 28 }}>💾</span>
          </div>
          <div className="card-body">
            <p
              style={{
                fontSize: "var(--text-sm)",
                color: "var(--color-text-secondary)",
                marginBottom: "var(--space-4)",
              }}
            >
              Exports all stores — users, tenants, permissions, roles, auth
              methods, JWT templates, connectors, attributes, and access keys.
            </p>
            <button
              id="export-snapshot-btn"
              data-testid="export-snapshot-btn"
              className="btn btn-primary w-full"
              onClick={handleExport}
              disabled={exporting}
            >
              {exporting ? (
                <span className="loading-spinner" />
              ) : (
                "⬇️ Export Snapshot"
              )}
            </button>
          </div>
        </div>

        {/* Import */}
        <div className="card">
          <div className="card-header">
            <div>
              <div className="card-title">Import</div>
              <div className="card-description">Restore from JSON file</div>
            </div>
            <span style={{ fontSize: 28 }}>📂</span>
          </div>
          <div className="card-body">
            <p
              style={{
                fontSize: "var(--text-sm)",
                color: "var(--color-text-secondary)",
                marginBottom: "var(--space-4)",
              }}
            >
              Imports a previously exported snapshot. Resets runtime state
              first, then restores all stores from the file.
            </p>
            <label
              className="btn btn-secondary w-full"
              style={{ justifyContent: "center" }}
            >
              {importing ? (
                <span className="loading-spinner" />
              ) : (
                "⬆️ Import Snapshot"
              )}
              <input
                id="import-snapshot-input"
                data-testid="import-snapshot-input"
                type="file"
                accept=".json"
                className="sr-only"
                onChange={(e) =>
                  e.target.files?.[0] && handleImport(e.target.files[0])
                }
              />
            </label>
          </div>
        </div>

        {/* Reset */}
        <div className="card">
          <div className="card-header">
            <div>
              <div className="card-title">Reset</div>
              <div className="card-description">Clear runtime state</div>
            </div>
            <span style={{ fontSize: 28 }}>🔄</span>
          </div>
          <div className="card-body">
            <p
              style={{
                fontSize: "var(--text-sm)",
                color: "var(--color-text-secondary)",
                marginBottom: "var(--space-4)",
              }}
            >
              Resets all runtime stores (users, tokens, OTPs). Configuration
              stores (permissions, roles, auth methods, etc.) are preserved.
            </p>
            <button
              id="reset-emulator-btn"
              className="btn btn-danger w-full"
              onClick={handleReset}
              disabled={resetting}
            >
              {resetting ? (
                <span className="loading-spinner" />
              ) : (
                "🔄 Reset Runtime"
              )}
            </button>
          </div>
        </div>
      </div>
    </Layout>
  );
}
