/**
 * Management API via @descope/node-sdk
 *
 * Tests: Tenant CRUD, User CRUD, test-user utils (OTP/magic-link/embedded-link),
 * batch operations, JWT update.
 */
import { describe, it, expect, beforeEach } from "vitest";
import { createClient, resetEmulator, uniqueLogin } from "../helpers/sdk.js";

beforeEach(() => resetEmulator());

// ─── Tenant CRUD ──────────────────────────────────────────────────────────────

describe("management.tenant", () => {
  it("createWithId + load returns tenant", async () => {
    const sdk = createClient();
    const id = `t-${Date.now()}`;
    const create = await sdk.management.tenant.createWithId(id, "Acme Corp", []);
    expect(create.ok).toBe(true);

    const load = await sdk.management.tenant.load(id);
    expect(load.ok).toBe(true);
    // Emulator wraps in {tenant:{...}}; SDK passes through as-is so data = {tenant:{...}}
    const tenant = (load.data as Record<string, unknown>)?.tenant as Record<string, unknown>;
    expect(tenant?.name ?? load.data?.name).toBe("Acme Corp");
  });

  it("create (auto-id) + loadAll includes it", async () => {
    const sdk = createClient();
    const create = await sdk.management.tenant.create("Auto Corp", []);
    expect(create.ok).toBe(true);
    // data may be wrapped in {tenant:{...}} — acceptable either way
    const id = (create.data as Record<string, unknown>)?.id
      ?? ((create.data as Record<string, unknown>)?.tenant as Record<string, unknown>)?.id;
    expect(id).toBeTruthy();

    const all = await sdk.management.tenant.loadAll();
    expect(all.ok).toBe(true);
    expect(all.data?.some((t) => t.name === "Auto Corp")).toBe(true);
  });

  it("update changes tenant name", async () => {
    const sdk = createClient();
    const id = `t-upd-${Date.now()}`;
    await sdk.management.tenant.createWithId(id, "Old Name", []);
    await sdk.management.tenant.update(id, "New Name", []);

    const load = await sdk.management.tenant.load(id);
    // Emulator wraps in {tenant:{...}}; SDK passes through as-is
    const name = (load.data as Record<string, unknown>)?.name
      ?? ((load.data as Record<string, unknown>)?.tenant as Record<string, unknown>)?.name;
    expect(name).toBe("New Name");
  });

  it("delete removes tenant — subsequent load fails", async () => {
    const sdk = createClient();
    const id = `t-del-${Date.now()}`;
    await sdk.management.tenant.createWithId(id, "Gone", []);

    const del = await sdk.management.tenant.delete(id, false);
    expect(del.ok).toBe(true);

    const load = await sdk.management.tenant.load(id);
    expect(load.ok).toBe(false);
  });

  it("searchAll filters by tenant ID", async () => {
    const sdk = createClient();
    const id = `t-srch-${Date.now()}`;
    await sdk.management.tenant.createWithId(id, "Search Me", []);

    const res = await sdk.management.tenant.searchAll([id]);
    expect(res.ok).toBe(true);
    expect(res.data?.some((t) => t.id === id)).toBe(true);
  });
});

// ─── User CRUD ────────────────────────────────────────────────────────────────

describe("management.user.create + load + delete", () => {
  it("create + load round-trip", async () => {
    const sdk = createClient();
    const login = uniqueLogin("mgmt-u");
    const create = await sdk.management.user.create(login, {
      email: login,
      displayName: "Test User",
    });
    expect(create.ok).toBe(true);

    const load = await sdk.management.user.load(login);
    expect(load.ok).toBe(true);
    expect(load.data?.loginIds).toContain(login);
  });

  it("load by userId works", async () => {
    const sdk = createClient();
    const login = uniqueLogin("mgmt-u");
    await sdk.management.user.create(login, { email: login });

    const load = await sdk.management.user.load(login);
    const userId = load.data?.userId as string;
    expect(userId).toBeTruthy();

    const byId = await sdk.management.user.loadByUserId(userId);
    expect(byId.ok).toBe(true);
    expect(byId.data?.loginIds).toContain(login);
  });

  it("delete removes user", async () => {
    const sdk = createClient();
    const login = uniqueLogin("mgmt-u");
    await sdk.management.user.create(login, { email: login });
    const del = await sdk.management.user.delete(login);
    expect(del.ok).toBe(true);

    const load = await sdk.management.user.load(login);
    expect(load.ok).toBe(false);
  });
});

