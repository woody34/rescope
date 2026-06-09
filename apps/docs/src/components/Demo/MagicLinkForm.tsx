import { useState } from 'react';
import JsonBlock from './JsonBlock';
import type { DemoProfile } from './profiles/types';
import type { useEmulator } from './useEmulator';

interface MagicLinkFormProps {
  profile: DemoProfile;
  emulator: ReturnType<typeof useEmulator>;
}

const s: Record<string, React.CSSProperties> = {
  form: { background: '#0a0a0f', border: '1px solid #1e1e1e', borderRadius: '8px', padding: '1.25rem' },
  label: { color: '#666', fontSize: '0.72rem', fontFamily: "'SF Mono','Fira Code',monospace", display: 'block', marginBottom: '0.3rem' },
  input: { width: '100%', background: '#111', border: '1px solid #2a2a2a', borderRadius: '5px', color: '#e0e0e0', padding: '0.5rem 0.75rem', fontSize: '0.85rem', fontFamily: "'SF Mono','Fira Code',monospace", boxSizing: 'border-box' as const, marginBottom: '0.75rem', outline: 'none' },
  btn: { width: '100%', background: '#39ff14', color: '#000', border: 'none', borderRadius: '6px', padding: '0.6rem', fontFamily: "'SF Mono','Fira Code',monospace", fontWeight: 700, fontSize: '0.85rem', cursor: 'pointer', marginBottom: '0.5rem' },
  tokenBox: { background: '#0d0d1a', border: '1px solid #39ff14', borderRadius: '6px', padding: '0.75rem', fontFamily: "'SF Mono','Fira Code',monospace", color: '#66b3ff', fontSize: '0.68rem', wordBreak: 'break-all' as const, marginBottom: '0.75rem' },
  hint: { color: '#555', fontSize: '0.72rem', textAlign: 'center' as const, marginBottom: '0.75rem' },
  responseBox: { background: '#0d0d0d', border: '1px solid #1e1e1e', borderRadius: '8px', padding: '1rem', fontFamily: "'SF Mono','Fira Code',monospace", fontSize: '0.75rem', color: '#9fff7e', overflowX: 'auto' as const, whiteSpace: 'pre' as const },
};

type Step = 'email' | 'token' | 'done';

export default function MagicLinkForm({ profile, emulator }: MagicLinkFormProps) {
  const [step, setStep] = useState<Step>('email');
  const [email, setEmail] = useState(profile.loginUI.prefillEmail ?? '');
  const [token, setToken] = useState('');
  const [loading, setLoading] = useState(false);
  const [resp, setResp] = useState<unknown>(null);
  const [error, setError] = useState('');

  const requestLink = async () => {
    setLoading(true); setError('');
    const r = await emulator.call('POST', '/v1/auth/magiclink/signin/email', {
      loginId: email,
      URI: window.location.origin + '/demo',
    });
    if (r.ok) {
      const t = await emulator.getOtp(email);
      setToken(t ?? '');
      setStep('token');
    } else {
      setError(JSON.stringify(r.data, null, 2));
    }
    setLoading(false);
  };

  const verifyToken = async () => {
    setLoading(true); setError('');
    const r = await emulator.call('POST', '/v1/auth/magiclink/verify', { token });
    if (r.ok) { setResp(r.data); setStep('done'); }
    else { setError(JSON.stringify(r.data, null, 2)); }
    setLoading(false);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', height: '100%' }}>
      <div style={s.form}>
        {step === 'email' && (
          <>
            <label style={s.label}>EMAIL</label>
            <input style={s.input} value={email} onChange={(e) => setEmail(e.target.value)} />
            <button style={s.btn} onClick={requestLink} disabled={loading}>{loading ? 'Sending…' : 'Send Magic Link'}</button>
          </>
        )}
        {(step === 'token' || step === 'done') && (
          <>
            <label style={s.label}>MAGIC LINK TOKEN (from emulator)</label>
            <div style={s.tokenBox}>{token || '—'}</div>
            <p style={s.hint}>↑ No real email sent — token intercepted by the emulator</p>
            {step === 'token' && (
              <button style={s.btn} onClick={verifyToken} disabled={loading}>{loading ? 'Verifying…' : '✓ Click Link (Verify Token)'}</button>
            )}
          </>
        )}
        {error && <JsonBlock value={error} error />}
      </div>
      {resp != null && <JsonBlock value={JSON.stringify(resp, null, 2)} />}
    </div>
  );
}
