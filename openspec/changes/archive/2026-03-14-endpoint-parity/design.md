## Context

The emulator is a Rust/Axum in-memory service that mimics the Descope REST API for integration testing. It currently implements ~43/175 Descope endpoints (~25%). The `emulator-completeness` change (currently finishing integration tests in section 8) brings OTP auth, logoutAll, user status, and tenant role mutations to completion. This change picks up from that baseline and closes the remaining testable gaps needed for full release parity.

The existing architecture is well-established:
- **Handlers**: `src/routes/{auth,mgmt}/` — thin Axum handlers that delegate to the store
- **Store**: `src/store/` — in-memory structs with O(1) indices (`UserStore`, `TenantStore`, `OtpStore`, `TokenStore`)
- **State**: `EmulatorState` — `Arc`-wrapped stores shared across handlers
- **JWT**: `src/jwt/` — RSA-keyed token generation and validation
- **Server**: `src/server.rs` — all route registrations in one place

## Goals / Non-Goals

**Goals:**
- Add all Descope endpoints that can be meaningfully emulated without third-party infrastructure (IdP, OAuth provider, WebAuthn hardware, TOTP app)
- Every new endpoint gets a unit test in-handler (handler-level) and an integration test in the SDK test suite
- No breaking changes to existing endpoint behavior or response shapes
- SAML `/start` path kept as alias so existing tests don't break

**Non-Goals:**
- OAuth/Social login — requires external IdP redirect flow; no meaningful emulation possible
- WebAuthn/Passkeys — requires browser credential API; no meaningful emulation possible
- TOTP — requires time-based code generation and authenticator app pairing; out of scope
- Enchanted Link polling — the `/pending-session` long-poll pattern is infrastructure-heavy; skip
- Audit log — read-only history that the emulator doesn't populate; stub only if SDK requires it
- AuthZ (Fine-grained authorization) — separate system with its own schema; out of scope for auth emulator
- Access Keys — machine-to-machine auth; out of scope for user-facing emulator
- Roles/Permissions/Groups management APIs — too expansive; the `role_names` field on users is sufficient
- Flow management and project export/import — server-side orchestration; not emulatable

## Decisions

### 1. SSO generic (`/v1/auth/sso/*`) reuses SAML logic
**Decision**: `sso/authorize` is an alias for `saml/start`; `sso/exchange` is an alias for `saml/exchange`.  
**Rationale**: The emulator doesn't run a real IdP. Generic SSO in Descope is OIDC-based but the exchange step produces the same JWT response shape. Reusing the SAML code path means zero test surface duplication.  
**Alternative considered**: Separate `sso.rs` module — rejected because it adds complexity with no behavioral difference.

### 2. `signup-in` composite = load-or-create
**Decision**: For OTP and magic link `signup-in` endpoints, attempt `load(login_id)`; if not found, create the user. Then proceed with normal signin flow (generate OTP code or magic link token).  
**Rationale**: This exactly matches Descope's documented behavior. The existing `magic_link::signup_email` already implements this pattern — we generalize it.  
**Alternative considered**: Always create, catch duplicate error, fallback to signin — more roundtrips, same result.

### 3. Magic link SMS = same TokenStore, different masked field
**Decision**: SMS magic link variants use the same `TokenStore` and `TokenType::Magic` as email. Response returns `maskedPhone` instead of `maskedEmail`. No actual SMS sent.  
**Rationale**: Consistent with how OTP already handles SMS — the emulator never sends anything; it returns the token in the body.

### 4. `magiclink/update/email` now persists the email change
**Decision**: Fix the existing no-op — after validating the refresh JWT, patch the user's email in `UserStore` and add the new email to `login_ids`.  
**Rationale**: Identified as a 🔴 High priority gap in `docs/gap-analysis.md`. The current implementation validates the token but discards the update, making the endpoint useless for test flows that check the updated email via `me`.

### 5. Tenant CRUD added to `mgmt/tenant.rs` — `Tenant` type is already fully defined
**Decision**: Add `create`, `update`, `delete`, `load` (single), and `search` (filter by name/id) to the existing `tenant.rs` handler file. The `TenantStore` already has the data structures; we need store methods and route registrations.  
**Source**: Confirmed from `src/types.rs` — `Tenant` struct already has `id`, `name`, `self_provisioning_domains: Vec<String>`, and `auth_type: AuthType`. No new fields needed; `create` and `update` accept `{ id?, name, selfProvisioningDomains? }`.  
**Rationale**: Currently only `load_all` and internal `find_by_email`/`load` are implemented. Runtime tenant CRUD is needed for test setups that can't rely on a seed file.

