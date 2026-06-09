## ADDED Requirements

### Requirement: Startup banner displays configuration summary
The emulator SHALL print a configuration summary to stdout on startup, before accepting requests.

#### Scenario: Default configuration startup
- **WHEN** the emulator starts with no environment variable overrides
- **THEN** stdout SHALL contain the port number (`4600`), project ID (`emulator-project`), and a line indicating the emulator is ready to accept requests

#### Scenario: Custom configuration startup
- **WHEN** the emulator starts with `DESCOPE_EMULATOR_PORT=3001` and `DESCOPE_PROJECT_ID=my-project`
- **THEN** stdout SHALL display port `3001` and project ID `my-project` in the startup banner

#### Scenario: Seed file loaded
- **WHEN** the emulator starts with a valid `DESCOPE_EMULATOR_SEED_FILE` pointing to a file with 3 users and 2 tenants
- **THEN** stdout SHALL indicate the seed file was loaded and display the count of seeded users and tenants

### Requirement: Startup warns on invalid configuration
The emulator SHALL warn on startup if the configured seed file does not exist.

#### Scenario: Missing seed file warning
- **WHEN** the emulator starts with `DESCOPE_EMULATOR_SEED_FILE=/nonexistent/seed.json`
- **THEN** the emulator SHALL print a warning to stderr and continue starting (not crash), OR exit with a clear error message indicating the seed file was not found

### Requirement: README env var names match code
The README SHALL document the same environment variable names that `config.rs` reads.

#### Scenario: All documented env vars are accurate
- **WHEN** a user reads the README's Environment Variables table
- **THEN** every variable name listed SHALL match the corresponding `env::var("...")` call in `config.rs`
