# rescope-playground

Minimal static playground for testing the rescope-wasm build in the browser.

## Commands

| Task | Command (from repo root) |
|------|---------|
| Dev | `cd apps/playground && npm run dev` (serves on port 4501) |
| Build WASM | `cd apps/playground && npm run build:wasm` (requires wasm-pack) |

## Conventions

- Static HTML served via `npx serve`
- WASM artifacts copied from `apps/rescope-wasm` build into `public/wasm/`
- No build step for the playground itself; just serves static files
