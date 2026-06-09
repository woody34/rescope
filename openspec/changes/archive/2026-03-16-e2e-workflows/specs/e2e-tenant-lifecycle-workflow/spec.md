## ADDED Requirements

### Requirement: Tenants can be created via Admin UI
The Tenants page SHALL provide a form to create a new tenant with an ID, name, and optional domains. Created tenants SHALL be retrievable via the management API.

#### Scenario: Create tenant in UI → visible via API
- **WHEN** user creates a tenant via the UI with a given ID and name
- **THEN** `GET /v1/mgmt/tenant/all` includes a tenant with that ID and name

---

### Requirement: Users can be assigned to tenants via Admin UI
The User detail view SHALL support assigning a user to a tenant (by ID) and optionally specifying a role within that tenant.

#### Scenario: Assigned user appears in tenant's user list
- **WHEN** a user is assigned to a tenant via the UI
- **THEN** fetching the user via `GET /v1/mgmt/user?loginid=...` shows `userTenants` containing that tenantId

---

### Requirement: Tenant membership appears in session JWT
When a user who belongs to a tenant authenticates, the resulting session JWT SHALL contain a `tenants` claim with the tenant's ID.

#### Scenario: Session JWT carries tenant claim
- **WHEN** a user belonging to a tenant signs in via password auth
- **THEN** decoding the `sessionJwt` reveals a `tenants` object containing the tenant's ID as a key

#### Scenario: Tenant role appears in JWT tenant claim
- **WHEN** a user is assigned to a tenant with a role and then signs in
- **THEN** the decoded `sessionJwt.tenants.<tenantId>.roles` array contains that role name
