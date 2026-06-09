## 1. Embedded UI Serving

- [x] 1.1 Add `rust-embed` dependency to `apps/api/Cargo.toml`; add `dev-ui` feature flag that disables embedding and falls back to filesystem
- [x] 1.2 Create an embedded asset handler module (`apps/api/src/embedded_ui.rs`) that serves files from the `rust-embed` struct with correct MIME types and SPA fallback
- [x] 1.3 Update `server.rs` `build_router()` to use the embedded handler by default; conditionally use `ServeDir` when `dev-ui` feature is active
- [x] 1.4 Verify: `cargo build --release` compiles with embedded assets; `cargo run` (debug) uses filesystem passthrough

## 2. Cross-Platform Release Script

- [x] 2.1 Create `scripts/release-build.sh` that builds the UI and then compiles release binaries for all 6 targets
- [x] 2.2 Add Rust target definitions and `cross` invocations for Linux targets; native `cargo build` for host platform
- [x] 2.3 Add consistent binary naming (`rescope-{os}-{arch}[.exe]`) and output to `dist/` directory
- [x] 2.4 Add graceful skip logic for unavailable cross-compilation toolchains

## 3. GitHub Actions Release Workflow

- [x] 3.1 Create `.github/workflows/release.yml` triggered on `v*` tags
- [x] 3.2 Add build matrix: linux-amd64, linux-arm64 (via `cross`), macos-amd64, macos-arm64, windows-amd64, windows-arm64
- [x] 3.3 Upload binaries as GitHub Release assets with checksums

## 4. Verification

- [x] 4.1 Build with `cargo build --release` and run the binary from a temp directory (not repo root) — confirm UI loads at `http://localhost:<port>/`
- [x] 4.2 Confirm SPA fallback: navigate to `/identity-providers` directly — should serve `index.html`
- [x] 4.3 Confirm API routes still work: `curl http://localhost:<port>/health`
- [x] 4.4 Confirm dev mode: `cargo run` (debug build) still serves from filesystem
