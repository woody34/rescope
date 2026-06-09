## 1. Rust — New Stores (Foundation)

- [x] 1.1 Create `src/store/permission_store.rs` — `Permission` struct (id, name, description), CRUD methods, uniqueness enforcement on name
- [x] 1.2 Create `src/store/role_store.rs` — `Role` struct (id, name, description, permissions: Vec<String>, is_default, is_hidden), CRUD methods, default-role query
- [x] 1.3 Create `src/store/auth_method_config.rs` — `AuthMethodConfig` struct with all 13 method sub-configs (OTP, magic link, enchanted link, embedded link, TOTP, passkeys, OAuth per-provider, SSO, passwords, security questions, recovery codes, device auth, nOTP); sane defaults matching Descope defaults
- [x] 1.4 Create `src/store/jwt_template_store.rs` — `JwtTemplate` struct (id, name, claims, active flag), CRUD, active-template query, built-in preset definitions
- [x] 1.5 Create `src/store/connector_store.rs` — `Connector` struct (id, name, type, config: JsonObject), CRUD
- [x] 1.6 Create `src/store/custom_attribute_store.rs` — `CustomAttribute` schema struct (name, machine_name, type, permissions), CRUD
- [x] 1.7 Create `src/store/access_key_store.rs` — `AccessKey` struct (id, name, key_hash, expires_at, permitted_ips, role_names, tenant_roles, status), CRUD, key-validation method
- [x] 1.8 Update `src/store/mod.rs` to re-export all new stores
- [x] 1.9 Unit tests for each new store (included in each store file, all passing)

## 2. Rust — EmulatorState & Config Wiring

- [x] 2.1 Update `src/state.rs` — add all 7 new stores to `EmulatorState`; wire into `reset_stores()` (auth method config NOT reset on reset — only user/tenant/token stores)
- [ ] 2.2 Update `src/state.rs` — add `project_config: Arc<RwLock<ProjectConfig>>` (mutable version of session/refresh TTLs, project name, approved domains)
- [x] 2.3 Update `src/types.rs` — extended `Tenant` with `SamlConfig`, `OidcConfig`, per-tenant TTL overrides, `enforce_sso`, `parent_tenant_id`
- [x] 2.4 Update `src/seed.rs` — extended `SeedFile` to accept `permissions`, `roles`, `auth_method_config`, `jwt_templates`, `connectors`; loads in dependency order
- [ ] 2.5 Update `src/config.rs` — promote immutable `EmulatorConfig` TTLs to initial values for the mutable `ProjectConfig`

## 3. Rust — Auth Middleware & Token Generation Changes

- [x] 3.1 Update `src/routes/mgmt/*.rs` — replace `check_mgmt_auth` with a function that checks `AccessKeyStore` first (expiry, IP allowlist), then falls back to `EmulatorConfig` bootstrap key
- [x] 3.2 Update `src/jwt/token_generator.rs` — `generate_session_jwt` gains optional `template: Option<&JwtTemplate>` param; evaluates dynamic/static claims and merges into payload before signing
- [x] 3.3 Create `src/auth_policy.rs` — `AuthPolicyGuard` helper: checks method enabled, validates expiry, checks retry/lockout state; add `LockoutStore` (in-memory map of loginId → failure count + timestamp)
- [x] 3.4 Update `src/routes/auth/otp.rs` — use `AuthPolicyGuard` for enabled check, expiry, and lockout
- [x] 3.5 Update `src/routes/auth/magic_link.rs` — use `AuthPolicyGuard` for enabled check and expiry
- [x] 3.6 Update `src/routes/auth/password.rs` — use `AuthPolicyGuard` for enabled check, enforce full password policy on signup/update, lockout on failures
- [x] 3.7 Update `src/routes/auth/session.rs` — read active JWT template from `JwtTemplateStore` at token generation time
- [x] 3.8 Unit tests for `AuthPolicyGuard` and all updated route handlers at ≥95% coverage

## 4. Rust — New Management API Routes

