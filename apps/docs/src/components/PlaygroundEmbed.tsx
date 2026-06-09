import { useEffect, useRef, useState } from 'react';

const WASM_BASE = '/playground-wasm';

type Phase = 'loading' | 'ready' | 'error' | 'unavailable';

interface Status {
  phase: Phase;
  message: string;
}

export default function PlaygroundEmbed() {
  const [status, setStatus] = useState<Status>({
    phase: 'loading',
    message: 'Initializing WASM emulator…',
  });
  const iframeRef = useRef<HTMLIFrameElement>(null);

  useEffect(() => {
    // The /playground page now redirects to the purpose-built /demo page
    // which has the full WASM sandbox with profile selection.
    window.location.replace('/demo');
  }, []);

  return (
    <div style={{ fontFamily: "'Inter', system-ui, sans-serif" }}>
      {status.phase === 'loading' && (
        <div style={styles.overlay}>
          <div style={styles.spinner} />
          <p style={styles.statusText}>{status.message}</p>
        </div>
      )}

      {status.phase === 'unavailable' && (
        <div style={styles.card}>
          <div style={styles.icon}>🔬</div>
          <h2 style={styles.cardTitle}>Playground coming soon</h2>
          <p style={styles.cardBody}>{status.message}</p>
          <div style={styles.installBox}>
            <p style={{ margin: '0 0 0.75rem', color: '#888', fontSize: '0.85rem' }}>Run locally in 30 seconds:</p>
            <code style={styles.code}>npx rescope</code>
          </div>
          <p style={{ marginTop: '1rem', color: '#555', fontSize: '0.8rem' }}>
            Then open <a href="http://localhost:5173" style={{ color: '#39ff14' }}>http://localhost:5173</a>
          </p>
        </div>
      )}

      {status.phase === 'error' && (
        <div style={styles.card}>
          <div style={styles.icon}>❌</div>
          <h2 style={{ ...styles.cardTitle, color: '#ff4444' }}>Failed to initialize</h2>
          <p style={styles.cardBody}>{status.message}</p>
          <button style={styles.btn} onClick={() => window.location.reload()}>Retry</button>
        </div>
      )}

      {status.phase === 'ready' && (
        <iframe
          ref={iframeRef}
          src={`${WASM_BASE}/ui/`}
          style={styles.iframe}
          title="Rescope Admin UI"
        />
      )}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: '400px',
    gap: '1.5rem',
    background: '#0a0a0f',
    borderRadius: '8px',
    border: '1px solid #1e1e1e',
  },
  spinner: {
    width: 48,
    height: 48,
    border: '3px solid #222',
    borderTopColor: '#39ff14',
    borderRadius: '50%',
    animation: 'spin 0.8s linear infinite',
  },
  statusText: {
    color: '#888',
    fontSize: '0.9rem',
    margin: 0,
  },
  card: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: '400px',
    gap: '1rem',
    background: '#0a0a0f',
    borderRadius: '8px',
    border: '1px solid #1e1e1e',
    padding: '3rem',
    textAlign: 'center',
  },
  icon: {
    fontSize: '3rem',
    lineHeight: 1,
  },
  cardTitle: {
    color: '#e0e0e0',
    fontSize: '1.4rem',
    fontFamily: "'SF Mono', 'Fira Code', monospace",
    margin: '0.5rem 0',
  },
  cardBody: {
    color: '#888',
    maxWidth: 420,
    lineHeight: 1.6,
    margin: 0,
  },
  installBox: {
    marginTop: '1.5rem',
    background: '#111',
    border: '1px solid #222',
    borderRadius: '8px',
    padding: '1.25rem 2rem',
  },
  code: {
    fontFamily: "'SF Mono', 'Fira Code', monospace",
    color: '#39ff14',
    fontSize: '1rem',
    background: 'none',
    border: 'none',
    padding: 0,
  },
  btn: {
    marginTop: '1rem',
    background: '#1a1a2e',
    color: '#39ff14',
    border: '1px solid #39ff14',
    padding: '0.6rem 1.5rem',
    borderRadius: '6px',
    cursor: 'pointer',
    fontFamily: "'SF Mono', 'Fira Code', monospace",
    fontSize: '0.85rem',
  },
  iframe: {
    width: '100%',
    height: '80vh',
    border: 'none',
    borderRadius: '8px',
    background: '#0a0a0f',
  },
};
