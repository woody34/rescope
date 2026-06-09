# Contributing to Rescope

Thank you for your interest in contributing! Rescope is an open source Descope API emulator for local development and testing.

## Getting Started

### Prerequisites

- **Rust** (latest stable) — `rustup update stable`
- **Node.js** 18+ — for integration tests and the admin UI
- **Nx** — installed globally or via `npx`

### Local Setup

```bash
# Clone the repo
git clone https://github.com/<your-username>/descope-emulator.git
cd descope-emulator

# Build the Rust API
cd apps/api && cargo build

# Install Node dependencies (for UI + tests)
npm install

# Run the emulator
cargo run --manifest-path apps/api/Cargo.toml

# Run all tests
cargo test --manifest-path apps/api/Cargo.toml --lib    # Rust unit tests
npx nx test integration-api                               # API integration tests
npx nx test ui                                                # Playwright E2E tests
```

## Development Workflow

We follow a **Test-Driven Development (TDD)** cycle:

1. **Spec** — understand what the Descope API endpoint does
2. **Failing test** — write a test that demonstrates the expected behavior
3. **Implement** — write the minimal code to make the test pass
4. **Refactor** — clean up while keeping tests green

### Project Structure

```
apps/
  api/          — Rust API server (Axum)
  ui/           — React admin UI (Vite)
  integration-api/  — Vitest integration tests
  sample-app/   — NestJS + Angular sample app with Playwright tests
```

## Pull Request Guidelines

Before submitting a PR, please ensure:

- [ ] All existing tests pass (`cargo test --lib` + `npx nx test integration-api`)
- [ ] New tests are added for any new functionality
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] README is updated if behavior changes are user-facing

### PR Checklist

1. Describe **what** changed and **why**
2. Link to any related issues
3. Include test output showing all tests pass
4. Keep PRs focused — one feature or fix per PR

## Code Style

- **Rust**: Follow standard Rust conventions. Run `cargo fmt` before committing.
- **TypeScript**: Use the project's ESLint configuration.
- **Commit messages**: Use conventional commits (`feat:`, `fix:`, `docs:`, `test:`, etc.)

## Reporting Issues

- Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md) for bugs
- Use the [feature request template](.github/ISSUE_TEMPLATE/feature_request.md) for new features

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.
