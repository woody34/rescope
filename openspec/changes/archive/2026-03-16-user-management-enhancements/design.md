## Context

The Rescope admin UI (`apps/ui/`) is a React + Radix UI application served alongside the Rust emulator API (`apps/api/`). The current `UsersPage.tsx` only supports creating and deleting users. The API already has full CRUD support: `update`, `patch`, `status_update`, `add_tenant`, `remove_tenant`, `set_tenant_role`. The Custom Attributes page currently lives under the "Settings" sidebar section as a standalone page.

The user management capabilities need to match the Descope console's UX patterns — specifically a combined create/edit modal, inline custom attribute editing, tenant + role assignment, and a configurable table with persistent column preferences.

## Goals / Non-Goals

**Goals:**
- Unified create/edit user modal that surfaces all editable fields
- Client-side diff → PATCH update semantics (compute changed fields, send only those)
- Custom Attributes as a sub-tab (`/users/attributes`) within the Users page
- Column picker with localStorage persistence
- Tenant + role assignment within the user modal
- Full E2E, integration, and unit test coverage

**Non-Goals:**
- Application management (Descope's "Define access to applications" — deferred to future change)
- User import/export functionality (scrapped)
- Login ID editing (login ID stays immutable)
- API changes (all needed endpoints already exist)

## Decisions

### 1. Combined Create/Edit Modal as a Shared Component
**Decision:** Extract the user modal into a reusable `UserModal` component that accepts an optional `User` prop. When a user is passed, it pre-fills the form and computes a diff on save. When no user is passed, it behaves as the create flow.

**Rationale:** Avoids duplicating form logic. The form state management is identical; only the save handler differs (POST to create vs. PATCH to update). This matches the Descope console pattern.

**Alternative considered:** Separate edit page (rejected — too much navigation friction for quick edits).

### 2. Client-Side Diff for PATCH Updates
**Decision:** On save, diff the form state against the original user object. Send only changed fields via the existing `PATCH /v1/mgmt/user/patch` endpoint.

**Rationale:** User explicitly requested diff → patch semantics. This prevents accidental field clearing when Save is clicked with unchanged fields. The existing API `patch` endpoint already supports partial updates.

### 3. Custom Attributes Sub-Tab via React Router Nested Routes
**Decision:** Add nested routes under `/users`: `/users` (user list, default) and `/users/attributes` (attribute definitions). Use a tab bar component to switch between them.

**Rationale:** Gives each tab a distinct URL for deep-linking. Matches the Descope console UX (screenshot shows "Users" and "Custom Attributes" tabs). Removing the Custom Attributes link from the sidebar simplifies navigation.

### 4. Column Picker with localStorage Persistence
**Decision:** Store visible column preferences in localStorage under a key like `rescope:users:visibleColumns`. Default to a sensible set (Login ID, Name, Status, Verified, Created). Column state survives page navigation and refresh.

**Rationale:** User explicitly requested persistence. localStorage is the simplest solution for this — no backend state needed. Two separate keys for users and custom attributes column preferences.

### 5. Tenant/Role Assignment as Inline Section in Modal
**Decision:** Add an "Authorization" section at the bottom of the user modal. It renders existing tenant assignments as rows with remove buttons, and provides an "+ Add Tenant / Role" button that expands a row with tenant dropdown + role multi-select. Tenants and roles are loaded from the existing API endpoints.

**Rationale:** Matches the Descope console layout. Avoids a separate modal-within-modal. Tenants and roles are fetched on modal open and cached for the session.

### 6. Custom Attribute Inputs by Type
**Decision:** Render attribute value inputs based on the attribute's defined type:
- `boolean` → checkbox
- `text` → text input
- `number` → number input with validation
- `datetime` → datetime-local input

**Rationale:** User requested type-appropriate inputs with validation. This keeps the form clean and prevents invalid data.

## Risks / Trade-offs

- **Large modal complexity** → The user edit modal will be a large component with many sections. Mitigation: decompose into sub-components (BasicFields, CustomAttributes, Authorization).
- **Diff computation edge cases** → Comparing nested objects (userTenants, customAttributes) requires deep diff. Mitigation: use JSON.stringify for nested equality checks, which is sufficient for this use case.
- **Custom attribute schema changes while editing** → If someone adds a new custom attribute while a user modal is open, the modal won't show it until re-opened. Mitigation: acceptable for a local dev tool.
