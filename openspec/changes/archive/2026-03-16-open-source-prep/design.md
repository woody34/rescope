## Context

The emulator is a complete, tested, working Rust binary implementing the Descope authentication API surface. It has 60 passing unit tests and a Vitest integration suite. The goal now is to make it safe and usable for others — which requires legal scaffolding (license), community scaffolding (contributing docs), CI/CD (PR gate and binary releases), a public-facing README, and fixing two behavioral bugs the gap analysis surfaced.

No architectural changes are needed. This is additive: new files, new workflows, and two small bug fixes in `src/routes/auth/magic_link.rs`.

## Goals / Non-Goals

**Goals:**

- Make the repo legally shareable under Apache 2.0
- Provide pre-built binaries for the four major platforms via GitHub Releases
- Fix `magicLink.update.email` so it actually persists the new email
- Establish a CI gate that enforces the existing TDD methodology for contributors
- Rewrite README for an external audience unfamiliar with the project's original context

**Non-Goals:**

- Docker image distribution (deferred to a later change)
- crates.io publish (deferred — needs stable public API designation)
- Website or landing page
- Expanding endpoint coverage (that is `emulator-completeness`)

## Decisions

**Apache 2.0 over MIT**
The emulator reimplements the public Descope REST API surface. Apache 2.0 includes an explicit patent grant that MIT lacks. Given the emulator exists in the same space as a commercial product, the patent grant provides a safer foundation for contributors and consumers. MIT would be acceptable but Apache 2.0 is the conservative and recommended choice for infrastructure tooling adjacent to commercial platforms.

**Pre-built binaries over crates.io**
The primary audience is developers who want to run the emulator in CI — they don't need to own a Rust toolchain. Pre-built binaries (`.tar.gz` for macOS/Linux, `.zip` for Windows) attached to GitHub Releases are the most accessible distribution path. Cross-compilation is handled in CI via the `cross` tool — no native runners for each platform needed.

**Targets: macOS arm64, macOS x86_64, Linux x86_64, Linux aarch64, Windows x86_64**
These cover the realistic install surfaces for a developer tool. Windows aarch64 is excluded (too niche, cross-compilation support immature).

**`magicLink.update.email` fix strategy**
The existing handler calls the auth-tier `update_email` path which doesn't mutate the user record. The fix is to call the management-tier `user_patch` function directly — the same function used by `PATCH /v1/mgmt/user/patch`. This is a one-line change in the handler but requires threading the `State` reference through to the user store write path, which is already the pattern used elsewhere.

## Risks / Trade-offs

- **Cross-compilation complexity** → Mitigated by using `cross` (Docker-based cross-compiler) with a GitHub Actions matrix; well-established pattern for Rust projects
- **Apache 2.0 + Descope ToS interaction** → The emulator implements a publicly documented API surface for interoperability/testing. This is the same legal basis as projects like `localstack` (AWS emulator) and `azurite` (Azure emulator). The NOTICE file includes an explicit disaffiliation disclaimer.
- **Coverage gate in CI** → The current gate is 95% (`--fail-under-coverage 95`). The `magicLink.update.email` fix must include a new test that covers the persisted-email path, otherwise coverage could drop.
