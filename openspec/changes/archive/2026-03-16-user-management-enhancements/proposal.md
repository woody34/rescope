## Why

The Rescope admin UI currently supports only creating and deleting users. Descope's console provides a much richer user management experience — editing all fields, toggling status, assigning tenants and roles, managing custom attribute schemas, and configuring column visibility. These gaps force developers to use the API directly for common user management tasks, undermining the purpose of the admin UI.

## What Changes

- **Combined create/edit user modal** — single modal for both creating and editing users, supporting all user fields (name, given name, family name, email, phone, status toggle, verified flags)
- **Custom attributes in user modal** — display attribute values inline using type-appropriate inputs (checkbox for boolean, text for string, number input for number) with validation
- **Tenant + role assignment in modal** — "Authorization" section with the ability to add/remove tenant associations and set roles per tenant, with helpful messaging when no tenants exist
- **Custom Attributes sub-tab** — move the Custom Attributes page from the Settings sidebar section to a sub-tab within the Users page at `/users/attributes`, removing it from the sidebar
- **Table column picker** — gear icon dropdown to toggle visible table columns, with preferences persisted to localStorage across page navigation
- **User table enhancements** — display core fields by default, with toggleable extra columns (email, phone, given name, etc.)
- **`api.ts` client additions** — add `update`, `status`, `addTenant`, `removeTenant`, `setTenantRoles` methods for users
- **Sub-tab routing** — `/users` for user list, `/users/attributes` for custom attribute definitions

## Capabilities

### New Capabilities
- `user-edit-modal`: Combined create/edit user dialog with full field support, custom attribute values, tenant/role assignment, and status toggle
- `user-table-columns`: Configurable table columns with a gear-icon picker and localStorage persistence
- `custom-attributes-subtab`: Custom Attributes as a sub-tab under Users with its own route

### Modified Capabilities
_(none — no existing specs to modify)_

## Impact

- **UI files**: `UsersPage.tsx` (major rewrite), `Sidebar.tsx` (remove Custom Attributes link), `App.tsx` (update routing), `api.ts` (add user update/status/tenant methods)
- **New UI files**: User modal component, column picker component, custom attributes sub-tab component
- **API**: No Rust changes needed — all endpoints already exist (`update`, `patch`, `status_update`, `add_tenant`, `remove_tenant`, `set_tenant_role`)
- **Tests**: New E2E Playwright tests (POM pattern) for user edit flow, column picker, custom attributes sub-tab. Integration tests for any new API client methods.
