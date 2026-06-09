## ADDED Requirements

### Requirement: Create user via modal
The system SHALL provide a modal dialog for creating new users. The modal MUST include fields for Login ID (required), Display Name, Given Name, Family Name, Email, Phone, and status (enabled by default).

#### Scenario: Create a new user with all fields
- **WHEN** the user clicks "+ Create User" and fills in Login ID, Name, Email, Phone, and clicks "Create"
- **THEN** a new user is created via `POST /v1/mgmt/user/create` and the user list refreshes with the new user visible

#### Scenario: Create a user with only required fields
- **WHEN** the user fills in only Login ID and clicks "Create"
- **THEN** a new user is created with only the loginId set and other fields empty

#### Scenario: Create user validation
- **WHEN** the user attempts to submit the create form without a Login ID
- **THEN** the Create button MUST be disabled

### Requirement: Edit user via modal
The system SHALL provide an edit modal pre-filled with the existing user's data. The modal MUST display the same fields as create, with Login ID shown as read-only. The user's status SHALL be toggleable via a badge/button in the modal header.

#### Scenario: Open edit modal
- **WHEN** the user clicks on a user row in the table (or an edit icon)
- **THEN** the edit modal opens pre-filled with the user's current field values and Login ID is read-only

#### Scenario: Edit user fields and save
- **WHEN** the user modifies one or more fields (e.g., changes Display Name) and clicks "Save"
- **THEN** only the changed fields are sent via `POST /v1/mgmt/user/patch` and the user list refreshes with updated data

#### Scenario: Toggle user status in edit modal
- **WHEN** the user clicks the status badge in the modal header (e.g., toggling from "Active" to "Disabled")
- **THEN** `POST /v1/mgmt/user/update/status` is called with the new status and the badge updates to reflect the change

#### Scenario: No changes on save
- **WHEN** the user opens the edit modal, makes no changes, and clicks "Save"
- **THEN** no API call is made and the modal closes

### Requirement: Custom attribute values in user modal
The system SHALL display custom attribute value editors within the user create/edit modal. The attributes shown MUST be based on the attribute definitions fetched from `GET /v1/mgmt/user/attribute/all`.

#### Scenario: Boolean attribute renders as checkbox
- **WHEN** a custom attribute of type `boolean` is defined
- **THEN** the user modal renders a labeled checkbox for that attribute

#### Scenario: Text attribute renders as text input
- **WHEN** a custom attribute of type `text` is defined
- **THEN** the user modal renders a labeled text input for that attribute

#### Scenario: Number attribute renders with validation
- **WHEN** a custom attribute of type `number` is defined
- **THEN** the user modal renders a number input that rejects non-numeric values

#### Scenario: Datetime attribute renders as datetime input
- **WHEN** a custom attribute of type `datetime` is defined
- **THEN** the user modal renders a datetime-local input for that attribute

#### Scenario: No custom attributes defined
- **WHEN** no custom attributes are defined in the system
- **THEN** the custom attributes section of the modal is hidden or shows "No custom attributes defined"

### Requirement: Tenant and role assignment in user modal
The system SHALL provide an "Authorization" section in the user modal for managing tenant and role assignments. Users MUST be able to belong to multiple tenants, each with independent role assignments.

#### Scenario: Add a tenant to a user
- **WHEN** the user clicks "+ Add Tenant / Role" and selects a tenant from the dropdown
- **THEN** the tenant is added to the user's `userTenants` via the update API

#### Scenario: Assign roles within a tenant
- **WHEN** the user selects one or more roles from the role dropdown for a given tenant assignment
- **THEN** the selected roles are sent as `roleNames` within that tenant assignment

#### Scenario: Remove a tenant assignment
- **WHEN** the user clicks the remove button on an existing tenant assignment
- **THEN** `POST /v1/mgmt/user/update/tenant/remove` is called and the assignment is removed

#### Scenario: No tenants exist
- **WHEN** no tenants are defined in the system and the user clicks "+ Add Tenant / Role"
- **THEN** a helpful message is displayed: "No tenants defined. Create a tenant first."

#### Scenario: Multiple tenants with different roles
- **WHEN** a user is assigned to multiple tenants, each with different roles
- **THEN** each tenant assignment displays independently with its own role list
