# Descope Emulator — Gap Analysis

_Updated: 2026-03-14 (post endpoint-parity change)_

This document crosswalks every Descope REST API feature area against what the emulator currently implements.

---

## Legend

| Symbol | Meaning                                             |
| ------ | --------------------------------------------------- |
| ✅     | Fully implemented                                   |
| ⚠️     | Partially implemented (see notes)                   |
| ❌     | Not implemented                                     |
| 🔧     | Emulator-specific extension (no Descope equivalent) |

---

## 1. Infrastructure & JWKS

| Endpoint                     | Emulator | Notes                                               |
| ---------------------------- | -------- | --------------------------------------------------- |
| `GET /.well-known/jwks.json` | ✅       | RSA public key served for SDK JWT verification      |
| `GET /v2/keys/:project_id`   | ✅       | Alternate JWKS path used by some SDK versions       |
| `GET /health`                | 🔧       | Emulator-specific; not a Descope API endpoint       |
| `POST /emulator/reset`       | 🔧       | Emulator-specific; clears state and re-applies seed |

---

## 2. Auth — Password

| Endpoint                         | Emulator | Notes                                                                          |
| -------------------------------- | -------- | ------------------------------------------------------------------------------ |
| `POST /v1/auth/password/signup`  | ✅       | Creates user + bcrypt hash; returns session & refresh JWTs                     |
| `POST /v1/auth/password/signin`  | ✅       | Verifies bcrypt hash; returns session & refresh JWTs                           |
| `POST /v1/auth/password/replace` | ✅       | Old password required; issues new tokens on success                            |
| `POST /v1/auth/password/reset`   | ✅       | No email sent; **returns token in body** (test convenience — Descope does not) |
| `POST /v1/auth/password/update`  | ✅       | Consumes reset token from header or body                                       |
| `GET /v1/auth/password/policy`   | ✅       | Returns static permissive policy (no real enforcement)                         |

---

## 3. Auth — Magic Link

| Endpoint                                        | Emulator | Notes                                                              |
| ----------------------------------------------- | -------- | ------------------------------------------------------------------ |
| `POST /v1/auth/magiclink/signin/email`          | ✅       | **Returns token in body** (test convenience — Descope does not)    |
| `POST /v1/auth/magiclink/signup/email`          | ✅       | Load-or-create user; **returns token in body**                     |
| `POST /v1/auth/magiclink/signin/phone/sms`     | ✅       | Load-or-create by phone; **returns token in body**                 |
| `POST /v1/auth/magiclink/signup/phone/sms`     | ✅       | Same as signin — load-or-create pattern                            |
| `POST /v1/auth/magiclink/verify`                | ✅       | Single-use token; returns session + refresh JWTs                   |
| `POST /v1/auth/magiclink/update/email`          | ✅       | Updates email on user record; returns new tokens                   |
| `POST /v1/auth/magiclink/update/phone/sms`     | ✅       | Updates phone on user record                                       |
| `POST /v1/auth/magiclink/signin/phone/whatsapp` | ❌       | WhatsApp — not implemented                                         |
| `POST /v1/auth/magiclink/update/phone/whatsapp` | ❌       | Update phone via WhatsApp — not implemented                        |

---

## 4. Auth — OTP

