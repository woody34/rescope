mod router;
mod store;
mod types;

use wasm_bindgen::prelude::*;
use std::sync::RwLock;

/// Global emulator state — single-threaded in WASM, so RwLock never contends.
static STATE: RwLock<Option<store::UserStore>> = RwLock::new(None);
static CONFIG: RwLock<Option<types::Config>> = RwLock::new(None);

/// Initialize the emulator with a config. Call once before handle_request.
#[wasm_bindgen]
pub fn init(config_json: &str) -> Result<(), JsValue> {
    let config: types::Config = serde_json::from_str(config_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid config: {e}")))?;

    *CONFIG.write().unwrap() = Some(config);
    *STATE.write().unwrap() = Some(store::UserStore::new());

    Ok(())
}

/// Handle an HTTP request. Input and output are JSON-serialized.
///
/// Design note: we acquire the lock *inside the router*, not here, to avoid
/// deadlocks when a handler needs to upgrade from read to write.
#[wasm_bindgen]
pub fn handle_request(request_json: &str) -> String {
    let req: types::WasmRequest = match serde_json::from_str(request_json) {
        Ok(r) => r,
        Err(e) => {
            return serde_json::to_string(&types::WasmResponse {
                status: 400,
                headers: Default::default(),
                body: format!("{{\"error\": \"Invalid request: {e}\"}}"),
            }).unwrap();
        }
    };

    // Pass the RwLock reference, let the router decide read vs write
    let response = router::route(&req, &STATE);
    serde_json::to_string(&response).unwrap()
}
