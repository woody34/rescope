## ADDED Requirements

### Requirement: Repository has a valid open source license

The repository SHALL contain a `LICENSE` file at the root with a complete Apache 2.0 license text, and a `NOTICE` file that states the project name, copyright holder, and an explicit disclaimer that the project is not affiliated with or endorsed by Descope, Inc.

#### Scenario: LICENSE file is present and valid

- **WHEN** the repository root is inspected
- **THEN** a `LICENSE` file SHALL exist containing the full Apache 2.0 license text

#### Scenario: NOTICE file includes disaffiliation disclaimer

- **WHEN** the NOTICE file is read
- **THEN** it SHALL contain the phrase "not affiliated with or endorsed by Descope"

### Requirement: Repository has contributor documentation

The repository SHALL contain `CONTRIBUTING.md` (setup, TDD methodology, PR workflow), `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1), and `SECURITY.md` (vulnerability reporting instructions).

#### Scenario: CONTRIBUTING.md describes the TDD workflow

- **WHEN** a new contributor reads CONTRIBUTING.md
- **THEN** they SHALL find instructions for running unit tests (`cargo test --lib`), running integration tests (`make test-integration`), and the required test-first development cycle

#### Scenario: SECURITY.md provides a reporting path

- **WHEN** a security researcher reads SECURITY.md
- **THEN** they SHALL find instructions for private disclosure (e.g., GitHub private vulnerability reporting or email)

### Requirement: Repository has a structured changelog

The repository SHALL contain a `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com) format, starting with a `[0.1.0]` entry documenting the initial release capabilities.

#### Scenario: CHANGELOG starts at v0.1.0

- **WHEN** CHANGELOG.md is read
- **THEN** an `[0.1.0]` section SHALL exist listing the initial feature set

### Requirement: Repository has GitHub issue and PR templates

The `.github/` directory SHALL contain `ISSUE_TEMPLATE/bug_report.md`, `ISSUE_TEMPLATE/feature_request.md`, and `pull_request_template.md` with checklists appropriate for TDD-based contributions.

#### Scenario: PR template includes test checklist

- **WHEN** a contributor opens a PR
- **THEN** the PR description SHALL be pre-populated with a checklist including test coverage and documentation updates

### Requirement: README targets an external audience

The `README.md` SHALL be rewritten to serve developers unfamiliar with the project. It SHALL include: quick-start instructions for downloading a pre-built binary by OS, SDK integration examples for Node.js (TypeScript), a table of emulator behavioral deviations from real Descope, and the OTP code retrieval pattern.

#### Scenario: User can get started from the README without Rust

- **WHEN** a developer reads the README without a Rust toolchain installed
- **THEN** they SHALL find a binary download link and a working quick-start example using curl or a Descope SDK