| Endpoint                               | Emulator | Notes                                                          |
| -------------------------------------- | -------- | -------------------------------------------------------------- |
| `POST /v1/auth/otp/signup/email`       | ✅       | Load-or-create; **returns code in body**                       |
| `POST /v1/auth/otp/signup/phone/sms`   | ✅       | Load-or-create by phone; **returns code in body**              |
| `POST /v1/auth/otp/signin/email`       | ✅       | Same as signup (load-or-create)                                |
| `POST /v1/auth/otp/signin/phone/sms`   | ✅       | Same as signup by phone                                        |
| `POST /v1/auth/otp/verify/email`       | ✅       | Verifies code; returns session + refresh JWTs                  |
| `POST /v1/auth/otp/verify/phone/sms`   | ✅       | Verifies code; returns session + refresh JWTs                  |
| `POST /v1/auth/otp/update/email`       | ✅       | Updates email on user record                                   |
| `POST /v1/auth/otp/update/phone/sms`   | ✅       | Updates phone field; no SMS sent                               |
| `POST /v1/auth/otp/signup/phone/voice` | ❌       | OTP voice — not implemented                                    |
| `POST /v1/auth/otp/signin/phone/voice` | ❌       | OTP voice — not implemented                                    |
| `POST /v1/auth/otp/update/phone/voice` | ❌       | OTP voice phone update — not implemented                       |

---

## 5. Auth — SAML / SSO

| Endpoint                      | Emulator | Notes                                                    |
| ----------------------------- | -------- | -------------------------------------------------------- |
| `POST /v1/auth/saml/start`    | ✅       | Returns `?code=` URL; skips real IdP redirect            |
| `POST /v1/auth/saml/exchange` | ✅       | Exchanges code for session + refresh JWTs                |
| `POST /v1/auth/sso/start`     | ✅       | Alias to SAML start (generic SSO)                        |
| `POST /v1/auth/sso/exchange`  | ✅       | Alias to SAML exchange (generic SSO)                     |

---

## 6. Auth — OAuth (Social Login)

| Endpoint                       | Emulator | Notes                                            |
| ------------------------------ | -------- | ------------------------------------------------ |
| `GET /v1/auth/oauth/authorize` | ❌       | Redirect to OAuth provider — not implemented     |
| `POST /v1/auth/oauth/exchange` | ❌       | Exchange OAuth code for tokens — not implemented |
| `POST /v1/auth/oauth/native`   | ❌       | Native OAuth (mobile) — not implemented          |

**Coverage: 0/3. Out of scope per PRD (§3 Non-Goals).**

---

## 7. Auth — TOTP (Authenticator Apps)

| Endpoint                    | Emulator | Notes                             |
| --------------------------- | -------- | --------------------------------- |
| `POST /v1/auth/totp/signup` | ❌       | TOTP enrollment — not implemented |
| `POST /v1/auth/totp/update` | ❌       | TOTP update — not implemented     |
| `POST /v1/auth/totp/verify` | ❌       | TOTP verify — not implemented     |

**Coverage: 0/3. Out of scope per PRD (§3 Non-Goals).**

---

## 8. Auth — Enchanted Link

| Endpoint                                   | Emulator | Notes                                    |
| ------------------------------------------ | -------- | ---------------------------------------- |
| `POST /v1/auth/enchantedlink/signin/email` | ❌       | Enchanted link sign-in — not implemented |
| `POST /v1/auth/enchantedlink/signup/email` | ❌       | Enchanted link sign-up — not implemented |
| `POST /v1/auth/enchantedlink/verify`       | ❌       | Enchanted link verify — not implemented  |
| `POST /v1/auth/enchantedlink/session`      | ❌       | Poll for session — not implemented       |
| `POST /v1/auth/enchantedlink/update/email` | ❌       | Update email — not implemented           |

**Coverage: 0/5. Out of scope per PRD (§3 Non-Goals).**

---

## 9. Auth — WebAuthn / Passkeys

| Endpoint                               | Emulator | Notes                                          |
| -------------------------------------- | -------- | ---------------------------------------------- |
| `POST /v1/auth/webauthn/signup/start`  | ❌       | Passkey enrollment challenge — not implemented |
| `POST /v1/auth/webauthn/signup/finish` | ❌       | Passkey enrollment finish — not implemented    |
| `POST /v1/auth/webauthn/signin/start`  | ❌       | Passkey sign-in challenge — not implemented    |
| `POST /v1/auth/webauthn/signin/finish` | ❌       | Passkey sign-in finish — not implemented       |
| `POST /v1/auth/webauthn/update/start`  | ❌       | Add passkey challenge — not implemented        |
| `POST /v1/auth/webauthn/update/finish` | ❌       | Add passkey finish — not implemented           |

