## Why

Rescope has no documentation site. The README covers installation and configuration, but there's no interactive guide, API reference, or "try it now" experience. With the WASM playground (proposed separately), we can build a docs site that lets users interact with a live emulator while reading the docs — zero install, instant feedback.

## What Changes

- Create a new Astro + Starlight app at `apps/docs/` as the Rescope documentation site
- Pages: Getting Started, API Reference, Guides (OTP, SSO, management API), and an embedded Playground page
- The Playground page loads the WASM module and service worker, rendering the admin UI inline
- Static deployment target (GitHub Pages, Vercel, or Netlify)

## Capabilities

### New Capabilities
- `docs-app-scaffold`: Astro/Starlight documentation site with MDX content pages, sidebar navigation, and the Rescope dark aesthetic
- `docs-playground-embed`: Dedicated playground page that loads the WASM emulator inline, allowing users to interact with a live Rescope instance while reading the docs

### Modified Capabilities

_(none)_

## Impact

- **New: `apps/docs/`** — Astro + Starlight app with MDX support
- **Workspace** — new `package.json` entry for docs app; Nx project config
- **Depends on** — `wasm-browser-playground` change (for the playground embed page); can scaffold the docs site independently and add the playground page later
- **Deployment** — static site build output, deployable to any static host
