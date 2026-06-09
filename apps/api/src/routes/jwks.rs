use axum::{extract::State, Json};

use crate::state::EmulatorState;

pub async fn jwks_handler(State(state): State<EmulatorState>) -> Json<serde_json::Value> {
    Json(state.km().await.jwks())
}
