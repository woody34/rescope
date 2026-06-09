## Why

Rescope currently requires running `cargo build` + distributing a binary alongside the `apps/ui/dist/` folder. Users must run the binary from the repo root so the `ServeDir` path resolves correctly. This makes distribution fragile and prevents "download one file and run it" workflows. A single self-contained binary with the UI embedded at compile time fixes both.

Additionally, there is no cross-platform release pipeline — building for Windows, Linux, and macOS (amd64/arm64) requires manual setup on each platform.

## What Changes

- Embed the React UI `dist/` output into the Rust binary at compile time using `rust-embed`
- Replace the `ServeDir`/`ServeFile` static file serving in `server.rs` with an in-memory handler
- Add a cross-platform build script that produces release binaries for 6 targets
- Add a Cargo feature flag `embed-ui` so dev mode can still use the filesystem for hot-reload

## Capabilities

### New Capabilities
- `embedded-ui-serving`: Embed and serve the React UI from within the compiled binary using `rust-embed`, replacing filesystem-based `ServeDir`
- `cross-platform-release`: Build script producing release binaries for linux-amd64, linux-arm64, macos-amd64, macos-arm64, windows-amd64, windows-arm64

### Modified Capabilities

_(none — no existing spec-level behavior changes)_

## Impact

- **`apps/api/Cargo.toml`** — new `rust-embed` dependency, `embed-ui` feature flag
- **`apps/api/src/server.rs`** — replace `ServeDir`/`ServeFile` with embedded asset handler
- **`scripts/`** — new cross-platform build script
- **CI/CD** — `.github/workflows/` will need a release workflow (out of scope for this change but noted)
- **Dev workflow** — unchanged; dev mode continues using Vite dev server + `cargo run`
