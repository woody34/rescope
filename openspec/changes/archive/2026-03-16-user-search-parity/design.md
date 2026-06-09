## Context

The emulator's `POST /v1/mgmt/user/search` accepts `emails`, `phones`, `customAttributes`, `withTestUser`, `page`, and `limit`. Descope's API supports 10+ additional filters: `loginIds`, `roleNames`, `tenantIds`, `statuses`, `text`, `sort`, and date range fields. SDK users hitting the emulator with these filters get unfiltered results, causing silent test failures.

The current search implementation in `user_store.rs` (`UserStore::search()`) iterates all users with simple email/phone/attribute matching. The UI does client-side text search via a `search` state variable — this should be wired server-side.

## Goals / Non-Goals

**Goals:**
- Accept all Descope-compatible filter fields in the search request
- Implement efficient in-memory filtering for all new fields
- Server-side text search (case-insensitive substring) across loginId, name, email, phone
- Sort support with field + direction (default: createdTime desc)
- Add UI filter dropdowns for status, tenant, role
- Wire UI filters to the search API, replacing client-side-only search

**Non-Goals:**
- Cursor-based pagination (Descope uses page-based, so do we)
- Fuzzy text search or search ranking — simple substring match is sufficient
- Saved/named filters in the UI
- Full Descope search query language (exists, arrays, etc.)

## Decisions

### 1. Filter composition: AND semantics
All filters are AND-composed (matching Descope behavior). A user must match ALL provided filters to appear in results. Within a list filter (e.g., `statuses: ["enabled", "invited"]`), the semantics are OR — the user must match at least one value in the list.

**Rationale**: Descope uses AND across fields, OR within lists. This is the least-surprise behavior.

### 2. Text search: substring match
The `text` filter performs a case-insensitive substring match across `loginIds[0]`, `name`, `email`, and `phone`. This replaces the client-side `search` state in the UI.

**Alternatives considered**: Regex search — too complex for an emulator, and Descope doesn't support it.

### 3. Sort: single-field with direction
Sort accepts a `field` and `desc` boolean. Field names map to user properties (e.g., `"createdTime"`, `"email"`, `"name"`). Default is `createdTime` descending.

**Alternatives considered**: Multi-field sort — Descope only supports single-field, so we match.

### 4. UI filter bar: inline dropdowns
Filter dropdowns render inline in the table toolbar (same row as search + column picker). Each dropdown populates dynamically (status from hardcoded list, tenants/roles from API).

**Alternatives considered**: Sidebar filter panel — too heavy for the current UI density.

## Risks / Trade-offs

- **Performance**: In-memory filtering is O(n) per filter × number of users. For the emulator's in-memory store (typically <1000 users), this is fine. Not a concern.
- **Filter field name casing**: Descope uses camelCase in JSON (`roleNames`, `tenantIds`). Serde's `rename_all = "camelCase"` handles this automatically.
- **Client-side vs server-side text search**: Moving text search server-side means an API call on every keystroke. Mitigation: debounce the search input (300ms).
