## ADDED Requirements

### Requirement: Cross-platform release build script
A build script SHALL produce self-contained release binaries for all supported platform/architecture combinations.

#### Scenario: Build for all 6 targets
- **WHEN** the release build script is invoked without arguments
- **THEN** it SHALL attempt to produce binaries for: `linux-amd64`, `linux-arm64`, `macos-amd64`, `macos-arm64`, `windows-amd64`, `windows-arm64`

#### Scenario: UI is built before Rust compilation
- **WHEN** the build script runs
- **THEN** it SHALL first run the React UI production build (`npm run build` in `apps/ui/`) before compiling the Rust binary with `--features embed-ui`

#### Scenario: Output binaries use consistent naming
- **WHEN** the build script produces a binary
- **THEN** the binary SHALL be named `rescope-{os}-{arch}` (e.g., `rescope-linux-amd64`), with `.exe` appended for Windows targets

#### Scenario: Graceful handling of unavailable targets
- **WHEN** a cross-compilation toolchain is not available for a target (e.g., no Docker for `cross`)
- **THEN** the build script SHALL skip that target with a warning and continue building remaining targets

#### Scenario: Output directory
- **WHEN** the build script completes
- **THEN** all produced binaries SHALL be placed in a `dist/` directory at the repo root
