## Context

The `apps/platform-tests` project has 11 scenario tests (1,117 lines) that validate emulator behavior using the Descope Node SDK. Only ~44 lines are emulator-specific: `beforeEach(reset)` in all files, `/emulator/otp/` fetches in 2 files, and `BASE_URL`/`MGMT_AUTH_HEADER` references in 4 files. The test logic itself — assertions, SDK calls, JWT inspection — is already target-agnostic.

The Descope Node SDK provides test user APIs (`createTestUser`, `generateOTPForTestUser`, `deleteAllTestUsers`) that work on both emulator and live Descope. The emulator implements all of these.

All emulator delete operations (users, roles, permissions, tenants) are unconditional — no ordering dependencies, no reference checking. This means cleanup is a flat loop, not a dependency graph.

## Goals / Non-Goals

**Goals:**
- Make the existing test suite runnable against both emulator and live Descope
- Discover behavioral gaps between emulator and live Descope
- Assert structural parity (response shapes, JWT claim keys), not just "both return ok"

**Non-Goals:**
- Creating a parallel test suite (refactor in place instead)
- Byte-level response comparison (UUIDs, timestamps will differ)
- Fixing discovered parity gaps (separate changes)
- Error code parity in v1 (catalog needed first — log for manual comparison during initial live runs)

## Decisions

### D1: Target switching and cleanup strategy

`helpers/platform.ts` reads `PARITY_TARGET` (`"emulator"` | `"live"`) to configure the SDK. The `cleanup()` function branches:

- **Emulator**: `POST /emulator/reset` (fast, resets everything)
- **Live**: `deleteAllTestUsers()` + flat loop of `try/catch` deletes for tracked resources (roles, permissions, tenants, JWT templates, regular users). No ordering required — emulator deletes are unconditional, and live Descope also handles dangling references gracefully (to be verified on first live run).

Live target requires `DESCOPE_PROJECT_ID` and `DESCOPE_MANAGEMENT_KEY` (required), `DESCOPE_BASE_URL` (optional, SDK defaults to Descope API), and `DESCOPE_PARITY_ACKNOWLEDGED=1` (safety guard since `deleteAllTestUsers()` is project-wide).

### D2: OTP and magic link code retrieval

Existing tests fetch codes from `/emulator/otp/{loginId}`. This is replaced with:

- `generateOTPForTestUser("email", loginId)` — works on both targets
- `generateMagicLinkForTestUser("email", loginId, uri)` — works on both targets

Tests that exercise OTP/magic link flows call the initiation endpoint (`otp.signUpOrIn.email()` / `magicLink.signUpOrIn.email()`) as a **separate, explicit assertion** — not hidden inside the generate helper. This tests the initiation path (user lookup, response shape) independently from code verification.

```
// Explicit in test body, not hidden in helper:
const initRes = await client.otp.signUpOrIn.email(loginId);
expect(initRes.ok).toBe(true);

// Then generate a usable code via mgmt API:
const otpRes = await client.management.user.generateOTPForTestUser("email", loginId);
const code = otpRes.data.code;

// Then verify:
const verifyRes = await client.otp.verify.email(loginId, code);
expect(verifyRes.ok).toBe(true);
```

### D3: Assertion philosophy — behavioral + structural

Tests assert on two levels:

1. **Behavioral**: `ok` status, JWT claim presence (`sub`, `exp`), error handling (wrong password → `ok: false`)
2. **Structural**: Response shape comparison — same top-level keys, same JWT claim keys, same value types. Not exact values.

A `assertMatchingKeys(actual, expected)` helper validates that two objects have the same key structure and value types without comparing values. This catches the real parity gaps: missing fields, differently-nested objects, strings-vs-numbers.

Strong passwords (`TEST_PASSWORD` constant, e.g., `"T3st!Parity#2026"`) used throughout to satisfy live Descope password policies.

JWT claim shorthand (`r`/`roles`, `t`/`tenants`, `p`/`permissions`) checked with fallback — tests check both keys since the target may use either form.

## Risks / Trade-offs

- **Live test flakiness** — Network latency and rate limits → generous timeouts (45s), sequential execution
- **Stale test users on live** — `deleteAllTestUsers()` in `beforeEach` clears prior leftovers; manual cleanup as fallback
- **Delete ordering on live may differ from emulator** — Emulator deletes are unconditional. Live Descope likely is too, but verify on first run. If any delete rejects due to references, add ordering only for that case.
- **Emulator parity gaps** — Tests may reveal emulator behaviors that don't match live. This is the point — discovered gaps become separate fix changes.
- **Error codes** — Not asserted in v1. Log error responses during first live run for manual catalog-building. Add assertions incrementally.

## Execution cadence

- **First run**: Manual, against a dedicated live Descope test project. Document all gaps found.
- **Ongoing**: Run live suite before each emulator release or when adding new emulator endpoints. Emulator-target runs remain part of normal dev (existing vitest config, with `reset()` fast path).
- **Owner**: Whoever is actively developing the emulator owns running the live suite when releasing.
