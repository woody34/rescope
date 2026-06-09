## Context

Rescope currently has no documentation beyond the README. Users need a dedicated site for API reference, guides, and interactive exploration. The existing admin UI uses a dark/hacker design system with CSS custom properties.

This change depends on the `wasm-browser-playground` change for the interactive playground embed, but the docs app scaffold can be built independently.

## Goals / Non-Goals

**Goals:**
- Astro + Starlight docs site at `apps/docs/` with MDX content pages
- Rescope dark aesthetic applied via Starlight's custom CSS
- Static build output deployable to GitHub Pages or any static host
- Playground page that embeds the WASM emulator when available

**Non-Goals:**
- Blog or changelog features — out of scope for v1
- Search (Algolia/Pagefind) — Starlight includes Pagefind by default, so this is "free"
- i18n/localization
- Auto-generated API reference from OpenAPI spec — future enhancement

## Decisions

### 1. Astro + Starlight

**Choice:** Use [Starlight](https://starlight.astro.build/) — Astro's purpose-built docs framework.

**Alternatives considered:**
- React Router 7 framework mode — Too heavy for a static docs site. Requires configuring SSR/SSG pipeline, MDX integration, and a layout system from scratch.
- Docusaurus — Opinionated, heavy, React 18 only, hard to customize aesthetics
- VitePress — Vue-based, doesn't share the React knowledge of the team
- Plain Vite + React SPA — Works but re-invents sidebar, navigation, search, SEO that Starlight provides out of the box

**Rationale:** Starlight gives us sidebar navigation, search (Pagefind), responsive layout, MDX support, and good SEO defaults — all out of the box. Custom CSS lets us apply the Rescope dark hacker aesthetic. For the playground page (which needs React), Astro's React integration lets us embed React islands in specific pages.

### 2. React island for playground page

**Choice:** The playground page uses Astro's `@astrojs/react` integration to render the WASM playground as a client-side React component. All other pages are pure Starlight/MDX.

**Rationale:** Only the playground needs React (for the admin UI embed). The rest of the docs are content pages that benefit from Starlight's zero-JS defaults. Astro's island architecture is designed for exactly this pattern.

### 3. Keep in monorepo (apps/docs/)

**Choice:** Place the docs site in `apps/docs/` within the existing Nx monorepo.

**Rationale:** Docs live next to the code they document. API changes and doc updates can be in the same PR. Can always extract to a separate repo later if maintenance friction grows.

## Risks / Trade-offs

- **Starlight customization limits** — Deep aesthetic customization may hit Starlight's CSS boundaries. → Mitigation: Starlight supports full CSS overrides and custom components. The dark theme is well within its capabilities.

- **Playground depends on WASM change** — The playground page is a placeholder until WASM is ready. → Mitigation: Build with a "coming soon" fallback state.

- **Content maintenance** — Docs need to stay in sync with API changes. → Mitigation: Same monorepo, same PR.
