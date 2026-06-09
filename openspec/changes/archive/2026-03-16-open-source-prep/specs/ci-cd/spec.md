## ADDED Requirements

### Requirement: CI pipeline validates every pull request

A GitHub Actions workflow (`ci.yml`) SHALL run on every pull request and push to `main`. It SHALL execute in order: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --lib`, `cargo llvm-cov --lib --fail-under-coverage 95`, and the Vitest integration suite (`make test-integration`). The workflow SHALL fail if any step fails.

#### Scenario: PR with failing tests is blocked

- **WHEN** a pull request is opened with a failing unit test
- **THEN** the `ci.yml` check SHALL fail and the PR SHALL be marked as not mergeable

#### Scenario: PR below coverage threshold is blocked

- **WHEN** a pull request reduces line coverage below 95%
- **THEN** the `cargo llvm-cov --fail-under-coverage 95` step SHALL exit non-zero and block the PR

#### Scenario: PR with clippy warnings is blocked

- **WHEN** a pull request introduces a clippy warning
- **THEN** the `cargo clippy -- -D warnings` step SHALL exit non-zero and block the PR

### Requirement: Binary releases are created automatically on version tags

A GitHub Actions workflow (`release.yml`) SHALL trigger on pushes of tags matching `v*.*.*`. It SHALL cross-compile the `descope-emulator` binary for the following targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-gnu`. Each binary SHALL be packaged as a `.tar.gz` (macOS/Linux) or `.zip` (Windows) and attached to the GitHub Release.

#### Scenario: Tag push triggers release

- **WHEN** a tag `v0.1.0` is pushed to the repository
- **THEN** the `release.yml` workflow SHALL run, produce five platform archives, and create a GitHub Release with all archives attached

#### Scenario: Release archive contains a runnable binary

- **WHEN** the macOS arm64 release archive is downloaded and extracted
- **THEN** the extracted `descope-emulator` binary SHALL be executable and respond to `./descope-emulator --help`
