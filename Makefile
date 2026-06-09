.PHONY: test test-integration coverage lint build

# Build the debug binary (required before integration tests)
build:
	cargo build

# Rust unit tests only (no emulator startup needed)
test:
	cargo test --lib

# Vitest integration tests — builds debug binary first, then runs test suite
test-integration: build
	cd integration && npm install --silent && npx vitest run

# Parity tests (requires DESCOPE_PARITY_PROJECT_ID and DESCOPE_PARITY_MANAGEMENT_KEY to be set)
test-parity:
	cargo test --features parity

# Code coverage — enforces 95% floor
coverage:
	cargo llvm-cov --lib --fail-under-coverage 95

# Lint
lint:
	cargo clippy -- -D warnings
	cargo fmt --check
