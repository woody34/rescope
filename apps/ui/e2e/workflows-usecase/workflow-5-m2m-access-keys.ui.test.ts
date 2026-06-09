/**
 * UI Workflow 5 — M2M / Agentic AI Access Keys
 *
 * Market use case: Teams building AI agents, backend services, and MCP servers
 * that authenticate non-interactively. Admins create and manage access keys
 * ("service accounts") for each agent.
 *
 * UI coverage:
 *   - Access Keys page: create key → cleartext banner visible → key row appears
 *   - Access Keys page: delete key → row removed
 *   - Access Keys page: multiple keys → count correct
 *   - Access Keys page: cleartext shown once at creation (verify text non-empty)
 */
import { test, expect } from "@playwright/test";

import { AccessKeysPage } from "../pom/AccessKeysPage";
import { resetEmulator } from "../helpers/api";

test.beforeEach(() => resetEmulator());

test.describe("Workflow 5 — M2M Access Keys: Admin UI", () => {
  test("create access key → cleartext shown, then row visible", async ({
    page,
  }) => {
    const keys = await new AccessKeysPage(page).goto();

    await keys.createKey({ name: "ai-agent-prod" });

    // ADMIN VERIFY: cleartext banner shown exactly once
    await expect(keys.locators.cleartextBanner).toBeVisible();
    const cleartext = await keys.getCleartextKey();
    expect(cleartext.length).toBeGreaterThan(10);

    // Dismiss banner and verify row persists
    await keys.dismissCleartextBanner();
    await expect(keys.keyRow("ai-agent-prod")).toBeVisible();
  });

  test("delete access key → removed from table", async ({ page }) => {
    const keys = await new AccessKeysPage(page).goto();
    await keys.createKey({ name: "temp-agent-key" });
    await keys.dismissCleartextBanner();

    await keys.deleteKey("temp-agent-key");

    await expect(keys.locators.emptyTitle).toBeVisible();
  });

  test("multiple agent keys → correct count in table", async ({ page }) => {
    const keys = await new AccessKeysPage(page).goto();

    await keys.createKey({ name: "agent-service-a" });
    await keys.dismissCleartextBanner();
    await keys.createKey({ name: "agent-service-b" });
    await keys.dismissCleartextBanner();
    await keys.createKey({ name: "agent-service-c" });
    await keys.dismissCleartextBanner();

    expect(await keys.getVisibleRowCount()).toBe(3);
    await expect(keys.keyRow("agent-service-a")).toBeVisible();
    await expect(keys.keyRow("agent-service-b")).toBeVisible();
    await expect(keys.keyRow("agent-service-c")).toBeVisible();
  });

  test("cleartext key format is valid (projectId:secret)", async ({ page }) => {
    const keys = await new AccessKeysPage(page).goto();
    await keys.createKey({ name: "format-check-key" });

    await expect(keys.locators.cleartextBanner).toBeVisible();
    const cleartext = await keys.getCleartextKey();
    // Emulator format: "K<uuid>:<uuid>" or "projectId:secret"
    expect(cleartext).toContain(":");
    await keys.dismissCleartextBanner();
  });
});
