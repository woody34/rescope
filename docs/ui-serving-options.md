# Serving a React Router UI via the Rust API

## Context

The `descope-emulator` API is built on **Axum** (Rust). This document captures a discovery discussion on whether and how to serve a React Router application on an `/ui` endpoint directly from that API.

---

## Options Considered

### Option 1: Axum as a Reverse Proxy

Run the React Router SSR server (e.g. `node server.js`) as a separate process. Axum proxies all `/ui/*` requests to it.

**Deps:** `hyper`, `hyper-util`

```rust
.route("/ui", get(proxy_ui))
.route("/ui/*path", get(proxy_ui))
```

| Pro                     | Con                     |
| ----------------------- | ----------------------- |
| Full SSR support        | Two processes to manage |
| Hot reload works in dev | Not a single binary     |

---

### Option 2: Static File Serving via `ServeDir` ✅ Selected

Build the React Router app as a static bundle and serve it using `tower-http`'s `ServeDir`. React Router handles client-side navigation (SPA mode, not true SSR).

**Deps:** `tower-http` with `fs` feature (already present)

```toml
tower-http = { version = "0.5", features = ["cors", "fs"] }
```

```rust
use tower_http::services::{ServeDir, ServeFile};

Router::new()
    // ... existing API routes ...
    .nest_service("/ui", ServeDir::new("ui/dist").fallback(ServeFile::new("ui/dist/index.html")))
```

The `.fallback(ServeFile::new("index.html"))` is critical — it ensures React Router's client-side routing works correctly for deep links (e.g. `/ui/some/nested/path`).

| Pro                     | Con                      |
| ----------------------- | ------------------------ |
| Single binary / process | No true SSR              |
| No Node runtime in prod | Requires a build step    |
| Simple — no proxy logic | Deep links need fallback |

---

### Option 3: Embed JS Runtime (Deno Core / WASM)

Embed a JS runtime directly in Rust. Complex, heavy dependency, overkill for this use case. **Not recommended.**

---

## Decision

**Option 2** was selected. The emulator UI is an internal developer tool where true SSR is not required. Serving the pre-built static bundle keeps the deployment simple (single Rust binary) and eliminates any Node.js runtime dependency in production.

## Implementation Notes

- React Router app lives in `ui/` at the project root
- Build output goes to `ui/dist/`
- The Rust binary serves `/ui` → `ui/dist/index.html` (SPA shell)
- `ServeFile` fallback handles all client-side routes under `/ui/*`
- The `fs` feature must be enabled on `tower-http`
