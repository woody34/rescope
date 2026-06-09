import { api } from "../lib/api";
import { Layout } from "../components/Layout";
import { PageHeader } from "../components/PageHeader";
import { useEffect, useState, useCallback } from "react";

export default function OtpInspectorPage() {
  const [otps, setOtps] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    const result = await api.snapshot.otps();
    setOtps(result.otps);
    setLoading(false);
  }, []);

  useEffect(() => {
    // eslint-disable-next-line
    load();
    const id = setInterval(load, 3000);
    return () => clearInterval(id);
  }, [load]);

  const entries = Object.entries(otps);

  return (
    <Layout title="Emulator">
      <PageHeader
        title="OTP Inspector"
        description="View pending OTP codes in real-time — auto-refreshes every 3 seconds"
        actions={
          <button
            id="refresh-otps-btn"
            data-testid="refresh-otps-btn"
            className="btn btn-secondary"
            onClick={load}
          >
            Refresh
          </button>
        }
      />

      {loading ? (
        <div style={{ display: "flex", justifyContent: "center", padding: 40 }}>
          <div className="loading-spinner" />
        </div>
      ) : entries.length === 0 ? (
        <div className="empty-state">
          <div className="empty-state-icon">
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <polyline points="4 17 10 11 4 5" />
              <line x1="12" y1="19" x2="20" y2="19" />
            </svg>
          </div>
          <div data-testid="empty-state" className="empty-state-title">
            No pending OTPs
          </div>
          <div className="empty-state-description">
            OTP codes will appear here as they are issued during authentication
            flows.
          </div>
        </div>
      ) : (
        <div className="card">
          <div className="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>User ID</th>
                  <th>OTP Code</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {entries.map(([userId, code]) => (
                  <tr
                    key={userId}
                    data-user-id={userId}
                    data-testid={`otp-row-${userId}`}
                  >
                    <td
                      className="font-mono"
                      style={{
                        fontSize: "var(--text-xs)",
                        color: "var(--color-text-secondary)",
                      }}
                    >
                      {userId}
                    </td>
                    <td>
                      <code
                        className="font-mono"
                        style={{
                          fontSize: "var(--text-lg)",
                          fontWeight: "var(--fw-semibold)",
                          color: "var(--color-primary)",
                          letterSpacing: "0.15em",
                        }}
                      >
                        {code}
                      </code>
                    </td>
                    <td>
                      <button
                        className="btn btn-ghost btn-sm"
                        onClick={() => navigator.clipboard.writeText(code)}
                      >
                        📋 Copy
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
