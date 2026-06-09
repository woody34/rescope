import { useCallback } from 'react';
import type { DemoProfile, DemoSeed } from './profiles/types';

const BASE = '/demo-wasm';

async function apiCall(method: string, path: string, body?: unknown) {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: body != null ? JSON.stringify(body) : undefined,
  });
  const text = await res.text();
  try { return { ok: res.ok, status: res.status, data: JSON.parse(text) }; }
  catch { return { ok: res.ok, status: res.status, data: text }; }
}

async function seedEmulator(seed: DemoSeed, managementKey: string) {
  const headers = {
    'Content-Type': 'application/json',
    Authorization: `Bearer ${managementKey}`,
  };

  // Create roles first
  for (const role of seed.roles ?? []) {
    await fetch(`${BASE}/v1/mgmt/role/create`, {
      method: 'POST', headers,
      body: JSON.stringify({ name: role.name, description: role.description ?? '' }),
    });
  }

  // Create tenants
  for (const tenant of seed.tenants ?? []) {
    await fetch(`${BASE}/v1/mgmt/tenant/create`, {
      method: 'POST', headers,
      body: JSON.stringify({ id: tenant.id, name: tenant.name }),
    });
  }

  // Create users (batch)
  await fetch(`${BASE}/v1/mgmt/user/create/batch`, {
    method: 'POST', headers,
    body: JSON.stringify({
      users: seed.users.map((u) => ({
        loginId: u.loginId,
        email: u.email,
        name: u.name,
        ...(u.password ? { password: u.password } : {}),
        roleNames: u.roleNames ?? [],
        userTenants: u.userTenants ?? [],
      })),
    }),
  });
}

export function useEmulator(profile: DemoProfile | null) {
  const getSnapshot = useCallback(async () => {
    const res = await fetch(`${BASE}/emulator/snapshot`);
    return res.ok ? res.json() : null;
  }, []);

  const reset = useCallback(async () => {
    await fetch(`${BASE}/emulator/reset`, { method: 'POST' });
    if (profile) {
      // Re-seed with current profile after reset
      const snap = await fetch(`${BASE}/health`).then((r) => r.json()).catch(() => ({}));
      const mgmtKey = snap.management_key ?? 'demo-key';
      await seedEmulator(profile.seed, mgmtKey);
    }
  }, [profile]);

  const seed = useCallback(async (p: DemoProfile) => {
    // Reset first, then seed
    await fetch(`${BASE}/emulator/reset`, { method: 'POST' });
    // Get management key from startup (use a known default or fetch from health)
    const healthRes = await fetch(`${BASE}/health`).catch(() => null);
    const health = healthRes?.ok ? await healthRes.json() : {};
    const mgmtKey = health.management_key ?? 'demo-key';
    await seedEmulator(p.seed, mgmtKey);
  }, []);

  const call = useCallback(apiCall, []);

  const getOtp = useCallback(async (loginId: string) => {
    const res = await fetch(`${BASE}/emulator/otp/${encodeURIComponent(loginId)}`);
    if (!res.ok) return null;
    const data = await res.json();
    return data.code as string;
  }, []);

  return { seed, reset, getSnapshot, call, getOtp };
}
