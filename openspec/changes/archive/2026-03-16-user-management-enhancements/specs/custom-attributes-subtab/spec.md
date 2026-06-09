## ADDED Requirements

### Requirement: Custom Attributes as sub-tab under Users
The system SHALL display Custom Attributes as a sub-tab within the Users page. The Users page MUST have two tabs: "Users" (default, at `/users`) and "Custom Attributes" (at `/users/attributes`). The Custom Attributes link SHALL be removed from the Settings section of the sidebar.

#### Scenario: Navigate to Custom Attributes tab
- **WHEN** the user clicks the "Custom Attributes" tab on the Users page
- **THEN** the URL changes to `/users/attributes` and the custom attribute definitions table is displayed

#### Scenario: Navigate to Users tab
- **WHEN** the user is on `/users/attributes` and clicks the "Users" tab
- **THEN** the URL changes to `/users` and the users table is displayed

#### Scenario: Deep link to Custom Attributes
- **WHEN** a user navigates directly to `/users/attributes`
- **THEN** the Custom Attributes tab is active and the attribute definitions table is displayed

#### Scenario: Sidebar link removed
- **WHEN** the user views the sidebar navigation
- **THEN** there is no "Custom Attributes" link under the Settings section

#### Scenario: Users sidebar link remains
- **WHEN** the user clicks "Users" in the sidebar
- **THEN** they are taken to `/users` with the Users tab active

### Requirement: Custom Attributes definition management
The Custom Attributes sub-tab SHALL continue to provide the same functionality as the current standalone page: listing, creating, and deleting custom attribute definitions. Each attribute has a name, machine name, type (text/number/boolean/datetime), and permissions.

#### Scenario: List existing attributes
- **WHEN** the user visits the Custom Attributes tab
- **THEN** all defined custom attributes are displayed in a table with columns for name, machine name, type, and permissions

#### Scenario: Create a new attribute
- **WHEN** the user fills in the create attribute form and submits
- **THEN** the new attribute is created via `POST /v1/mgmt/user/attribute` and appears in the list

#### Scenario: Delete an attribute
- **WHEN** the user clicks delete on an existing attribute
- **THEN** `POST /v1/mgmt/user/attribute/delete` is called and the attribute is removed from the list
