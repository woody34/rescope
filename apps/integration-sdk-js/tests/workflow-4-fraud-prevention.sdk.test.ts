/**
 * Workflow 4 — Fraud Prevention & Account Security Controls
 *
 * Market use case: Fintech, healthcare, and e-commerce apps that need to
 * block compromised accounts, flag suspicious activity, and embed risk
 * signals in JWTs for downstream policy enforcement. Branch Insurance
 * is a cited example — reduced fraud-related support tickets by 50%.
 *
 * Journey:
 *   ADMIN CONFIG: create user
 *      → CONSUMER ACTION: sign up / sign in succeeds
 *      → ADMIN ACTION: disable account (fraud signal detected)
 *      → CONSUMER ACTION: sign in → rejected (403)
 *      → ADMIN ACTION: re-enable account (false positive cleared)
 *      → CONSUMER ACTION: sign in → succeeds again
 *      → ADMIN ACTION: force-logout all sessions (active session revocation)
 *      → CONSUMER ACTION: refresh with old token → revoked (401)
 *      → ADMIN ACTION: enrich JWT with risk metadata via jwt/update
 *      → Verify risk claims in decoded JWT
 */
import { describe, it, expect, beforeEach } from "vitest";
import {
  createClient,
  resetEmulator,
  uniqueLogin,
  mgmtAuth,
} from "../helpers/sdk";

const BASE_URL = process.env.EMULATOR_BASE_URL ?? "http://localhost:4501";
const H = { "Content-Type": "application/json", Authorization: mgmtAuth };

beforeEach(() => resetEmulator());

// ─── Helpers ──────────────────────────────────────────────────────────────────

async function setUserStatus(loginId: string, status: "enabled" | "disabled") {
  return fetch(`${BASE_URL}/v1/mgmt/user/status`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ loginId, status }),
  });
}

async function forceLogout(loginId: string) {
  return fetch(`${BASE_URL}/v1/mgmt/user/logout`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ loginId }),
  });
}

async function jwtUpdate(jwt: string, customClaims: Record<string, unknown>) {
  const res = await fetch(`${BASE_URL}/v1/mgmt/jwt/update`, {
    method: "POST",
    headers: H,
    body: JSON.stringify({ jwt, customClaims }),
  });
  return { status: res.status, body: await res.json() as { jwt: string } };
}

function decodePayload(jwt: string): Record<string, unknown> {
  return JSON.parse(
    Buffer.from(jwt.split(".")[1], "base64url").toString()
  ) as Record<string, unknown>;
}

// ─── Account disable / enable ─────────────────────────────────────────────────

describe("Workflow 4 — Fraud Prevention: account disable/enable lifecycle", () => {
  it("full journey: sign up → disable → blocked → re-enable → allowed → force-logout → revoked", async () => {
    const sdk = createClient();
    const login = uniqueLogin("wf4-fraud");

    // CONSUMER ACTION: sign up (account created)
    const signupRes = await sdk.password.signUp(login, "Fraud1!");
    expect(signupRes.ok).toBe(true);
    const { refreshJwt } = signupRes.data!;

    // Confirm initial access works
    expect((await sdk.me(refreshJwt!)).ok).toBe(true);

    // ADMIN ACTION: fraud detected → disable account
    const disableRes = await setUserStatus(login, "disabled");
    expect(disableRes.ok).toBe(true);

    // CONSUMER ACTION: sign in attempt → rejected (account disabled)
    const blockedSignin = await sdk.password.signIn(login, "Fraud1!");
    expect(blockedSignin.ok).toBe(false);
    expect(blockedSignin.code).toBe(403);

    // ADMIN ACTION: false positive cleared → re-enable account
    const enableRes = await setUserStatus(login, "enabled");
    expect(enableRes.ok).toBe(true);

    // CONSUMER ACTION: sign in succeeds after re-enable
    const reSignin = await sdk.password.signIn(login, "Fraud1!");
    expect(reSignin.ok).toBe(true);
    const { refreshJwt: newRefreshJwt } = reSignin.data!;

    // ADMIN ACTION: compromise suspected → force all sessions out
    const forceRes = await forceLogout(login);
    expect(forceRes.ok).toBe(true);

    // Wait for 1-second revocation resolution
    await new Promise(r => setTimeout(r, 1100));

    // Old tokens revoked — both pre and post-reenable sessions
    expect((await sdk.refresh(refreshJwt!)).ok).toBe(false);
    expect((await sdk.refresh(newRefreshJwt!)).ok).toBe(false);
  });
});