**Coverage: 0/6. Out of scope per PRD (§3 Non-Goals).**

---

## 10. Session Management

| Endpoint                    | Emulator | Notes                                                         |
| --------------------------- | -------- | ------------------------------------------------------------- |
| `POST /v1/auth/refresh`     | ✅       | Supports header, DSR cookie, and body token; revocation check |
| `POST /v1/auth/logout`      | ✅       | Adds to revocation set; subsequent refresh fails              |
| `POST /v1/auth/logoutall`   | ✅       | Revokes all tokens for user by setting revocation timestamp   |
| `GET /v1/auth/me`           | ✅       | Accepts Bearer or DSR cookie                                  |
| `GET /v1/auth/me/history`   | ✅       | Stubbed — returns empty list                                  |
| `POST /v1/auth/validate`    | ✅       | Validates session JWT; returns decoded claims                 |
| `POST /v1/auth/tenant/select` | ✅     | Issues new session JWT with `dct` (selected tenant) claim     |

---

## 11. Management — User

| Endpoint                                   | Emulator | Notes                                                               |
| ------------------------------------------ | -------- | ------------------------------------------------------------------- |
| `POST /v1/mgmt/user/create`                | ✅       | Full field set including tenants, roles, custom attrs               |
| `POST /v1/mgmt/user/create/test`           | ✅       | Marked `_is_test_user`; included in `deleteAll`                     |
| `POST /v1/mgmt/user/create/batch`          | ✅       | Creates multiple users; non-transactional                           |
| `GET /v1/mgmt/user?loginid=…`              | ✅       | Lookup by loginId                                                   |
| `GET /v1/mgmt/user/userid?userid=…`        | ✅       | Lookup by userId                                                    |
| `POST /v1/mgmt/user/search`                | ✅       | Filters by email, phone, customAttributes, withTestUser, pagination |
| `POST /v1/mgmt/user/update`                | ✅       | Full replace of user fields including userTenants                   |
| `PATCH /v1/mgmt/user/patch`                | ✅       | Partial update (preserves unspecified fields; ignores userTenants)  |
| `POST /v1/mgmt/user/update/email`          | ✅       | Updates email + verified flag via patch                             |
| `POST /v1/mgmt/user/update/name`           | ✅       | Updates display name                                                |
| `POST /v1/mgmt/user/update/phone`          | ✅       | Updates phone number                                                |
| `POST /v1/mgmt/user/update/loginid`        | ✅       | Renames loginId; old loginId no longer valid                        |
| `POST /v1/mgmt/user/update/role/set`       | ✅       | Replaces global role set for user                                   |
| `POST /v1/mgmt/user/update/role/remove`    | ✅       | Removes specified global roles                                      |
| `POST /v1/mgmt/user/password/set/active`   | ✅       | Sets bcrypt password hash for user                                  |
| `POST /v1/mgmt/user/password/expire`       | ✅       | Marks password as expired                                           |
| `POST /v1/mgmt/user/password/set/temporary` | ✅      | Sets temporary password; user can sign in with it                   |
| `POST /v1/mgmt/user/logout`                | ✅       | Force-revokes all sessions for user                                 |
| `DELETE /v1/mgmt/user?loginid=…`           | ✅       | Delete by loginId                                                   |
| `DELETE /v1/mgmt/user/userid?userid=…`     | ✅       | Delete by userId                                                    |
| `DELETE /v1/mgmt/user/test/delete/all`     | ✅       | Removes all `_is_test_user` users                                   |
| `POST /v1/mgmt/user/delete/batch`          | ✅       | Deletes multiple users; unknown IDs silently ignored                |
| `POST /v1/mgmt/user/tenant/add`            | ✅       | Associates user with a tenant + roles                               |
| `POST /v1/mgmt/user/tenant/remove`         | ✅       | Removes user from tenant                                            |
| `POST /v1/mgmt/user/tenant/setRole`        | ✅       | Sets tenant-scoped roles for user                                   |
| `POST /v1/mgmt/user/embeddedlink`          | ✅       | Issues `Embedded` token (login-by-token flow)                       |
| `POST /v1/mgmt/user/status/enable`         | ✅       | Enables user (via status update handler)                            |
| `POST /v1/mgmt/user/status/disable`        | ✅       | Disables user (via status update handler)                           |
| `POST /v1/mgmt/user/ssoapp/add`            | ❌       | Add user to SSO app — not implemented                               |
| `POST /v1/mgmt/user/ssoapp/remove`         | ❌       | Remove user from SSO app — not implemented                          |
| `GET /v1/mgmt/user/providers`              | ❌       | List providers for user — not implemented                           |

