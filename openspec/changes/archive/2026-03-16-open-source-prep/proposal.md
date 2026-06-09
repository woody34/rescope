## Why

The `descope-emulator` is ready to be shared as a personal open source project. Without a license, CI, or contributor scaffolding, the project cannot be legally used by others and will be difficult to maintain or accept contributions for. This change establishes everything needed to publish the repo, and also fixes two high-priority behavioral bugs discovered during the gap analysis.

## What Changes

- **New**: `LICENSE` — Apache 2.0 (patent grant; most permissive while protecting against IP claims given the emulator reimplements a commercial API surface)
- **New**: `NOTICE` — Required Apache 2.0 attribution file; includes disclaimer of Descope affiliation
- **New**: `CONTRIBUTING.md` — Setup instructions, TDD methodology, PR guidelines
- **New**: `CODE_OF_CONDUCT.md` — Contributor Covenant 2.1
- **New**: `SECURITY.md` — Vulnerability reporting policy
- **New**: `CHANGELOG.md` — Starts the change log at v0.1.0
- **New**: `.github/workflows/ci.yml` — PR gate: `cargo test --lib`, `cargo clippy`, `cargo fmt --check`, `cargo llvm-cov --fail-under-coverage 95`, Vitest integration tests
- **New**: `.github/workflows/release.yml` — On tag push: cross-compile binaries for macOS arm64, macOS x86_64, Linux x86_64, Linux aarch64, Windows x86_64; create GitHub Release with attached artifacts
- **New**: `.github/ISSUE_TEMPLATE/bug_report.md` — Structured bug report template
- **New**: `.github/ISSUE_TEMPLATE/feature_request.md` — Feature request template
- **New**: `.github/pull_request_template.md` — PR checklist (tests, coverage, docs)
- **Modified**: `README.md` — Rewrite for external audience: quick start by OS (binary download), SDK integration examples (Node.js, Python, Go, curl), emulator deviation table, OTP code retrieval pattern
- **Fixed**: `POST /v1/auth/magiclink/update/email` — Now persists the new email on the user record
- **Fixed**: `GET /v1/auth/me` — Documented and verified correct JWT source behavior

## Capabilities

### New Capabilities

- `oss-scaffolding`: License, notice, contributing guide, code of conduct, security policy, changelog, and issue/PR templates — the non-code artifacts required for a trustworthy open source repository
- `ci-cd`: GitHub Actions workflows for PR validation and binary release builds (macOS arm64/x86_64, Linux x86_64/aarch64, Windows x86_64)

### Modified Capabilities

- `auth-flows`: `magicLink.update.email` must persist the new email to the user record (currently a known no-op)

## Impact

- **No Rust API changes** — all fixes are behavioral, no endpoint signatures change
- **New files**: `LICENSE`, `NOTICE`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CHANGELOG.md`, `.github/**`
- **Modified files**: `README.md`, `src/routes/auth/magic_link.rs`
- **CI dependency**: `cross` crate for cross-compilation in release workflow
- **Consumers**: Binary download URL pattern will be `github.com/<user>/descope-emulator/releases/latest`
