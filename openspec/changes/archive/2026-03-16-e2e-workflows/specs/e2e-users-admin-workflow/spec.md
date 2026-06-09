## ADDED Requirements

### Requirement: Users table displays API-created users
The Users page at `/ui/users` SHALL display all non-test users created via the management API in a table with at least the loginId visible.

#### Scenario: User created via API appears in table
- **WHEN** a user is created via `POST /v1/mgmt/user/create` and the Users page is loaded
- **THEN** a table row containing the user's loginId is visible

#### Scenario: Deleted user no longer appears
- **WHEN** a user is deleted via `DELETE /v1/mgmt/user` and the Users page is refreshed
- **THEN** no table row containing that loginId is present

---

### Requirement: User can be edited via Admin UI
The Users page SHALL provide the ability to edit user details (name, email) via a form/drawer, and the changes SHALL be reflected when the user record is subsequently fetched via the management API.

#### Scenario: Name update in UI persists via API
- **WHEN** user clicks edit on a user row, updates the name field, and saves
- **THEN** `GET /v1/mgmt/user?loginid=...` returns the updated name

---

### Requirement: User count is visible
The Users page SHALL display a count or list that reflects the current number of non-test users.

#### Scenario: Count matches API user count
- **WHEN** N users exist (created via mgmt API) and the Users page is loaded
- **THEN** the table contains exactly N rows
