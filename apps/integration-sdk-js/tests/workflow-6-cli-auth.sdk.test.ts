/**
 * Workflow 6 — CLI Application Auth
 *
 * Market use case: Developer tool companies (think Stripe CLI, GitHub CLI,
 * Vercel CLI) that want their end-users to authenticate via browser and have
 * the CLI receive a long-lived refresh token for ongoing API access.
 *
 * In a real CLI flow: the tool opens a browser → user authenticates → browser
 * redirects to a local callback → CLI exchanges the code for tokens. In the
 * emulator's test context we bypass the browser using the test-user magic link
 * mechanism (which represents the "code received at callback").
 *
 * Journey:
 *   ADMIN CONFIG: create test user (represents a developer who has a Descope account)
 *      → CLI ACTION (simulated): send magic link / "start login" request
 *      → CLI CALLBACK: receive token out-of-band, exchange for session tokens
 *      → CLI stores refresh token (simulated by variable)
 *      → CLI uses sessionJwt to call protected API (/v1/auth/validate)
 *      → CLI silently refreshes session when sessionJwt expires (refresh call)
 *      → CLI `logout` command → session fully revoked
 *      → Verify subsequent me/refresh rejected
 *
 * Also covers: enchanted links (used in CLI for cross-device flows),
 * re-authentication after explicit logout, and persistent token lifecycle.
 */
import { describe, it, expect, beforeEach } from "vitest";
import {
  createClient,
  resetEmulator,
  uniqueLogin,
  mgmtCreateTestUser,
  mgmtGenerateMagicLink,
  mgmtAuth,
} from "../helpers/sdk";

const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";
const H = { "Content-Type": "application/json", Authorization: mgmtAuth };

beforeEach(() => resetEmulator());

// ─── Full CLI auth journey (magic link as PKCE proxy) ─────────────────────────

describe("Workflow 6 — CLI Auth: full login → use → refresh → logout lifecycle", () => {
  it("full journey: create dev user → simulate browser auth → receive tokens → use API → refresh → logout → revoked", async () => {
    const sdk = createClient();
    const login = uniqueLogin("wf6-cli-dev");

    // ADMIN CONFIG: user has a Descope developer account
    await mgmtCreateTestUser(login);

    // CLI ACTION: "descope login" — generates an auth link (magic link = PKCE callback proxy)
    const token = await mgmtGenerateMagicLink(login);
    expect(typeof token).toBe("string");

    // CLI CALLBACK: browser redirects back to localhost:PORT/callback with token
    // SDK processes the token (equivalent to exchanging the OAuth code)
    const verifyRes = await sdk.magicLink.verify(token);
    expect(verifyRes.ok).toBe(true);
    const { sessionJwt, refreshJwt } = verifyRes.data!;

    // CLI "persists" refresh token to disk (simulated by variable)
    const storedRefreshJwt = refreshJwt!;

    // CLI uses sessionJwt to call protected API
    const validateRes = await fetch(`${BASE_URL}/v1/auth/validate`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ sessionJwt }),
    });
    expect(validateRes.ok).toBe(true);
    const validateBody = await validateRes.json() as { token: Record<string, unknown> };
    expect(typeof validateBody.token.sub).toBe("string");

    // CLI uses stored token to fetch identity (e.g., for `descope whoami`)
    const meRes = await sdk.me(storedRefreshJwt);
    expect(meRes.ok).toBe(true);
    expect(meRes.data?.user?.loginIds).toContain(login);

    // CLI silently refreshes session (called automatically when sessionJwt nears expiry)
    const refreshRes = await sdk.refresh(storedRefreshJwt);
    expect(refreshRes.ok).toBe(true);
    const newSessionJwt = refreshRes.data!.sessionJwt;
    expect(newSessionJwt.split(".").length).toBe(3);
    // NOTE: the emulator issues JWTs with second-precision iat; if refresh happens
    // in the same second as sign-in, the JWT may be identical — that's valid behavior.

    // CLI `logout` command — user explicitly signs out
    const logoutRes = await sdk.logout(storedRefreshJwt);
    expect(logoutRes.ok).toBe(true);

    // Post-logout: stored token no longer works
    const postLogoutMe = await sdk.me(storedRefreshJwt);
    expect(postLogoutMe.ok).toBe(false);

    const postLogoutRefresh = await sdk.refresh(storedRefreshJwt);
    expect(postLogoutRefresh.ok).toBe(false);
    expect(postLogoutRefresh.code).toBe(401);
  });
});

