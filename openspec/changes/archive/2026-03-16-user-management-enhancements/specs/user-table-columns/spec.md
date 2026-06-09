## ADDED Requirements

### Requirement: Column picker toggle
The system SHALL provide a gear/settings icon button in the users table header area. Clicking the button MUST open a dropdown/popover that lists all available columns with checkboxes.

#### Scenario: Toggle a column off
- **WHEN** the user unchecks a column (e.g., "Phone") in the column picker
- **THEN** the Phone column is immediately hidden from the table

#### Scenario: Toggle a column on
- **WHEN** the user checks a column (e.g., "Email") in the column picker
- **THEN** the Email column is immediately shown in the table

#### Scenario: Default visible columns
- **WHEN** the user visits the Users page for the first time (no saved preferences)
- **THEN** the following columns MUST be visible by default: Login ID, Status, Display Name, Email, Phone, Created Time

### Requirement: Column preference persistence
The system SHALL persist column visibility preferences to localStorage. Preferences MUST survive page navigation and page refresh.

#### Scenario: Preferences survive page navigation
- **WHEN** the user hides the "Phone" column and navigates to another page then back to Users
- **THEN** the "Phone" column remains hidden

#### Scenario: Preferences survive page refresh
- **WHEN** the user hides the "Phone" column and refreshes the browser
- **THEN** the "Phone" column remains hidden

#### Scenario: Separate persistence for Users and Custom Attributes
- **WHEN** the user configures column visibility on the Users tab and the Custom Attributes tab
- **THEN** each tab MUST maintain its own independent column preferences

### Requirement: Available columns
The users table SHALL support the following columns, each toggleable via the column picker: Login ID, Status, Display Name, Email, Phone, Verified, Roles, Tenants, Created Time.

#### Scenario: Extra columns display correct data
- **WHEN** the user enables the "Roles" column
- **THEN** the table displays the user's role names as badges

#### Scenario: Tenants column display
- **WHEN** the user enables the "Tenants" column
- **THEN** the table displays the user's tenant names as badges
