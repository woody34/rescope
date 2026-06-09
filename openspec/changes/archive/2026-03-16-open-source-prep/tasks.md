## 1. Legal & Community Scaffolding

- [ ] 1.1 Create `LICENSE` — Apache 2.0 full text
- [ ] 1.2 Create `NOTICE` — project name, copyright, explicit Descope disaffiliation disclaimer
- [ ] 1.3 Create `CONTRIBUTING.md` — local setup, TDD cycle (spec → failing test → implement → passing), `make` command reference, PR requirements
- [ ] 1.4 Create `CODE_OF_CONDUCT.md` — Contributor Covenant 2.1
- [ ] 1.5 Create `SECURITY.md` — GitHub private vulnerability reporting instructions
- [ ] 1.6 Create `CHANGELOG.md` — Keep a Changelog format, `[0.1.0]` entry with full initial feature set

## 2. GitHub Community Files

- [ ] 2.1 Create `.github/ISSUE_TEMPLATE/bug_report.md` — steps to reproduce, expected vs actual behavior, emulator version, OS
- [ ] 2.2 Create `.github/ISSUE_TEMPLATE/feature_request.md` — problem statement, proposed solution, Descope API reference link
- [ ] 2.3 Create `.github/pull_request_template.md` — checklist: tests added, coverage maintained, README updated if behavior changed

## 3. CI — PR Gate

- [ ] 3.1 Create `.github/workflows/ci.yml` — triggered on PR and push to `main`
- [ ] 3.2 Add `cargo fmt --check` step to ci.yml
- [ ] 3.3 Add `cargo clippy -- -D warnings` step to ci.yml
- [ ] 3.4 Add `cargo test --lib` step to ci.yml
- [ ] 3.5 Add `cargo llvm-cov --lib --fail-under-coverage 95` step to ci.yml (requires `cargo-llvm-cov` install)
- [ ] 3.6 Add Vitest integration test step to ci.yml (`make test-integration`) — build Rust binary first, then run Vitest
- [ ] 3.7 Verify ci.yml runs cleanly on current `main` branch state

## 4. CI — Binary Release

- [ ] 4.1 Create `.github/workflows/release.yml` — triggered on `v*.*.*` tags
- [ ] 4.2 Add cross-compilation matrix: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-gnu`
- [ ] 4.3 Use `cross` tool for Linux and Windows targets; native runner for macOS (GitHub macOS runner covers both arm64 and x86_64)
- [ ] 4.4 Package macOS/Linux as `.tar.gz` named `descope-emulator-{target}.tar.gz`
- [ ] 4.5 Package Windows as `.zip` named `descope-emulator-{target}.zip`
- [ ] 4.6 Attach all archives to the GitHub Release using `softprops/action-gh-release`

## 5. Bug Fix — magicLink.update.email

- [ ] 5.1 Write a failing test in `src/routes/auth/magic_link.rs` that calls `magicLink.update.email` and then `GET /v1/auth/me` and asserts the returned email matches the updated value
- [ ] 5.2 Fix `magic_link_update_email` handler in `src/routes/auth/magic_link.rs` to call `state.users.write().await.patch(loginId, {email, verifiedEmail: false})`
- [ ] 5.3 Confirm all unit tests pass at 95%+ coverage (`make coverage`)
- [ ] 5.4 Add integration test to `integration/sdk-js/tests/magic-link.sdk.test.ts` covering the update+me round-trip

## 6. README Rewrite

- [ ] 6.1 Rewrite `README.md` for external audience — remove internal references, add binary download quick-start section by OS
- [ ] 6.2 Add Node.js SDK integration example with `baseUrl` pointing to emulator
- [ ] 6.3 Add "Emulator Deviations" table documenting all intentional differences from real Descope behavior (password reset token in body, magic link token in body, OTP code in body, no email/SMS delivery)
- [ ] 6.4 Add "OTP Code Retrieval" section explaining the `GET /emulator/otp/:loginId` escape hatch (forward-reference for `emulator-completeness` change — mark as "coming in next release" if not yet implemented)
- [ ] 6.5 Review README end-to-end for external reader experience
