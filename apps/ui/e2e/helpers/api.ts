/**
 * Shared API client for e2e tests — mirrors integration/api/helpers/client.ts.
 * Talks directly to the emulator REST API (bypassing the UI) for seeding/asserting state.
 */

export const EMULATOR_BASE = "http://localhost:4600";

const projectId = process.env.EMULATOR_PROJECT_ID ?? "emulator-project";
const mgmtKey = process.env.EMULATOR_MANAGEMENT_KEY ?? "emulator-key";
const MGMT_AUTH = `Bearer ${projectId}:${mgmtKey}`;

async function request(path: string, init: RequestInit): Promise<Response> {
  return fetch(`${EMULATOR_BASE}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(init.headers as Record<string, string>),
    },
  });
}

export const api = {
  /** Unauthenticated POST */
  post: (path: string, body?: unknown) =>
    request(path, {
      method: "POST",
      body: body !== undefined ? JSON.stringify(body) : undefined,
    }),

  /** Unauthenticated GET */
  get: (path: string) => request(path, { method: "GET" }),

  /** Management POST */
  mgmtPost: (path: string, body?: unknown) =>
    request(path, {
      method: "POST",
      headers: { Authorization: MGMT_AUTH },
      body: body !== undefined ? JSON.stringify(body) : undefined,
    }),

  /** Management GET */
  mgmtGet: (path: string) =>
    request(path, { method: "GET", headers: { Authorization: MGMT_AUTH } }),

  /** Management DELETE */
  mgmtDelete: (path: string, body?: unknown) =>
    request(path, {
      method: "DELETE",
      headers: { Authorization: MGMT_AUTH },
      body: body !== undefined ? JSON.stringify(body) : undefined,
    }),
};

/**
 * Reset emulator state between tests.
 * Also clears config stores (permissions, roles, access keys) that
 * /emulator/reset intentionally preserves (they mirror Descope project config).
 */
export async function resetEmulator(): Promise<void> {
  // 1. Reset runtime state (users, tokens, OTPs, sessions)
  await api.post("/emulator/reset");

  // 2. Clear permissions (config store) — delete endpoint is POST, not DELETE
  try {
    const permResp = await api.mgmtGet("/v1/mgmt/authz/permission/all");
    const permData = await permResp.json();
    for (const perm of permData.permissions ?? []) {
      await api.mgmtPost("/v1/mgmt/authz/permission/delete", {
        name: perm.name,
      });
    }
  } catch {
    /* ignore */
  }

  // 3. Clear roles (config store) — delete endpoint is POST, not DELETE
  try {
    const roleResp = await api.mgmtGet("/v1/mgmt/authz/role/all");
    const roleData = await roleResp.json();
    for (const role of roleData.roles ?? []) {
      await api.mgmtPost("/v1/mgmt/authz/role/delete", { name: role.name });
    }
  } catch {
    /* ignore */
  }

  // 4. Clear access keys (config store) — delete endpoint is POST, not DELETE
  try {
    const keysResp = await api.mgmtGet("/v1/mgmt/accesskey/all");
    const keysData = await keysResp.json();
    for (const key of keysData.keys ?? []) {
      await api.mgmtPost("/v1/mgmt/accesskey/delete", { id: key.id });
    }
  } catch {
    /* ignore */
  }
}

/** Generate a unique login ID to avoid cross-test pollution */
let _counter = 0;
export const uniqueLogin = (prefix = "user") =>
  `${prefix}-${Date.now()}-${++_counter}@test.example`;
