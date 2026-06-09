## ADDED Requirements

### Requirement: Permissions are creatable, listable, updatable, and deletable
The emulator SHALL maintain a `PermissionStore` with permission entities (`id`, `name`, `description`). Management API endpoints SHALL follow Descope shapes: `POST /v1/mgmt/authz/permission`, `GET /v1/mgmt/authz/permission/all`, `POST /v1/mgmt/authz/permission/update`, `DELETE /v1/mgmt/authz/permission/delete`. Permission `name` SHALL be unique within the project.

#### Scenario: Create and retrieve a permission
- **WHEN** `POST /v1/mgmt/authz/permission` is called with `{name: "read:reports", description: "Can view reports"}`
- **THEN** response is `200 OK`
- **WHEN** `GET /v1/mgmt/authz/permission/all` is called
- **THEN** response includes the created permission with its `name` and `description`

#### Scenario: Duplicate permission name returns error
- **WHEN** a permission named `"admin"` already exists
- **WHEN** `POST /v1/mgmt/authz/permission` is called with `name: "admin"`
- **THEN** response is `409 Conflict`

### Requirement: Roles are creatable, listable, updatable, and deletable
The emulator SHALL maintain a `RoleStore` with role entities (`id`, `name`, `description`, `permissions: Vec<String>`, `is_default: bool`, `is_hidden: bool`). Role `name` SHALL be unique. `permissions` references permission names. Management API endpoints SHALL follow Descope shapes: `POST /v1/mgmt/authz/role`, `GET /v1/mgmt/authz/role/all`, `POST /v1/mgmt/authz/role/update`, `DELETE /v1/mgmt/authz/role/delete`.

#### Scenario: Role references only defined permissions
- **WHEN** `POST /v1/mgmt/authz/role` is called with `permissions: ["nonexistent-perm"]`
- **THEN** response is `400 Bad Request` indicating the permission does not exist

#### Scenario: Default role is assigned to new users automatically
- **WHEN** a role is configured with `is_default: true`
- **WHEN** a new user is created via any auth flow
- **THEN** the new user has the default role in their `role_names`

### Requirement: Roles and permissions are reflected in JWT session claims
The emulator SHALL include the user's resolved role names in the session JWT claims. If a JWT template is configured, its `authorization_claims_format` setting determines the shape of the `roles` claim.

#### Scenario: Session JWT includes roles claim
- **WHEN** a user has `role_names: ["viewer", "editor"]`
- **WHEN** a session JWT is generated for that user
- **THEN** the JWT payload contains a `roles` field with those role names

### Requirement: Tenant-scoped roles are supported
Users MAY have roles scoped to specific tenants via `UserTenant.role_names`. The emulator SHALL support setting and reading tenant-scoped roles independently from project-level roles.

#### Scenario: Tenant role is visible in user tenants
- **WHEN** user is assigned role `"tenant-admin"` on tenant `"t1"`
- **WHEN** user's profile is retrieved
- **THEN** `userTenants[0].roleNames` includes `"tenant-admin"`