- [x] 4.1 Create `src/routes/mgmt/permissions.rs` — `POST /v1/mgmt/authz/permission`, `GET /v1/mgmt/authz/permission/all`, `POST /v1/mgmt/authz/permission/update`, `POST /v1/mgmt/authz/permission/delete`
- [x] 4.2 Create `src/routes/mgmt/roles.rs` — `POST /v1/mgmt/authz/role`, `GET /v1/mgmt/authz/role/all`, `POST /v1/mgmt/authz/role/update`, `POST /v1/mgmt/authz/role/delete`
- [x] 4.3 Create `src/routes/mgmt/auth_method_config.rs` — `GET /v1/mgmt/config/auth-methods`, `PUT /v1/mgmt/config/auth-methods`
- [x] 4.4 Create `src/routes/mgmt/jwt_templates.rs` — `POST /v1/mgmt/jwt/template`, `GET /v1/mgmt/jwt/template/all`, `POST /v1/mgmt/jwt/template/update`, `POST /v1/mgmt/jwt/template/delete`, `POST /v1/mgmt/jwt/template/set-active`, `GET /v1/mgmt/jwt/template/active`
- [x] 4.5 Create `src/routes/mgmt/connectors.rs` — `POST /v1/mgmt/connector`, `GET /v1/mgmt/connector/all`, `POST /v1/mgmt/connector/update`, `POST /v1/mgmt/connector/delete`
- [x] 4.6 Create `src/routes/mgmt/custom_attributes.rs` — `POST /v1/mgmt/user/attribute`, `GET /v1/mgmt/user/attribute/all`, `POST /v1/mgmt/user/attribute/delete`
- [x] 4.7 Create `src/routes/mgmt/access_keys.rs` — `POST /v1/mgmt/accesskey`, `GET /v1/mgmt/accesskey/all`, `POST /v1/mgmt/accesskey/update`, `POST /v1/mgmt/accesskey/delete`, `POST /v1/mgmt/accesskey/disable`
- [x] 4.8 Update `src/routes/mgmt/tenant.rs` — add SAML/OIDC config fields to create/update endpoints
- [x] 4.9 Register all new routes in `src/server.rs`
- [x] 4.10 Unit tests for all new route handlers (included in each route file, all passing)

## 5. Rust — Connector Invocation

- [x] 5.1 Add `reqwest` to `Cargo.toml` (async HTTP client)
- [x] 5.2 Create `src/connector/invoker.rs` — `invoke_connector(connector: &Connector, payload: JsonObject)` function; fire-and-forget in `passthrough` mode; logs failures at WARN level
- [x] 5.3 Add `connector_mode: ConnectorMode` (passthrough|strict) to `EmulatorConfig` from `DESCOPE_EMULATOR_CONNECTOR_MODE` env var; default: `passthrough`
- [x] 5.4 Update `src/routes/auth/otp.rs` — after storing OTP, invoke email/SMS connector if configured (non-blocking)
- [x] 5.5 Update `src/routes/auth/magic_link.rs` — after storing token, invoke email connector if configured
- [x] 5.6 Update `src/routes/auth/password.rs` — on password reset request, invoke reset connector if configured

## 6. Rust — Snapshot Export & Import

- [x] 6.1 Add `keys` field to snapshot — serialize RSA private + public key PEMs; on import, replace `KeyManager` if present
- [x] 6.2 Ensure `User._password_hash` included in snapshot serialization (override `skip_serializing`)
- [x] 6.3 Create `src/routes/emulator/snapshot.rs` — `GET /emulator/snapshot` (full export), `POST /emulator/snapshot` (import with full restore)
- [x] 6.4 Add `GET /emulator/otps` — returns map of userId → pending OTP code
- [x] 6.5 Register snapshot routes in `src/server.rs`
- [x] 6.6 Unit tests for snapshot round-trip, bcrypt hash preservation

## 7. Admin UI — Setup & Infrastructure

- [x] 7.1 Add Radix UI primitives to `ui/package.json`: `@radix-ui/react-dialog`, `@radix-ui/react-select`, `@radix-ui/react-tabs`, `@radix-ui/react-switch`, `@radix-ui/react-checkbox`, `@radix-ui/react-toast`, `@radix-ui/react-dropdown-menu`
- [x] 7.2 Create `ui/src/lib/api.ts` — typed fetch wrapper pointing to `window.location.origin` (emulator base URL); exports typed request functions for every management API endpoint
- [x] 7.3 Create `ui/src/components/` — shared base components: `Layout.tsx` (sidebar + topnav shell), `Sidebar.tsx`, `PageHeader.tsx`, `DataTable.tsx`, `FormField.tsx`, `Toast.tsx`, `ConfirmDialog.tsx`, `StatusBadge.tsx`
- [x] 7.4 Create `ui/src/index.css` — design tokens (colors matching Descope console: navy sidebar, white content, blue accents), typography (Inter font), layout utilities
- [x] 7.5 Update `ui/src/App.tsx` — configure React Router with all routes matching the emulator URL structure
- [x] 7.6 Set up Playwright: install `@playwright/test`, create `ui/playwright.config.ts` with `baseURL: http://localhost:4600/ui`, add `test:e2e` script to `ui/package.json`

