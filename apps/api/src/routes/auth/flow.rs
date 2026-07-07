//! Self-contained Descope FLOW runtime for `sign-up-or-in-passwords`.
//!
//! The `descope-wc` web component drives a flow by POSTing to `/v2/flow/start`
//! then `/v2/flow/next`, rendering the `screen` returned in each FLAT response
//! envelope. The emulator owns this one flow end-to-end: a two-screen flow
//! (email → password → completed) that authenticates against the same password
//! machinery as `POST /v1/auth/password/signin`.
//!
//! The response envelope is FLAT (no `data` wrapper) and mirrors the real
//! Descope `/v2/flow/start` response field-for-field so the SDK's parser is
//! happy. The two screen ids (`signIn`, `signInPassword`) are emulator-owned and
//! resolve to the static screens served by `flow_assets` at
//! `/pages/{projectId}/{version}/{screenId}.html`.

use crate::{
    auth_policy::AuthPolicyGuard,
    cookies::build_auth_cookies,
    error::EmulatorError,
    jwt::token_generator::{generate_refresh_jwt, generate_session_jwt},
    routes::auth::password::build_jwt_response,
    state::EmulatorState,
    store::flow_store::{FlowExecution, FlowStatus},
    store::token_store::generate_token,
};
use axum::{body::Bytes, extract::State, http::HeaderMap, Json};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

const FLOW_ID: &str = "sign-up-or-in-passwords";
/// Emulator-owned screen ids. Each MUST resolve to `<id>.html` in `flow_assets`
/// and be consistent with `config.json`'s `startScreenId` — the widget fetches
/// the screen HTML at `{screen.id}.html`, so any id emitted here must be served.
const SCREEN_SIGNIN: &str = "signIn";
const SCREEN_PASSWORD: &str = "signInPassword";

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Parse a request body permissively: ignore `Content-Type`, tolerate an empty
/// or malformed body by falling back to `{}`. The flow endpoints must never 500
/// on attacker-controlled input — a bad body just yields an empty object, which
/// flows through to an "unknown flow"/re-show path.
fn parse_body(bytes: &Bytes) -> Value {
    serde_json::from_slice(bytes).unwrap_or_else(|_| json!({}))
}

/// The static per-screen `state` sub-object, mirrored from the real
/// `/v2/flow/start` response verbatim.
fn screen_state() -> Value {
    json!({
        "componentsConfig": {
            "samlGroupMappings": {},
            "ssoApplications": {},
            "thirdPartyAppApproveScopes": {},
            "userRoles": {},
            "userSelectedTenant": {}
        },
        "form": { "spAcsUrl": "", "spEntityId": "", "spMetadataUrl": "" },
        "inputs": { "spAcsUrl": "", "spEntityId": "", "spMetadataUrl": "" },
        "project": { "name": "local" }
    })
}

/// Build a FLAT `waiting` envelope showing a screen.
fn waiting_envelope(execution_id: &str, step_id: &str, screen_id: &str, step_name: &str) -> Value {
    json!({
        "executionId": execution_id,
        "stepId": step_id,
        "status": "waiting",
        "action": "",
        "screen": { "id": screen_id, "state": screen_state() },
        "redirect": { "url": "", "isPopup": false },
        "webauthn": { "transactionId": "", "options": "" },
        "authInfo": null,
        "error": null,
        "lastAuth": null,
        "stepName": step_name,
        "samlIdpResponse": null,
        "openInNewTabUrl": "",
        "runnerLogs": [],
        "nativeResponse": null,
        "output": {}
    })
}

/// Build a FLAT `completed` envelope carrying the minted auth info.
fn completed_envelope(execution_id: &str, step_id: &str, auth_info: Value) -> Value {
    json!({
        "executionId": execution_id,
        "stepId": step_id,
        "status": "completed",
        "action": "",
        "screen": null,
        "redirect": { "url": "", "isPopup": false },
        "webauthn": { "transactionId": "", "options": "" },
        "authInfo": auth_info,
        "error": null,
        "lastAuth": null,
        "stepName": "",
        "samlIdpResponse": null,
        "openInNewTabUrl": "",
        "runnerLogs": [],
        "nativeResponse": null,
        "output": {}
    })
}

