## Why

Rescope's SSO flow is currently a code-swap shortcut — no real IdP is involved. Developers testing SSO integrations can't validate the actual redirect flow, attribute mapping, or IdP configuration without deploying to a cloud environment. A built-in IdP emulator lets developers test the full SSO lifecycle (OIDC and SAML) against a local identity provider, keeping the zero-external-dependency promise.

## What Changes

- **New**: Built-in OIDC Identity Provider — discovery, authorize (user picker page), token, and JWKS endpoints served by Rescope itself at `/emulator/idp/:idp_id/...`
- **New**: Built-in SAML Identity Provider — metadata XML, SSO redirect (user picker page), signed SAML Response generation using `samael` crate
- **New**: `IdpEmulator` store — CRUD for IdP configurations with protocol selection (OIDC/SAML), attribute mapping, and snapshot/seed/import/export support
- **New**: IdP login page — HTML user picker showing all users in the linked tenant; user clicks to "log in" and gets redirected back
- **New**: Attribute mapping — configurable IdP claim → user field mapping, applied to id_token (OIDC) and SAML Assertion attributes
- **New**: Admin UI "Identity Providers" page — create/edit/delete IdP emulators, attribute mapping editor, test SSO button
- **Modified**: `saml/start` + `sso/start` — resolve tenant's IdP config and redirect to the emulated IdP's authorize/SSO endpoint instead of generating a code directly
- **Modified**: `saml/exchange` + `sso/exchange` — validate tokens from the emulated IdP, apply attribute mapping
- **Modified**: `EmulatorSnapshot` + `SeedFile` — include `idpEmulators` field for import/export/seed support
- **New dependency**: `samael` crate for SAML assertion signing and XML generation

## Capabilities

### New Capabilities
- `idp-oidc`: OIDC Identity Provider emulator — discovery, authorize, token, JWKS endpoints with a separate IdP signing key
- `idp-saml`: SAML Identity Provider emulator — metadata XML, SSO endpoint, signed SAML Response/Assertion generation
- `idp-store`: IdP emulator configuration store — CRUD, attribute mapping, snapshot/seed/import/export
- `idp-login-page`: Browser-based user picker page served at the IdP authorize/SSO endpoint — lists tenant users, redirects on selection
- `idp-admin-ui`: Admin UI page for managing IdP emulator configurations, attribute mapping, and testing SSO flows

### Modified Capabilities
_(none — no existing specs to modify)_

## Impact

- **Rust API**: New route module `routes/emulator/idp.rs`, new store `store/idp_store.rs`, separate `KeyManager` instance for IdP signing
- **Dependencies**: Adds `samael` crate (SAML XML generation + signing)
- **Snapshot/Seed**: `EmulatorSnapshot` and `SeedFile` structs gain `idp_emulators` field
- **Admin UI**: New "Identity Providers" page with POM + Playwright E2E tests
- **Testing**: Unit tests for all store/route logic, integration tests for full OIDC and SAML flows, E2E tests for UI workflows