---

## 12. Management — Test User Flows

| Endpoint                                     | Emulator | Notes                                                    |
| -------------------------------------------- | -------- | -------------------------------------------------------- |
| `POST /v1/mgmt/tests/generate/magiclink`     | ✅       | Returns `link`, `token`, `maskedEmail`; test users only  |
| `POST /v1/mgmt/tests/generate/otp`           | ✅       | Returns OTP code; test users only                        |
| `POST /v1/mgmt/tests/generate/enchantedlink` | ✅       | Returns token consumable via `magiclink/verify`; test users only |

---

## 13. Management — Tenant

| Endpoint                      | Emulator | Notes                                                          |
| ----------------------------- | -------- | -------------------------------------------------------------- |
| `GET /v1/mgmt/tenant/all`     | ✅       | Returns all seeded/created tenants                             |
| `POST /v1/mgmt/tenant/create` | ✅       | Creates tenant; duplicate ID → 409                             |
| `POST /v1/mgmt/tenant/update` | ✅       | Updates tenant name/settings                                   |
| `POST /v1/mgmt/tenant/search` | ✅       | Filters by tenant name(s); empty filter returns all            |
| `DELETE /v1/mgmt/tenant`      | ✅       | Deletes tenant by ID                                           |
| `GET /v1/mgmt/tenant?id=…`    | ✅       | Load single tenant by ID                                       |

---

## 14. Management — JWT

| Endpoint                  | Emulator | Notes                                                                     |
| ------------------------- | -------- | ------------------------------------------------------------------------- |
| `POST /v1/mgmt/jwt/update` | ✅      | Accepts session JWT + `customClaims`; issues new JWT with claims merged   |

---

## 15. Management — Access Keys

| Endpoint                             | Emulator | Notes           |
| ------------------------------------ | -------- | --------------- |
| `POST /v1/mgmt/accesskey/create`     | ❌       | Not implemented |
| `GET /v1/mgmt/accesskey`             | ❌       | Not implemented |
| `POST /v1/mgmt/accesskey/search`     | ❌       | Not implemented |
| `POST /v1/mgmt/accesskey/update`     | ❌       | Not implemented |
| `DELETE /v1/mgmt/accesskey`          | ❌       | Not implemented |
| `POST /v1/mgmt/accesskey/activate`   | ❌       | Not implemented |
| `POST /v1/mgmt/accesskey/deactivate` | ❌       | Not implemented |

**Coverage: 0/7. Not in PRD scope.**

---

## 16. Management — Roles, Permissions, Groups

| Endpoint                          | Emulator | Notes           |
| --------------------------------- | -------- | --------------- |
| `POST /v1/mgmt/role/create`       | ❌       | Not implemented |
| `POST /v1/mgmt/permission/create` | ❌       | Not implemented |
| `POST /v1/mgmt/group/members`     | ❌       | Not implemented |

**Coverage: 0/many. Not in PRD scope.**

---

## 17. Management — Flows & Audit

