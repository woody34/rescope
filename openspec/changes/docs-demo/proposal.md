## Why

The Rescope docs site has no interactive demo. Users must install Rescope locally to experience it — a significant barrier. A WASM-powered demo page lets anyone try a real auth flow in their browser, zero install, with curated scenarios showing Rescope's core value.

## What Changes

- Add a `/demo` page to `apps/docs/` with a WASM-backed interactive sandbox
- Profile selector lets users pick an auth scenario (password, OTP, magic link, multi-tenant)
- Selecting a profile initializes the WASM emulator and seeds it with the scenario's data
- A live file tree shows the seeded resources and updates in real-time after interactions
- A sample login form lets users trigger actual auth flows against the in-browser emulator
- A `build:wasm` step copies the WASM pkg into `apps/docs/public/wasm/` as part of the docs build

## Capabilities

### New Capabilities
- `demo-page`: The `/demo` route — profile selector, live resource tree, and inline login form powered by the WASM emulator
- `demo-profiles`: Profile system defining seed data + login UI per scenario (password, OTP, magic link, multi-tenant)
- `wasm-docs-integration`: WASM build pipeline wiring `apps/rescope-wasm` output into `apps/docs/public/wasm/`

### Modified Capabilities

_(none)_

## Impact

- **New: `apps/docs/src/components/Demo/`** — React island components for the demo page
- **New: `apps/docs/public/wasm/`** — WASM build artifacts (added to `.gitignore`, generated at build time)
- **New: `apps/docs/public/demo-sw.js`** — Service worker scoped to `/demo-wasm/` only
- **Modified: `apps/docs/package.json`** — adds `build:wasm` script
- **Modified: `package.json`** — adds `docs:build:wasm` workspace script
- **Depends on** — `apps/rescope-wasm` crate (already exists and builds)
