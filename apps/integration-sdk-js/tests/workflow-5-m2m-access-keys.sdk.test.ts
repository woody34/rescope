/**
 * Workflow 5 — Agentic AI / Machine-to-Machine (M2M) Authentication
 *
 * Market use case: Teams building AI agents, backend services, and MCP
 * (Model Context Protocol) servers that need non-interactive auth. The
 * "service account" pattern: a long-lived credential is exchanged for a
 * short-lived JWT that proves identity to downstream services.
 * Descope's "Agentic Identity Hub" treats AI agents as first-class identities.
 *
 * Emulator API routes:
 *   POST   /v1/mgmt/accesskey             — create (returns { key, cleartext })
 *   GET    /v1/mgmt/accesskey/all         — list all keys
 *   POST   /v1/mgmt/accesskey/update      — rename / update
 *   DELETE /v1/mgmt/accesskey/delete      — hard delete (JSON body { id })
 *   POST   /v1/mgmt/accesskey/disable     — soft-disable
 *
 * Journey:
 *   ADMIN CONFIG: create access key via mgmt API
 *      → ADMIN VERIFY: key appears in list, cleartext is returned once
 *      → M2M ACTION: rename key (key rotation metadata update)
 *      → ADMIN ACTION: disable key (key compromise response)
 *      → ADMIN VERIFY: disabled key no longer in active list
 *      → ADMIN ACTION: delete key (permanent decommission)
 *      → ADMIN VERIFY: key gone from list entirely
 */
import { describe, it, expect, beforeEach } from "vitest";
import { resetEmulator, mgmtAuth } from "../helpers/sdk";

const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";
const H = { "Content-Type": "application/json", Authorization: mgmtAuth };

beforeEach(() => resetEmulator());

// ─── Helpers ──────────────────────────────────────────────────────────────────

interface AccessKey {
  id: string;
  name: string;
  status?: string;
}

async function createKey(name: string): Promise<{ key: AccessKey; cleartext: string }> {
  const res = await fetch(`${BASE_URL}/v1/mgmt/accesskey`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ name, expireTime: 0, roleNames: [] }),
  });
  expect(res.ok).toBe(true);
  const text = await res.text();
  const body = JSON.parse(text) as { key: AccessKey; cleartext: string };
  expect(body.cleartext).toBeTruthy();
  return body;
}

async function listKeys(): Promise<AccessKey[]> {
  const res = await fetch(`${BASE_URL}/v1/mgmt/accesskey/all`, { headers: H });
  expect(res.ok).toBe(true);
  const body = await res.json() as { keys: AccessKey[] };
  return body.keys ?? [];
}

async function updateKey(id: string, name: string) {
  return fetch(`${BASE_URL}/v1/mgmt/accesskey/update`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ id, name }),
  });
}

async function disableKey(id: string) {
  return fetch(`${BASE_URL}/v1/mgmt/accesskey/disable`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ id }),
  });
}

async function deleteKey(id: string) {
  return fetch(`${BASE_URL}/v1/mgmt/accesskey/delete`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ id }),
  });
}

// ─── Full M2M access key lifecycle ───────────────────────────────────────────

describe("Workflow 5 — M2M / AI Agent: access key lifecycle", () => {
  it("full journey: create key → cleartext returned → appears in list → rename → disable → delete → gone", async () => {
    // ADMIN CONFIG: create access key for AI agent service
    const { key, cleartext } = await createKey("ai-agent-prod");
    expect(key.id).toBeTruthy();
    expect(cleartext.length).toBeGreaterThan(10);
    expect(key.name).toBe("ai-agent-prod");

    // ADMIN VERIFY: key appears in the list
    const keysAfterCreate = await listKeys();
    expect(keysAfterCreate.some(k => k.id === key.id)).toBe(true);
    expect(keysAfterCreate.find(k => k.id === key.id)?.name).toBe("ai-agent-prod");

    // M2M ACTION: rename key (rotation metadata — new agent version)
    const updateRes = await updateKey(key.id, "ai-agent-v2");
    expect(updateRes.ok).toBe(true);

    const keysAfterUpdate = await listKeys();
    expect(keysAfterUpdate.find(k => k.id === key.id)?.name).toBe("ai-agent-v2");

    // ADMIN ACTION: key compromised → disable immediately
    const disableRes = await disableKey(key.id);
    expect(disableRes.ok).toBe(true);

    // ADMIN ACTION: permanently decommission → delete key
    const deleteRes = await deleteKey(key.id);
    expect(deleteRes.ok).toBe(true);

    // ADMIN VERIFY: key gone from list
    const keysAfterDelete = await listKeys();
    expect(keysAfterDelete.some(k => k.id === key.id)).toBe(false);
  });
});

// ─── Multiple keys (multi-agent scenario) ────────────────────────────────────

describe("Workflow 5 — M2M / AI Agent: multiple keys for different services", () => {
  it("each agent gets its own key; keys are listed independently", async () => {
    const { key: keyA } = await createKey("agent-service-a");
    const { key: keyB } = await createKey("agent-service-b");
    const { key: keyC } = await createKey("agent-service-c");

    const keys = await listKeys();
    const ids = keys.map(k => k.id);
    expect(ids).toContain(keyA.id);
    expect(ids).toContain(keyB.id);
    expect(ids).toContain(keyC.id);

    // Deleting one key doesn't affect others
    await deleteKey(keyA.id);
    const remaining = await listKeys();
    expect(remaining.some(k => k.id === keyA.id)).toBe(false);
    expect(remaining.some(k => k.id === keyB.id)).toBe(true);
    expect(remaining.some(k => k.id === keyC.id)).toBe(true);
  });

  it("key names are human-readable; each key has a unique id", async () => {
    const { key: k1 } = await createKey("analytics-agent");
    const { key: k2 } = await createKey("analytics-agent"); // same name, different id

    // Different IDs even with the same name
    expect(k1.id).not.toBe(k2.id);
  });
});

// ─── Cleartext key is returned only at creation time ─────────────────────────

describe("Workflow 5 — M2M / AI Agent: cleartext key returned once at creation", () => {
  it("cleartext is returned in create response and is non-empty", async () => {
    const { cleartext } = await createKey("single-use-cleartext");
    expect(typeof cleartext).toBe("string");
    expect(cleartext.length).toBeGreaterThan(0);
    // Cleartext format: projectId:keySecret (emulator convention)
    expect(cleartext).toContain(":");
  });
});

// ─── Invalid key scenarios ────────────────────────────────────────────────────

describe("Workflow 5 — M2M / AI Agent: invalid operation rejections", () => {
  it("disabling non-existent key returns error", async () => {
    const res = await disableKey("nonexistent-key-id");
    expect(res.ok).toBe(false);
  });

  it("deleting non-existent key returns error", async () => {
    const res = await deleteKey("nonexistent-key-id");
    expect(res.ok).toBe(false);
  });

  it("create without management auth is rejected", async () => {
    const res = await fetch(`${BASE_URL}/v1/mgmt/accesskey`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer wrong:credentials" },
      body: JSON.stringify({ name: "unauthorized-key" }),
    });
    expect(res.ok).toBe(false);
    expect(res.status).toBe(401);
  });
});