## 8. Admin UI — Section Pages

- [x] 8.1 Create `ui/src/pages/auth-methods/` — one component per method (OtpSettings, MagicLinkSettings, EnchantedLinkSettings, EmbeddedLinkSettings, ToTpSettings, PasskeySettings, OAuthSettings, SsoSettings, PasswordSettings, SecurityQuestionsSettings, RecoveryCodesSettings, DeviceAuthSettings, NotpSettings); left nav within auth-methods for switching methods
- [x] 8.2 Create `ui/src/pages/authorization/` — `RbacPage.tsx` with Permissions tab and Roles tab; `AddPermissionDialog.tsx`, `AddRoleDialog.tsx`, `FgaPage.tsx` (placeholder)
- [x] 8.3 Create `ui/src/pages/users/` — `UsersPage.tsx` (table with search/filter/bulk actions), `CustomAttributesTab.tsx`, `CreateEditUserDrawer.tsx` (with login IDs, user attributes, tenant+role assignment)
- [x] 8.4 Create `ui/src/pages/tenants/` — `TenantsPage.tsx` (list), `TenantDetailLayout.tsx` (sidebar with 5 sub-routes), `TenantSettingsPage.tsx`, `TenantAuthPage.tsx` (SAML/OIDC radio + fields), `TenantAuthorizationPage.tsx`, `TenantUsersPage.tsx`, `TenantSubTenantsPage.tsx`
- [x] 8.5 Create `ui/src/pages/settings/` — `ProjectGeneralPage.tsx`, `SessionManagementPage.tsx`, `JwtTemplatesPage.tsx` (list + editor + template library picker)
- [x] 8.6 Create `ui/src/pages/access-keys/` — `AccessKeysPage.tsx` (list), `CreateKeyDialog.tsx` (shows cleartext key once)
- [x] 8.7 Create `ui/src/pages/connectors/` — `ConnectorsPage.tsx` (list by type), `CreateConnectorDialog.tsx` (type picker → type-specific fields)
- [x] 8.8 Create `ui/src/pages/emulator/` — `SnapshotPage.tsx` (export button, import drop zone, mode toggle), `OtpInspectorPage.tsx` (auto-refreshing OTP list), `ResetPage.tsx`

## 9. Admin UI — Playwright E2E (POM)

- [x] 9.1 Create `ui/e2e/pom/SnapshotPage.ts` — Page Object Model for `/ui/emulator/snapshot` (export(), importFile(path), getModeToggle())
- [x] 9.2 Create `ui/e2e/pom/UsersPage.ts` — Page Object Model for `/ui/users` (getUserCount(), getUserRow(loginId))
- [x] 9.3 Create `ui/e2e/smoke.test.ts` — Snapshot round-trip test: seed emulator with 1 user, export snapshot to temp file, POST `/emulator/reset`, assert users = 0, import snapshot from file, assert user is restored via `UsersPage.getUserRow()`

## 10. Testing & Polish

- [x] 10.1 Run `cargo test --lib` — confirm all unit tests pass at ≥95% library coverage
- [x] 10.2 Run `cargo clippy -- -D warnings` — fix all warnings
- [x] 10.3 Run `cargo fmt --check` — fix formatting
- [x] 10.4 Run existing Vitest integration tests (`make test-integration`) — confirm no regressions
- [ ] 10.5 Run `npm run test:e2e` from `ui/` — confirm Playwright smoke test passes
- [ ] 10.6 Update `docs/descope-admin-ui-spec.md` with any spec changes discovered during implementation
- [ ] 10.7 Update `CHANGELOG.md` with v0.2.0 entry covering all new capabilities
