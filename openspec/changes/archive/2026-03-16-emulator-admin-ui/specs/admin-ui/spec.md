## ADDED Requirements

### Requirement: Admin UI is served from the Rust server at /ui
The Rust server SHALL serve the compiled Vite React app under `/ui`. The React app SHALL use React Router for client-side navigation. All routes in the app SHALL mirror the Descope console URL structure (e.g. `/ui/auth-methods/otp`, `/ui/authorization`, `/ui/users`, `/ui/tenants/:id/settings`). Server-side routing SHALL return the `index.html` for any unmatched `/ui/*` path to support SPA navigation.

#### Scenario: Any /ui/* route returns the app shell
- **WHEN** a browser navigates directly to `/ui/users`
- **THEN** the server responds with the React app `index.html`
- **THEN** React Router renders the Users section

### Requirement: Sidebar navigation matches Descope console section grouping
The admin UI SHALL have a persistent left sidebar with sections: **Build** (Authentication Methods, Authorization, Connectors), **Manage** (Users, Access Keys, Tenants), **Settings** (Project, Session Management, JWT Templates), and **Emulator** (Snapshot, OTP Inspector, Reset). Active route SHALL be highlighted.

#### Scenario: Navigating to a section highlights it in the sidebar
- **WHEN** user navigates to `/ui/users`
- **THEN** "Users" is highlighted as active in the sidebar

### Requirement: Each admin UI section reads from and writes to the emulator API
Each section's forms SHALL call the corresponding management API endpoints on save. Changes SHALL be reflected immediately (optimistic update or re-fetch). Error responses from the API SHALL surface as inline form error messages.

#### Scenario: Saving auth method config persists via API
- **WHEN** user changes OTP expiry to 300 seconds in the UI and clicks Save
- **THEN** the UI calls `PUT /v1/mgmt/config/auth-methods` with the updated config
- **THEN** the UI shows a success toast
- **THEN** re-opening the page shows 300 seconds

### Requirement: Admin UI has a fully accessible, Radix UI-based component system
All interactive elements (dropdowns, dialogs, checkboxes, toggles, tabs) SHALL use Radix UI headless primitives styled with vanilla CSS. UI SHALL meet WCAG 2.1 AA for keyboard navigation and screen reader support.

#### Scenario: Dialogs are accessible via keyboard
- **WHEN** user opens the "Add Role" dialog
- **THEN** focus is trapped within the dialog
- **THEN** pressing Escape closes the dialog

### Requirement: Playwright E2E smoke test for snapshot round-trip exists
The `ui/e2e/` directory SHALL contain a Playwright test suite with Page Object Models. At minimum, one test SHALL cover: (1) export snapshot, (2) reset emulator, (3) import snapshot, (4) verify state is restored.

#### Scenario: Snapshot round-trip smoke test passes
- **WHEN** the emulator is running with seed data (at least one user and tenant)
- **WHEN** the Playwright test runs `npm run test:e2e`
- **THEN** all assertions pass: exported snapshot contains the user, reset clears users, imported snapshot restores the user
