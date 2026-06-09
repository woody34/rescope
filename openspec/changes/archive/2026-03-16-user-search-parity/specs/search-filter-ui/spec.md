## ADDED Requirements

### Requirement: Status filter dropdown
The users table toolbar SHALL include a "Status" dropdown filter. When a status is selected, the table SHALL display only users matching that status by passing `statuses` to the search API.

#### Scenario: Filter by enabled status
- **WHEN** user selects "Enabled" from the status dropdown
- **THEN** the table shows only users with status "enabled"

#### Scenario: Clear filter shows all
- **WHEN** user selects "All" (default) from the status dropdown
- **THEN** the table shows all users regardless of status

### Requirement: Tenant filter dropdown
The users table toolbar SHALL include a "Tenant" dropdown filter populated dynamically from the tenants API. When a tenant is selected, the table SHALL display only users belonging to that tenant.

#### Scenario: Filter by tenant
- **WHEN** user selects "Acme Corp" from the tenant dropdown
- **THEN** the table shows only users who have a tenant assignment to "Acme Corp"

### Requirement: Role filter dropdown
The users table toolbar SHALL include a "Role" dropdown filter populated dynamically from the roles API. When a role is selected, the table SHALL display only users who have that role.

#### Scenario: Filter by role
- **WHEN** user selects "admin" from the role dropdown
- **THEN** the table shows only users who have the "admin" role (project-level or tenant-level)

### Requirement: Search input wired to API
The existing search input SHALL send its value as the `text` parameter to the search API with a 300ms debounce, replacing client-side filtering.

#### Scenario: Text search hits API
- **WHEN** user types "alice" in the search input and waits 300ms
- **THEN** the search API is called with `text: "alice"` and results are updated

### Requirement: Filters compose in UI
Multiple filter selections SHALL be passed together to the search API. The API composes them with AND semantics.

#### Scenario: Status + tenant filter
- **WHEN** user selects status "Enabled" AND tenant "Acme Corp"
- **THEN** the table shows only users who are enabled AND belong to Acme Corp
