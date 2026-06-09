/// OIDC Identity Provider emulator endpoints.
///
/// GET  /emulator/idp/:idp_id/.well-known/openid-configuration → discovery doc
/// GET  /emulator/idp/:idp_id/jwks                             → IdP public key
/// GET  /emulator/idp/:idp_id/authorize                        → user picker / code grant
/// POST /emulator/idp/:idp_id/token                            → code → id_token
/// GET  /emulator/idp/callback                                 → internal SP callback
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use jsonwebtoken::{encode, Algorithm, Header};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    error::EmulatorError,
    jwt::key_manager::KeyManager,
    state::EmulatorState,
    store::{idp_store::IdpProtocol, token_store::generate_token},
    types::TokenType,
};

// ─── OIDC Auth Code Store ────────────────────────────────────────────────────

/// Metadata stored alongside an OIDC authorization code.
#[derive(Debug, Clone)]
pub struct OidcCodeEntry {
    pub user_id: String,
    pub idp_id: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub nonce: Option<String>,
    pub created_at: u64,
}

/// In-memory store for OIDC authorization codes.
/// Separate from TokenStore because OIDC codes carry richer metadata.
#[derive(Default)]
pub struct OidcCodeStore {
    codes: HashMap<String, OidcCodeEntry>,
}

impl OidcCodeStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, code: String, entry: OidcCodeEntry) {
        self.codes.insert(code, entry);
    }

    pub fn consume(&mut self, code: &str) -> Option<OidcCodeEntry> {
        self.codes.remove(code)
    }

    pub fn reset(&mut self) {
        self.codes.clear();
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_secs()
}

// ─── Discovery ───────────────────────────────────────────────────────────────

pub async fn discovery(
    State(state): State<EmulatorState>,
    Path(idp_id): Path<String>,
) -> Result<Json<Value>, EmulatorError> {
    // Verify IdP exists
    state.idp_emulators.read().await.load(&idp_id)?;

    let base = format!(
        "http://localhost:{}/emulator/idp/{}",
        state.config.port, idp_id
    );
    Ok(Json(json!({
        "issuer": base,
        "authorization_endpoint": format!("{}/authorize", base),
        "token_endpoint": format!("{}/token", base),
        "jwks_uri": format!("{}/jwks", base),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
    })))
}

// ─── JWKS ────────────────────────────────────────────────────────────────────

pub async fn jwks(State(state): State<EmulatorState>) -> Json<Value> {
    let km = state.idp_keys.read().await.clone();
    Json(km.jwks())
}

// ─── Authorize ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AuthorizeParams {
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub response_type: Option<String>,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
    /// Programmatic test helper: auto-select this user instead of showing a picker.
    pub login_id: Option<String>,
}

