## Context

The emulator is an in-memory Rust HTTP server (Axum) that mimics the Descope auth API surface. It has a thin React/Vite UI shell (`/ui`) that currently shows one placeholder page. All state is stored across per-concern in-memory stores (`UserStore`, `TenantStore`, etc.) protected by `Arc<RwLock<_>>`. Config is injected at startup from environment variables and is currently immutable at runtime. There is no admin UI, no snapshot mechanism, and no way to configure auth-method behavior (e.g. OTP expiry, password policy) without code changes.

## Goals / Non-Goals

**Goals:**
- Mirror the Descope console UX for all config-bearing sections (auth methods, RBAC, JWT templates, connectors, users, tenants, project settings, access keys)
- Enforce all auth-method config at runtime (OTP expiry, lockout, password policy, magic link redirect, etc.)
- Apply JWT templates at token-generation time
- Invoke connectors when auth methods trigger notifications
- Validate management API requests against `AccessKeyStore` (multi-key, with expiry and IP restrictions)
- Full snapshot export/import (`GET` + `POST /emulator/snapshot`) — includes password hashes, SAML/OIDC secrets, RSA key pair optional; mirrors Descope clone semantics
- Admin UI served at `/ui`, Radix UI headless + vanilla CSS, React Router mirroring Descope console URL structure
- Playwright E2E suite (POM) with one smoke test: snapshot export → reset → import round-trip

**Non-Goals:**
- FGA (Fine-Grained Authorization / ReBAC) — UI tab shows a "not supported" placeholder; no Rust implementation
- Flows / Styles / Themes / Localization editor — emulator has no flow engine; these sections show placeholder
- Real email/SMS delivery — connector invocations are logged and stored but not actually sent (test mode override)
- Multi-project support — emulator remains single-project
- Persistence to disk — emulator remains in-memory; snapshot file is the user's persistence mechanism

## Decisions

**D1: Radix UI (headless) + vanilla CSS for the UI**
- Considered: shadcn/ui (Tailwind dependency), MUI (opinionated design), hand-rolled
- Chose Radix because it provides accessible, unstyled primitives (Dialog, Select, Tabs, Toast, etc.) without forcing a CSS framework. We style with vanilla CSS matching the project's existing stack rule.

**D2: Snapshot replaces RSA key pair by default; opt-out via flag**
- Descope clone semantics = full replacement including secrets. Snapshot includes the PEM-encoded key pair.
- On import, if `keys` field is present → replace `KeyManager`; if absent → keep current keys.
- Risk: imported JWTs (if any) from a different key pair become invalid. Acceptable — snapshot implies fresh state.

**D3: `AuthMethodConfig` is a single `Arc<RwLock<AuthMethodConfigStore>>` struct in `EmulatorState`**
- All auth route handlers read from it on each request (cheap lock, small struct).
- Runtime mutation via new `PUT /emulator/config/auth-methods` (called by admin UI save actions).
- Alternative considered: baking config into each handler via Arc<RwLock<T>> per method — rejected as too fragmented.

**D4: JWT template evaluation within `token_generator.rs`**
- `generate_session_jwt` gains an optional `template: Option<&JwtTemplate>` parameter.
- Template resolution: server looks up the "active" template (or default) from `JwtTemplateStore` before calling generator.
- Dynamic claims resolved against the `User` struct at call time; static claims are literal values.
- No DSL/expression evaluator in scope — dynamic mappings are field references only (e.g., `user.custom_attributes["plan"]`).

**D5: Connectors are invoked in-process via `reqwest` but responses are always treated as success in test mode**
- Emulator is a test tool. Connector calls are fire-and-forget by default (log request, don't block on response).
- A `connector_mode` config flag: `passthrough` (default, fire-and-forget) vs `strict` (fail auth if connector call fails).
- Connector secrets are stored in `ConnectorStore` encrypted at rest? No — emulator is local-only, plaintext acceptable.

**D6: AccessKeyStore validates management API keys; single-key `EmulatorConfig.management_key` remains as a bootstrap override**
- The existing `check_mgmt_auth` function is updated to check `AccessKeyStore` first, then fall back to the `EmulatorConfig` bootstrap key.
- This means the emulator is never fully locked out even if the `AccessKeyStore` is emptied.

**D7: Password hashes accepted as-is on snapshot import**
- Snapshot includes `_password_hash` (bcrypt string). On import, hashes are stored directly without re-hashing.
- Plaintext passwords in the existing seed format are still hashed on load.

**D8: Admin UI management endpoints live under `/v1/mgmt/*` (matching real Descope API shapes) where they exist**
- New config-only endpoints (auth method config, JWT templates, connectors, custom attributes) live under `/v1/mgmt/` with Descope-compatible shapes where defined, otherwise emulator-specific shapes.
- Emulator-specific endpoints (snapshot, OTP inspector) remain under `/emulator/*`.

## Risks / Trade-offs

- **`AuthMethodConfig` enforcement breadth** → Risk: adding runtime config reads to every auth handler increases handler complexity and test matrix significantly. Mitigation: introduce an `AuthPolicyGuard` helper struct that encapsulates all policy checks (expiry, retries, lockout) so handlers stay lean.
- **Connector invocation adds external HTTP calls** → Risk: tests that don't mock connectors may fail in CI. Mitigation: `connector_mode = passthrough` by default; integration tests always run in passthrough mode.
- **JWT template evaluation correctness** → Risk: complex claim mappings that don't match user struct fields silently produce empty claims. Mitigation: template validation on save (warn if referenced field doesn't exist on `User`).
- **Snapshot size** → Risk: large user stores produce huge snapshots. Not a practical concern for emulator-scale workloads (hundreds not millions of users).
- **Radix UI accessibility** → low risk; Radix is built for accessibility by default.
- **Playwright E2E flakiness** → Risk: UI tests against a local server are timing-sensitive. Mitigation: use Playwright's built-in `waitForResponse` and element-visibility assertions rather than arbitrary sleeps.