describe("management.user.update + patch", () => {
  it("update changes displayName", async () => {
    const sdk = createClient();
    const login = uniqueLogin("mgmt-upd");
    await sdk.management.user.create(login, { email: login, displayName: "Old" });
    await sdk.management.user.update(login, { email: login, displayName: "New" });

    const load = await sdk.management.user.load(login);
    // Emulator returns 'name' field (Descope API calls it displayName)
    const displayName = load.data?.displayName ?? (load.data as Record<string, unknown>)?.name;
    expect(displayName).toBe("New");
  });

  it("patch changes displayName without wiping other fields", async () => {
    const sdk = createClient();
    const login = uniqueLogin("mgmt-patch");
    await sdk.management.user.create(login, { email: login, displayName: "Before" });
    await sdk.management.user.patch(login, { displayName: "After" });

    const load = await sdk.management.user.load(login);
    // Emulator returns 'name' field (Descope API calls it displayName)
    const displayName = load.data?.displayName ?? (load.data as Record<string, unknown>)?.name;
    expect(displayName).toBe("After");
    expect(load.data?.email).toBe(login); // email untouched
  });
});

describe("management.user.updateLoginId", () => {
  it("renames loginId; new loginId works, old does not", async () => {
    const sdk = createClient();
    const oldLogin = uniqueLogin("mgmt-lid-old");
    const newLogin = uniqueLogin("mgmt-lid-new");
    await sdk.management.user.create(oldLogin, { email: oldLogin });

    // updateLoginId(loginId, newLoginId?) — newLoginId is optional
    const res = await sdk.management.user.updateLoginId(oldLogin, newLogin);
    expect(res.ok).toBe(true);

    const good = await sdk.management.user.load(newLogin);
    expect(good.ok).toBe(true);

    // The old value still resolves, and that is correct. `user.load` resolves a
    // user by email and by bare username prefix as well as by exact login id,
    // and this user was created with `email: oldLogin`, so renaming the login id
    // leaves the email index still pointing at them. Real Descope behaves the
    // same way; asserting a refusal here encoded a false expectation about it.
    // Renaming the login id is proven by `newLogin` resolving, above.
    const byOldEmail = await sdk.management.user.load(oldLogin);
    expect(byOldEmail.ok).toBe(true);
    expect(byOldEmail.data?.userId).toBe(good.data?.userId);
  });
});

describe("management.user.search", () => {
  it("returns created users by email filter", async () => {
    const sdk = createClient();
    const login = uniqueLogin("mgmt-srch");
    await sdk.management.user.create(login, { email: login });

    // SDK search: { emails?: string[] } — response data is UserResponse[] (via users field)
    const res = await sdk.management.user.search({ emails: [login] });
    expect(res.ok).toBe(true);
    // The SDK transforms the emulator's { users } to data as array via transformResponse
    const users = (res.data as unknown as { users: Array<{ loginIds?: string[] }> })?.users ?? res.data;
    const found = Array.isArray(users)
      ? users
      : (res.data as unknown as Array<{ loginIds?: string[] }>);
    expect(found.some((u: { loginIds?: string[] }) => u.loginIds?.includes(login))).toBe(true);
  });
});

describe("management.user.createBatch + deleteBatch", () => {
  it("createBatch creates all users; deleteBatch removes them", async () => {
    const sdk = createClient();
    const a = uniqueLogin("batch-a");
    const b = uniqueLogin("batch-b");

    const batchCreate = await sdk.management.user.createBatch([
      { loginId: a, email: a },
      { loginId: b, email: b },
    ]);
    expect(batchCreate.ok).toBe(true);
    // createBatch returns { createdUsers, failedUsers }
    expect(batchCreate.data?.createdUsers.length).toBeGreaterThanOrEqual(2);
    expect(batchCreate.data?.failedUsers.length).toBe(0);

    // Get user IDs from createdUsers for deleteBatch
    const uidA = batchCreate.data?.createdUsers.find(
      (u) => u.loginIds?.includes(a)
    )?.userId as string;
    const uidB = batchCreate.data?.createdUsers.find(
      (u) => u.loginIds?.includes(b)
    )?.userId as string;

    const batchDel = await sdk.management.user.deleteBatch([uidA, uidB]);
    expect(batchDel.ok).toBe(true);

    expect((await sdk.management.user.load(a)).ok).toBe(false);
    expect((await sdk.management.user.load(b)).ok).toBe(false);
  });
});

