# apps-docs

Astro/Starlight documentation site for Rescope. Includes an interactive WASM-powered demo.

## Commands

| Task | Command (from repo root) |
|------|---------|
| Dev | `npm run docs:dev` (or `npx nx run docs:dev`) |
| Build | `npx nx run docs:build` |
| Build WASM demo | `npm run docs:build:wasm` (requires wasm-pack) |
| Preview | `npx nx run docs:preview` |

## Conventions

- Built with Astro 6 + Starlight theme
- React components used for interactive elements via @astrojs/react
- WASM demo: builds rescope-wasm and copies to `public/wasm/`, served via service worker (`public/demo-sw.js`)
- Dev server runs on port 4321
