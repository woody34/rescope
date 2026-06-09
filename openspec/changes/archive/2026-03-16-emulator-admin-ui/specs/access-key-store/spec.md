## ADDED Requirements

### Requirement: Custom user attribute schemas are stored and surfaced
The emulator SHALL maintain a `CustomAttributeStore` with schema definitions (`name`, `machine_name`, `type: string|number|bool|date`, `permissions: "all"|"admin"`). Management API: `POST /v1/mgmt/user/attribute`, `GET /v1/mgmt/user/attribute/all`. Users may store values for any custom attribute key; schema definitions describe available fields for the admin UI.

#### Scenario: Custom attribute schema is created and listed
- **WHEN** `POST /v1/mgmt/user/attribute` is called with `{name: "Plan", machineName: "plan", type: "string", permissions: "all"}`
- **THEN** response is `200 OK`
- **WHEN** `GET /v1/mgmt/user/attribute/all` is called
- **THEN** the attribute schema appears in the list

#### Scenario: Users can store values for custom attributes
- **WHEN** a custom attribute schema `plan` exists
- **WHEN** a user is created with `custom_attributes: {plan: "enterprise"}`
- **THEN** the user's profile includes `customAttributes.plan = "enterprise"`

## ADDED Requirements

### Requirement: Multiple access keys are stored with expiry and role scoping
The emulator SHALL maintain an `AccessKeyStore`. Each key has: `id`, `name`, `key_hash` (bcrypt of the raw key value), `expires_at: Option<u64>`, `permitted_ips: Vec<String>` (CIDR), `role_names: Vec<String>`, `tenant_roles: Vec<UserTenant>`, `created_at: u64`, `status: active|expired|disabled`. Management API follows Descope shapes: `POST /v1/mgmt/accesskey`, `GET /v1/mgmt/accesskey/all`, `POST /v1/mgmt/accesskey/update`, `DELETE /v1/mgmt/accesskey/delete`.

#### Scenario: Access key is created and returned once
- **WHEN** `POST /v1/mgmt/accesskey` is called with `{name: "ci-key", expireTime: 0}`
- **THEN** response includes a `cleartext` field with the raw key value (shown once only)
- **THEN** subsequent calls to `GET /v1/mgmt/accesskey/all` do NOT include the raw key, only masked fields

#### Scenario: Management auth accepts a valid stored access key
- **WHEN** an access key with value `"proj:secret"` is stored
- **WHEN** a management API is called with `Authorization: Bearer proj:secret`
- **THEN** the request is authorized

#### Scenario: Expired access key is rejected
- **WHEN** an access key has `expires_at` in the past
- **WHEN** a management API is called with that key
- **THEN** response is `401 Unauthorized`

#### Scenario: Bootstrap key from EmulatorConfig always authorizes
- **WHEN** the `AccessKeyStore` is empty
- **WHEN** the bootstrap management key from `EmulatorConfig` is used
- **THEN** the request is authorized (prevents lockout)
