## ADDED Requirements

### Requirement: Permissions can be created via Admin UI
The Authorization / RBAC page SHALL provide a form for creating a new permission with a name and optional description. Created permissions SHALL be immediately visible via the management API.

#### Scenario: Create permission in UI → visible via API
- **WHEN** user fills in a permission name and clicks Create on the permissions page
- **THEN** `GET /v1/mgmt/authz/permission/all` includes the new permission

#### Scenario: Created permission appears in the UI table
- **WHEN** a permission is created via the UI form
- **THEN** a table row containing the permission name is visible on the page

---

### Requirement: Roles can be created via Admin UI
The Authorization / RBAC page SHALL provide a form for creating a new role with a name and assignable permissions list. Created roles SHALL be immediately visible via the management API.

#### Scenario: Create role in UI → visible via API
- **WHEN** user fills in a role name and clicks Create on the roles page
- **THEN** `GET /v1/mgmt/authz/role/all` includes the new role

#### Scenario: Role with permission assigned reflects in API
- **WHEN** user creates a role and assigns an existing permission to it
- **THEN** `GET /v1/mgmt/authz/role/all` returns the role with that permission in its permissions list

---

### Requirement: Permissions and roles deleted via UI are removed via API
The Admin UI SHALL provide delete controls for permissions and roles. Deletion SHALL be reflected in the management API.

#### Scenario: Deleted permission removed from API
- **WHEN** user clicks delete on a permission row in the UI
- **THEN** `GET /v1/mgmt/authz/permission/all` no longer contains that permission
