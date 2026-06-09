## 1. API Client Extensions

- [x] 1.1 Add `users.update(loginId, params)` method to `api.ts` calling `POST /v1/mgmt/user/update`
- [x] 1.2 Add `users.patch(loginId, params)` method to `api.ts` calling `POST /v1/mgmt/user/patch`
- [x] 1.3 Add `users.setStatus(loginId, status)` method to `api.ts` calling `POST /v1/mgmt/user/update/status`
- [x] 1.4 Add `users.addTenant(loginId, tenantId, roleNames)` method to `api.ts` calling `POST /v1/mgmt/user/update/tenant/add`
- [x] 1.5 Add `users.removeTenant(loginId, tenantId)` method to `api.ts` calling `POST /v1/mgmt/user/update/tenant/remove`
- [x] 1.6 Add `users.setTenantRoles(loginId, tenantId, roleNames)` method to `api.ts` calling `POST /v1/mgmt/user/update/tenant/set-role`

## 2. User Modal Component

- [x] 2.1 Create `UserModal.tsx` component with create/edit mode based on optional `user` prop
- [x] 2.2 Implement basic fields section: Login ID (read-only in edit mode), Display Name, Given Name, Family Name, Email, Phone
- [x] 2.3 Implement status toggle in modal header (badge showing Active/Disabled, clickable to toggle)
- [x] 2.4 Implement client-side diff logic: on save, compute changed fields from original user and call `users.patch()` with only the diff
- [x] 2.5 Implement no-op save behavior: if no fields changed, close modal without API call

## 3. Custom Attributes in User Modal

- [x] 3.1 Fetch custom attribute definitions on modal open via `customAttributes.list()`
- [x] 3.2 Render boolean attributes as labeled checkboxes
- [x] 3.3 Render text attributes as text inputs
- [x] 3.4 Render number attributes as number inputs with numeric validation
- [x] 3.5 Render datetime attributes as datetime-local inputs
- [x] 3.6 Handle empty state: hide section or show "No custom attributes defined" when none exist

## 4. Tenant & Role Assignment in User Modal

- [x] 4.1 Add "Authorization" section to `UserModal` with list of current tenant assignments
- [x] 4.2 Implement "+ Add Tenant / Role" button that expands a row with tenant dropdown (fetched from `tenants.list()`)
- [x] 4.3 Implement role multi-select dropdown for each tenant assignment (fetched from `roles.list()`)
- [x] 4.4 Implement remove button for existing tenant assignments
- [x] 4.5 Handle empty state: show "No tenants defined. Create a tenant first." when no tenants exist
- [x] 4.6 Save tenant changes via `users.addTenant()` / `users.removeTenant()` / `users.setTenantRoles()` API calls

## 5. Users Table Enhancements

- [x] 5.1 Create `ColumnPicker` component with gear icon button and checkbox dropdown
- [x] 5.2 Define available columns: Login ID, Status, Display Name, Email, Phone, Verified, Roles, Tenants, Created Time
- [x] 5.3 Implement localStorage persistence for column preferences under `rescope:users:visibleColumns`
- [x] 5.4 Set default visible columns: Login ID, Status, Display Name, Email, Phone, Created Time
- [x] 5.5 Update `UsersPage` table rendering to respect visible columns
- [x] 5.6 Add click handler on user rows to open the edit modal

## 6. Custom Attributes Sub-Tab

- [x] 6.1 Add nested routes in `App.tsx`: `/users` (user list) and `/users/attributes` (attribute definitions)
- [x] 6.2 Create tab bar component with "Users" and "Custom Attributes" tabs using NavLink
- [x] 6.3 Move existing Custom Attributes page content into a sub-component of the Users page
- [x] 6.4 Remove "Custom Attributes" link from `Sidebar.tsx` Settings section
- [x] 6.5 Implement localStorage persistence for Custom Attributes table column preferences under `rescope:attributes:visibleColumns`

## 7. E2E Tests (Playwright, POM pattern)

- [x] 7.1 Create `UsersPage` POM class with methods for create, edit, search, column picker interactions
- [x] 7.2 Write E2E test: create user via modal, verify in table
- [x] 7.3 Write E2E test: edit user fields, verify diff-based patch updates correctly
- [x] 7.4 Write E2E test: toggle user status via modal header
- [x] 7.5 Write E2E test: add tenant and role to user, verify in modal
- [x] 7.6 Write E2E test: column picker toggle and localStorage persistence
- [x] 7.7 Write E2E test: custom attributes sub-tab navigation and CRUD
- [x] 7.8 Write E2E test: custom attribute values in user modal (boolean, text, number)
- [x] 7.9 Write E2E test: navigate between Users and Custom Attributes tabs via URL
