## 1. App Scaffold

- [x] 1.1 Initialize Astro + Starlight app at `apps/docs/` using `npx create-astro`; add `@astrojs/starlight` and `@astrojs/react` integrations
- [x] 1.2 Add to Nx workspace with `project.json`
- [x] 1.3 Apply Rescope dark aesthetic via Starlight custom CSS overrides (dark background, neon green accent, monospace headings)
- [x] 1.4 Configure sidebar navigation: Getting Started, Guides, API Reference, Playground

## 2. Documentation Content

- [x] 2.1 Create Getting Started page (installation, config, basic usage)
- [x] 2.2 Create at least one Guide page (e.g., OTP Authentication flow with code examples)
- [x] 2.3 Create API Reference overview page (link to `/docs` Swagger UI on running instance)

## 3. Playground Embed

- [x] 3.1 Create `/playground` page using Astro's React island integration for the WASM playground component
- [x] 3.2 Implement loading indicator during WASM initialization
- [x] 3.3 Implement graceful fallback when WASM is not available
- [x] 3.4 Scope service worker to `/playground` path only

## 4. Verification

- [x] 4.1 `npm run build` produces static output; serve with `npx serve dist/` and verify all pages render
- [x] 4.2 Verify responsive layout: sidebar visible on desktop, collapsible on mobile
- [x] 4.3 Verify MDX content renders with syntax highlighting and proper heading hierarchy
- [x] 4.4 Verify playground page shows loading/fallback state (live WASM integration verified after that change lands)
