/// rescope-core: Framework-agnostic business logic for the Rescope emulator.
///
/// This crate contains all stores, types, JWT handling, and config that can
/// be shared between the native async API (apps/api) and the WASM playground.
/// It intentionally avoids async runtime dependencies (tokio, axum, reqwest).
pub mod config;
pub mod error;
#[cfg(feature = "native-crypto")]
pub mod jwt;
pub mod store;
pub mod types;
