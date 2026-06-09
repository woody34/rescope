## ADDED Requirements

### Requirement: Tenant can be created at runtime
`POST /v1/mgmt/tenant/create` SHALL accept `{ id?, name, selfProvisioningDomains? }` and persist the tenant in TenantStore. If `id` is omitted a UUID SHALL be generated. Returns `{ id }`.

#### Scenario: Create tenant with explicit id
- **WHEN** `POST /v1/mgmt/tenant/create` is called with `{ id: "t1", name: "Acme" }`
- **THEN** tenant is created and response returns `{ id: "t1" }`

#### Scenario: Create tenant with auto-generated id
- **WHEN** `POST /v1/mgmt/tenant/create` is called without an `id`
- **THEN** tenant is created with a generated id and response returns `{ id: "<generated>" }`

#### Scenario: Duplicate tenant id returns conflict
- **WHEN** `POST /v1/mgmt/tenant/create` is called with an id that already exists
- **THEN** response is 409 Conflict

### Requirement: Tenant can be updated
`POST /v1/mgmt/tenant/update` SHALL accept `{ id, name?, selfProvisioningDomains? }` and update the existing tenant. Returns `{ ok: true }`.

#### Scenario: Update tenant name
- **WHEN** `POST /v1/mgmt/tenant/update` is called with a valid id and new name
- **THEN** tenant name is updated and response returns `{ ok: true }`

#### Scenario: Update unknown tenant returns 404
- **WHEN** `POST /v1/mgmt/tenant/update` is called with an unknown id
- **THEN** response is 404

### Requirement: Tenant can be deleted
`DELETE /v1/mgmt/tenant` SHALL accept `?id=<tenantId>` and remove the tenant. Idempotent — deleting a non-existent tenant returns `{ ok: true }`.

#### Scenario: Delete existing tenant
- **WHEN** `DELETE /v1/mgmt/tenant?id=t1` is called
- **THEN** tenant is removed and response returns `{ ok: true }`

### Requirement: Single tenant can be loaded
`GET /v1/mgmt/tenant` SHALL accept `?id=<tenantId>` and return `{ tenant }`. Returns 404 if not found.

#### Scenario: Load existing tenant
- **WHEN** `GET /v1/mgmt/tenant?id=t1` is called
- **THEN** response returns `{ tenant: { id, name, ... } }`

#### Scenario: Load unknown tenant returns 404
- **WHEN** `GET /v1/mgmt/tenant?id=missing` is called
- **THEN** response is 404

### Requirement: Tenants can be searched
`POST /v1/mgmt/tenant/search` SHALL accept `{ ids?, names? }` and return matching tenants. Empty filters return all tenants.

#### Scenario: Search by name substring
- **WHEN** `POST /v1/mgmt/tenant/search` is called with `{ names: ["Acme"] }`
- **THEN** response returns tenants whose name matches