// ─── Re-authentication after logout ──────────────────────────────────────────

describe("Workflow 6 — CLI Auth: re-authentication after logout", () => {
  it("user can re-login after explicit logout and get a fresh valid session", async () => {
    const sdk = createClient();
    const login = uniqueLogin("wf6-cli-reauth");

    await mgmtCreateTestUser(login);

    // First session
    const token1 = await mgmtGenerateMagicLink(login);
    const session1 = await sdk.magicLink.verify(token1);
    expect(session1.ok).toBe(true);

    await sdk.logout(session1.data!.refreshJwt!);

    // Wait 1.1s — the emulator uses second-precision iat for revocation checks.
    // A new session issued within the same second as logout would be incorrectly
    // rejected. This is a known timing invariant, not a test flake.
    await new Promise(r => setTimeout(r, 1100));

    // Re-login generates a new token (new browser auth)
    const token2 = await mgmtGenerateMagicLink(login);
    // token2 must be different from token1 (single-use)
    expect(token2).not.toBe(token1);
    const session2 = await sdk.magicLink.verify(token2);
    expect(session2.ok).toBe(true);

    // New session is valid — verify by refreshing it
    const newRefresh = await sdk.refresh(session2.data!.refreshJwt!);
    expect(newRefresh.ok).toBe(true);

    // Old session is revoked (the logout happened before token2 was issued)
    const oldRefresh = await sdk.refresh(session1.data!.refreshJwt!);
    expect(oldRefresh.ok).toBe(false);
  });
});

// ─── Enchanted link (cross-device CLI flow) ───────────────────────────────────

describe("Workflow 6 — CLI Auth: enchanted link (cross-device flow)", () => {
  it("admin-generated enchanted link token can be consumed once", async () => {
    const login = uniqueLogin("wf6-enchanted");

    // Create test user
    await fetch(`${BASE_URL}/v1/mgmt/user/create/test`, {
      method: "POST",
      headers: H,
      body: JSON.stringify({ loginId: login, email: login }),
    });

    // ADMIN generates enchanted link (cross-device: user scans QR on phone)
    const enchantedRes = await fetch(`${BASE_URL}/v1/mgmt/tests/generate/enchantedlink`, {
      method: "POST",
      headers: H,
      body: JSON.stringify({ loginId: login, uri: "http://localhost/cli-callback" }),
    });
    expect(enchantedRes.ok).toBe(true);
    const { token } = await enchantedRes.json() as { token: string };

    // Token is consumable via magic link verify
    const verifyRes = await fetch(`${BASE_URL}/v1/auth/magiclink/verify`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ token }),
    });
    expect(verifyRes.status).toBe(200);
    const { sessionJwt } = await verifyRes.json() as { sessionJwt: string };
    expect(sessionJwt.split(".").length).toBe(3);

    // Enchanted link is single-use — second verify fails
    const secondVerify = await fetch(`${BASE_URL}/v1/auth/magiclink/verify`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ token }),
    });
    expect(secondVerify.status).toBe(401);
  });
});

// ─── Embedded link (admin-issued session for CLI onboarding) ──────────────────

describe("Workflow 6 — CLI Auth: embedded link for admin-provisioned CLi access", () => {
  it("admin can generate embedded link for existing user; link produces valid session", async () => {
    const sdk = createClient();
    const login = uniqueLogin("wf6-embedded");

    // Admin creates the user (e.g., enterprise SSO provisioned via SCIM)
    await fetch(`${BASE_URL}/v1/mgmt/user/create`, {
      method: "POST",
      headers: H,
      body: JSON.stringify({ loginId: login, email: login }),
    });

    // Admin generates an embedded link (e.g., for initial CLI setup)
    const linkRes = await fetch(`${BASE_URL}/v1/mgmt/user/embeddedlink`, {
      method: "POST",
      headers: H,
      body: JSON.stringify({ loginId: login }),
    });
    expect(linkRes.ok).toBe(true);
    const { token } = await linkRes.json() as { token: string };

    // CLI callback: verify the link → get session
    const verifyRes = await sdk.magicLink.verify(token);
    expect(verifyRes.ok).toBe(true);
    expect(verifyRes.data?.sessionJwt.split(".").length).toBe(3);
    expect(verifyRes.data?.user?.loginIds).toContain(login);
  });
});
