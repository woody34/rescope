import type { DemoProfile } from './profiles/types';

interface ProfileBarProps {
  profiles: DemoProfile[];
  active: DemoProfile | null;
  onSelect: (profile: DemoProfile) => void;
  disabled?: boolean;
}

const s: Record<string, React.CSSProperties> = {
  bar: {
    display: 'flex',
    gap: '0.5rem',
    flexWrap: 'wrap',
    marginBottom: '1.25rem',
  },
  btn: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.4rem',
    padding: '0.45rem 0.9rem',
    borderRadius: '6px',
    border: '1px solid #2a2a2a',
    background: '#111',
    color: '#888',
    cursor: 'pointer',
    fontFamily: "'SF Mono', 'Fira Code', monospace",
    fontSize: '0.8rem',
    transition: 'all 0.15s',
    whiteSpace: 'nowrap' as const,
  },
  btnActive: {
    borderColor: '#39ff14',
    background: 'rgba(57,255,20,0.08)',
    color: '#39ff14',
  },
};

export default function ProfileBar({ profiles, active, onSelect, disabled }: ProfileBarProps) {
  return (
    <div style={s.bar}>
      {profiles.map((p) => {
        const isActive = active?.id === p.id;
        return (
          <button
            key={p.id}
            onClick={() => !disabled && onSelect(p)}
            style={{ ...s.btn, ...(isActive ? s.btnActive : {}) }}
            title={p.description}
            aria-pressed={isActive}
          >
            <span style={{ fontSize: '0.85em', lineHeight: 1 }}>{p.icon}</span>
            <span>{p.label}</span>
          </button>
        );
      })}
    </div>
  );
}
