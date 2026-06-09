## Context

Rescope's current SSO flow is a shortcut: `saml/start` generates a code, `saml/exchange` swaps it for JWTs. No real identity provider is involved — `SamlConfig` and `OidcConfig` on tenants are stored but unused. Developers cannot validate redirect flows, attribute mapping, or IdP configuration without deploying to a cloud environment with a real IdP like Okta or Azure AD.

The goal is to serve both sides — the Service Provider (Descope emulation) and the Identity Provider — from a single Rescope binary.

## Goals / Non-Goals

**Goals:**
- Serve a fully functional OIDC Identity Provider from Rescope (discovery, authorize, token, JWKS)
- Serve a fully functional SAML Identity Provider from Rescope (metadata, SSO, signed assertions)
- Browser-based user picker page at the IdP authorize/SSO endpoint
- Configurable attribute mapping (IdP claim → user field)
- IdP config included in snapshot/seed/import/export
- Admin UI page for managing IdP emulators
- Full test coverage: unit + integration + E2E

**Non-Goals:**
- Real cryptographic validation of SAML AuthnRequests (emulator trusts itself)
- SAML assertion encryption (nobody does this locally)
- OIDC PKCE flow (can be added later)
- Multiple IdP instances per tenant (one IdP per tenant is sufficient)
- Social login emulation (Google, GitHub, etc. — separate concern)

## Decisions

### 1. Separate IdP Signing Key
**Decision:** The IdP uses its own RSA key pair, separate from the SP's key. Auto-generated on startup, included in snapshots.

**Rationale:** In production, the SP and IdP are separate entities with separate trust anchors. Using a shared key would not test the real trust boundary. The IdP key signs `id_token` (OIDC) and SAML Assertions; the SP key signs session/refresh JWTs.

### 2. SAML via `samael` Crate
**Decision:** Use the `samael` crate for SAML Response/Assertion XML generation and signing.

**Rationale:** Hand-rolling XML signing is error-prone and untested. `samael` is the most active Rust SAML crate and handles the XML canonicalization + enveloped signature complexity. The tradeoff is a dependency on `libxml2`, but this is acceptable for a dev tool.

**Alternative:** Template-based XML generation — rejected because XML signing requires proper canonicalization that templates can't provide.

### 3. IdP Serves All Users in Linked Tenant
**Decision:** The IdP user picker shows all users in the tenant associated with the SSO request, rather than maintaining a separate user list per IdP.

**Rationale:** Mirrors how real IdPs work — the IdP returns whoever authenticates, and the SP (Descope) resolves them. Avoids duplicating user management.

### 4. User Picker Page (Not Auto-Approve)
**Decision:** The IdP login page shows an HTML user picker listing all users in the tenant. No auto-approve mode.

**Rationale:** User explicitly requested a picker for interactive and E2E testing. Auto-approve can be added later as a query parameter option.

### 5. Phased Execution (Smallest Testable Pieces)
**Decision:** Break into 5 parallel-optimized changes:

| Phase | Change | Parallel With |
|-------|--------|---------------|
| 1 | `idp-store` — IdP config store + CRUD + snapshot/seed | — |
| 2 | `idp-oidc` — OIDC endpoints + SSO wire-up | Phase 3 |
| 3 | `idp-saml` — SAML endpoints + SSO wire-up | Phase 2 |
| 4 | `idp-login-page` — HTML user picker | After 2+3 |
| 5 | `idp-admin-ui` — Admin UI page | After 1 |

Phases 2 and 3 can execute in parallel since they share the store (Phase 1) but don't depend on each other.

### 6. IdP Endpoints Namespaced Under `/emulator/idp/`
**Decision:** All IdP endpoints live under `/emulator/idp/:idp_id/` to cleanly separate them from the SP-side Descope API.

**Rationale:** Makes it obvious these are emulator-specific endpoints, not part of the Descope API surface. The `:idp_id` parameter allows multiple IdP configs (one per tenant).

## Risks / Trade-offs

- **`samael` dependency size** → Acceptable for a dev tool; doesn't affect production apps using Rescope
- **`libxml2` system dependency** → May require build instructions for some platforms; CI workflows need to install it
- **SAML complexity** → Mitigated by not validating incoming AuthnRequests and using `samael` for the hard parts
- **Scope creep** → Mitigated by strict phasing; each phase ships independently with full tests