| Endpoint                        | Emulator | Notes                             |
| ------------------------------- | -------- | --------------------------------- |
| `POST /v1/mgmt/flow/run`        | ❌       | Run a Descope Flow — not in scope |
| `POST /v1/mgmt/auditlog/search` | ❌       | Audit log search — not in scope   |

---

## 18. OIDC Provider Endpoints

| Endpoint                   | Emulator | Notes                                 |
| -------------------------- | -------- | ------------------------------------- |
| `GET /oauth2/v1/authorize` | ❌       | OIDC authorization — not implemented  |
| `POST /oauth2/v1/token`    | ❌       | OIDC token exchange — not implemented |
| `GET /oauth2/v1/userinfo`  | ❌       | OIDC userinfo — not implemented       |
| `GET /oauth2/v1/keys`      | ❌       | OIDC JWKS variant — not implemented   |
| `GET /oauth2/v1/apps-info` | ❌       | App metadata — not implemented        |

---

## Summary Scorecard

| Feature Area              | Implemented | Total (in-scope) | Coverage               |
| ------------------------- | ----------- | ---------------- | ---------------------- |
| Infrastructure / JWKS     | 2           | 2                | ✅ 100%                |
| Password Auth             | 6           | 6                | ✅ 100%                |
| Magic Link                | 7           | 7 (in-scope)     | ✅ 100%                |
| OTP                       | 8           | 8 (in-scope)     | ✅ 100%                |
| SAML / SSO                | 4           | 4                | ✅ 100%                |
| Session Mgmt              | 7           | 7                | ✅ 100%                |
| Mgmt — User               | 28          | 28 (in-scope)    | ✅ 100%                |
| Mgmt — Test Flows         | 3           | 3                | ✅ 100%                |
| Mgmt — Tenant             | 6           | 6                | ✅ 100%                |
| Mgmt — JWT                | 1           | 1                | ✅ 100%                |
| **In-scope total**        | **72**      | **72**           | **✅ 100% of PRD scope** |

---

## Remaining Gaps (Outside Original PRD Scope)

These were out-of-scope but could be added if needed:

| Area                                       | Endpoints |
| ------------------------------------------ | --------- |
| OTP voice (signup/signin/update)           | 3         |
| Magic link WhatsApp                        | 2         |
| OAuth / Social login                       | 3         |
| TOTP / Authenticator                       | 3         |
| Enchanted Link (auth flow)                 | 5         |
| WebAuthn / Passkeys                        | 6         |
| OIDC Provider                              | 5         |
| Access Keys                                | 7         |
| Roles / Permissions / Groups               | many      |
| Audit Log / Flow execution                 | many      |
| SSO App user management                    | 2         |
| User provider listing                      | 1         |

---

## Emulator-Specific Deviations from Descope Behavior

These are intentional differences that aid testability but differ from real Descope:

| Behavior                          | Emulator                                         | Real Descope                      |
| --------------------------------- | ------------------------------------------------ | --------------------------------- |
| `password/reset` response         | Returns `token` in body for test access          | Sends email; no token in response |
| `magiclink/signin/email` response | Returns `token` in body for test access          | Sends email; no token in response |
| `otp/signup` response             | Returns `code` in body for test access           | Sends email/SMS; no code in response |
| SAML start                        | Returns redirect URL with code; no real IdP call | Redirects browser to IdP          |
| `password/policy`                 | Static permissive policy; not configurable       | Project-specific policy           |
| `me/history`                      | Always returns empty list                        | Returns real session history      |
| `POST /emulator/reset`            | Clears all state + re-seeds                      | Does not exist                    |
| `GET /health`                     | Health check                                     | Does not exist                    |
| RSA key                           | Generated at startup (or loaded from file)       | Managed by Descope platform       |
| Token TTL                         | Configurable via env vars                        | Managed by project settings       |
| Rate limiting                     | None                                             | Enforced                          |
| Email/SMS delivery                | Never sends                                      | Actually delivers                 |
