import { useEffect, useRef, useState } from 'react';

interface SnapshotUser {
  loginId?: string;
  email?: string;
  name?: string;
  roleNames?: string[];
  status?: string;
}
interface SnapshotRole { name: string }
interface SnapshotTenant { id: string; name: string }

interface Snapshot {
  users?: SnapshotUser[];
  roles?: SnapshotRole[];
  tenants?: SnapshotTenant[];
}

interface ResourceTreeProps {
  pollUrl: string;
  active: boolean;
}

function TreeLine({ depth = 0, last = false, label }: { depth?: number; last?: boolean; label: React.ReactNode }) {
  const indent = '  '.repeat(depth);
  const branch = last ? '└── ' : '├── ';
  return (
    <div style={{ fontFamily: "'SF Mono','Fira Code',monospace", fontSize: '0.78rem', color: '#c8c8c8', lineHeight: 1.6 }}>
      {indent}{branch}{label}
    </div>
  );
}

function Badge({ count }: { count: number }) {
  return (
    <span style={{
      background: 'rgba(57,255,20,0.12)',
      color: '#39ff14',
      fontSize: '0.65rem',
      padding: '0 5px',
      borderRadius: '10px',
      marginLeft: '0.4rem',
    }}>
      {count}
    </span>
  );
}

export default function ResourceTree({ pollUrl, active }: ResourceTreeProps) {
  const [snap, setSnap] = useState<Snapshot | null>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (!active) { setSnap(null); return; }

    async function poll() {
      try {
        const res = await fetch(pollUrl);
        if (res.ok) setSnap(await res.json());
      } catch { /* ignore */ }
    }

    poll();
    intervalRef.current = setInterval(poll, 500);
    return () => { if (intervalRef.current) clearInterval(intervalRef.current); };
  }, [pollUrl, active]);

  const users = snap?.users ?? [];
  const roles = snap?.roles ?? [];
  const tenants = snap?.tenants ?? [];

  const containerStyle: React.CSSProperties = {
    background: '#0a0a0f',
    border: '1px solid #1e1e1e',
    borderRadius: '8px',
    padding: '1rem',
    minHeight: '300px',
    fontFamily: "'SF Mono','Fira Code',monospace",
    overflowY: 'auto' as const,
  };

  if (!active) {
    return (
      <div style={{ ...containerStyle, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <span style={{ color: '#333', fontSize: '0.8rem' }}>← Select a profile to see state</span>
      </div>
    );
  }

  const sections = [
    { key: 'users', items: users, icon: '📁', renderItem: (u: SnapshotUser, i: number, arr: SnapshotUser[]) => (
      <TreeLine key={i} depth={1} last={i === arr.length - 1}
        label={<span><span style={{ color: '#39ff14' }}>{u.email ?? u.loginId ?? '?'}</span>{u.roleNames?.length ? <span style={{ color: '#555' }}>{' [' + u.roleNames.join(', ') + ']'}</span> : null}</span>}
      />
    )},
    { key: 'roles', items: roles, icon: '📁', renderItem: (r: SnapshotRole, i: number, arr: SnapshotRole[]) => (
      <TreeLine key={i} depth={1} last={i === arr.length - 1} label={<span style={{ color: '#9fff7e' }}>{r.name}</span>} />
    )},
    { key: 'tenants', items: tenants, icon: '📁', renderItem: (t: SnapshotTenant, i: number, arr: SnapshotTenant[]) => (
      <TreeLine key={i} depth={1} last={i === arr.length - 1} label={<span style={{ color: '#66b3ff' }}>{t.name} <span style={{ color: '#444' }}>({t.id})</span></span>} />
    )},
  ];

  return (
    <div style={containerStyle}>
      {/* Live indicator */}
      <div style={{ marginBottom: '0.75rem', display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
        <span style={{ width: 6, height: 6, borderRadius: '50%', background: '#39ff14', display: 'inline-block', boxShadow: '0 0 4px #39ff14' }} />
        <span style={{ color: '#444', fontSize: '0.7rem' }}>live</span>
      </div>

      {/* Root */}
      <div style={{ color: '#39ff14', fontSize: '0.8rem', marginBottom: '0.25rem', fontWeight: 700 }}>rescope/</div>

      {sections.map(({ key, items, renderItem }, si) => {
        const isLastSection = si === sections.filter(s => s.items.length > 0).length - 1 &&
          sections.slice(si + 1).every(s => s.items.length === 0);
        if (!items.length) return null;
        return (
          <div key={key}>
            <TreeLine last={isLastSection} label={<span style={{ color: '#aaa' }}>{key}/<Badge count={items.length} /></span>} />
            {/* eslint-disable-next-line @typescript-eslint/no-explicit-any */}
            {(items as any[]).map((item, i, arr) => (renderItem as any)(item, i, arr))}
          </div>
        );
      })}
    </div>
  );
}
