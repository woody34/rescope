/// `PermissiveJson<T>` — like Axum's `Json<T>` but ignores `Content-Type`.
///
/// Some SDK clients (e.g. `@descope/core-js-sdk` v1.x) omit the
/// `Content-Type: application/json` header on POST requests. Axum's built-in
/// `Json<T>` extractor rejects those with 415. This extractor reads the raw
/// request body and deserializes it with `serde_json` regardless of the
/// declared content type, acting as a permissive drop-in replacement.
use axum::{body::Body, extract::FromRequest, http::Request};
use serde::de::DeserializeOwned;

use crate::error::EmulatorError;

pub struct PermissiveJson<T>(pub T);

impl<T> std::ops::Deref for PermissiveJson<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

#[axum::async_trait]
impl<T, S> FromRequest<S> for PermissiveJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = EmulatorError;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = axum::body::Bytes::from_request(req, state)
            .await
            .map_err(|e| EmulatorError::Internal(e.to_string()))?;

        // For empty bodies (e.g. logout, refresh with no body) fall back to `{}`
        let bytes = if bytes.is_empty() {
            b"{}".as_ref().into()
        } else {
            bytes
        };

        let value = serde_json::from_slice::<T>(&bytes)
            .map_err(|e| EmulatorError::Internal(format!("JSON parse error: {e}")))?;

        Ok(PermissiveJson(value))
    }
}