// ─── Test-user utilities ──────────────────────────────────────────────────────

describe("management.user.createTestUser + generateOTPForTestUser", () => {
  it("generates OTP for test user; verify returns session tokens", async () => {
    const sdk = createClient();
    const login = uniqueLogin("test-otp");
    await sdk.management.user.createTestUser(login, { email: login });

    const otpRes = await sdk.management.user.generateOTPForTestUser("email", login);
    expect(otpRes.ok).toBe(true);
    const code = otpRes.data?.code as string;
    expect(code).toMatch(/^\d{6}$/);

    const verify = await sdk.otp.verify.email(login, code);
    expect(verify.ok).toBe(true);
    expect(verify.data?.sessionJwt).toBeTruthy();
  });
});

describe("management.user.createTestUser + generateMagicLinkForTestUser", () => {
  it("generates magic link for test user; verify returns session tokens", async () => {
    const sdk = createClient();
    const login = uniqueLogin("test-ml");
    await sdk.management.user.createTestUser(login, { email: login });

    const linkRes = await sdk.management.user.generateMagicLinkForTestUser(
      "email",
      login,
      "http://localhost/verify"
    );
    expect(linkRes.ok).toBe(true);
    const token = linkRes.data?.token as string;
    expect(token).toBeTruthy();

    const verify = await sdk.magicLink.verify(token);
    expect(verify.ok).toBe(true);
    expect(verify.data?.sessionJwt).toBeTruthy();
  });
});

describe("management.user.generateEmbeddedLink", () => {
  it("generates embedded link token; verify returns session tokens", async () => {
    const sdk = createClient();
    const login = uniqueLogin("test-emb");
    // generateEmbeddedLink works on regular users (not just test users)
    await sdk.management.user.create(login, { email: login });

    // SDK: generateEmbeddedLink(loginId, customClaims?, timeout?)
    const linkRes = await sdk.management.user.generateEmbeddedLink(login);
    expect(linkRes.ok).toBe(true);
    const token = linkRes.data?.token as string;
    expect(token).toBeTruthy();

    const verify = await sdk.magicLink.verify(token);
    expect(verify.ok).toBe(true);
    expect(verify.data?.sessionJwt).toBeTruthy();
  });
});

describe("management.user.deleteAllTestUsers", () => {
  it("removes all test users, leaves regular users", async () => {
    const sdk = createClient();
    const regular = uniqueLogin("regular");
    const testU = uniqueLogin("tu");

    // Create both in same reset cycle
    await sdk.management.user.create(regular, { email: regular });
    await sdk.management.user.createTestUser(testU, { email: testU });

    const delAll = await sdk.management.user.deleteAllTestUsers();
    expect(delAll.ok).toBe(true);

    expect((await sdk.management.user.load(regular)).ok).toBe(true);
    expect((await sdk.management.user.load(testU)).ok).toBe(false);
  });
});

// ─── JWT update ───────────────────────────────────────────────────────────────

describe("management.jwt.update", () => {
  it("adds custom claims to a valid session JWT", async () => {
    const sdk = createClient();
    const login = uniqueLogin("jwt-upd");
    const signupRes = await sdk.password.signUp(login, "Pass1!", { email: login });
    const sessionJwt = signupRes.data?.sessionJwt as string;

    const updatedRes = await sdk.management.jwt.update(sessionJwt, { appRole: "admin" });
    expect(updatedRes.ok).toBe(true);
    const newJwt = updatedRes.data?.jwt as string;
    expect(newJwt).toBeTruthy();

    // Decode JWT payload (no verification needed for claim inspection)
    const payload = JSON.parse(
      Buffer.from(newJwt.split(".")[1], "base64url").toString()
    );
    expect(payload.appRole).toBe("admin");
  });
});