/// Build a FLAT `failed` envelope. Keeps every top-level key so the SDK parser
/// stays happy; the `error` object carries the reason.
fn failed_envelope(execution_id: &str, code: &str, description: &str) -> Value {
    json!({
        "executionId": execution_id,
        "stepId": "",
        "status": "failed",
        "action": "",
        "screen": null,
        "redirect": { "url": "", "isPopup": false },
        "webauthn": { "transactionId": "", "options": "" },
        "authInfo": null,
        "error": { "errorCode": code, "errorDescription": description },
        "lastAuth": null,
        "stepName": "",
        "samlIdpResponse": null,
        "openInNewTabUrl": "",
        "runnerLogs": [],
        "nativeResponse": null,
        "output": {}
    })
}

/// First string value found in a JSON object (fallback input extraction).
fn first_string(obj: &Value) -> Option<String> {
    obj.as_object()?
        .values()
        .find_map(|v| v.as_str().map(|s| s.to_string()))
}

fn field(input: &Value, key: &str) -> Option<String> {
    input
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

// ─── /v{1,2}/flow/start ─────────────────────────────────────────────────────

pub async fn start(
    State(state): State<EmulatorState>,
    body: Bytes,
) -> Result<(HeaderMap, Json<Value>), EmulatorError> {
    let body = parse_body(&body);
    let flow_id = field(&body, "flowId").unwrap_or_default();
    if flow_id != FLOW_ID {
        return Ok((
            HeaderMap::new(),
            Json(failed_envelope("", "E102001", "Unknown flow")),
        ));
    }

    let execution_id = format!("{FLOW_ID}---{}", &generate_token()[..27]);

    let exec = FlowExecution {
        execution_id: execution_id.clone(),
        flow_id: FLOW_ID.to_string(),
        status: FlowStatus::Waiting,
        step: 1,
        screen_id: SCREEN_SIGNIN.to_string(),
        login_id: None,
        created_at: now_secs(),
    };
    {
        let mut flows = state.flows.write().await;
        flows.sweep_expired(now_secs());
        flows.insert(exec);
    }

    let env = waiting_envelope(&execution_id, "1", SCREEN_SIGNIN, "Welcome");
    Ok((HeaderMap::new(), Json(env)))
}

// ─── /v{1,2}/flow/next ──────────────────────────────────────────────────────

pub async fn next(
    State(state): State<EmulatorState>,
    body: Bytes,
) -> Result<(HeaderMap, Json<Value>), EmulatorError> {
    let body = parse_body(&body);
    let execution_id = field(&body, "executionId").unwrap_or_default();
    let input = body.get("input").cloned().unwrap_or_else(|| json!({}));

    // Load the current step for this execution.
    let step = {
        let flows = state.flows.read().await;
        match flows.get(&execution_id) {
            Some(e) if e.status == FlowStatus::Waiting => e.step,
            _ => {
                return Ok((
                    HeaderMap::new(),
                    Json(failed_envelope(
                        &execution_id,
                        "E102103",
                        "Did not find next task",
                    )),
                ));
            }
        }
    };

    if step == 1 {
        // Email screen submitted → advance to the password screen.
        let login_id = field(&input, "externalId")
            .or_else(|| field(&input, "email"))
            .or_else(|| field(&input, "loginId"))
            .or_else(|| first_string(&input))
            .unwrap_or_default();

        state
            .flows
            .write()
            .await
            .advance_to_password(&execution_id, login_id);

        let env = waiting_envelope(&execution_id, "2", SCREEN_PASSWORD, "Sign In");
        return Ok((HeaderMap::new(), Json(env)));
    }

    // step == 2: password screen submitted → verify credentials.
    let login_id = {
        let flows = state.flows.read().await;
        flows
            .get(&execution_id)
            .and_then(|e| e.login_id.clone())
            .unwrap_or_default()
    };
    let password = field(&input, "password")
        .or_else(|| first_string(&input))
        .unwrap_or_default();

    // Any auth failure re-shows the password screen (never a 500), so the widget
    // re-renders its form; the descope-alert surfaces validation errors itself.
    let re_show = || {
        (
            HeaderMap::new(),
            Json(waiting_envelope(
                &execution_id,
                "2",
                SCREEN_PASSWORD,
                "Sign In",
            )),
        )
    };

    if AuthPolicyGuard::check_method_enabled(&state, "password")
        .await
        .is_err()
        || AuthPolicyGuard::check_not_locked_out(&state, &login_id)
            .await
            .is_err()
    {
        return Ok(re_show());
    }

    // Look up the stored hash. Unknown user → re-show, not 500.
    let hash = {
        let users = state.users.read().await;
        match users.load(&login_id) {
            Ok(u) => u._password_hash.clone(),
            Err(_) => None,
        }
    };
    let Some(hash) = hash else {
        AuthPolicyGuard::record_failure(&state, &login_id).await;
        return Ok(re_show());
    };

    let valid = {
        let pwd = password.clone();
        tokio::task::spawn_blocking(move || bcrypt::verify(&pwd, &hash))
            .await
            .map_err(|e| EmulatorError::Internal(e.to_string()))?
            .unwrap_or(false)
    };
    if !valid {
        AuthPolicyGuard::record_failure(&state, &login_id).await;
        return Ok(re_show());
    }
    AuthPolicyGuard::clear_failures(&state, &login_id).await;

    // Success — mint session + refresh JWTs exactly like password::signin.
    let users = state.users.read().await;
    let user = users.load(&login_id)?;
    if user.status == "disabled" {
        drop(users);
        return Ok(re_show());
    }
    let tmpl_store = state.jwt_templates.read().await;
    let active_tmpl = tmpl_store.active();
    let session_jwt = generate_session_jwt(
        &*state.km().await,
        user,
        &state.config.project_id,
        state.config.session_ttl,
        active_tmpl,
        &*state.roles.read().await,
        "pwd",
    )
    .map_err(|e| EmulatorError::Internal(e.to_string()))?;
    let refresh_jwt = generate_refresh_jwt(
        &*state.km().await,
        &user.user_id,
        &state.config.project_id,
        state.config.refresh_ttl,
    )
    .map_err(|e| EmulatorError::Internal(e.to_string()))?;

    let user_resp = serde_json::to_value(user.to_response()).unwrap();
    let cookies = build_auth_cookies(&session_jwt, &refresh_jwt, state.config.session_ttl);
    let auth_info =
        build_jwt_response(&state, &user.user_id, &session_jwt, &refresh_jwt, user_resp);
    drop(tmpl_store);
    drop(users);

    state.flows.write().await.complete(&execution_id);
    let _ = state.users.write().await.record_login(&login_id);

    Ok((
        cookies,
        Json(completed_envelope(&execution_id, "2", auth_info)),
    ))
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::config::EmulatorConfig;
    use crate::jwt::token_validator::validate_session_jwt;
    use crate::server::build_router;
    use crate::state::EmulatorState;
    use axum_test::TestServer;
    use serde_json::json;

    async fn setup() -> (TestServer, EmulatorState) {
        let state = EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap();
        let server = TestServer::new(build_router(state.clone())).unwrap();
        (server, state)
    }

    #[tokio::test]
    async fn descope_wc_flow_logs_in_via_rescope() {
        let (server, state) = setup().await;
        let login_id = "flowuser@test.com";
        let password = "SuperSecret123!";

        // Sign up a user via the password signup route.
        server
            .post("/v1/auth/password/signup")
            .json(&json!({
                "loginId": login_id,
                "password": password,
                "user": { "email": login_id }
            }))
            .await
            .assert_status_ok();

        // Start the flow.
        let start = server
            .post("/v2/flow/start")
            .json(&json!({ "flowId": "sign-up-or-in-passwords" }))
            .await;
        start.assert_status_ok();
        let start_body = start.json::<serde_json::Value>();
        assert_eq!(start_body["status"], "waiting");
        assert_eq!(start_body["screen"]["id"], "signIn");
        assert_eq!(start_body["stepName"], "Welcome");
        assert!(start_body["authInfo"].is_null());
        let execution_id = start_body["executionId"].as_str().unwrap().to_string();
        assert!(execution_id.starts_with("sign-up-or-in-passwords---"));

        // Regression: the widget fetches the screen HTML at the EXACT id the
        // envelope returned — that path must resolve (this is what breaks the
        // real widget if screen.id and the served asset names diverge).
        let start_screen = start_body["screen"]["id"].as_str().unwrap();
        server
            .get(&format!("/pages/PROJ/v2-beta/{start_screen}.html"))
            .await
            .assert_status_ok();

        // Step 1: submit the email → the password screen (a DIFFERENT screen).
        let step1 = server
            .post("/v2/flow/next")
            .json(&json!({
                "executionId": execution_id,
                "input": { "externalId": login_id }
            }))
            .await;
        step1.assert_status_ok();
        let step1_body = step1.json::<serde_json::Value>();
        assert_eq!(step1_body["status"], "waiting");
        assert_eq!(step1_body["screen"]["id"], "signInPassword");
        assert_ne!(
            step1_body["screen"]["id"], start_body["screen"]["id"],
            "password step must show a different screen than the email step"
        );
        // ...and that screen HTML must also resolve.
        let pw_screen = step1_body["screen"]["id"].as_str().unwrap();
        server
            .get(&format!("/pages/PROJ/v2-beta/{pw_screen}.html"))
            .await
            .assert_status_ok();

        // Step 2: submit the password → completed.
        let step2 = server
            .post("/v2/flow/next")
            .json(&json!({
                "executionId": execution_id,
                "input": { "password": password }
            }))
            .await;
        step2.assert_status_ok();

        // Cookies carry DS= and DSR=.
        let set_cookies: Vec<String> = step2
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect();
        let joined = set_cookies.join("\n");
        assert!(joined.contains("DS="), "expected DS cookie: {joined}");
        assert!(joined.contains("DSR="), "expected DSR cookie: {joined}");

        let step2_body = step2.json::<serde_json::Value>();
        assert_eq!(step2_body["status"], "completed");
        assert!(step2_body["screen"].is_null());
        let session_jwt = step2_body["authInfo"]["sessionJwt"].as_str().unwrap();
        assert!(!session_jwt.is_empty());

        // Session JWT decodes with amr ["pwd"] and the right subject.
        let claims = validate_session_jwt(&*state.km().await, session_jwt).unwrap();
        assert_eq!(claims.amr, vec!["pwd".to_string()]);
        let user_id = step2_body["authInfo"]["user"]["userId"].as_str().unwrap();
        assert_eq!(claims.sub, user_id);
    }

    #[tokio::test]
    async fn config_and_screen_assets_are_served() {
        let (server, _state) = setup().await;
        // config.json advertises startScreenId == the id the start envelope emits.
        let cfg = server
            .get("/pages/PROJ/v2-beta/config.json")
            .await
            .json::<serde_json::Value>();
        assert_eq!(
            cfg["flows"]["sign-up-or-in-passwords"]["startScreenId"], "signIn",
            "config startScreenId must match the emitted screen.id"
        );
        for tail in ["signIn.html", "signInPassword.html"] {
            server
                .get(&format!("/pages/PROJ/v2-beta/{tail}"))
                .await
                .assert_status_ok();
        }
    }

    #[tokio::test]
    async fn wrong_password_reshows_without_500() {
        let (server, _state) = setup().await;
        let login_id = "wrongpw@test.com";
        server
            .post("/v1/auth/password/signup")
            .json(&json!({
                "loginId": login_id, "password": "Correct1!", "user": { "email": login_id }
            }))
            .await
            .assert_status_ok();

        let start = server
            .post("/v2/flow/start")
            .json(&json!({ "flowId": "sign-up-or-in-passwords" }))
            .await;
        let execution_id = start.json::<serde_json::Value>()["executionId"]
            .as_str()
            .unwrap()
            .to_string();

        server
            .post("/v2/flow/next")
            .json(&json!({ "executionId": execution_id, "input": { "externalId": login_id } }))
            .await;

        let bad = server
            .post("/v2/flow/next")
            .json(&json!({ "executionId": execution_id, "input": { "password": "WrongOne!" } }))
            .await;
        bad.assert_status_ok(); // no 500
        let body = bad.json::<serde_json::Value>();
        assert_ne!(body["status"], "completed");
        assert_eq!(body["status"], "waiting");
        assert_eq!(body["screen"]["id"], "signInPassword");
        assert!(body["authInfo"].is_null());
    }

    #[tokio::test]
    async fn unknown_execution_id_fails() {
        let (server, _state) = setup().await;
        let resp = server
            .post("/v2/flow/next")
            .json(&json!({ "executionId": "does-not-exist", "input": { "password": "x" } }))
            .await;
        resp.assert_status_ok();
        assert_eq!(resp.json::<serde_json::Value>()["status"], "failed");
    }

    #[tokio::test]
    async fn unknown_flow_id_fails() {
        let (server, _state) = setup().await;
        let resp = server
            .post("/v2/flow/start")
            .json(&json!({ "flowId": "some-other-flow" }))
            .await;
        resp.assert_status_ok();
        let body = resp.json::<serde_json::Value>();
        assert_eq!(body["status"], "failed");
        assert_eq!(body["error"]["errorCode"], "E102001");
    }

    #[tokio::test]
    async fn malformed_body_does_not_500() {
        let (server, _state) = setup().await;
        // A non-JSON body must yield a 200 failed/unknown-flow envelope, never a 500.
        let resp = server
            .post("/v2/flow/start")
            .bytes("{not valid json".into())
            .await;
        resp.assert_status_ok();
        assert_eq!(resp.json::<serde_json::Value>()["status"], "failed");

        let resp2 = server.post("/v2/flow/next").bytes("garbage".into()).await;
        resp2.assert_status_ok();
        assert_eq!(resp2.json::<serde_json::Value>()["status"], "failed");
    }
}
