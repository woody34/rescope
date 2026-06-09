import { useHealthCheck } from "../hooks/useHealthCheck";

/**
 * Full-screen overlay shown when the API is unreachable.
 * Renders nothing when connected.
 */
export function ConnectionOverlay() {
  const connected = useHealthCheck();

  if (connected) return null;

  return (
    <div
      data-testid="connection-overlay"
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 9999,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        background: "rgba(0, 0, 0, 0.85)",
        backdropFilter: "blur(6px)",
        color: "var(--color-text-primary, #fff)",
        fontFamily: "var(--font-mono)",
        gap: 16,
      }}
    >
      {/* Pulsing indicator */}
      <div
        style={{
          width: 14,
          height: 14,
          borderRadius: "50%",
          background: "var(--color-danger, #ef4444)",
          animation: "pulse-dot 1.5s ease-in-out infinite",
        }}
      />

      <div style={{ fontSize: 20, fontWeight: 600, letterSpacing: "0.04em" }}>
        API Disconnected
      </div>

      <div
        style={{
          fontSize: 13,
          color: "var(--color-text-tertiary, #888)",
          maxWidth: 360,
          textAlign: "center",
          lineHeight: 1.5,
        }}
      >
        The emulator API is unreachable. Make sure the server is running and
        this page will reconnect automatically.
      </div>

      {/* Spinner */}
      <div
        style={{
          marginTop: 8,
          width: 24,
          height: 24,
          border: "2px solid var(--color-text-tertiary, #555)",
          borderTopColor: "var(--color-primary, #22c55e)",
          borderRadius: "50%",
          animation: "spin 0.8s linear infinite",
        }}
      />

      <style>{`
        @keyframes pulse-dot {
          0%, 100% { opacity: 1; transform: scale(1); }
          50% { opacity: 0.4; transform: scale(0.8); }
        }
        @keyframes spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}
