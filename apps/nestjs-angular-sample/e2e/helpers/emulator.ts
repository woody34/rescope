/**
 * Emulator management API helpers for E2E tests.
 * All calls go directly to the emulator — no CORS issue from Playwright's context.
 */
import * as dotenv from 'dotenv';
dotenv.config();

const BASE_URL = process.env.EMULATOR_BASE_URL ?? 'http://localhost:4600';
const PROJECT_ID = process.env.EMULATOR_PROJECT_ID ?? 'emulator-project';
const MGMT_KEY = process.env.EMULATOR_MANAGEMENT_KEY ?? 'emulator-key';

export const MGMT_AUTH = `Bearer ${PROJECT_ID}:${MGMT_KEY}`;
const H = { 'Content-Type': 'application/json', Authorization: MGMT_AUTH };

let _counter = 0;
export const uniqueLogin = (prefix = 'e2e') =>
  `${prefix}-${Date.now()}-${++_counter}@e2e.test`;

export async function resetEmulator(): Promise<void> {
  await fetch(`${BASE_URL}/emulator/reset`, { method: 'POST' });
}

export async function createUser(loginId: string): Promise<void> {
  await fetch(`${BASE_URL}/v1/mgmt/user/create`, {
    method: 'POST', headers: H,
    body: JSON.stringify({ loginId, email: loginId }),
  });
}

export async function createUserWithPassword(loginId: string, password: string): Promise<void> {
  await createUser(loginId);
  await fetch(`${BASE_URL}/v1/mgmt/user/password/set/active`, {
    method: 'POST', headers: H,
    body: JSON.stringify({ loginId, password }),
  });
}

export async function createTestUser(loginId: string): Promise<void> {
  await fetch(`${BASE_URL}/v1/mgmt/user/create/test`, {
    method: 'POST', headers: H,
    body: JSON.stringify({ loginId, email: loginId }),
  });
}

/** Creates a test user with a password — used for flows needing both password sign-in and OTP bypass */
export async function createTestUserWithPassword(loginId: string, password: string): Promise<void> {
  await createTestUser(loginId);
  await fetch(`${BASE_URL}/v1/mgmt/user/password/set/active`, {
    method: 'POST', headers: H,
    body: JSON.stringify({ loginId, password }),
  });
}


export async function generateMagicLink(loginId: string): Promise<string> {
  const res = await fetch(`${BASE_URL}/v1/mgmt/tests/generate/magiclink`, {
    method: 'POST', headers: H,
    body: JSON.stringify({ loginId, URI: 'http://localhost:4444/verify' }),
  });
  const body = await res.json() as { token: string };
  return body.token;
}

export async function generateOtp(loginId: string): Promise<string> {
  const res = await fetch(`${BASE_URL}/v1/mgmt/tests/generate/otp`, {
    method: 'POST', headers: H,
    body: JSON.stringify({ loginId, deliveryMethod: 'email' }),
  });
  const body = await res.json() as { code: string };
  return body.code;
}

export async function createTenant(id: string, name: string): Promise<void> {
  await fetch(`${BASE_URL}/v1/mgmt/tenant/create`, {
    method: 'POST', headers: H,
    body: JSON.stringify({ id, name }),
  });
}

export async function addUserToTenant(loginId: string, tenantId: string, roleNames: string[]): Promise<void> {
  await fetch(`${BASE_URL}/v1/mgmt/user/tenant/add`, {
    method: 'POST', headers: H,
    body: JSON.stringify({ loginId, tenantId, roleNames }),
  });
}

export async function createAccessKey(name: string): Promise<{ id: string; cleartext: string }> {
  const res = await fetch(`${BASE_URL}/v1/mgmt/accesskey`, {
    method: 'POST', headers: H,
    body: JSON.stringify({ name, expireTime: 0, roleNames: [] }),
  });
  const body = await res.json() as { key: { id: string }; cleartext: string };
  return { id: body.key.id, cleartext: body.cleartext };
}

/** Delete all access keys (resetEmulator does NOT clear them) */
export async function deleteAllAccessKeys(): Promise<void> {
  const res = await fetch(`${BASE_URL}/v1/mgmt/accesskey/all`, { method: 'GET', headers: H });
  const body = await res.json() as { keys?: Array<{ id: string }> };
  const keys = body.keys ?? [];
  await Promise.all(
    keys.map(k =>
      // Emulator registers delete as POST (not DELETE)
      fetch(`${BASE_URL}/v1/mgmt/accesskey/delete`, {
        method: 'POST', headers: H,
        body: JSON.stringify({ id: k.id }),
      }),
    ),
  );
}

export async function forceLogout(loginId: string): Promise<void> {
  await fetch(`${BASE_URL}/v1/mgmt/user/logout`, {
    method: 'POST', headers: H,
    body: JSON.stringify({ loginId }),
  });
}

export async function setUserStatus(loginId: string, status: 'enabled' | 'disabled'): Promise<void> {
  await fetch(`${BASE_URL}/v1/mgmt/user/status`, {
    method: 'POST', headers: H,
    body: JSON.stringify({ loginId, status }),
  });
}

/** Inject JWTs into localStorage (mirrors sample-app auth.setup.ts pattern) */
export async function injectSession(page: import('@playwright/test').Page, sessionJwt: string, refreshJwt?: string): Promise<void> {
  await page.evaluate(([ds, dsr]: [string, string | undefined]) => {
    window.localStorage.setItem('DS', ds);
    if (dsr) window.localStorage.setItem('DSR', dsr);
  }, [sessionJwt, refreshJwt] as [string, string | undefined]);
}
