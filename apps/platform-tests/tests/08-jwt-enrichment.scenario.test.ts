/**
 * Scenario: JWT Enrichment and Custom Claims
 *
 * One of Descope's most powerful features is the ability to inject
 * custom claims into JWTs at runtime. This lets apps embed application
 * context (subscription tier, feature flags, org plan) directly into
 * the token so the backend doesn't need extra DB lookups on every request.
 *
 * This scenario tests:
 * - Management API can enrich a JWT with arbitrary claims post-issuance
 * - Enriched claims exist and are correctly typed in the new token
 * - Original claims (sub, exp, etc.) are preserved after enrichment
 * - JWT template list endpoint returns templates (verified via raw fetch
 *   since jwt.template is not part of the Node SDK's typed surface)
 */
import { describe, it, expect, beforeEach } from "vitest";
import { sdk, reset, email, decodeJwt, signUpPassword, BASE_URL, MGMT_AUTH_HEADER } from "../helpers/platform.js";

beforeEach(reset);

describe("Scenario: JWT Enrichment — Custom Claims", () => {
  it("management.jwt.update enriches a token with application claims", async () => {
    const client = sdk();
    const loginId = email("jwt-enrich");
    const sessionJwt = await signUpPassword(client, loginId);

    // After authentication the app backend may want to inject
    // application-level context into the token: subscription plan,
    // feature flags, org tier, etc. This avoids a DB lookup on every
    // authenticated request — the info rides along in the token.
    const enriched = await client.management.jwt.update(sessionJwt, {
      subscriptionTier: "pro",
      maxSeats: 50,
      featureFlags: ["advanced-export", "api-access"],
    });

    expect(enriched.ok, "JWT enrichment should succeed").toBe(true);
    const jwt = enriched.data?.jwt as string;
    expect(jwt).toBeTruthy();

    const claims = decodeJwt(jwt);

    // Application claims should be present exactly as passed.
    expect(claims.subscriptionTier).toBe("pro");
    expect(claims.maxSeats).toBe(50);
    expect(claims.featureFlags).toEqual(["advanced-export", "api-access"]);

    // Standard claims must be preserved — apps rely on sub for user identity.
    const originalClaims = decodeJwt(sessionJwt);
    expect(claims.sub).toBe(originalClaims.sub);
    expect(typeof claims.exp).toBe("number");
  });

  it("JWT template list endpoint returns the template after creation via management API", async () => {
    // jwt.template is not part of the Node SDK's typed management surface,
    // but the emulator exposes it as a REST endpoint that the Admin UI and
    // other tools use. We call it directly here to verify the emulator's
    // template store works end-to-end, using the management key for auth.
    const MGMT_KEY = process.env.EMULATOR_MANAGEMENT_KEY ?? "emulator-key";

    // Create a template via the management REST API directly.
    const createRes = await fetch(`${BASE_URL}/v1/mgmt/jwt/template`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: MGMT_AUTH_HEADER,
      },
      body: JSON.stringify({
        name: "platform-claims",
        type: "user",
        authSchema: JSON.stringify({ claims: {} }),
        template: JSON.stringify({
          userType: "standard",
        }),
      }),
    });

    expect(createRes.ok, "template create should return 200").toBe(true);

    // Verify it's retrievable — the Admin UI depends on this listing
    // to show configured templates in the project dashboard.
    const listRes = await fetch(`${BASE_URL}/v1/mgmt/jwt/template/all`, {
      headers: { Authorization: MGMT_AUTH_HEADER },
    });
    expect(listRes.ok).toBe(true);
    const list = (await listRes.json()) as { templates?: Array<{ name?: string }> };
    const templates = list.templates ?? [];
    const found = templates.find((t: { name?: string }) => t.name === "platform-claims");
    expect(found, "created template should appear in the list").toBeTruthy();
  });
});
