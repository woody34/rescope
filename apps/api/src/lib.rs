pub mod auth_policy;
pub mod connector;
pub mod cookies;
#[cfg(not(feature = "dev-ui"))]
pub mod embedded_ui;
pub mod extractor;
pub mod mgmt_auth;
pub mod openapi;
pub mod request_log;
pub mod routes;
pub mod seed;
pub mod server;
pub mod state;

// ── Re-exports from rescope-core ──────────────────────────────────────────────
// This lets all existing `crate::store`, `crate::types`, etc. references
// continue working without changing every import across the codebase.
pub use rescope_core::config;
pub use rescope_core::error;
pub use rescope_core::jwt;
pub use rescope_core::store;
pub use rescope_core::types;
