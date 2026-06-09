## ADDED Requirements

### Requirement: JWT templates are stored and applied at token generation time
The emulator SHALL maintain a `JwtTemplateStore` with named templates. Each template SHALL define: `name`, `authorization_claims_format` (e.g. `"flat"` or `"nested"`), and `custom_claims` (array of `{key, type: "dynamic"|"static", value}`). Dynamic claims resolve a field path from the `User` struct (e.g. `"custom_attributes.plan"`). Static claims emit a literal value. When a template is active and token generation is triggered, the emulator SHALL evaluate and merge the template claims into the session JWT body.

#### Scenario: Dynamic claim from user attribute appears in JWT
- **WHEN** an active JWT template has claim `{key: "plan", type: "dynamic", value: "custom_attributes.plan"}`
- **WHEN** a user with `custom_attributes.plan = "enterprise"` authenticates
- **THEN** the generated session JWT contains `"plan": "enterprise"` in the payload

#### Scenario: Static claim appears in all JWTs
- **WHEN** an active JWT template has claim `{key: "env", type: "static", value: "staging"}`
- **WHEN** any user authenticates
- **THEN** the generated session JWT contains `"env": "staging"`

#### Scenario: Dynamic claim with undefined field emits null
- **WHEN** an active JWT template has claim `{key: "plan", type: "dynamic", value: "custom_attributes.plan"}`
- **WHEN** a user without `custom_attributes.plan` set authenticates
- **THEN** the generated session JWT contains `"plan": null`

### Requirement: Template library presets are available
The emulator SHALL provide built-in template presets (Basic User JWT, OIDC, AWS, Hasura) that can be selected as the starting point when creating a new JWT template. Presets are read-only definitions; the user creates a mutable copy.

#### Scenario: OIDC preset is selectable
- **WHEN** creating a new JWT template via the admin UI
- **THEN** the template library shows at minimum: Basic User JWT, OIDC, AWS, Hasura options

### Requirement: JWT templates are managed via management API
`POST /v1/mgmt/jwt/template`, `GET /v1/mgmt/jwt/template/all`, `POST /v1/mgmt/jwt/template/update`, `DELETE /v1/mgmt/jwt/template/delete`. One template may be marked active at a time.

#### Scenario: Setting a template as active affects subsequent tokens
- **WHEN** template `"my-template"` is created and set active
- **WHEN** a user authenticates
- **THEN** the session JWT includes all claims defined in `"my-template"`
