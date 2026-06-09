/**
 * Scenario: M2M / AI Agent Authentication (Access Keys)
 *
 * Access keys enable machine-to-machine (M2M) authentication — any
 * automated process (CI pipeline, AI agent, background worker) that
 * needs to call your API without a human user session.
 *
 * NOTE: The access key exchange endpoint (/v1/auth/accesskey/exchange) is not
 * yet implemented in the emulator. These tests cover the management lifecycle
 * (create, list, disable, delete) which is what the WF5 UI tests exercise.
 *
 * This scenario tests:
 * - Creating a key returns an ID and one-time cleartext secret
 * - A newly created key appears in the listing
 * - Disabling a key changes its status in the listing
 * - Deleting a key removes it from the listing
 * - Multiple keys coexist independently — deleting one doesn't affect others
 */
import { describe, it, expect, beforeEach } from "vitest";
import { reset, BASE_URL, MGMT_AUTH_HEADER } from "../helpers/platform.js";

const H = { "Content-Type": "application/json", Authorization: MGMT_AUTH_HEADER };

/** Create an access key via the emulator's POST /v1/mgmt/accesskey. */
async function createKey(name: string): Promise<{ id: string; cleartext: string }> {
  const res = await fetch(`${BASE_URL}/v1/mgmt/accesskey`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ name, expireTime: 0, roleNames: [] }),
  });
  if (!res.ok) throw new Error(`createKey failed: ${res.status}`);
  const body = (await res.json()) as { key: { id: string }; cleartext: string };
  return { id: body.key.id, cleartext: body.cleartext };
}

async function listKeys(): Promise<Array<{ id: string; name: string; status?: string }>> {
  const res = await fetch(`${BASE_URL}/v1/mgmt/accesskey/all`, { headers: H });
  const body = (await res.json()) as { keys?: Array<{ id: string; name: string; status?: string }> };
  return body.keys ?? [];
}

async function disableKey(id: string): Promise<void> {
  const res = await fetch(`${BASE_URL}/v1/mgmt/accesskey/disable`, {
    method: "POST", headers: H,
    body: JSON.stringify({ id }),
  });
  if (!res.ok) throw new Error(`disableKey failed: ${res.status}`);
}

async function deleteKey(id: string): Promise<void> {
  await fetch(`${BASE_URL}/v1/mgmt/accesskey/delete`, {
    method: "POST", headers: H,
    body: JSON.stringify({ id }),
  });
}

async function deleteAllKeys(): Promise<void> {
  const keys = await listKeys();
  await Promise.all(keys.map((k) => deleteKey(k.id)));
}

beforeEach(async () => {
  await reset();
  await deleteAllKeys();
});

describe("Scenario: M2M / AI Agent (Access Keys)", () => {
  it("create returns key ID and one-time cleartext; key appears in listing", async () => {
    const { id, cleartext } = await createKey("ai-agent-prod");
    expect(id, "key ID must be returned on creation").toBeTruthy();
    expect(cleartext, "cleartext secret must be returned on creation (write once)").toBeTruthy();

    // The key must immediately appear in the management listing.
    const keys = await listKeys();
    const found = keys.find((k) => k.id === id);
    expect(found, "newly created key must appear in the listing").toBeTruthy();
    expect(found?.name).toBe("ai-agent-prod");
  });

  it("disabled key has 'inactive' status in the listing", async () => {
    const { id } = await createKey("disable-test");

    await disableKey(id);

    const keys = await listKeys();
    const found = keys.find((k) => k.id === id);
    // Emulator serializes AccessKeyStatus::Disabled as 'disabled' (camelCase serde).
    expect(found?.status, "disabled key status should be 'disabled'").toBe("disabled");
  });

  it("deleted key does not appear in the listing", async () => {
    const { id } = await createKey("to-delete");

    await deleteKey(id);

    const keys = await listKeys();
    const ids = keys.map((k) => k.id);
    expect(ids).not.toContain(id);
  });

  it("multiple keys coexist; deleting one does not remove others", async () => {
    const keyA = await createKey("agent-a");
    const keyB = await createKey("agent-b");
    const keyC = await createKey("agent-c");

    await deleteKey(keyA.id);

    const keys = await listKeys();
    const ids = keys.map((k) => k.id);
    expect(ids).not.toContain(keyA.id); // deleted
    expect(ids).toContain(keyB.id);     // unaffected
    expect(ids).toContain(keyC.id);     // unaffected
  });

  it("listing after sequential creates returns all keys", async () => {
    const names = ["worker-1", "worker-2", "worker-3"];
    const created = await Promise.all(names.map(createKey));

    const keys = await listKeys();
    for (const { id } of created) {
      expect(keys.some((k) => k.id === id), `key ${id} must appear in listing`).toBe(true);
    }
    expect(keys.length, "listing should have exactly 3 keys").toBe(names.length);
  });
});
