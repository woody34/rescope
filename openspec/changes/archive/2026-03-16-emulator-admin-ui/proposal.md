## Why

The emulator today is a headless API — teams can seed users/tenants via a JSON file, but there is no way to inspect state, configure auth method behavior, or round-trip a full environment snapshot without writing curl commands. This change delivers a Descope-console-mirroring admin UI and a complete export/import snapshot mechanism so that developers can seed, inspect, and replay any production-like configuration in isolation.

## What Changes

- **New**: `ui/` Vite+React admin UI served at `/ui` — mirrors the Descope console UX across all sections
- **New**: `GET /emulator/snapshot` — exports full emulator state (users, tenants, roles, permissions, auth method config, JWT templates, connectors, custom attributes, access keys, project config) as a versioned JSON blob, including bcrypt password hashes
- **New**: `POST /emulator/snapshot` — imports a snapshot, replacing all emulator state (mirrors Descope clone semantics: replaces RSA keys if included, bcrypt hashes accepted as-is)
- **New**: `GET /emulator/otps` — lists all pending OTP codes and magic link tokens for the UI inspector
- **New**: `RoleStore` — role entities (name, description, permissions[], is_default, is_hidden)
- **New**: `PermissionStore` — permission entities (name, description)
- **New**: `AuthMethodConfig` store — per-method policy config that is enforced at runtime (OTP expiry, password strength, lockout, magic link redirect URL, etc.)
- **New**: `JwtTemplateStore` — named JWT templates with claim mappings, evaluated at token generation time
- **New**: `ConnectorStore` — connector definitions (HTTP, SMTP, Twilio, etc.) that are invoked when auth methods trigger them; secrets stripped on export
- **New**: `CustomAttributeStore` — user attribute schema definitions (display name, machine name, type, permissions)
- **New**: `AccessKeyStore` — management key entities (name, expiry, permitted IPs, role/tenant scoping); auth middleware validates against this store
- **Modified**: `TenantStore` — add SAML/OIDC config fields (entity ID, ACS URL, cert, OIDC discovery URL, client credentials, scopes, attribute mapping) and per-tenant session TTL overrides
- **Modified**: `EmulatorConfig` / `ProjectConfig` — promote session/refresh TTLs to runtime-mutable state
- **Modified**: `EmulatorState` — add all new stores; wire into `reset_stores()` and seed loading
- **Modified**: `seed.rs` — extend seed format to accept roles, permissions, auth method config, JWT templates alongside existing users/tenants
- **Modified**: `token_generator.rs` — evaluate active JWT template (if configured) at generation time
- **Modified**: auth route handlers — read `AuthMethodConfig` for expiry/retry/lockout enforcement
- **New**: Playwright E2E test suite for UI (`ui/e2e/`) with POM pattern; one smoke test covering snapshot export → reset → import round-trip

## Capabilities

### New Capabilities

- `snapshot-export-import`: Full emulator state round-trip — export all config + user data (including password hashes) to a single versioned JSON file; import replaces all state
- `auth-method-config`: Per-method policy configuration (OTP, magic link, enchanted link, embedded link, TOTP, passkeys, OAuth/social, SSO, passwords, security questions, recovery codes, device auth, nOTP) stored and enforced at runtime
- `rbac-store`: Role and permission entities with hierarchy (permission → role → user/tenant), runtime CRUD, reflected in JWT claims
- `jwt-templates`: Named JWT templates with dynamic/static claim mappings evaluated at token generation; template library presets (OIDC, AWS, Hasura, etc.)
- `connector-store`: Connector definitions (HTTP, SMTP, Twilio, Datadog, etc.) referenced by auth methods; invoked at runtime when methods trigger notifications
- `custom-attributes`: User attribute schema management (types, permissions) stored and surfaced in user create/edit forms
- `access-key-store`: Multi-key management API authentication; keys have expiry, IP CIDR restrictions, and role/tenant scoping
- `admin-ui`: React+Vite admin UI served at `/ui`, mirroring Descope console navigation and behavior for all sections above plus emulator-specific controls (snapshot, OTP inspector, reset)

### Modified Capabilities

- `auth-flows`: OTP, magic link, password, and SSO handlers now read `AuthMethodConfig` for expiry, retry limits, and lockout enforcement instead of hardcoded defaults
- `tenant-management`: `TenantStore` gains SAML entity ID, ACS URL, certificate, OIDC discovery URL, client credentials, scopes, attribute mapping, and per-tenant session/refresh TTL overrides

## Impact

- **Rust crates added**: `zip` (snapshot serialization), `bcrypt` already present, potentially `reqwest` (connector invocation)
- **New Rust files**: `src/store/role_store.rs`, `src/store/permission_store.rs`, `src/store/auth_method_config.rs`, `src/store/jwt_template_store.rs`, `src/store/connector_store.rs`, `src/store/custom_attribute_store.rs`, `src/store/access_key_store.rs`, `src/routes/emulator/snapshot.rs`, `src/routes/mgmt/roles.rs`, `src/routes/mgmt/permissions.rs`, `src/routes/mgmt/connectors.rs`, `src/routes/mgmt/access_keys.rs`
- **Modified Rust files**: `src/state.rs`, `src/types.rs`, `src/seed.rs`, `src/config.rs`, `src/routes/auth/otp.rs`, `src/routes/auth/magic_link.rs`, `src/routes/auth/password.rs`, `src/routes/auth/session.rs`, `src/routes/mgmt/tenant.rs`, `src/jwt/token_generator.rs`, `src/server.rs`
- **UI**: `ui/` Vite+React app — Radix UI (headless) + vanilla CSS; `ui/e2e/` Playwright suite with POM
- **Breaking for seed file users**: Seed file format extended; old format remains valid (new fields optional)
- **No breaking Descope SDK API changes** — all new endpoints are under `/emulator/*` or `/v1/mgmt/*`; existing endpoint signatures unchanged
- **Depends on**: `open-source-prep` change completing first (licensing, CI baseline must be in place)