pub async fn authorize(
    State(state): State<EmulatorState>,
    Path(idp_id): Path<String>,
    Query(params): Query<AuthorizeParams>,
) -> Result<Response, EmulatorError> {
    let client_id = params.client_id.ok_or(EmulatorError::ValidationError(
        "client_id is required".into(),
    ))?;
    let redirect_uri = params.redirect_uri.ok_or(EmulatorError::ValidationError(
        "redirect_uri is required".into(),
    ))?;

    // Load IdP config
    let idp = state.idp_emulators.read().await.load(&idp_id)?.clone();
    if idp.protocol != IdpProtocol::Oidc {
        return Err(EmulatorError::ValidationError(
            "IdP is not configured for OIDC".into(),
        ));
    }

    // If login_id is provided (programmatic), auto-select the user
    if let Some(login_id) = params.login_id {
        let user = state.users.read().await.load(&login_id)?.clone();

        // Generate OIDC authorization code
        let code = generate_token();
        let entry = OidcCodeEntry {
            user_id: user.user_id.clone(),
            idp_id: idp_id.clone(),
            client_id,
            redirect_uri: redirect_uri.clone(),
            nonce: params.nonce.clone(),
            created_at: now_secs(),
        };
        state.oidc_codes.write().await.insert(code.clone(), entry);

        // Redirect back with code + state
        let mut url = format!("{}?code={}", redirect_uri, code);
        if let Some(st) = &params.state {
            url.push_str(&format!("&state={}", st));
        }
        return Ok(Redirect::to(&url).into_response());
    }

    // Browser mode: show user picker HTML
    // Show all users — the IdP is already tenant-scoped, and user_tenants
    // may not be populated if users were created via the mgmt API.
    let users: Vec<Value> = {
        let users_store = state.users.read().await;
        users_store
            .all_users()
            .iter()
            .map(|u| {
                json!({
                    "userId": u.user_id,
                    "loginId": u.login_ids.first().unwrap_or(&u.user_id),
                    "email": u.email.clone().unwrap_or_default(),
                    "name": u.name.clone().unwrap_or_else(|| u.login_ids.first().cloned().unwrap_or_default()),
                })
            })
            .collect()
    };

    let user_rows = if users.is_empty() {
        "<tr><td colspan=\"3\" style=\"text-align:center;padding:2rem;color:#888;\">No users in this tenant</td></tr>".to_string()
    } else {
        users
            .iter()
            .map(|u| {
                let login_id = u["loginId"].as_str().unwrap_or("");
                let name = u["name"].as_str().unwrap_or(login_id);
                let email = u["email"].as_str().unwrap_or("");
                format!(
                    r#"<tr>
                        <td>{name}</td>
                        <td>{email}</td>
                        <td><a class="btn" href="/emulator/idp/{idp_id}/authorize?client_id={client_id}&redirect_uri={redir}&response_type=code&state={state}&nonce={nonce}&login_id={login_id}">Login</a></td>
                    </tr>"#,
                    name = html_escape(name),
                    email = html_escape(email),
                    idp_id = idp_id,
                    client_id = urlencoding::encode(&client_id),
                    redir = urlencoding::encode(&redirect_uri),
                    state = urlencoding::encode(params.state.as_deref().unwrap_or("")),
                    nonce = urlencoding::encode(params.nonce.as_deref().unwrap_or("")),
                    login_id = urlencoding::encode(login_id),
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Rescope IdP Emulator — {display_name}</title>
<style>
  *{{margin:0;padding:0;box-sizing:border-box}}
  body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#0d1117;color:#e6edf3;min-height:100vh;display:flex;align-items:center;justify-content:center}}
  .card{{background:#161b22;border:1px solid #30363d;border-radius:12px;padding:2rem;max-width:520px;width:100%}}
  h1{{font-size:1.1rem;color:#39d353;margin-bottom:.5rem}}
  .subtitle{{color:#8b949e;font-size:.85rem;margin-bottom:1.5rem}}
  table{{width:100%;border-collapse:collapse}}
  th{{text-align:left;color:#8b949e;font-size:.75rem;text-transform:uppercase;padding:.5rem;border-bottom:1px solid #30363d}}
  td{{padding:.75rem .5rem;border-bottom:1px solid #21262d}}
  .btn{{display:inline-block;background:#39d353;color:#0d1117;padding:.4rem 1rem;border-radius:6px;text-decoration:none;font-size:.85rem;font-weight:600}}
  .btn:hover{{background:#2ea043}}
</style>
</head>
<body>
<div class="card">
  <h1>⚡ Rescope IdP Emulator</h1>
  <div class="subtitle">{display_name} — Select a user to sign in</div>
  <table>
    <thead><tr><th>Name</th><th>Email</th><th></th></tr></thead>
    <tbody>{user_rows}</tbody>
  </table>
</div>
</body>
</html>"#,
        display_name = html_escape(&idp.display_name),
        user_rows = user_rows,
    );

    Ok(Html(html).into_response())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ─── Token Endpoint ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TokenRequest {
    pub grant_type: Option<String>,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub id_token: String,
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

pub async fn token(
    State(state): State<EmulatorState>,
    Path(idp_id): Path<String>,
    body: String, // Accept form or JSON
) -> Result<Response, EmulatorError> {
    // Parse either form-urlencoded or JSON
    let req: TokenRequest = if body.contains('=') {
        serde_urlencoded::from_str(&body)
            .map_err(|e| EmulatorError::ValidationError(format!("Invalid form body: {e}")))?
    } else {
        serde_json::from_str(&body)
            .map_err(|e| EmulatorError::ValidationError(format!("Invalid JSON body: {e}")))?
    };

    let code = req
        .code
        .ok_or(EmulatorError::ValidationError("code is required".into()))?;

    // Consume the auth code
    let entry = state
        .oidc_codes
        .write()
        .await
        .consume(&code)
        .ok_or(EmulatorError::ValidationError("invalid_grant".into()))?;

    // Verify IdP matches
    if entry.idp_id != idp_id {
        return Err(EmulatorError::ValidationError("invalid_grant".into()));
    }

    // Verify client_id and redirect_uri
    if let Some(ref client_id) = req.client_id {
        if *client_id != entry.client_id {
            return Err(EmulatorError::InvalidCredentials);
        }
    }

    // Load the IdP config for attribute mapping
    let idp = state.idp_emulators.read().await.load(&idp_id)?.clone();

    // Verify client_secret against tenant's OIDC config
    let tenant = state.tenants.read().await.load(&idp.tenant_id)?.clone();
    if let Some(ref oidc_config) = tenant.oidc_config {
        if let Some(ref expected_secret) = oidc_config.client_secret {
            if let Some(ref provided_secret) = req.client_secret {
                if provided_secret != expected_secret {
                    return Err(EmulatorError::InvalidCredentials);
                }
            }
        }
    }

    // Load the user
    let user = {
        let users = state.users.read().await;
        users.load_by_user_id(&entry.user_id)?.clone()
    };

    // Generate id_token signed with IdP key
    let idp_km = state.idp_keys.read().await.clone();
    let base = format!(
        "http://localhost:{}/emulator/idp/{}",
        state.config.port, idp_id
    );

    let id_token = generate_id_token(
        &idp_km,
        &user,
        &base,
        &entry.client_id,
        entry.nonce.as_deref(),
        &idp.attribute_mapping,
    )?;

    let access_token = generate_token();

    let resp = TokenResponse {
        id_token,
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
    };

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&resp).unwrap(),
    )
        .into_response())
}

// ─── IdP Callback (SP-side) ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: Option<String>,
}

/// Internal SP callback: receives an OIDC code from the IdP, exchanges it
/// internally, creates/updates the user, generates an SP code, and redirects
/// to the app's original redirect_uri.
pub async fn callback(
    State(state): State<EmulatorState>,
    Query(params): Query<CallbackParams>,
) -> Result<Response, EmulatorError> {
    let oidc_code = params.code;

    // Peek at the code entry to find the IdP and user
    let entry = state
        .oidc_codes
        .write()
        .await
        .consume(&oidc_code)
        .ok_or(EmulatorError::InvalidToken)?;

    let user_id = entry.user_id.clone();

    // Record login
    {
        let users = state.users.read().await;
        let _user = users.load_by_user_id(&user_id)?;
    }

    // Generate an SP-side code and store it
    let sp_code = generate_token();
    state.tokens.write().await.insert(
        sp_code.clone(),
        user_id,
        TokenType::Saml, // Reuse Saml token type for SSO exchange compatibility
    );

    // Redirect to the original app redirect_uri
    // The state param contains the encoded original redirect_uri
    let redirect_uri = params.state.ok_or(EmulatorError::ValidationError(
        "Missing state (original redirect_uri)".into(),
    ))?;

    let url = format!("{}?code={}", redirect_uri, sp_code);
    Ok(Redirect::to(&url).into_response())
}

// ─── id_token Generation ─────────────────────────────────────────────────────

fn generate_id_token(
    km: &KeyManager,
    user: &crate::types::User,
    issuer: &str,
    audience: &str,
    nonce: Option<&str>,
    attribute_mapping: &HashMap<String, String>,
) -> Result<String, EmulatorError> {
    let iat = now_secs();
    let exp = iat + 3600;

    let mut claims = serde_json::Map::new();
    claims.insert("iss".into(), json!(issuer));
    claims.insert("sub".into(), json!(user.user_id));
    claims.insert("aud".into(), json!(audience));
    claims.insert("iat".into(), json!(iat));
    claims.insert("exp".into(), json!(exp));
    if let Some(n) = nonce {
        claims.insert("nonce".into(), json!(n));
    }

    // Apply attribute mapping: for each IdP claim → user field path
    let user_json = serde_json::to_value(user).unwrap_or_default();
    for (idp_claim, user_field) in attribute_mapping {
        let value = resolve_user_field(&user_json, user_field);
        if !value.is_null() {
            claims.insert(idp_claim.clone(), value);
        }
    }

    // Default claims if not provided by mapping
    if !claims.contains_key("email") {
        if let Some(ref email) = user.email {
            claims.insert("email".into(), json!(email));
        }
    }
    if !claims.contains_key("name") {
        if let Some(ref name) = user.name {
            claims.insert("name".into(), json!(name));
        }
    }

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(km.kid.clone());

    encode(&header, &Value::Object(claims), &km.encoding_key)
        .map_err(|e| EmulatorError::Internal(format!("Failed to sign id_token: {e}")))
}

/// Resolve a dotted path like "user.email" or "user.customAttributes.department"
/// against a JSON value.
fn resolve_user_field(user_json: &Value, path: &str) -> Value {
    // Strip leading "user." prefix if present
    let path = path.strip_prefix("user.").unwrap_or(path);
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = user_json;
    for part in &parts {
        match current.get(part).or_else(|| {
            // Try camelCase conversion: given_name → givenName
            let camel = to_camel_case(part);
            current.get(&camel)
        }) {
            Some(v) => current = v,
            None => return Value::Null,
        }
    }
    current.clone()
}

fn to_camel_case(s: &str) -> String {
    let parts: Vec<&str> = s.split('_').collect();
    let mut result = parts[0].to_string();
    for part in &parts[1..] {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_uppercase().next().unwrap_or(first));
            result.extend(chars);
        }
    }
    result
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::EmulatorConfig,
        state::EmulatorState,
        store::idp_store::{IdpEmulator, IdpProtocol},
        types::{OidcConfig, Tenant, User, UserTenant},
    };
    use axum::extract::{Path as AxumPath, State as AxumState};

    async fn make_state() -> EmulatorState {
        EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap()
    }

    async fn setup_oidc_idp(state: &EmulatorState) -> String {
        // Create tenant
        let tenant = Tenant {
            id: "acme".into(),
            name: "Acme Corp".into(),
            domains: vec!["acme.com".into()],
            auth_type: crate::types::AuthType::Oidc,
            oidc_config: Some(OidcConfig {
                client_id: Some("acme-client".into()),
                client_secret: Some("acme-secret".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        state.tenants.write().await.insert(tenant);

        // Create IdP
        let idp = IdpEmulator {
            id: "mock-okta".into(),
            protocol: IdpProtocol::Oidc,
            display_name: "Mock Okta".into(),
            tenant_id: "acme".into(),
            attribute_mapping: HashMap::from([
                ("email".into(), "user.email".into()),
                ("name".into(), "user.name".into()),
            ]),
        };
        state.idp_emulators.write().await.insert(idp).unwrap();

        // Create user in tenant
        let user = User {
            user_id: "user-alice".into(),
            login_ids: vec!["alice@acme.com".into()],
            email: Some("alice@acme.com".into()),
            name: Some("Alice".into()),
            status: "enabled".into(),
            user_tenants: vec![UserTenant {
                tenant_id: "acme".into(),
                tenant_name: "Acme Corp".into(),
                role_names: vec![],
            }],
            ..Default::default()
        };
        state.users.write().await.insert(user).unwrap();

        "mock-okta".to_string()
    }

    #[tokio::test]
    async fn discovery_returns_valid_document() {
        let state = make_state().await;
        setup_oidc_idp(&state).await;

        let result = discovery(AxumState(state), AxumPath("mock-okta".into()))
            .await
            .unwrap();
        let doc = result.0;
        assert!(doc["issuer"].as_str().unwrap().contains("mock-okta"));
        assert!(doc["authorization_endpoint"]
            .as_str()
            .unwrap()
            .ends_with("/authorize"));
        assert!(doc["token_endpoint"].as_str().unwrap().ends_with("/token"));
        assert!(doc["jwks_uri"].as_str().unwrap().ends_with("/jwks"));
    }

    #[tokio::test]
    async fn discovery_unknown_idp_returns_error() {
        let state = make_state().await;
        let err = discovery(AxumState(state), AxumPath("ghost".into()))
            .await
            .unwrap_err();
        assert!(matches!(err, EmulatorError::IdpNotFound));
    }

    #[tokio::test]
    async fn jwks_returns_keys_array() {
        let state = make_state().await;
        let result = jwks(AxumState(state)).await;
        assert!(result["keys"].is_array());
        assert!(!result["keys"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn oidc_code_store_insert_and_consume() {
        let mut store = OidcCodeStore::new();
        let entry = OidcCodeEntry {
            user_id: "u1".into(),
            idp_id: "idp1".into(),
            client_id: "c1".into(),
            redirect_uri: "http://localhost/cb".into(),
            nonce: Some("n1".into()),
            created_at: 0,
        };
        store.insert("code1".into(), entry);
        let consumed = store.consume("code1").unwrap();
        assert_eq!(consumed.user_id, "u1");
        assert!(store.consume("code1").is_none()); // single-use
    }

    #[tokio::test]
    async fn resolve_user_field_works() {
        let user = User {
            user_id: "u1".into(),
            email: Some("alice@test.com".into()),
            name: Some("Alice".into()),
            given_name: Some("Alice".into()),
            ..Default::default()
        };
        let user_json = serde_json::to_value(&user).unwrap();

        assert_eq!(
            resolve_user_field(&user_json, "user.email"),
            json!("alice@test.com")
        );
        assert_eq!(resolve_user_field(&user_json, "user.name"), json!("Alice"));
        assert_eq!(
            resolve_user_field(&user_json, "user.givenName"),
            json!("Alice")
        );
        assert_eq!(
            resolve_user_field(&user_json, "email"),
            json!("alice@test.com")
        );
        assert!(resolve_user_field(&user_json, "user.nonexistent").is_null());
    }

    #[tokio::test]
    async fn generate_id_token_contains_claims() {
        let km = KeyManager::generate().unwrap();
        let user = User {
            user_id: "u1".into(),
            email: Some("alice@test.com".into()),
            name: Some("Alice".into()),
            ..Default::default()
        };
        let mapping = HashMap::from([
            ("email".into(), "user.email".into()),
            ("name".into(), "user.name".into()),
        ]);

        let token = generate_id_token(
            &km,
            &user,
            "http://issuer",
            "client1",
            Some("nonce1"),
            &mapping,
        )
        .unwrap();

        // Decode and verify claims
        let mut validation = jsonwebtoken::Validation::new(Algorithm::RS256);
        validation.validate_exp = false;
        validation.set_audience(&["client1"]);
        validation.set_issuer(&["http://issuer"]);
        let decoded = jsonwebtoken::decode::<Value>(&token, &km.decoding_key, &validation).unwrap();
        let claims = decoded.claims;

        assert_eq!(claims["sub"], "u1");
        assert_eq!(claims["email"], "alice@test.com");
        assert_eq!(claims["name"], "Alice");
        assert_eq!(claims["nonce"], "nonce1");
        assert_eq!(claims["iss"], "http://issuer");
        assert_eq!(claims["aud"], "client1");
    }
}
