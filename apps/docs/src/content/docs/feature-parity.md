---
title: Feature Parity
description: Map of every Descope feature to its Rescope implementation across the server API and WASM bridge.
---

# Rescope Feature Parity Map

**Legend:** ✅ Implemented · ⚡ WASM-only (no server handler) · 🔶 Partial · ❌ Not implemented  
**Two runtimes:** `API` = `apps/api` (server, full Rust/Axum) · `WASM` = `apps/rescope-wasm` (in-browser, service worker)

---

## 🔐 Auth — Password

| Descope Feature | Endpoint | API | WASM | Location |
|---|---|:---:|:---:|---|
| Sign up | `POST /v1/auth/password/signup` | ✅ | ✅ | [password.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/password.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Sign in | `POST /v1/auth/password/signin` | ✅ | ✅ | [password.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/password.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Replace password | `POST /v1/auth/password/replace` | ✅ | ❌ | [password.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/password.rs) |
| Send reset | `POST /v1/auth/password/reset` | ✅ | ❌ | [password.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/password.rs) |
| Update password | `POST /v1/auth/password/update` | ✅ | ❌ | [password.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/password.rs) |
| Password policy | `GET /v1/auth/password/policy` | ✅ | ❌ | [password.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/password.rs) |

---

## 📧 Auth — OTP (Email & SMS)

| Descope Feature | Endpoint | API | WASM | Location |
|---|---|:---:|:---:|---|
| Sign up (email) | `POST /v1/auth/otp/signup/email` | ✅ | ❌ | [otp.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/otp.rs) |
| Sign up (SMS) | `POST /v1/auth/otp/signup/phone/sms` | ✅ | ❌ | [otp.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/otp.rs) |
| Sign in (email) | `POST /v1/auth/otp/signin/email` | ✅ | ✅ | [otp.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/otp.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Sign in (SMS) | `POST /v1/auth/otp/signin/phone/sms` | ✅ | ❌ | [otp.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/otp.rs) |
| Verify (email) | `POST /v1/auth/otp/verify/email` | ✅ | ✅ | [otp.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/otp.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Verify (SMS) | `POST /v1/auth/otp/verify/phone/sms` | ✅ | ❌ | [otp.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/otp.rs) |
| Sign-up-in (email) | `POST /v1/auth/otp/signup-in/email` | ✅ | ❌ | [otp.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/otp.rs) |
| Sign-up-in (SMS) | `POST /v1/auth/otp/signup-in/sms` | ✅ | ❌ | [otp.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/otp.rs) |
| Update phone | `POST /v1/auth/otp/update/phone/sms` | ✅ | ❌ | [otp.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/otp.rs) |
| OTP retrieval (emulator) | `GET /emulator/otp/:loginId` | ✅ | ✅ | [snapshot.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/snapshot.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |

---

## 🔗 Auth — Magic Link

| Descope Feature | Endpoint | API | WASM | Location |
|---|---|:---:|:---:|---|
| Sign up (email) | `POST /v1/auth/magiclink/signup/email` | ✅ | ❌ | [magic_link.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/magic_link.rs) |
| Sign in (email) | `POST /v1/auth/magiclink/signin/email` | ✅ | ✅ | [magic_link.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/magic_link.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Verify | `POST /v1/auth/magiclink/verify` | ✅ | ✅ | [magic_link.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/magic_link.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Sign-up-in (email) | `POST /v1/auth/magiclink/signup-in/email` | ✅ | ❌ | [magic_link.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/magic_link.rs) |
| Sign-up-in (SMS) | `POST /v1/auth/magiclink/signup-in/sms` | ✅ | ❌ | [magic_link.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/magic_link.rs) |
| Update email | `POST /v1/auth/magiclink/update/email` | ✅ | ❌ | [magic_link.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/magic_link.rs) |
| Sign up (SMS) | `POST /v1/auth/magiclink/signup/sms` | ✅ | ❌ | [magic_link.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/magic_link.rs) |
| Sign in (SMS) | `POST /v1/auth/magiclink/signin/sms` | ✅ | ❌ | [magic_link.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/magic_link.rs) |
| Update phone | `POST /v1/auth/magiclink/update/phone/sms` | ✅ | ❌ | [magic_link.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/magic_link.rs) |

---

## 🏢 Auth — SAML / SSO (Social/Enterprise)

| Descope Feature | Endpoint | API | WASM | Location |
|---|---|:---:|:---:|---|
| SAML start | `POST /v1/auth/saml/start` | ✅ | ❌ | [saml.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/saml.rs) |
| SAML authorize | `POST /v1/auth/saml/authorize` | ✅ | ❌ | [saml.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/saml.rs) |
| SAML exchange | `POST /v1/auth/saml/exchange` | ✅ | ❌ | [saml.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/saml.rs) |
| SSO authorize | `POST /v1/auth/sso/authorize` | ✅ | ❌ | [saml.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/saml.rs) |
| SSO exchange | `POST /v1/auth/sso/exchange` | ✅ | ❌ | [saml.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/saml.rs) |
| OAuth / Social | _(not mapped)_ | ❌ | ❌ | — |
| Enchanted Link | _(not mapped)_ | ❌ | ❌ | — |
| TOTP | _(not mapped)_ | ❌ | ❌ | — |

---

## 🔄 Session Management

| Descope Feature | Endpoint | API | WASM | Location |
|---|---|:---:|:---:|---|
| Refresh session | `POST /v1/auth/refresh` | ✅ | ❌ | [session.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/session.rs) |
| Try-refresh (SDK alias) | `POST /v1/auth/try-refresh` | ✅ | ❌ | [session.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/session.rs) |
| Logout | `POST /v1/auth/logout` | ✅ | ❌ | [session.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/session.rs) |
| Logout all devices | `POST /v1/auth/logoutall` | ✅ | ❌ | [session.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/session.rs) |
| Get current user | `GET /v1/auth/me` | ✅ | ❌ | [session.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/session.rs) |
| Login history | `GET /v1/auth/me/history` | ✅ | ❌ | [session.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/session.rs) |
| Validate token | `POST /v1/auth/validate` | ✅ | ❌ | [session.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/session.rs) |
| Select tenant | `POST /v1/auth/tenant/select` | ✅ | ❌ | [session.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/auth/session.rs) |
| JWKS | `GET /.well-known/jwks.json` | ✅ | ❌ | [jwks.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/jwks.rs) |
| JWKS (v2 alias) | `GET /v2/keys/:project_id` | ✅ | ❌ | [jwks.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/jwks.rs) |

---

## 👤 Management — Users

| Descope Feature | Endpoint | API | WASM | Location |
|---|---|:---:|:---:|---|
| Create user | `POST /v1/mgmt/user/create` | ✅ | ✅ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Create test user | `POST /v1/mgmt/user/create/test` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Create batch | `POST /v1/mgmt/user/create/batch` | ✅ | ✅ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Load by loginId | `GET /v1/mgmt/user` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Load by userId | `GET /v1/mgmt/user/userid` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Search users | `POST /v1/mgmt/user/search` | ✅ | ✅ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Update user | `POST /v1/mgmt/user/update` | ✅ | ✅ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Patch user | `PATCH /v1/mgmt/user/patch` | ✅ | ✅ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Delete user (GET/DELETE) | `DELETE /v1/mgmt/user` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Delete user (POST) | `POST /v1/mgmt/user/delete` | ✅ | ✅ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Delete batch | `POST /v1/mgmt/user/delete/batch` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Delete all test users | `DELETE /v1/mgmt/user/test/delete/all` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Set status | `POST /v1/mgmt/user/update/status` | ✅ | ✅ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Update email | `POST /v1/mgmt/user/update/email` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Update name | `POST /v1/mgmt/user/update/name` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Update phone | `POST /v1/mgmt/user/update/phone` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Update loginId | `POST /v1/mgmt/user/update/loginid` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Update picture | `POST /v1/mgmt/user/update/picture` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Update custom attribute | `POST /v1/mgmt/user/update/customAttribute` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Set roles | `POST /v1/mgmt/user/update/role/set` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Add roles | `POST /v1/mgmt/user/update/role/add` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Remove roles | `POST /v1/mgmt/user/update/role/remove` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Add to tenant | `POST /v1/mgmt/user/tenant/add` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Remove from tenant | `POST /v1/mgmt/user/tenant/remove` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Set tenant role | `POST /v1/mgmt/user/tenant/setRole` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Set active password | `POST /v1/mgmt/user/password/set/active` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Set temp password | `POST /v1/mgmt/user/password/set/temporary` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Expire password | `POST /v1/mgmt/user/password/expire` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Force logout | `POST /v1/mgmt/user/logout` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Embedded link | `POST /v1/mgmt/user/embeddedlink` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Generate OTP (test) | `POST /v1/mgmt/tests/generate/otp` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Generate magic link (test) | `POST /v1/mgmt/tests/generate/magiclink` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Generate enchanted link (test) | `POST /v1/mgmt/tests/generate/enchantedlink` | ✅ | ❌ | [user.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/user.rs) |
| Custom attributes — list | `GET /v1/mgmt/user/attribute/all` | ✅ | ✅ | [custom_attributes.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/custom_attributes.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Custom attributes — create | `POST /v1/mgmt/user/attribute` | ✅ | ✅ | [custom_attributes.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/custom_attributes.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Custom attributes — delete | `POST /v1/mgmt/user/attribute/delete` | ✅ | ✅ | [custom_attributes.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/custom_attributes.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |

---

## 🏢 Management — Tenants

| Descope Feature | Endpoint | API | WASM | Location |
|---|---|:---:|:---:|---|
| Create | `POST /v1/mgmt/tenant/create` | ✅ | ✅ | [tenant.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/tenant.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Load | `GET /v1/mgmt/tenant` | ✅ | ❌ | [tenant.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/tenant.rs) |
| List all | `GET /v1/mgmt/tenant/all` | ✅ | ✅ | [tenant.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/tenant.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Search | `POST /v1/mgmt/tenant/search` | ✅ | ✅ | [tenant.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/tenant.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Update | `POST /v1/mgmt/tenant/update` | ✅ | ✅ | [tenant.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/tenant.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Delete | `DELETE /v1/mgmt/tenant` | ✅ | ❌ | [tenant.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/tenant.rs) |
| Delete (POST) | `POST /v1/mgmt/tenant/delete` | ✅ | ✅ | [tenant.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/tenant.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| SAML config | _(part of update)_ | 🔶 | ❌ | [tenant.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/tenant.rs) |
| OIDC config | _(part of update)_ | 🔶 | ❌ | [tenant.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/tenant.rs) |

---

## 🛡 Management — Roles & Permissions

| Descope Feature | Endpoint | API | WASM | Location |
|---|---|:---:|:---:|---|
| Permissions — list | `GET /v1/mgmt/authz/permission/all` | ✅ | ✅ | [permissions.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/permissions.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Permissions — create | `POST /v1/mgmt/authz/permission` | ✅ | ✅ | [permissions.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/permissions.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Permissions — update | `POST /v1/mgmt/authz/permission/update` | ✅ | ✅ | [permissions.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/permissions.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Permissions — delete | `POST /v1/mgmt/authz/permission/delete` | ✅ | ✅ | [permissions.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/permissions.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Roles — list | `GET /v1/mgmt/authz/role/all` | ✅ | ✅ | [roles.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/roles.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Roles — create | `POST /v1/mgmt/authz/role` | ✅ | ✅ | [roles.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/roles.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Roles — update | `POST /v1/mgmt/authz/role/update` | ✅ | ✅ | [roles.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/roles.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Roles — delete | `POST /v1/mgmt/authz/role/delete` | ✅ | ✅ | [roles.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/roles.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |

---

## 🔑 Management — Access Keys

| Descope Feature | Endpoint | API | WASM | Location |
|---|---|:---:|:---:|---|
| Create | `POST /v1/mgmt/accesskey` | ✅ | ✅ | [access_keys.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/access_keys.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| List all | `GET /v1/mgmt/accesskey/all` | ✅ | ✅ | [access_keys.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/access_keys.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Update | `POST /v1/mgmt/accesskey/update` | ✅ | ❌ | [access_keys.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/access_keys.rs) |
| Delete | `POST /v1/mgmt/accesskey/delete` | ✅ | ✅ | [access_keys.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/access_keys.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Disable | `POST /v1/mgmt/accesskey/disable` | ✅ | ❌ | [access_keys.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/access_keys.rs) |

---

## 📜 Management — JWT Templates

| Descope Feature | Endpoint | API | WASM | Location |
|---|---|:---:|:---:|---|
| List all | `GET /v1/mgmt/jwt/template/all` | ✅ | ✅ | [jwt_templates.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/jwt_templates.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Get active | `GET /v1/mgmt/jwt/template/active` | ✅ | ✅ | [jwt_templates.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/jwt_templates.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Create | `POST /v1/mgmt/jwt/template` | ✅ | ✅ | [jwt_templates.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/jwt_templates.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Update | `POST /v1/mgmt/jwt/template/update` | ✅ | ❌ | [jwt_templates.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/jwt_templates.rs) |
| Set active | `POST /v1/mgmt/jwt/template/set-active` | ✅ | ❌ | [jwt_templates.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/jwt_templates.rs) |
| Delete | `POST /v1/mgmt/jwt/template/delete` | ✅ | ✅ | [jwt_templates.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/jwt_templates.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Update claims (JWT) | `POST /v1/mgmt/jwt/update` | ✅ | ❌ | [jwt.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/jwt.rs) |

---

## 🔌 Management — Connectors

| Descope Feature | Endpoint | API | WASM | Location |
|---|---|:---:|:---:|---|
| List all | `GET /v1/mgmt/connector/all` | ✅ | ✅ | [connectors.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/connectors.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Create | `POST /v1/mgmt/connector` | ✅ | ✅ | [connectors.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/connectors.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Update | `POST /v1/mgmt/connector/update` | ✅ | ❌ | [connectors.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/connectors.rs) |
| Delete | `POST /v1/mgmt/connector/delete` | ✅ | ✅ | [connectors.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/connectors.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |

---

## 🔒 Management — Auth Method Config

| Descope Feature | Endpoint | API | WASM | Location |
|---|---|:---:|:---:|---|
| Get config | `GET /v1/mgmt/config/auth-methods` | ✅ | ✅ | [auth_method_config.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/auth_method_config.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Update config | `PUT /v1/mgmt/config/auth-methods` | ✅ | ✅ | [auth_method_config.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/auth_method_config.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |

---

## 🏭 Management — IdP Emulators (SAML/OIDC)

| Descope Feature | Endpoint | API | WASM | Location |
|---|---|:---:|:---:|---|
| Create IdP | `POST /v1/mgmt/idp` | ✅ | ✅ | [idp.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/idp.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| List IdPs | `GET /v1/mgmt/idp/all` | ✅ | ✅ | [idp.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/idp.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Update IdP | `POST /v1/mgmt/idp/update` | ✅ | ❌ | [idp.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/idp.rs) |
| Delete IdP | `POST /v1/mgmt/idp/delete` | ✅ | ✅ | [idp.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/mgmt/idp.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| OIDC discovery | `GET /emulator/idp/:id/.well-known/openid-configuration` | ✅ | ❌ | [idp_oidc.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/idp_oidc.rs) |
| OIDC JWKS | `GET /emulator/idp/:id/jwks` | ✅ | ❌ | [idp_oidc.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/idp_oidc.rs) |
| OIDC authorize | `GET /emulator/idp/:id/authorize` | ✅ | ❌ | [idp_oidc.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/idp_oidc.rs) |
| OIDC token | `POST /emulator/idp/:id/token` | ✅ | ❌ | [idp_oidc.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/idp_oidc.rs) |
| OIDC callback | `GET /emulator/idp/callback` | ✅ | ❌ | [idp_oidc.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/idp_oidc.rs) |
| SAML metadata | `GET /emulator/idp/:id/metadata` | ✅ | ❌ | [idp_saml.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/idp_saml.rs) |
| SAML SSO | `GET /emulator/idp/:id/sso` | ✅ | ❌ | [idp_saml.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/idp_saml.rs) |
| SAML ACS | `POST /emulator/idp/saml/acs` | ✅ | ❌ | [idp_saml.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/idp_saml.rs) |

---

## 🔧 Emulator Controls

| Feature | Endpoint | API | WASM | Location |
|---|---|:---:|:---:|---|
| Health | `GET /health` | ✅ | ✅ | [emulator/mod.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/mod.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Reset state | `POST /emulator/reset` | ✅ | ✅ | [emulator/mod.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/mod.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Export snapshot | `GET /emulator/snapshot` | ✅ | ✅ | [snapshot.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/snapshot.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Import snapshot | `POST /emulator/snapshot` | ✅ | ❌ | [snapshot.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/snapshot.rs) |
| List all OTPs | `GET /emulator/otps` | ✅ | ✅ | [snapshot.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/snapshot.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Get OTP by loginId | `GET /emulator/otp/:loginId` | ✅ | ✅ | [emulator/mod.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/mod.rs) · [lib.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/rescope-wasm/src/lib.rs) |
| Create tenant shortcut | `POST /emulator/tenant` | ✅ | ❌ | [emulator/mod.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/routes/emulator/mod.rs) |
| OpenAPI spec | `GET /openapi.json` | ✅ | ❌ | [openapi.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/openapi.rs) |
| Swagger UI | `GET /docs` | ✅ | ❌ | [server.rs](file:///Users/mattwoodruff/repos/descope-emulator/apps/api/src/server.rs) |

---

## 📊 Coverage Summary

| Domain | API Routes | WASM Routes |
|---|:---:|:---:|
| Auth — Password | 6/6 ✅ | 2/6 🔶 |
| Auth — OTP | 9/9 ✅ | 3/9 🔶 |
| Auth — Magic Link | 9/9 ✅ | 3/9 🔶 |
| Auth — SAML/SSO | 5/5 ✅ | 0/5 ❌ |
| Auth — OAuth/Social | ❌ | ❌ |
| Auth — TOTP/Enchanted | ❌ | ❌ |
| Session Management | 8/8 ✅ | 0/8 ❌ |
| Users (CRUD) | 32/32 ✅ | 8/32 🔶 |
| Tenants | 8/8 ✅ | 6/8 🔶 |
| Roles | 4/4 ✅ | 4/4 ✅ |
| Permissions | 4/4 ✅ | 4/4 ✅ |
| Access Keys | 5/5 ✅ | 3/5 🔶 |
| JWT Templates | 7/7 ✅ | 4/7 🔶 |
| Connectors | 4/4 ✅ | 3/4 🔶 |
| Auth Method Config | 2/2 ✅ | 2/2 ✅ |
| IdP Emulators (SAML/OIDC) | 12/12 ✅ | 3/12 🔶 |
| Emulator Controls | 9/9 ✅ | 6/9 🔶 |

> **WASM gap note**: The WASM bridge targets the demo-page use cases. Missing routes are primarily: SMS flows, OAuth/Social, TOTP, session management (refresh/logout), and snapshot import. The server-side API has full coverage.