### 6. Batch user operations use existing `UserStore::insert` / `remove_by_uid` in a loop
**Decision**: `create/batch` and `delete/batch` loop over the request array and call existing single-item store methods. Partial failure on batch create returns the first error; existing users are not rolled back. Both are registered as **POST** endpoints matching the Descope spec (`POST /v1/mgmt/user/create/batch`, `POST /v1/mgmt/user/delete/batch`).  
**Source**: Confirmed from Descope YAML — `delete/batch` uses `post` method, not `delete`.  
**Rationale**: Descope's own batch operations are non-transactional and use POST. Matches real behavior without needing transaction logic.

### 7. `password/policy` returns a static permissive policy
**Decision**: Respond with `{ "active": true, "minLength": 6, "maxLength": 128 }` regardless of configuration.  
**Rationale**: The emulator doesn't enforce password complexity. SDKs may call this on init to display UI hints. A static permissive policy lets those SDKs proceed without error.

### 8. `me/history` returns `{ "users": [] }`
**Decision**: Stub — always return an empty list.  
**Rationale**: The emulator has no login event history. SDKs that call this should receive a valid empty response, not a 404.

### 9. `tenant/select` re-issues tokens with `dct` claim
**Decision**: Validate the refresh JWT (from `Authorization: Bearer <projectId:refreshJwt>` header), extract `userId`, verify the user belongs to the requested tenant, then issue new session + refresh JWTs. The session JWT SHALL include `dct = tenantId` as a top-level claim alongside the standard user claims.  
**Source**: Confirmed from Descope YAML: `"Set the active tenant for the user's current session — new session token and refresh token with the dct claim on the JWT which shows the active selected tenant"`. Response is a standard `JWTResponse` (same shape as login).  
**Rationale**: The `dct` claim is the authoritative field; apps use it to determine active tenant. Must be added to `generate_session_jwt` as an optional extra-claims parameter.

### 10. `jwt/update` takes a session JWT and returns an updated session JWT
**Decision**: Accept `{ jwt, customClaims }` where `jwt` is the **session JWT** (not refresh). Validate it, decode the sub (`userId`), load the user, issue a new session JWT with `customClaims` merged into the standard claims. Returns `{ jwt: <newSessionJwt> }` (single field).  
**Source**: Confirmed from Descope YAML — request schema is `UpdateJWTRequest`, response is `managementv1.JWTResponse: { jwt: string }`. The description says "updates a JWT with custom claims" — the JWT in question is the session token.  
**Rationale**: Mirrors existing behavior. The management key auth means the caller is trusted to enrich tokens without needing a refresh JWT in scope.

### 11. User field update endpoints delegate to `UserStore::patch`
**Decision**: `update/name`, `update/phone`, `update/loginid`, `update/role/set`, `update/role/remove` all call `users.patch(login_id, UserPatch { ... })` with the specific field set. `update/loginid` also updates the `by_login_id` index.  
**Rationale**: `UserStore::patch` already handles partial updates correctly. Adding `update_login_id` as a dedicated store method is needed to handle the index re-keying.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| `update/loginid` leaves stale index entries if not carefully implemented | Add a dedicated `UserStore::update_login_id` method that removes the old key from `by_login_id` and inserts the new one atomically |
| Batch create partial failure leaves orphaned users | Document as intentional non-transactional behavior in code comments; matches Descope's own behavior |
| `delete/batch` was planned as `DELETE` with body | **Corrected**: Descope spec uses `POST /v1/mgmt/user/delete/batch` — registered as POST to match spec |
| `signup-in` race condition (read-then-write) | Acceptable — the emulator is single-threaded at the store level (uses `RwLock`); the write lock is held for the create step |
| SAML `/start` alias — SDK may only call `/authorize` | Both paths registered; existing tests use `/start` so they continue to pass |
| `tenant/select` JWT re-issue may produce a longer-lived token than expected | Tokens are issued with standard TTL from config — same as all other auth endpoints |

## Migration Plan

No migration needed — all changes are additive. The emulator is stateless between restarts; no stored data format changes.

1. Merge `emulator-completeness` first (currently in progress — section 8 tests)  
2. Apply this change on top  
3. Run `make test` (unit) + `make test-integration` (SDK integration)  
4. Update `docs/gap-analysis.md` to reflect new coverage

## Open Questions

- Should `enchanted-link` test generate (`/v1/mgmt/tests/generate/enchantedlink`) reuse the magic link `TokenType::Magic` or get its own `TokenType::Enchanted`? → Use `Magic` for simplicity; the SDK only cares about the token value, not its type metadata.
- Does the JS SDK call `/v1/auth/saml/authorize` or `/v1/auth/saml/start`? → Test both paths in integration tests; register both as aliases.
