## Why

The Rescope emulator's user search endpoint (`POST /v1/mgmt/user/search`) only supports filtering by `emails`, `phones`, and `customAttributes`. Descope's search API supports filtering by loginIds, roles, tenants, status, free-text, date ranges, and sorting — all commonly used by SDKs. When a developer's code filters by `tenantIds` or `statuses`, the emulator silently ignores those filters and returns all users, causing test failures and confusion.

## What Changes

- Expand `SearchRequest` / `SearchQuery` to accept `loginIds`, `roleNames`, `tenantIds`, `statuses`, `text`, `sort`, `createdAfter`, `createdBefore`
- Implement server-side filtering logic in `UserStore::search()` for all new filter fields
- `text` filter performs case-insensitive substring match across loginId, name, email, phone
- `sort` field supports any user property with asc/desc direction, defaults to `createdTime` desc
- Add filter bar to Users table UI: Status dropdown, Tenant dropdown, Role dropdown
- Wire UI filters to the search API (replace client-side-only search)

## Capabilities

### New Capabilities
- `search-filters`: Server-side search filters for loginIds, roleNames, tenantIds, statuses, text, sort, and date range
- `search-filter-ui`: UI filter bar with dropdowns for status, tenant, and role in the users table

### Modified Capabilities
_(none — no existing specs to modify)_

## Impact

- **API**: `user.rs` (SearchRequest struct, search handler), `user_store.rs` (SearchQuery struct, search method)
- **UI**: `UsersPage.tsx` (filter bar, wired to API search params), `api.ts` (search params type)
- **Tests**: New Rust unit tests for each filter, E2E tests for UI filter interactions
