## ADDED Requirements

### Requirement: Filter by login IDs
The search endpoint SHALL accept a `loginIds` field (array of strings). When provided, results SHALL only include users whose primary login ID matches any value in the array.

#### Scenario: Filter returns matching users only
- **WHEN** search is called with `loginIds: ["alice@test.com", "bob@test.com"]`
- **THEN** only users with login ID `alice@test.com` or `bob@test.com` are returned

#### Scenario: No matches returns empty
- **WHEN** search is called with `loginIds: ["nonexistent@test.com"]`
- **THEN** an empty user array is returned

### Requirement: Filter by role names
The search endpoint SHALL accept a `roleNames` field (array of strings). When provided, results SHALL only include users who have at least one of the specified roles (project-level or tenant-level).

#### Scenario: Filter by project role
- **WHEN** search is called with `roleNames: ["admin"]`
- **THEN** only users with "admin" in their `roleNames` or in any tenant's `roleNames` are returned

### Requirement: Filter by tenant IDs
The search endpoint SHALL accept a `tenantIds` field (array of strings). When provided, results SHALL only include users who are associated with at least one of the specified tenants.

#### Scenario: Filter by tenant
- **WHEN** search is called with `tenantIds: ["tenant-acme"]`
- **THEN** only users with a `userTenants` entry where `tenantId` is `"tenant-acme"` are returned

### Requirement: Filter by status
The search endpoint SHALL accept a `statuses` field (array of strings). When provided, results SHALL only include users whose status matches any value in the array.

#### Scenario: Filter for enabled users only
- **WHEN** search is called with `statuses: ["enabled"]`
- **THEN** only users with status "enabled" are returned

#### Scenario: Filter for multiple statuses
- **WHEN** search is called with `statuses: ["enabled", "disabled"]`
- **THEN** users with status "enabled" or "disabled" are returned (not "invited")

### Requirement: Free-text search
The search endpoint SHALL accept a `text` field (string). When provided, results SHALL only include users where the text appears (case-insensitive) in any of: login ID, display name, email, or phone.

#### Scenario: Text matches email
- **WHEN** search is called with `text: "alice"`
- **THEN** a user with email "alice@example.com" is included in results

#### Scenario: Text is case-insensitive
- **WHEN** search is called with `text: "ALICE"`
- **THEN** a user with name "alice" is included in results

### Requirement: Sort results
The search endpoint SHALL accept a `sort` field with `field` (string) and `desc` (boolean). Results SHALL be sorted by the specified user property. Default sort is `createdTime` descending.

#### Scenario: Sort by name ascending
- **WHEN** search is called with `sort: { field: "name", desc: false }`
- **THEN** results are sorted alphabetically by display name, A→Z

#### Scenario: Default sort is creation time descending
- **WHEN** search is called with no `sort` field
- **THEN** results are sorted by `createdTime` descending (newest first)

### Requirement: Filter by creation date range
The search endpoint SHALL accept `createdAfter` and `createdBefore` fields (unix timestamps). When provided, results SHALL only include users created within the specified range (inclusive).

#### Scenario: Filter by date range
- **WHEN** search is called with `createdAfter: 1700000000` and `createdBefore: 1710000000`
- **THEN** only users with `createdTime` between those timestamps (inclusive) are returned

### Requirement: Filters compose with AND semantics
All filter fields SHALL be composed with AND logic. A user must match ALL provided filters to appear in results. Within a single list filter, values are OR-composed.

#### Scenario: Combined filters narrow results
- **WHEN** search is called with `statuses: ["enabled"]` and `tenantIds: ["acme"]`
- **THEN** only users who are enabled AND belong to tenant "acme" are returned
