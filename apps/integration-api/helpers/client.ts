/**
 * Typed fetch wrapper for the Descope Emulator.
 * All tests import from here via: { client, resetEmulator, uniqueLogin }
 */

const baseUrl = () =>
  process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";

// ─── Config ───────────────────────────────────────────────────────────────────

const projectId = process.env.EMULATOR_PROJECT_ID ?? "emulator-project";
const mgmtKey = process.env.EMULATOR_MANAGEMENT_KEY ?? "emulator-key";
const MANAGEMENT_AUTH = `Bearer ${projectId}:${mgmtKey}`;

// ─── Raw helpers ──────────────────────────────────────────────────────────────

function doFetch(path: string, init: RequestInit): Promise<Response> {
  return fetch(`${baseUrl()}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(init.headers as Record<string, string>),
    },
  });
}

function doPost(path: string, body?: unknown, headers?: Record<string, string>) {
  return doFetch(path, {
    method: "POST",
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
}

function doGet(path: string, headers?: Record<string, string>) {
  return doFetch(path, { method: "GET", headers });
}

function doDelete(path: string, headers?: Record<string, string>) {
  return doFetch(path, { method: "DELETE", headers });
}

function doPatch(path: string, body?: unknown, headers?: Record<string, string>) {
  return doFetch(path, {
    method: "PATCH",
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
}

// ─── client object ────────────────────────────────────────────────────────────

export const client = {
  /** Unauthenticated POST */
  post: (path: string, body?: unknown) => doPost(path, body),

  /** Unauthenticated GET */
  get: (path: string) => doGet(path),

  /** Management POST (Authorization header included) */
  mgmtPost: (path: string, body?: unknown) => doPost(path, body, { Authorization: MANAGEMENT_AUTH }),

  /** Management GET */
  mgmtGet: (path: string) => doGet(path, { Authorization: MANAGEMENT_AUTH }),

  /** Management PATCH */
  mgmtPatch: (path: string, body?: unknown) => doPatch(path, body, { Authorization: MANAGEMENT_AUTH }),

  /** Management DELETE */
  mgmtDelete: (path: string) => doDelete(path, { Authorization: MANAGEMENT_AUTH }),
};

// ─── Test utilities ───────────────────────────────────────────────────────────

/** Reset emulator state between tests (clears all users/tenants/tokens) */
export const resetEmulator = async (): Promise<void> => {
  await client.post("/emulator/reset");
};

/** Generate unique login IDs to prevent cross-test pollution */
let _counter = 0;
export const uniqueLogin = (prefix = "user") =>
  `${prefix}-${Date.now()}-${++_counter}@test.example`;
