## Context

Rescope is a Rust (axum + tokio) API server with a React (Vite) admin UI. Currently, the server serves the UI's `dist/` folder from the filesystem via `tower-http`'s `ServeDir` (line ~577 of `server.rs`). This couples the binary to a specific working directory and makes distribution a multi-file affair.

The API has ~60 route handlers and no native dependencies — all crates are pure Rust. The UI builds to a standard Vite `dist/` output (~500KB gzipped).

## Goals / Non-Goals

**Goals:**
- Single binary ships with UI embedded — `./rescope` and it just works
- Embedded UI is the **default** — no feature flag needed for production builds
- Cross-platform release builds for 6 targets (linux/macos/windows × amd64/arm64)
- Dev workflow option — `cargo run --features dev-ui` uses filesystem serving for hot-reload
- GitHub Actions release workflow producing binaries for GitHub Releases
- Build script is simple and CI-friendly

**Non-Goals:**
- Docker image builds — separate concern
- Auto-update mechanism
- Code-signing for macOS/Windows
- Homebrew tap or `cargo install` support (yet)

## Decisions

### 1. `rust-embed` with embedded-by-default, dev filesystem opt-in

**Choice:** Use [`rust-embed`](https://crates.io/crates/rust-embed) for embedding. UI is embedded by **default**. A `dev-ui` feature flag switches to filesystem serving for local development.

**Alternatives considered:**
- Feature flag `embed-ui` for production opt-in — Wrong default. Every release build, CI job, and source-builder would need to remember `--features embed-ui`. Embedding by default means the simple path produces the right artifact.
- `include_dir` — Less mature, no MIME detection
- Manual `include_bytes!` — Too much boilerplate

**Rationale:** The common case (run the binary) should be zero-config. Only developers actively working on the UI need the filesystem fallback, and they know to pass `--features dev-ui`.

### 2. `rust-embed` debug-mode passthrough

`rust-embed` has a built-in debug-mode feature: in debug builds, it reads from disk instead of embedding. This means `cargo run` (debug profile) automatically uses the filesystem — no feature flag needed for local dev at all. The `dev-ui` feature is a belt-and-suspenders option for release builds that still want filesystem serving.

### 3. Cross-compilation via `cross` + GitHub Actions

**Choice:** A `scripts/release-build.sh` script for local builds, plus a GitHub Actions workflow for automated releases.

**Approach:**
- `cross` for Linux cross-compilation (Docker-based)
- GitHub Actions matrix for macOS and Windows native builds
- Triggered on version tags (`v*`)
- Uploads binaries as GitHub Release assets

**Rationale:** Hybrid approach. Local script for testing, CI for production releases. macOS and Windows targets need native runners anyway.

### 4. Binary naming convention

Output: `rescope-{os}-{arch}[.exe]`

Examples: `rescope-linux-amd64`, `rescope-darwin-arm64`, `rescope-windows-amd64.exe`

## Risks / Trade-offs

- **Binary size increase** — Embedding UI assets adds ~2-3MB to the binary. Acceptable for a dev tool.
  → Mitigation: `rust-embed` supports compression. Can enable later if needed.

- **UI build must precede Rust build** — The embedded path must exist at compile time.
  → Mitigation: Build script runs `npm run build` first. Debug mode reads from disk so this only matters for release builds.

- **Cross-compilation toolchain setup** — `cross` requires Docker. macOS/Windows ARM builds need native runners.
  → Mitigation: GitHub Actions handles this. Local script gracefully skips unavailable targets.
