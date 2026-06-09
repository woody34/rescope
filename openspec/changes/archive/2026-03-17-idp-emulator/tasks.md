## 1. IdP Store (`idp-store`)

- [x] 1.1 Create `IdpEmulator` struct: `id`, `protocol` (oidc|saml), `display_name`, `tenant_id`, `attribute_mapping` (HashMap<String, String>)
- [x] 1.2 Create `IdpStore` with CRUD: `insert`, `load`, `list`, `update`, `delete`
- [x] 1.3 Add `snapshot()` and `restore()` methods to `IdpStore`
- [x] 1.4 Add `idp_emulators` field to `EmulatorSnapshot` struct
- [x] 1.5 Wire `IdpStore` snapshot/restore into `GET/POST /emulator/snapshot`
- [x] 1.6 Add `idpEmulators` to `SeedFile` struct and `seed::load()`
- [x] 1.7 Add `Arc<RwLock<IdpStore>>` to `EmulatorState`
- [x] 1.8 Create CRUD mgmt endpoints: `POST /v1/mgmt/idp`, `GET /v1/mgmt/idp/all`, `POST /v1/mgmt/idp/update`, `POST /v1/mgmt/idp/delete`
- [x] 1.9 Unit tests: CRUD, snapshot/restore roundtrip, seed loading
- [x] 1.10 Integration tests: CRUD endpoints, snapshot roundtrip with IdP config, seed file with IdP

## 2. OIDC IdP (`idp-oidc`)

- [x] 2.1 Generate separate IdP RSA key pair on startup (stored in `EmulatorState`)
- [x] 2.2 Implement `GET /emulator/idp/:idp_id/.well-known/openid-configuration` (discovery)
- [x] 2.3 Implement `GET /emulator/idp/:idp_id/jwks` (IdP public key)
- [x] 2.4 Implement `GET /emulator/idp/:idp_id/authorize` (renders user picker, generates auth code)
- [x] 2.5 Implement `POST /emulator/idp/:idp_id/token` (code → id_token + access_token)
- [x] 2.6 Generate `id_token` JWT signed with IdP key, claims populated from attribute mapping
- [x] 2.7 OidcCodeStore + SP callback flow (GET /emulator/idp/callback)
- [x] 2.8 End-to-end: authorize → callback → SP code → saml/exchange → session JWTs
- [x] 2.9 Unit tests: discovery, token endpoint, id_token generation, attribute mapping
- [x] 2.10 Integration tests: full OIDC flow (create IdP → configure tenant → discovery → authorize → token → callback → exchange)

## 3. SAML IdP (`idp-saml`)

- [x] 3.1 Use `rcgen` for X.509 cert generation (lightweight alternative to `samael`)
- [x] 3.2 Generate self-signed X.509 certificate for SAML signing on startup
- [x] 3.3 Implement `GET /emulator/idp/:idp_id/metadata` (EntityDescriptor XML)
- [x] 3.4 Implement `GET /emulator/idp/:idp_id/sso` (renders user picker, generates SAML Response)
- [x] 3.5 SAML Response generation: Response + Assertion with NameID, AuthnStatement, AttributeStatement
- [x] 3.6 Populate AttributeStatement from attribute mapping configuration
- [x] 3.7 SP-side ACS callback: `POST /emulator/idp/saml/acs` → extract NameID → SP code → redirect
- [x] 3.8 End-to-end: SSO → auto-submit form → ACS → SP code → saml/exchange → session JWTs
- [x] 3.9 Unit tests: utc_iso, xml_escape, extract_xml_value, SAML Response generation, X.509 cert, auto_submit_form
- [x] 3.10 Integration tests: metadata, SSO programmatic + browser, ACS callback, full SAML flow

## 4. Login Page (`idp-login-page`)

- [x] 4.1 Create HTML template for IdP user picker page (Rescope dark theme) — built into OIDC authorize + SAML SSO
- [x] 4.2 Header: "Rescope IdP Emulator" branding + IdP display name
- [x] 4.3 User list: name, email, "Login" button per user
- [x] 4.4 Empty state: "No users in this tenant" message
- [x] 4.5 OIDC mode: clicking Login → redirect to redirect_uri with code + state
- [x] 4.6 SAML mode: clicking Login → auto-submit form POST with SAMLResponse + RelayState
- [x] 4.7 E2E tests: OIDC user picker renders, user selection redirects, full SSO flow in browser
- [x] 4.8 E2E tests: SAML user picker renders, user selection auto-submits, full SSO flow in browser

## 5. Admin UI (`idp-admin-ui`)

- [x] 5.1 Add "Identity Providers" sidebar nav link and route
- [x] 5.2 Add IdP API client methods to `lib/api.ts` (create, list, update, delete)
- [x] 5.3 Identity Providers list page: table with display name, protocol, tenant, actions
- [x] 5.4 Empty state: "No identity providers configured"
- [x] 5.5 Create IdP dialog: display name, protocol dropdown (OIDC/SAML), tenant dropdown
- [x] 5.6 Auto-wire: creating an IdP auto-configures the tenant's oidcConfig/samlConfig
- [x] 5.7 Attribute mapping editor: table with IdP claim ↔ user field rows, add/remove
- [x] 5.8 Default mappings pre-populated on creation (email, name)
- [x] 5.9 Delete IdP with confirmation dialog (clears tenant SSO config)
- [x] 5.10 "Test SSO" button: opens IdP login page in new tab
- [x] 5.11 POM class: `IdpPage` with locators and helpers
- [x] 5.12 E2E tests: empty state, create OIDC IdP, create SAML IdP, attribute mapping, delete, test SSO

