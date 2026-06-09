## 1. Expand Search API (Rust)

- [x] 1.1 Add new fields to `SearchRequest` in `user.rs`: `loginIds`, `roleNames`, `tenantIds`, `statuses`, `text`, `sort` (field + desc), `createdAfter`, `createdBefore`
- [x] 1.2 Add corresponding fields to `SearchQuery` in `user_store.rs`
- [x] 1.3 Implement `loginIds` filter in `UserStore::search()` — match if user's primary loginId is in the list
- [x] 1.4 Implement `statuses` filter — match if user's status is in the list
- [x] 1.5 Implement `tenantIds` filter — match if any of user's tenant assignments has a matching tenantId
- [x] 1.6 Implement `roleNames` filter — match if user has the role at project level or in any tenant
- [x] 1.7 Implement `text` filter — case-insensitive substring match across loginId, name, email, phone
- [x] 1.8 Implement `createdAfter` / `createdBefore` filter — compare against user's `createdTime`
- [x] 1.9 Implement `sort` — sort results by specified field with direction, default `createdTime` desc

## 2. Rust Unit Tests for Search Filters

- [x] 2.1 Test: `search_by_login_ids_returns_matching_users`
- [x] 2.2 Test: `search_by_statuses_returns_matching_users`
- [x] 2.3 Test: `search_by_tenant_ids_returns_matching_users`
- [x] 2.4 Test: `search_by_role_names_returns_matching_users`
- [x] 2.5 Test: `search_by_text_matches_name_email_phone`
- [x] 2.6 Test: `search_by_text_is_case_insensitive`
- [x] 2.7 Test: `search_by_date_range_filters_correctly`
- [x] 2.8 Test: `search_sort_by_name_ascending`
- [x] 2.9 Test: `search_combined_filters_use_and_semantics`

## 3. UI Filter Bar

- [x] 3.1 Add `statusFilter`, `tenantFilter`, `roleFilter` state to `UsersListTab`
- [x] 3.2 Create status dropdown (options: All, Enabled, Disabled, Invited)
- [x] 3.3 Create tenant dropdown (populated from `api.tenants.list()`)
- [x] 3.4 Create role dropdown (populated from `api.roles.list()`)
- [x] 3.5 Wire filter values into the `api.users.search()` call as `statuses`, `tenantIds`, `roleNames`
- [x] 3.6 Add 300ms debounce to search input and send as `text` parameter
- [x] 3.7 Remove client-side text filtering (replace with server-side `text` filter)
- [x] 3.8 Style filter bar to align with existing toolbar (search + column picker)

## 4. E2E Tests (Playwright)

- [x] 4.1 Test: status filter dropdown shows only enabled users when "Enabled" selected
- [x] 4.2 Test: tenant filter dropdown shows only users in selected tenant
- [x] 4.3 Test: combining status + tenant filters narrows results correctly
- [x] 4.4 Test: clearing filters restores full user list