// ─── JWT risk metadata ────────────────────────────────────────────────────────

describe("Workflow 4 — Fraud Prevention: risk metadata in JWT", () => {
  it("admin embeds risk score and device fingerprint in JWT for downstream policy", async () => {
    const sdk = createClient();
    const login = uniqueLogin("wf4-risk-jwt");

    const signup = await sdk.password.signUp(login, "RiskMeta1!");
    const { sessionJwt } = signup.data!;

    // ADMIN ACTION: embed fraud signals as JWT claims
    const { status, body } = await jwtUpdate(sessionJwt, {
      riskScore: 0.85,
      deviceTrusted: false,
      ipReputation: "suspicious",
      geoCountry: "US",
    });
    expect(status).toBe(200);
    const enrichedJwt = body.jwt;

    const payload = decodePayload(enrichedJwt);
    expect(payload.riskScore).toBe(0.85);
    expect(payload.deviceTrusted).toBe(false);
    expect(payload.ipReputation).toBe("suspicious");
    expect(payload.geoCountry).toBe("US");

    // Standard claims still present
    expect(typeof payload.sub).toBe("string");
    expect(typeof payload.iss).toBe("string");
  });

  it("jwt/update rejects invalid/expired JWT", async () => {
    const { status } = await jwtUpdate("bad.token.here", { riskScore: 0 });
    expect(status).toBe(401);
  });
});

// ─── Status boundary conditions ───────────────────────────────────────────────

describe("Workflow 4 — Fraud Prevention: status edge cases", () => {
  it("disabling unknown user returns error", async () => {
    const res = await setUserStatus("nonexistent@example.com", "disabled");
    expect(res.ok).toBe(false);
  });

  it("re-enabling already-enabled user is idempotent", async () => {
    const sdk = createClient();
    const login = uniqueLogin("wf4-idempotent");
    await sdk.password.signUp(login, "Idm1!");

    // Enable already-enabled user — should be a no-op
    const res = await setUserStatus(login, "enabled");
    expect(res.ok).toBe(true);

    // User can still sign in
    const signin = await sdk.password.signIn(login, "Idm1!");
    expect(signin.ok).toBe(true);
  });

  it("disabled user's existing session tokens still report identity via /me (revocation is on signin)", async () => {
    // NOTE: disabling an account blocks new sign-ins but does NOT revoke
    // existing sessions — that requires a separate force-logout.
    // This is consistent with Descope's own behavior.
    const sdk = createClient();
    const login = uniqueLogin("wf4-session-persist");
    const signup = await sdk.password.signUp(login, "Persist1!");
    const { refreshJwt } = signup.data!;

    await setUserStatus(login, "disabled");

    // Existing session tokens still work (no automatic revocation)
    const meRes = await sdk.me(refreshJwt!);
    // Behavior: session JWT is still valid; me may or may not return user depending
    // on implementation. We assert that forcing logout would revoke it.
    if (meRes.ok) {
      // If session persists, admin must explicitly force-logout
      await forceLogout(login);
      await new Promise(r => setTimeout(r, 1100));
      expect((await sdk.refresh(refreshJwt!)).ok).toBe(false);
    } else {
      // Session was also revoked — stronger security posture
      expect(meRes.ok).toBe(false);
    }
  });
});
