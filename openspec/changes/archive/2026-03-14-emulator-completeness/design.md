## Context

The emulator has a complete auth and management foundation from the Phase 1 build. This design covers three domains: OTP authentication flows, user status management, and tenant membership mutations. All work is inside the existing Rust binary — no new services, no new dependencies. The TDD methodology from Phase 1 (spec → failing test → implement → passing) applies to every task.

All API shapes in this document were verified against the Descope Node SDK source at `github.com/descope/node-sdk`.

## Goals / Non-Goals

**Goals:**

- Implement OTP sign-up, sign-in, verify for email and SMS channels
- Implement `GET /emulator/otp/:loginId` escape hatch for retrieving codes in SDK-driven test flows
- Implement `POST /v1/mgmt/tests/generate/otp` for test-user OTP automation
- Implement `POST /v1/auth/logoutall` for full user session revocation
- Add `disabled` to `User.status` field to block sign-in (matches real Descope `UserStatus` type)
- Implement `user/status` management endpoint (single endpoint, not separate enable/disable)
- Implement `tenant/remove` and `tenant/setRole` mutations
- Maintain 95%+ coverage gate throughout

**Non-Goals:**

- OTP voice channel (`/phone/voice` endpoints)
- OTP update flows (`update/email`, `update/phone/voice`) — `update/phone/sms` already exists
- TOTP / authenticator, Enchanted Link, WebAuthn — out of scope

## Decisions

---

### Decision 1: Status endpoint — single `POST /v1/mgmt/user/status` with `status` field

**Verified from SDK source** (`manage/user.ts` chunk 4):

```ts
activate: (loginId) =>
  httpClient.post(apiPaths.user.updateStatus, { loginId, status: "enabled" });
deactivate: (loginId) =>
  httpClient.post(apiPaths.user.updateStatus, { loginId, status: "disabled" });
```

Both SDK methods hit the **same endpoint** with a `status` string. The response is `{ user: UserResponse }`.

**Decision**: One handler at `POST /v1/mgmt/user/status`. Request body: `{ "loginId": "<id>", "status": "enabled" | "disabled" }`. Response: `{ "user": UserResponse }`.

**`User.status` field** already exists as `String` in `types.rs`. We do NOT add a separate `disabled: bool`. Instead:

- `status: String` is the source of truth
- All auth-issuing handlers check `user.status == "disabled"` → `403`
- `UserStore::enable(login_id)` sets `user.status = "enabled"`
- `UserStore::disable(login_id)` sets `user.status = "disabled"`

---

### Decision 2: Tenant mutations — `tenant/remove` and `tenant/setRole`

**Verified from SDK source**:

```ts
removeTenant: (loginId, tenantId)              → POST user.removeTenant, { loginId, tenantId }          → { user }
setTenantRoles: (loginId, tenantId, roles)     → POST user.setRole,       { loginId, tenantId, roleNames } → { user }
```

Note: `setTenantRoles` reuses the same `setRole` path as global `setRoles`, distinguished by the presence of `tenantId`. The emulator already has `POST /v1/mgmt/user/update/role` for global roles. The tenant-scoped `setRole` hits `POST /v1/mgmt/user/tenant/setRole`.

- **`tenant/remove`**: Removes the `UserTenant` entry where `tenant_id == tenantId`. Idempotent — if not found, return current user state (`200 OK`).
- **`tenant/setRole`**: Finds the `UserTenant` entry, replaces `role_names`. Returns `404` if user is not in that tenant.

---

### Decision 3: `logoutAll` — per-user revocation timestamp in `EmulatorState`

New field in `EmulatorState`: `user_revocations: Arc<RwLock<HashMap<String, u64>>>`.

`logoutAll` handler:

1. Extracts and validates refresh JWT (same multi-source extraction as `logout`)
2. Records `state.user_revocations.write().await.insert(user_id, now_secs())`
3. Returns `{ "ok": true }`

`refresh` and `me` validation — additional check after string-level revocation:

```rust
let revoked_at = state.user_revocations.read().await
    .get(&user_id).copied().unwrap_or(0);
if claims.iat <= revoked_at {
    return Err(EmulatorError::TokenExpired);
}
```

`RefreshClaims` already has `iat: u64` — confirmed in `types.rs`. No JWT changes needed.

`reset_stores()` in `state.rs` must also clear `user_revocations`.

---

### Decision 4: OTP storage — separate `OtpStore`

A dedicated `OtpStore` with peek-not-consume semantics, keyed by `userId`:

```rust
pub struct OtpStore {
    // userId → 6-digit code string
    pending: HashMap<String, String>,
}

impl OtpStore {
    pub fn store(&mut self, user_id: &str, code: String)
    pub fn peek(&self, user_id: &str) -> Option<&str>        // non-destructive
    pub fn consume(&mut self, user_id: &str, code: &str) -> Result<(), EmulatorError>
    pub fn reset(&mut self)
}
```

`consume` verifies the code matches and removes it. Returns `EmulatorError::InvalidToken` on mismatch or absence.

Added to `EmulatorState` as `otps: Arc<RwLock<OtpStore>>`.

---

### Decision 5: OTP signup — `loginId` sets both `login_ids[0]` and `email`

The Descope OTP signup for email uses the loginId as both the login identifier and the email channel. Confirmed from Descope docs: "the email address provided acts as the `loginId` and is also the channel through which the OTP will be sent."

Created user will have:

- `login_ids: [loginId]`
- `email: Some(loginId.clone())`
- `verified_email: false` (requires successful verify to be set true)
- `status: "enabled"`

After successful `otp/verify`, set `verified_email: true` on the user.

---

### Decision 6: `generate/otp` request and response shape

**Verified from SDK source** (chunk 6):

```ts
generateOTPForTestUser: (deliveryMethod, loginId, loginOptions?) =>
  httpClient.post(apiPaths.user.generateOTPForTest, { deliveryMethod, loginId, loginOptions })
  → GenerateOTPForTestResponse  // { code, loginId }
```

Request: `POST /v1/mgmt/tests/generate/otp` with `{ "deliveryMethod": "email" | "sms", "loginId": "<id>" }`  
Response: `{ "code": "<6-digit>", "loginId": "<id>" }`

No `maskedEmail` in this response — that matches the real SDK type `GenerateOTPForTestResponse`.

---

### Decision 7: No OTP code TTL

No expiry. Consistent with how magic link tokens work in the emulator. OtpStore uses the same count-eviction philosophy — a pending code is overwritten if the user requests a new one (store on the same `userId` key).

---

## Risks / Trade-offs

- **`logoutAll` timestamp precision** → `iat` is epoch-seconds. If a token is issued in the same second as `logoutAll` is called, `iat == revoked_at` → rejected (the check is `<=`). Acceptable for an emulator.
- **OTP `consume` with wrong code** → Returns `InvalidToken`. Handler must NOT reveal whether the userId exists — always return same 401.
- **Coverage gate** → All new handlers need unit tests before the integration test layer. `OtpStore` is simple enough for full unit test coverage. The disabled-user check must be added to every token-issuing handler, each requiring a new test case.
