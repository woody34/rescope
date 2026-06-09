## Context

The docs site (`apps/docs/`) is an Astro + Starlight static site. The `apps/rescope-wasm` crate already exists and produces a WASM module via wasm-pack. The `apps/playground` app demonstrates a working pattern for WASM + service worker interception. The goal is to wire that pattern into a purpose-built demo page in the docs site.

## Goals / Non-Goals

**Goals:**
- A `/demo` route in `apps/docs/` with a working WASM auth sandbox
- Profile-driven seed loading (select a scenario → emulator resets + seeds)
- Live file tree that polls `GET /emulator/snapshot` and updates as state changes
- Inline login form that makes real API calls to the WASM emulator
- Build pipeline wiring: `wasm-pack build apps/rescope-wasm → apps/docs/public/wasm/`

**Non-Goals:**
- Embedding the Rescope Admin UI iframe (no `apps/ui/dist` dependency in docs)
- Persisting state across browser sessions
- Server-side rendering of the demo (client-side only, `client:only="react"`)

## Decisions

### 1. Service Worker scoped to `/demo-wasm/`

**Choice:** The service worker is registered with `scope: '/demo-wasm/'`. All WASM API calls are made to `/demo-wasm/v1/...` paths. The SW intercepts these and routes them to `wasm_bindgen.handle_request()`.

**Rationale:** Scoping prevents the SW from intercepting Starlight's own navigation or pagefind search requests. Isolates the demo completely from the docs site's own network activity.

**Alternative considered:** Scoping to `/demo/` — rejected because it would intercept the full page navigation requests for the demo page itself.

### 2. Profile system as TypeScript config objects

**Choice:** Each profile is a plain TS object conforming to a `DemoProfile` interface, co-located in `src/components/Demo/profiles/`. The profile defines: seed payload, login UI type, prefill values, and description.

**Rationale:** Config-as-code is extensible (add a new file = new profile), type-safe, tree-shakeable at build time. Avoids a separate JSON config layer.

### 3. Live tree via polling (not WebSocket/SSE)

**Choice:** The ResourceTree component polls `GET /demo-wasm/emulator/snapshot` every 500ms while the demo is mounted.

**Rationale:** WASM runs in a service worker — there's no persistent connection to push. Polling at 500ms is imperceptible latency for a demo and avoids SSE/WS complexity. Stops polling on component unmount.

### 4. WASM assets as static files, not bundled

**Choice:** WASM pkg files sit in `apps/docs/public/wasm/` — served verbatim as static assets, not bundled by Vite/Astro.

**Rationale:** `.wasm` files must be served with `application/wasm` MIME type. Putting them in `public/` guarantees the static server serves them correctly. The JS glue code (`rescope_wasm.js`) is loaded via `importScripts()` in the service worker, not as an ES module.

### 5. Layout: two-panel (tree + form), profile bar at top

**Choice:** Profile picker as a horizontal tab bar, then a two-column layout: left = ResourceTree (30%), right = LoginForm + response panel (70%).

**Rationale:** Matches the mental model from the explore session. Keeps the layout simple without a third Admin UI panel.

## Risks / Trade-offs

- **WASM binary size** — The WASM module may be large (500KB–2MB). The demo page should lazy-load it with a progress indicator. → Use `client:only="react"` with a loading state; the WASM is only fetched when the user visits `/demo`.

- **Browser support** — Service workers require HTTPS (or localhost). The static docs deployment needs HTTPS. → GitHub Pages, Netlify, Vercel all serve HTTPS. No mitigation needed for production; localhost works for dev.

- **WASM build in CI** — `wasm-pack` must be installed in CI. → Add `cargo install wasm-pack` step; alternatively, check in the built WASM pkg if we want to avoid build complexity in CI.

- **SW update race** — If the user visited `/demo` with an old SW, a new deploy won't take effect until they close+reopen the tab. → Use `skipWaiting()` + `clients.claim()` in the SW; acceptable for a docs demo.
