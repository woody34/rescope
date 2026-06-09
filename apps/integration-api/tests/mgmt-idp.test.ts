import { describe, it, expect, beforeEach } from "vitest";
import { client, resetEmulator } from "../helpers/client";

beforeEach(() => resetEmulator());

// ─── POST /v1/mgmt/idp ───────────────────────────────────────────────────────

describe("POST /v1/mgmt/idp", () => {
  it("creates an OIDC IdP emulator", async () => {
    const res = await client.mgmtPost("/v1/mgmt/idp", {
      protocol: "oidc",
      displayName: "Mock Okta",
      tenantId: "acme",
      attributeMapping: { email: "user.email", name: "user.name" },
    });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.idp.id).toBeTruthy();
    expect(body.idp.protocol).toBe("oidc");
    expect(body.idp.displayName).toBe("Mock Okta");
    expect(body.idp.tenantId).toBe("acme");
    expect(body.idp.attributeMapping.email).toBe("user.email");
  });

  it("creates a SAML IdP emulator", async () => {
    const res = await client.mgmtPost("/v1/mgmt/idp", {
      protocol: "saml",
      displayName: "Mock Azure AD",
      tenantId: "contoso",
    });
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.idp.protocol).toBe("saml");
  });
});

// ─── GET /v1/mgmt/idp/all ────────────────────────────────────────────────────

describe("GET /v1/mgmt/idp/all", () => {
  it("returns empty list initially", async () => {
    const res = await client.mgmtGet("/v1/mgmt/idp/all");
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.idps).toEqual([]);
  });

  it("returns all created IdPs", async () => {
    await client.mgmtPost("/v1/mgmt/idp", {
      protocol: "oidc",
      displayName: "IdP 1",
      tenantId: "t1",
    });
    await client.mgmtPost("/v1/mgmt/idp", {
      protocol: "saml",
      displayName: "IdP 2",
      tenantId: "t2",
    });
    const res = await client.mgmtGet("/v1/mgmt/idp/all");
    const body = await res.json();
    expect(body.idps).toHaveLength(2);
  });
});

// ─── POST /v1/mgmt/idp/update ────────────────────────────────────────────────

describe("POST /v1/mgmt/idp/update", () => {
  it("updates display name", async () => {
    const created = await client.mgmtPost("/v1/mgmt/idp", {
      protocol: "oidc",
      displayName: "Old Name",
      tenantId: "acme",
    });
    const { idp } = await created.json();
    const res = await client.mgmtPost("/v1/mgmt/idp/update", {
      id: idp.id,
      displayName: "New Name",
    });
    expect(res.status).toBe(200);

    const list = await client.mgmtGet("/v1/mgmt/idp/all");
    const { idps } = await list.json();
    expect(idps[0].displayName).toBe("New Name");
  });

  it("updates attribute mapping", async () => {
    const created = await client.mgmtPost("/v1/mgmt/idp", {
      protocol: "oidc",
      displayName: "IdP",
      tenantId: "acme",
      attributeMapping: { email: "user.email" },
    });
    const { idp } = await created.json();
    await client.mgmtPost("/v1/mgmt/idp/update", {
      id: idp.id,
      attributeMapping: { department: "user.customAttributes.department" },
    });

    const list = await client.mgmtGet("/v1/mgmt/idp/all");
    const { idps } = await list.json();
    expect(idps[0].attributeMapping.department).toBe("user.customAttributes.department");
    expect(idps[0].attributeMapping.email).toBeUndefined(); // replaced, not merged
  });

  it("rejects update for unknown IdP", async () => {
    const res = await client.mgmtPost("/v1/mgmt/idp/update", {
      id: "ghost",
      displayName: "Nope",
    });
    expect(res.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── POST /v1/mgmt/idp/delete ────────────────────────────────────────────────

describe("POST /v1/mgmt/idp/delete", () => {
  it("deletes an IdP emulator", async () => {
    const created = await client.mgmtPost("/v1/mgmt/idp", {
      protocol: "oidc",
      displayName: "To Delete",
      tenantId: "acme",
    });
    const { idp } = await created.json();
    const res = await client.mgmtPost("/v1/mgmt/idp/delete", { id: idp.id });
    expect(res.status).toBe(200);

    const list = await client.mgmtGet("/v1/mgmt/idp/all");
    const { idps } = await list.json();
    expect(idps).toHaveLength(0);
  });

  it("rejects delete for unknown IdP", async () => {
    const res = await client.mgmtPost("/v1/mgmt/idp/delete", { id: "ghost" });
    expect(res.status).toBeGreaterThanOrEqual(400);
  });
});

// ─── Snapshot Roundtrip ──────────────────────────────────────────────────────

describe("Snapshot with IdP emulators", () => {
  it("persists IdP configs through export/import", async () => {
    // Create an IdP
    await client.mgmtPost("/v1/mgmt/idp", {
      protocol: "oidc",
      displayName: "Snapshot IdP",
      tenantId: "acme",
      attributeMapping: { email: "user.email" },
    });

    // Export snapshot
    const exportRes = await client.get("/emulator/snapshot");
    const snapshot = await exportRes.json();
    expect(snapshot.idpEmulators).toHaveLength(1);
    expect(snapshot.idpEmulators[0].displayName).toBe("Snapshot IdP");

    // Reset clears IdPs
    await resetEmulator();
    const afterReset = await client.mgmtGet("/v1/mgmt/idp/all");
    expect((await afterReset.json()).idps).toHaveLength(0);

    // Import restores them
    await client.post("/emulator/snapshot", snapshot);
    const afterImport = await client.mgmtGet("/v1/mgmt/idp/all");
    const { idps } = await afterImport.json();
    expect(idps).toHaveLength(1);
    expect(idps[0].displayName).toBe("Snapshot IdP");
    expect(idps[0].attributeMapping.email).toBe("user.email");
  });
});
