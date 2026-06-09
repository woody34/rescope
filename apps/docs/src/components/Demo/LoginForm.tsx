import { useState } from 'react';
import OtpForm from './OtpForm';
import MagicLinkForm from './MagicLinkForm';
import JsonBlock from './JsonBlock';
import type { DemoProfile } from './profiles/types';
import type { useEmulator } from './useEmulator';

interface LoginFormProps {
  profile: DemoProfile;
  emulator: ReturnType<typeof useEmulator>;
}

interface ApiResponse {
  ok: boolean;
  status: number;
  data: unknown;
}

const s: Record<string, React.CSSProperties> = {
  wrapper: { display: 'flex', flexDirection: 'column', gap: '1rem', height: '100%' },
  form: {
    background: '#0a0a0f',
    border: '1px solid #1e1e1e',
    borderRadius: '8px',
    padding: '1.25rem',
  },
  label: { color: '#666', fontSize: '0.72rem', fontFamily: "'SF Mono','Fira Code',monospace", display: 'block', marginBottom: '0.3rem' },
  input: {
    width: '100%',
    background: '#111',
    border: '1px solid #2a2a2a',
    borderRadius: '5px',
    color: '#e0e0e0',
    padding: '0.5rem 0.75rem',
    fontSize: '0.85rem',
    fontFamily: "'SF Mono','Fira Code',monospace",
    boxSizing: 'border-box' as const,
    marginBottom: '0.75rem',
    outline: 'none',
  },
  btn: {
    width: '100%',
    background: '#39ff14',
    color: '#000',
    border: 'none',
    borderRadius: '6px',
    padding: '0.6rem',
    fontFamily: "'SF Mono','Fira Code',monospace",
    fontWeight: 700,
    fontSize: '0.85rem',
    cursor: 'pointer',
  },
  responseBox: {
    background: '#0d0d0d',
    border: '1px solid #1e1e1e',
    borderRadius: '8px',
    padding: '1rem',
    fontFamily: "'SF Mono','Fira Code',monospace",
    fontSize: '0.75rem',
    color: '#9fff7e',
    overflowX: 'auto' as const,
    flex: 1,
    minHeight: 120,
    whiteSpace: 'pre' as const,
  },
  responseErr: { color: '#ff6666' },
};

function PasswordForm({ profile, emulator }: LoginFormProps) {
  const [email, setEmail] = useState(profile.loginUI.prefillEmail ?? '');
  const [password, setPassword] = useState(profile.loginUI.prefillPassword ?? '');
  const [resp, setResp] = useState<ApiResponse | null>(null);
  const [loading, setLoading] = useState(false);

  const submit = async () => {
    setLoading(true);
    const r = await emulator.call('POST', '/v1/auth/password/signin', { loginId: email, password });
    setResp(r);
    setLoading(false);
  };

  return (
    <div style={s.wrapper}>
      <div style={s.form}>
        <label style={s.label}>EMAIL</label>
        <input style={s.input} value={email} onChange={(e) => setEmail(e.target.value)} />
        <label style={s.label}>PASSWORD</label>
        <input style={s.input} type="password" value={password} onChange={(e) => setPassword(e.target.value)} />
        <button style={s.btn} onClick={submit} disabled={loading}>{loading ? 'Signing in…' : 'Sign In'}</button>
      </div>
      {resp && (
        <JsonBlock value={JSON.stringify(resp.data, null, 2)} error={!resp.ok} />
      )}
    </div>
  );
}

export default function LoginForm({ profile, emulator }: LoginFormProps) {
  if (profile.loginUI.type === 'otp') return <OtpForm profile={profile} emulator={emulator} />;
  if (profile.loginUI.type === 'magic-link') return <MagicLinkForm profile={profile} emulator={emulator} />;
  return <PasswordForm profile={profile} emulator={emulator} />;
}
