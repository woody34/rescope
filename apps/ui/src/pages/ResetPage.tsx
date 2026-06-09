import { api } from "../lib/api";
import { Layout } from "../components/Layout";
import { PageHeader } from "../components/PageHeader";
import { useState } from "react";

export default function ResetPage() {
  const [status, setStatus] = useState<"idle" | "resetting" | "done" | "error">(
    "idle",
  );
  const [message, setMessage] = useState<string>("");

  const handleReset = async () => {
    if (
      !confirm(
        "Reset all runtime state? This clears users, tokens, and OTPs. Config (auth methods, roles, permissions) is preserved.",
      )
    )
      return;
    setStatus("resetting");
    try {
      await api.emulator.reset();
      setStatus("done");
      setMessage(
        "Runtime state reset successfully. Users, tokens, and OTPs have been cleared.",
      );
    } catch (e: unknown) {
      setStatus("error");
      setMessage(e instanceof Error ? e.message : "Reset failed");
    }
  };

  return (
    <Layout title="Reset Emulator">
      <PageHeader
        title="Reset Emulator"
        description="Clear all runtime state to start fresh"
      />

      <div
        className="card"
        style={{ maxWidth: 560, padding: "var(--space-6)" }}
      >
        <div style={{ marginBottom: "var(--space-6)" }}>
          <h3
            style={{
              fontSize: "var(--text-lg)",
              fontWeight: "var(--fw-semibold)",
              marginBottom: "var(--space-3)",
            }}
          >
            Runtime State Reset
          </h3>
          <p
            style={{
              color: "var(--color-text-secondary)",
              fontSize: "var(--text-sm)",
              lineHeight: 1.6,
            }}
          >
            This will clear all <strong>runtime state</strong> from the
            emulator:
          </p>
          <ul
            style={{
              marginTop: "var(--space-3)",
              paddingLeft: "var(--space-5)",
              color: "var(--color-text-secondary)",
              fontSize: "var(--text-sm)",
              lineHeight: 2,
            }}
          >
            <li>All users</li>
            <li>All sessions and refresh tokens</li>
            <li>All pending OTPs and magic link tokens</li>
            <li>Lockout state</li>
          </ul>
          <div
            style={{
              marginTop: "var(--space-4)",
              padding: "var(--space-3) var(--space-4)",
              background: "var(--color-warning-bg)",
              borderRadius: "var(--radius-md)",
              border: "1px solid rgba(210,153,34,0.3)",
              fontSize: "var(--text-sm)",
              color: "var(--color-warning-text)",
            }}
          >
            <strong>Note:</strong> Config is preserved — auth methods, roles,
            permissions, JWT templates, and connectors are <em>not</em>{" "}
            affected.
          </div>
        </div>

        {status === "done" && (
          <div
            style={{
              padding: "var(--space-3) var(--space-4)",
              marginBottom: "var(--space-4)",
              background: "var(--color-success-bg)",
              borderRadius: "var(--radius-md)",
              color: "var(--color-success-text)",
              fontSize: "var(--text-sm)",
              border: "1px solid rgba(63,185,80,0.25)",
            }}
            id="reset-success-msg"
            data-testid="reset-success-msg"
          >
            ✓ {message}
          </div>
        )}

        {status === "error" && (
          <div
            style={{
              padding: "var(--space-3) var(--space-4)",
              marginBottom: "var(--space-4)",
              background: "var(--color-danger-bg)",
              borderRadius: "var(--radius-md)",
              color: "var(--color-danger)",
              fontSize: "var(--text-sm)",
              border: "1px solid rgba(248,81,73,0.25)",
            }}
          >
            ✕ {message}
          </div>
        )}

        <button
          id="reset-btn"
          className="btn btn-danger"
          data-testid="reset-btn"
          onClick={handleReset}
          disabled={status === "resetting"}
          style={{ width: "100%" }}
        >
          {status === "resetting" ? "Resetting…" : "Reset Runtime State"}
        </button>
      </div>
    </Layout>
  );
}
