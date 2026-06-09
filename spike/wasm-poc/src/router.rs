use crate::store::UserStore;
use crate::types::*;
use std::collections::HashMap;
use std::sync::RwLock;

/// Sync request router — maps (method, path) to handler functions.
/// Takes a reference to the RwLock so handlers can acquire read or write as needed.
pub fn route(req: &WasmRequest, state: &RwLock<Option<UserStore>>) -> WasmResponse {
    match (req.method.as_str(), req.path.as_str()) {
        ("POST", "/v1/mgmt/user/create") => handle_create_user(req, state),
        ("POST", "/v1/mgmt/user/search") => handle_search_users(state),
        ("GET", "/health") => WasmResponse {
            status: 200,
            headers: json_headers(),
            body: r#"{"status":"ok","wasm":true}"#.to_string(),
        },
        _ => WasmResponse {
            status: 404,
            headers: json_headers(),
            body: format!(r#"{{"error":"Not found: {} {}"}}"#, req.method, req.path),
        },
    }
}

fn handle_create_user(req: &WasmRequest, state: &RwLock<Option<UserStore>>) -> WasmResponse {
    let body = match &req.body {
        Some(b) => b,
        None => {
            return WasmResponse {
                status: 400,
                headers: json_headers(),
                body: r#"{"error":"Missing request body"}"#.to_string(),
            };
        }
    };

    let create_req: CreateUserRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => {
            return WasmResponse {
                status: 400,
                headers: json_headers(),
                body: format!(r#"{{"error":"Invalid body: {e}"}}"#),
            };
        }
    };

    let user = User {
        user_id: uuid::Uuid::new_v4().to_string(),
        login_ids: vec![create_req.login_id.clone()],
        email: create_req.email,
        name: create_req.name,
    };

    // Acquire write lock directly — no outer read lock to deadlock with
    let mut state_guard = state.write().unwrap();
    if let Some(store) = state_guard.as_mut() {
        let response_user = user.clone();
        store.create(user);
        WasmResponse {
            status: 200,
            headers: json_headers(),
            body: serde_json::to_string(&serde_json::json!({ "user": response_user })).unwrap(),
        }
    } else {
        WasmResponse {
            status: 500,
            headers: json_headers(),
            body: r#"{"error":"Not initialized"}"#.to_string(),
        }
    }
}

fn handle_search_users(state: &RwLock<Option<UserStore>>) -> WasmResponse {
    let state_guard = state.read().unwrap();
    if let Some(store) = state_guard.as_ref() {
        let users: Vec<_> = store.list();
        WasmResponse {
            status: 200,
            headers: json_headers(),
            body: serde_json::to_string(&serde_json::json!({ "users": users })).unwrap(),
        }
    } else {
        WasmResponse {
            status: 500,
            headers: json_headers(),
            body: r#"{"error":"Not initialized"}"#.to_string(),
        }
    }
}

fn json_headers() -> HashMap<String, String> {
    let mut h = HashMap::new();
    h.insert("content-type".to_string(), "application/json".to_string());
    h
}
