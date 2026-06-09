import { useState } from 'react';
import JsonBlock from './JsonBlock';
import type { DemoProfile } from './profiles/types';
import type { useEmulator } from './useEmulator';

interface OtpFormProps {
  profile: DemoProfile;
  emulator: ReturnType<typeof useEmulator>;
}

const s: Record<string, React.CSSProperties> = {
  form: { background: '#0a0a0f', border: '1px solid #1e1e1e', borderRadius: '8px', padding: '1.25rem' },
  label: { color: '#666', fontSize: '0.72rem', fontFamily: "'SF Mono','Fira Code',monospace", display: 'block', marginBottom: '0.3rem' },
  input: { width: '100%', background: '#111', border: '1px solid #2a2a2a', borderRadius: '5px', color: '#e0e0e0', padding: '0.5rem 0.75rem', fontSize: '0.85rem', fontFamily: "'SF Mono','Fira Code',monospace", boxSizing: 'border-box' as const, marginBottom: '0.75rem', outline: 'none' },
  btn: { width: '100%', background: '#39ff14', color: '#000', border: 'none', borderRadius: '6px', padding: '0.6rem', fontFamily: "'SF Mono','Fira Code',monospace", fontWeight: 700, fontSize: '0.85rem', cursor: 'pointer', marginBottom: '0.5rem' },
  codeBox: { background: '#0d1a0d', border: '1px solid #39ff14', borderRadius: '6px', padding: '0.75rem 1rem', fontFamily: "'SF Mono','Fira Code',monospace", color: '#39ff14', fontSize: '1.2rem', letterSpacing: '0.2em', textAlign: 'center' as const, marginBottom: '0.75rem' },
  hint: { color: '#555', fontSize: '0.72rem', textAlign: 'center' as const, marginBottom: '0.75rem' },
  responseBox: { background: '#0d0d0d', border: '1px solid #1e1e1e', borderRadius: '8px', padding: '1rem', fontFamily: "'SF Mono','Fira Code',monospace", fontSize: '0.75rem', color: '#9fff7e', overflowX: 'auto' as const, whiteSpace: 'pre' as const },
};

type Step = 'email' | 'code' | 'done';

export default function OtpForm({ profile, emulator }: OtpFormProps) {
  const [step, setStep] = useState<Step>('email');
  const [email, setEmail] = useState(profile.loginUI.prefillEmail ?? '');
  const [code, setCode] = useState('');
  const [retrievedCode, setRetrievedCode] = useState('');
  const [loading, setLoading] = useState(false);
  const [resp, setResp] = useState<unknown>(null);
  const [error, setError] = useState('');

  const requestOtp = async () => {
    setLoading(true); setError('');
    const r = await emulator.call('POST', '/v1/auth/otp/signin/email', { loginId: email });
    if (r.ok) {
      const fetched = await emulator.getOtp(email);
      setRetrievedCode(fetched ?? '');
      setCode(fetched ?? '');
      setStep('code');
    } else {
      setError(JSON.stringify(r.data, null, 2));
    }
    setLoading(false);
  };

  const verifyOtp = async () => {
    setLoading(true); setError('');
    const r = await emulator.call('POST', '/v1/auth/otp/verify/email', { loginId: email, code });
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
            <button style={s.btn} onClick={requestOtp} disabled={loading}>{loading ? 'Sending…' : 'Send OTP'}</button>
          </>
        )}
        {(step === 'code' || step === 'done') && (
          <>
            <label style={s.label}>OTP CODE (from emulator)</label>
            <div style={s.codeBox}>{retrievedCode || '—'}</div>
            <p style={s.hint}>↑ No real email sent — Rescope exposes the code instantly</p>
            {step === 'code' && (
              <>
                <label style={s.label}>ENTER CODE</label>
                <input style={s.input} value={code} onChange={(e) => setCode(e.target.value)} />
                <button style={s.btn} onClick={verifyOtp} disabled={loading}>{loading ? 'Verifying…' : 'Verify'}</button>
              </>
            )}
          </>
        )}
        {error && <JsonBlock value={error} error />}
      </div>
      {resp != null && <JsonBlock value={JSON.stringify(resp, null, 2)} />}
    </div>
  );
}
