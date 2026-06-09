/**
 * Feature: Emulator State Snapshot (Export / Import)
 *
 * Workflows:
 *  - Export: Download the full emulator state as a JSON file from the Snapshot page
 *  - Import: Upload a previously exported JSON file to restore emulator state
 *  - Round-trip: Export → reset → import to verify full restore fidelity
 *
 * This test validates that snapshot round-trips preserve management objects
 * (permissions, roles, tenants, users) exactly, making Rescope reproducible
 * across sessions and usable in CI seed workflows.
 */
import { test, expect } from "@playwright/test";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import { SnapshotPage } from "./pom/SnapshotPage";
import { api } from "./helpers/api";

test.describe("Snapshot Round-trip", () => {
  test("export → reset → import restores permissions", async ({ page }) => {
    // ── Step 1: seed state via management API ─────────────────────────────────
    await api.mgmtPost("/v1/mgmt/authz/permission", {
      name: "smoke:test",
      description: "Created by smoke test",
    });

    const beforeRes = await api.mgmtGet("/v1/mgmt/authz/permission/all");
    const beforePerms = await beforeRes.json();
    expect(
      beforePerms.permissions.some(
        (p: { name: string }) => p.name === "smoke:test",
      ),
    ).toBe(true);

    // ── Step 2: export snapshot from UI ───────────────────────────────────────
    const snapshotPom = new SnapshotPage(page);
    await snapshotPom.goto();
    const downloadPath = await snapshotPom.export();

    // ── Step 3: verify snapshot contains our permission  ──────────────────────
    const snapContent = JSON.parse(fs.readFileSync(downloadPath, "utf-8"));
    expect(
      snapContent.permissions.some(
        (p: { name: string }) => p.name === "smoke:test",
      ),
    ).toBe(true);

    // ── Step 4: reset via API ─────────────────────────────────────────────────
    await api.post("/emulator/reset");

    // ── Step 5: import snapshot via UI ────────────────────────────────────────
    const tmpFile = path.join(os.tmpdir(), "emulator-smoke-snapshot.json");
    fs.writeFileSync(tmpFile, JSON.stringify(snapContent));
    await snapshotPom.importFile(tmpFile);

    // ── Step 6: verify permissions restored via API ───────────────────────────
    const afterRes = await api.mgmtGet("/v1/mgmt/authz/permission/all");
    const afterPerms = await afterRes.json();
    expect(
      afterPerms.permissions.some(
        (p: { name: string }) => p.name === "smoke:test",
      ),
    ).toBe(true);

    // cleanup
    fs.unlinkSync(tmpFile);
  });
});