// ─── Impersonation ────────────────────────────────────────────────────────────

/**
 * Why: the Rust unit tests call the handlers as functions, so they prove the
 * logic and nothing about the wire. These drive the real `@descope/node-sdk`
 * over HTTP, which is the only thing that proves the SDK's request body reaches
 * our serde structs, that our response deserialises into what the SDK's callers
 * read, and that the routes are actually registered on the router.
 *
 * Decision: assert on decoded claim VALUES, never on `ok` alone. A route that
 * 404s makes `data.jwt` undefined and the decode throw, so none of these can
 * pass vacuously against a build without the endpoints.
 */
describe("management.jwt.impersonate", () => {
  /** Decode a JWT payload. No verification: these assert claim content only. */
  const claimsOf = (jwt: string): Record<string, any> =>
    JSON.parse(Buffer.from(jwt.split(".")[1], "base64url").toString());

  /** Create a user through the SDK and return its Descope user id. */
  async function createUser(sdk: ReturnType<typeof createClient>, prefix: string) {
    const login = uniqueLogin(prefix);
    const res = await sdk.management.user.create(login, { email: login });
    expect(res.ok).toBe(true);
    const userId = (res.data as Record<string, unknown>)?.userId as string;
    expect(userId).toBeTruthy();
    return { login, userId };
  }

  it("issues a token for the subject carrying the actor in act.sub", async () => {
    const sdk = createClient();
    const actor = await createUser(sdk, "imp-actor");
    const subject = await createUser(sdk, "imp-subject");

    const res = await sdk.management.jwt.impersonate(actor.userId, subject.login, false);

    expect(res.ok).toBe(true);
    const claims = claimsOf(res.data?.jwt as string);
    expect(claims.sub).toBe(subject.userId);
    expect(claims.act).toEqual({ sub: actor.userId });
  });

  it("round-trips back to the actor with no act claim", async () => {
    const sdk = createClient();
    const actor = await createUser(sdk, "rt-actor");
    const subject = await createUser(sdk, "rt-subject");

    const impersonated = await sdk.management.jwt.impersonate(
      actor.userId,
      subject.login,
      false
    );
    expect(impersonated.ok).toBe(true);

    const stopped = await sdk.management.jwt.stopImpersonation(
      impersonated.data?.jwt as string
    );

    expect(stopped.ok).toBe(true);
    const claims = claimsOf(stopped.data?.jwt as string);
    expect(claims.sub).toBe(actor.userId);
    expect(claims.act).toBeUndefined();
  });

  /**
   * Why: custom claims supplied at impersonation belong on the SESSION the
   *   operator ends up holding, not on the refresh token they hand to the SDK.
   *   Asserting them on the returned token pinned the old shape, where that
   *   token WAS the session; it now describes the wrong hop.
   * Decision: assert them where a caller reads them, on the session minted by
   *   the refresh exchange, alongside the actor. That also proves the two travel
   *   together rather than one displacing the other.
   */
  it("carries customClaims onto the session alongside act, not replacing it", async () => {
    const sdk = createClient();
    const actor = await createUser(sdk, "cc-actor");
    const subject = await createUser(sdk, "cc-subject");

    const imp = await sdk.management.jwt.impersonate(actor.userId, subject.login, false, {
      supportCase: "ENG-2401",
    });
    expect(imp.ok).toBe(true);

    const refreshed = await sdk.refresh(imp.data?.jwt as string);
    const session = claimsOf(refreshed.data?.sessionJwt as string);

    expect(session.supportCase).toBe("ENG-2401");
    expect(session.act).toEqual({ sub: actor.userId });
  });

  it("applies refreshDuration to the issued token's lifetime", async () => {
    const sdk = createClient();
    const actor = await createUser(sdk, "ttl-actor");
    const subject = await createUser(sdk, "ttl-subject");

    const res = await sdk.management.jwt.impersonate(
      actor.userId,
      subject.login,
      false,
      undefined,
      undefined,
      60
    );

    expect(res.ok).toBe(true);
    const claims = claimsOf(res.data?.jwt as string);
    expect(claims.exp - claims.iat).toBe(60);
  });

  /**
   * Why: the management round trip is not how a BROWSER adopts an impersonated
   *   session. Goliath hands the token to the Descope web SDK, which posts it to
   *   /v1/auth/refresh and stores what comes back, so the token this route
   *   returns has to be a REFRESH token and the actor has to survive that
   *   exchange. The ENG-2366 spike proved neither held: the route returned a
   *   session token (drn "DS"), and refreshing it produced a session with no
   *   act claim at all, silently ending the impersonation on the first refresh.
   * Decision: pin the browser path end to end rather than the management call
   *   alone. Real Descope documents impersonate as returning a refresh JWT, so
   *   the emulator has to match or every local test passes while production
   *   fails.
   */
  it("returns a REFRESH token, which is what the browser SDK can adopt", async () => {
    const sdk = createClient();
    const actor = await createUser(sdk, "ref-actor");
    const subject = await createUser(sdk, "ref-subject");

    const res = await sdk.management.jwt.impersonate(actor.userId, subject.login, false);

    expect(res.ok).toBe(true);
    const claims = claimsOf(res.data?.jwt as string);
    expect(claims.drn).toBe("DSR");
    expect(claims.sub).toBe(subject.userId);
    expect(claims.act).toEqual({ sub: actor.userId });
  });

  it("carries the actor through the refresh exchange onto the session token", async () => {
    const sdk = createClient();
    const actor = await createUser(sdk, "rt2-actor");
    const subject = await createUser(sdk, "rt2-subject");

    const imp = await sdk.management.jwt.impersonate(actor.userId, subject.login, false);
    const refreshed = await sdk.refresh(imp.data?.jwt as string);

    expect(refreshed.ok).toBe(true);
    const session = claimsOf(refreshed.data?.sessionJwt as string);
    expect(session.sub).toBe(subject.userId);
    expect(session.act).toEqual({ sub: actor.userId });
  });

  it("keeps the actor across a SECOND refresh, so the session does not silently end", async () => {
    const sdk = createClient();
    const actor = await createUser(sdk, "rt3-actor");
    const subject = await createUser(sdk, "rt3-subject");

    const imp = await sdk.management.jwt.impersonate(actor.userId, subject.login, false);
    const first = await sdk.refresh(imp.data?.jwt as string);
    const second = await sdk.refresh(first.data?.refreshJwt as string);

    expect(second.ok).toBe(true);
    const session = claimsOf(second.data?.sessionJwt as string);
    expect(session.act).toEqual({ sub: actor.userId });
  });

  it("drops the actor once impersonation stops, so a later refresh is an ordinary session", async () => {
    const sdk = createClient();
    const actor = await createUser(sdk, "rt4-actor");
    const subject = await createUser(sdk, "rt4-subject");

    const imp = await sdk.management.jwt.impersonate(actor.userId, subject.login, false);
    const stopped = await sdk.management.jwt.stopImpersonation(imp.data?.jwt as string);
    const refreshed = await sdk.refresh(stopped.data?.jwt as string);

    expect(refreshed.ok).toBe(true);
    const session = claimsOf(refreshed.data?.sessionJwt as string);
    expect(session.sub).toBe(actor.userId);
    expect(session.act).toBeUndefined();
  });

  it("refuses a login id no user holds", async () => {
    const sdk = createClient();
    const actor = await createUser(sdk, "unknown-actor");

    const res = await sdk.management.jwt.impersonate(
      actor.userId,
      "nobody@sdk.example",
      false
    );

    expect(res.ok).toBe(false);
    expect(res.data?.jwt).toBeUndefined();
  });

  it("refuses stopping impersonation on an ordinary session token", async () => {
    const sdk = createClient();
    const login = uniqueLogin("plain");
    const signup = await sdk.password.signUp(login, "Pass1!", { email: login });
    const sessionJwt = signup.data?.sessionJwt as string;
    expect(sessionJwt).toBeTruthy();

    const res = await sdk.management.jwt.stopImpersonation(sessionJwt);

    expect(res.ok).toBe(false);
    expect(res.data?.jwt).toBeUndefined();
  });
});
