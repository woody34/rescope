import { useState, useEffect, useCallback } from 'react';
import { useWasm } from './useWasm';
import { useEmulator } from './useEmulator';
import ProfileBar from './ProfileBar';
import ResourceTree from './ResourceTree';
import LoginForm from './LoginForm';
import { DEMO_PROFILES } from './profiles';
import type { DemoProfile } from './profiles/types';

const SNAPSHOT_URL = '/demo-wasm/emulator/snapshot';

const s: Record<string, React.CSSProperties> = {
  root: {
    fontFamily: "'Inter', system-ui, sans-serif",
    minHeight: '600px',
  },
  loadingOverlay: {
    display: 'flex', flexDirection: 'column', alignItems: 'center',
    justifyContent: 'center', minHeight: 400, gap: '1.5rem',
    background: '#0a0a0f', borderRadius: 8, border: '1px solid #1e1e1e',
  },
  spinner: {
    width: 48, height: 48, border: '3px solid #222',
    borderTopColor: '#39ff14', borderRadius: '50%',
    animation: 'spin 0.8s linear infinite',
  },
  statusText: { color: '#888', fontSize: '0.9rem', margin: 0 },
  unavailableCard: {
    display: 'flex', flexDirection: 'column', alignItems: 'center',
    justifyContent: 'center', minHeight: 400, gap: '1rem',
    background: '#0a0a0f', border: '1px solid #1e1e1e', borderRadius: 8,
    padding: '3rem', textAlign: 'center' as const,
  },
  headerRow: {
    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
    marginBottom: '0.5rem',
  },
  heading: {
    color: '#39ff14', fontFamily: "'SF Mono','Fira Code',monospace",
    fontSize: '0.85rem', fontWeight: 700, margin: 0, letterSpacing: '0.05em',
  },
  resetBtn: {
    background: 'none', border: '1px solid #2a2a2a', borderRadius: 5,
    color: '#666', fontSize: '0.72rem', padding: '0.25rem 0.6rem',
    fontFamily: "'SF Mono','Fira Code',monospace", cursor: 'pointer',
  },
  panels: {
    display: 'grid', gridTemplateColumns: '1fr 1.6fr', gap: '1rem', alignItems: 'start',
  },
  panelLabel: {
    color: '#555', fontSize: '0.7rem', fontFamily: "'SF Mono','Fira Code',monospace",
    letterSpacing: '0.08em', textTransform: 'uppercase' as const, marginBottom: '0.4rem',
  },
};

export default function DemoSandbox() {
  const { status, error } = useWasm();
  const [profile, setProfile] = useState<DemoProfile | null>(DEMO_PROFILES[0]);
  const [seeding, setSeeding] = useState(false);

  const emulator = useEmulator(profile);

  // Auto-seed the default profile once WASM is ready
  useEffect(() => {
    if (status !== 'ready') return;
    setSeeding(true);
    emulator.seed(DEMO_PROFILES[0]).finally(() => setSeeding(false));
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status]);

  const handleSelectProfile = useCallback(async (p: DemoProfile) => {
    setProfile(p);
    setSeeding(true);
    await emulator.seed(p);
    setSeeding(false);
  }, [emulator]);

  const handleReset = useCallback(async () => {
    if (!profile) return;
    setSeeding(true);
    await emulator.reset();
    setSeeding(false);
  }, [emulator, profile]);

  if (status === 'loading') {
    return (
      <div style={s.loadingOverlay}>
        <div style={s.spinner} />
        <p style={s.statusText}>Initializing WASM emulator…</p>
      </div>
    );
  }

  if (status === 'unavailable' || status === 'error') {
    return (
      <div style={s.unavailableCard}>
        <div style={{ fontSize: '2.5rem' }}>🔬</div>
        <h2 style={{ color: '#e0e0e0', fontFamily: "'SF Mono','Fira Code',monospace", fontSize: '1.3rem', margin: 0 }}>
          Run it locally
        </h2>
        <p style={{ color: '#666', maxWidth: 380, lineHeight: 1.6, margin: 0 }}>
          {error ?? 'The WASM sandbox works in the npm dev environment. Clone the repo and run:'}
        </p>
        <code style={{ color: '#39ff14', fontFamily: "'SF Mono','Fira Code',monospace", fontSize: '0.9rem', background: '#111', padding: '0.75rem 1.5rem', borderRadius: 6, border: '1px solid #222' }}>
          npm run docs:build:wasm && npm run docs:dev
        </code>
      </div>
    );
  }

  return (
    <div style={s.root}>
      {/* Profile selector */}
      <div style={s.headerRow}>
        <span style={s.heading}>SELECT A SCENARIO</span>
        {profile && (
          <button style={s.resetBtn} onClick={handleReset} disabled={seeding}>
            {seeding ? '…' : '↺ reset'}
          </button>
        )}
      </div>
      <ProfileBar profiles={DEMO_PROFILES} active={profile} onSelect={handleSelectProfile} disabled={seeding} />

      {/* Two-panel layout */}
      {profile && (
        <div style={s.panels}>
          <div>
            <div style={s.panelLabel}>Emulated State</div>
            <ResourceTree pollUrl={SNAPSHOT_URL} active={!seeding} />
          </div>
          <div>
            <div style={s.panelLabel}>Try It</div>
            {seeding ? (
              <div style={{ ...s.loadingOverlay, minHeight: 200 }}>
                <div style={{ ...s.spinner, width: 32, height: 32 }} />
                <p style={s.statusText}>Seeding emulator…</p>
              </div>
            ) : (
              <LoginForm profile={profile} emulator={emulator} />
            )}
          </div>
        </div>
      )}

      {!profile && (
        <div style={{ color: '#333', textAlign: 'center', padding: '3rem', fontFamily: "'SF Mono','Fira Code',monospace", fontSize: '0.8rem' }}>
          ↑ Pick a scenario above to start
        </div>
      )}

      {/* Spinner animation */}
      <style>{`@keyframes spin { to { transform: rotate(360deg); } }`}</style>
    </div>
  );
}
