use crate::extractor::PermissiveJson;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{
    error::EmulatorError,
    state::EmulatorState,
    store::{
        token_store::generate_token,
        user_store::{new_user_id, SearchQuery, SortSpec, UserPatch, UserUpdate},
    },
    types::{TokenType, User, UserTenant},
};

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// ─── Create user ─────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub login_id: Option<String>,
    /// Array form — UI sends `loginIds: [...]`, we use the first element
    pub login_ids: Option<Vec<String>>,
    pub email: Option<String>,
    pub phone: Option<String>,
    // Accept both 'name' (legacy) and 'displayName' (Node SDK field)
    pub name: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub given_name: Option<String>,
    pub middle_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub role_names: Option<Vec<String>>,
    pub user_tenants: Option<Vec<UserTenantInput>>,
    pub custom_attributes: Option<HashMap<String, Value>>,
    pub verified_email: Option<bool>,
    pub verified_phone: Option<bool>,
    /// When true, marks the user as a test user (allows mgmt OTP/magic-link helper endpoints).
    #[serde(default)]
    pub test: bool,
}

impl CreateUserRequest {
    pub fn resolved_login_id(&self) -> Option<String> {
        self.login_id
            .clone()
            .or_else(|| self.login_ids.as_ref()?.first().cloned())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserTenantInput {
    pub tenant_id: String,
    pub role_names: Option<Vec<String>>,
}

pub(crate) async fn create_user_impl(
    state: &EmulatorState,
    req: CreateUserRequest,
    is_test: bool,
    use_uid_as_user_id: bool,
) -> Result<Json<Value>, EmulatorError> {
    let login_id = req
        .resolved_login_id()
        .ok_or_else(|| EmulatorError::Internal("loginId is required".into()))?;

    // Descope lowercases email-based loginIds
    let login_id = if login_id.contains('@') {
        login_id.to_lowercase()
    } else {
        login_id
    };

    let email = req
        .email
        .clone()
        .or_else(|| {
            if login_id.contains('@') {
                Some(login_id.clone())
            } else {
                None
            }
        })
        .map(|e| e.to_lowercase());

    // On the seed path, honor a caller-provided `uid` as the userId: the backend
    // seed stores that uid as the user's descopeId and resolves admins via
    // findById(descopeId)/loadByUserId, so the emulator's userId must equal it.
    // Runtime creates keep a generated id — there, the backend sets descopeId to
    // the id the emulator returns (createTestUser: dto.descopeId = resp.userId), so
    // the two already agree, and overriding it would break flows that round-trip the
    // returned userId (e.g. the support change-password / cOTP flow).
    let user_id = if use_uid_as_user_id {
        req.custom_attributes
            .as_ref()
            .and_then(|a| a.get("uid"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(new_user_id)
    } else {
        new_user_id()
    };
    let mut user = User {
        user_id,
        login_ids: vec![login_id.clone()],
        email,
        phone: req.phone,
        name: req.display_name.or(req.name),
        given_name: req.given_name,
        middle_name: req.middle_name,
        family_name: req.family_name,
        picture: req.picture,
        role_names: req.role_names.unwrap_or_default(),
        custom_attributes: req.custom_attributes.unwrap_or_default(),
        verified_email: req.verified_email.unwrap_or(false),
        verified_phone: req.verified_phone.unwrap_or(false),
        status: "invited".into(),
        created_time: now(),
        _is_test_user: is_test || req.test,
        ..Default::default()
    };

    if let Some(tenants) = req.user_tenants {
        for t in tenants {
            let name = state
                .tenants
                .read()
                .await
                .load(&t.tenant_id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|_| t.tenant_id.clone());
            user.user_tenants.push(UserTenant {
                tenant_id: t.tenant_id,
                tenant_name: name,
                role_names: t.role_names.unwrap_or_default(),
            });
        }
    }

    // Idempotent for test users: e2e suites (and Playwright retries that re-run
    // setup) re-create the same loginId within a single run, where the user from
    // the prior attempt still lives in the emulator. Real Descope's test-user flow
    // tolerates this; the strict insert errors with UserAlreadyExists and derails
    // e2e setup. For test users, treat a duplicate loginId as success and return
    // the existing user. Real (non-test) user creation keeps the strict behavior.
    match state.users.write().await.insert(user) {
        Ok(()) => {}
        Err(EmulatorError::UserAlreadyExists) if is_test => {}
        Err(e) => return Err(e),
    }
    let users = state.users.read().await;
    let user = users.load(&login_id)?;
    Ok(Json(json!({ "user": user.to_response() })))
}

pub async fn create(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<CreateUserRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let is_test = req.test;
    create_user_impl(&state, req, is_test, false).await
}

pub async fn create_test(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<CreateUserRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    create_user_impl(&state, req, true, false).await
}

// ─── Load user ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LoginIdQuery {
    // Accept both camelCase (loginId — Node SDK) and lowercase (loginid — legacy)
    #[serde(rename = "loginId", alias = "loginid")]
    pub login_id: Option<String>,
    // The Node SDK also sends userId to the same /v1/mgmt/user path for loadByUserId
    #[serde(rename = "userId", alias = "userid")]
    pub user_id: Option<String>,
}

pub async fn load(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Query(q): Query<LoginIdQuery>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let users = state.users.read().await;
    let user = if let Some(ref login_id) = q.login_id {
        // Real Descope falls back to userId lookup when loginId doesn't match.
        // This supports SDK patterns where a userId is passed to load(loginId).
        users
            .load(login_id)
            .or_else(|_| users.load_by_user_id(login_id))?
    } else if let Some(ref user_id) = q.user_id {
        users.load_by_user_id(user_id)?
    } else {
        return Err(EmulatorError::UserNotFound);
    };
    Ok(Json(json!({ "user": user.to_response() })))
}

#[derive(Deserialize)]
pub struct UserIdQuery {
    // Accept both camelCase (userId — Node SDK) and lowercase (userid — legacy)
    #[serde(rename = "userId", alias = "userid")]
    pub user_id: Option<String>,
}

pub async fn load_by_user_id(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Query(q): Query<UserIdQuery>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let user_id = q.user_id.ok_or(EmulatorError::UserNotFound)?;
    let users = state.users.read().await;
    let user = users.load_by_user_id(&user_id)?;
    Ok(Json(json!({ "user": user.to_response() })))
}

// ─── Search ──────────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SortRequest {
    pub field: Option<String>,
    pub desc: Option<bool>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub emails: Option<Vec<String>>,
    pub phones: Option<Vec<String>>,
    pub custom_attributes: Option<HashMap<String, Value>>,
    pub with_test_user: Option<bool>,
    pub page: Option<usize>,
    pub limit: Option<usize>,
    // New filter fields
    pub login_ids: Option<Vec<String>>,
    pub statuses: Option<Vec<String>>,
    pub tenant_ids: Option<Vec<String>>,
    pub role_names: Option<Vec<String>>,
    pub text: Option<String>,
    pub sort: Option<Vec<SortRequest>>,
    pub created_after: Option<u64>,
    pub created_before: Option<u64>,
}

pub async fn search(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<SearchRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let sort_spec = req.sort.as_ref().and_then(|s| s.first()).map(|s| SortSpec {
        field: s.field.clone().unwrap_or_else(|| "createdTime".to_string()),
        desc: s.desc.unwrap_or(true),
    });
    let query = SearchQuery {
        emails: req.emails,
        phones: req.phones,
        custom_attributes: req.custom_attributes,
        with_test_user: req.with_test_user.unwrap_or(false),
        page: req.page.unwrap_or(0),
        limit: req.limit.unwrap_or(100),
        login_ids: req.login_ids,
        statuses: req.statuses,
        tenant_ids: req.tenant_ids,
        role_names: req.role_names,
        text: req.text,
        sort: sort_spec,
        created_after: req.created_after,
        created_before: req.created_before,
    };
    let users = state.users.read().await;
    let results: Vec<_> = users
        .search(&query)
        .into_iter()
        .map(|u| u.to_response())
        .collect();
    Ok(Json(json!({ "users": results })))
}

// ─── Update / Patch ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRequest {
    pub login_id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    // Accept both 'name' (legacy) and 'displayName' (Node SDK field)
    pub name: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub given_name: Option<String>,
    pub middle_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub verified_email: Option<bool>,
    pub verified_phone: Option<bool>,
    pub role_names: Option<Vec<String>>,
    pub custom_attributes: Option<HashMap<String, Value>>,
    pub user_tenants: Option<Vec<UserTenantInput>>,
}

pub async fn update(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<UpdateRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let user_tenants = if let Some(tenants) = req.user_tenants {
        let tenant_store = state.tenants.read().await;
        let mut resolved = vec![];
        for t in tenants {
            let name = tenant_store
                .load(&t.tenant_id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|_| t.tenant_id.clone());
            resolved.push(UserTenant {
                tenant_id: t.tenant_id,
                tenant_name: name,
                role_names: t.role_names.unwrap_or_default(),
            });
        }
        Some(resolved)
    } else {
        None
    };

    let u = UserUpdate {
        email: req.email,
        phone: req.phone,
        name: req.display_name.or(req.name),
        given_name: req.given_name,
        middle_name: req.middle_name,
        family_name: req.family_name,
        picture: req.picture,
        verified_email: req.verified_email,
        verified_phone: req.verified_phone,
        role_names: req.role_names,
        custom_attributes: req.custom_attributes,
        user_tenants,
    };
    let mut users = state.users.write().await;
    // Resolve loginId — fall back to userId lookup (same pattern as load)
    let resolved_login_id = if users.load(&req.login_id).is_ok() {
        req.login_id.clone()
    } else {
        // Try as userId and get the user's actual loginId
        let user = users.load_by_user_id(&req.login_id)?;
        user.login_ids
            .first()
            .cloned()
            .unwrap_or(req.login_id.clone())
    };
    let user = users.update(&resolved_login_id, u)?;
    Ok(Json(json!({ "user": user.to_response() })))
}

pub async fn user_patch(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<UpdateRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let p = UserPatch {
        email: req.email,
        phone: req.phone,
        name: req.display_name.or(req.name),
        given_name: req.given_name,
        middle_name: req.middle_name,
        family_name: req.family_name,
        picture: req.picture,
        verified_email: req.verified_email,
        verified_phone: req.verified_phone,
        role_names: req.role_names,
        custom_attributes: req.custom_attributes,
        user_tenants: None,
    };
    let mut users = state.users.write().await;
    let resolved_login_id = if users.load(&req.login_id).is_ok() {
        req.login_id.clone()
    } else {
        let user = users.load_by_user_id(&req.login_id)?;
        user.login_ids
            .first()
            .cloned()
            .unwrap_or(req.login_id.clone())
    };
    let user = users.patch(&resolved_login_id, p)?;
    Ok(Json(json!({ "user": user.to_response() })))
}

// ─── Update email ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEmailRequest {
    pub login_id: String,
    pub email: String,
    pub verified: Option<bool>,
}

pub async fn update_email(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<UpdateEmailRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let p = UserPatch {
        email: Some(req.email),
        verified_email: req.verified,
        ..Default::default()
    };
    let mut users = state.users.write().await;
    let resolved_login_id = if users.load(&req.login_id).is_ok() {
        req.login_id.clone()
    } else {
        let user = users.load_by_user_id(&req.login_id)?;
        user.login_ids
            .first()
            .cloned()
            .unwrap_or(req.login_id.clone())
    };
    let user = users.patch(&resolved_login_id, p)?;
    Ok(Json(json!({ "user": user.to_response() })))
}

// ─── Set active password ──────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPasswordRequest {
    pub login_id: String,
    pub password: String,
}

pub async fn set_active_password(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<SetPasswordRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    // Enforce password policy
    {
        let cfg = state.auth_method_config.read().await;
        let policy = &cfg.config.password;
        if (req.password.len() as u32) < policy.min_length {
            return Err(EmulatorError::PasswordPolicy(format!(
                "Password must be at least {} characters",
                policy.min_length
            )));
        }
    }
    let hash = {
        let pwd = req.password.clone();
        tokio::task::spawn_blocking(move || {
            bcrypt::hash(&pwd, 10).map_err(|e| EmulatorError::Internal(e.to_string()))
        })
        .await
        .map_err(|e| EmulatorError::Internal(e.to_string()))??
    };
    state
        .users
        .write()
        .await
        .set_password(&req.login_id, hash)?;
    Ok(Json(json!({ "ok": true })))
}

// ─── Delete ──────────────────────────────────────────────────────────────────

pub async fn delete_user(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Query(q): Query<LoginIdQuery>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    if let Some(login_id) = q.login_id {
        state.users.write().await.delete_by_login_id(&login_id);
    }
    Ok(Json(json!({ "ok": true })))
}

pub async fn delete_by_user_id(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Query(q): Query<UserIdQuery>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    if let Some(user_id) = q.user_id {
        state.users.write().await.delete_by_user_id(&user_id);
    }
    Ok(Json(json!({ "ok": true })))
}

pub async fn delete_all_test_users(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state.users.write().await.delete_all_test_users();
    Ok(Json(json!({ "ok": true })))
}

// ─── Delete (POST variant) ────────────────────────────────────────────────────
// The Node SDK calls POST /v1/mgmt/user/delete with body { loginId } or { userId }
// instead of DELETE /v1/mgmt/user?loginId=...

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeleteUserPostRequest {
    pub login_id: Option<String>,
    pub user_id: Option<String>,
}

pub async fn delete_user_by_login_id_post(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<DeleteUserPostRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    if let Some(ref login_id) = req.login_id {
        let mut users = state.users.write().await;
        // Try loginId first, fall back to userId (same pattern as load)
        if users.load(login_id).is_ok() {
            users.delete_by_login_id(login_id);
        } else {
            users.delete_by_user_id(login_id);
        }
    } else if let Some(ref user_id) = req.user_id {
        state.users.write().await.delete_by_user_id(user_id);
    }
    Ok(Json(json!({ "ok": true })))
}

// ─── Add tenant ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTenantRequest {
    pub login_id: String,
    pub tenant_id: String,
    pub role_names: Option<Vec<String>>,
}

pub async fn add_tenant(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<AddTenantRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let name = state
        .tenants
        .read()
        .await
        .load(&req.tenant_id)
        .map(|t| t.name.clone())
        .unwrap_or_else(|_| req.tenant_id.clone());
    let tenant = UserTenant {
        tenant_id: req.tenant_id,
        tenant_name: name,
        role_names: req.role_names.unwrap_or_default(),
    };
    state
        .users
        .write()
        .await
        .add_tenant(&req.login_id, tenant)?;
    Ok(Json(json!({ "ok": true })))
}

// ─── Generate embedded link ───────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedLinkRequest {
    pub login_id: String,
}

pub async fn generate_embedded_link(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<EmbeddedLinkRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let users = state.users.read().await;
    let user = users.load(&req.login_id)?;
    let uid = user.user_id.clone();
    drop(users);

    let token = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(token.clone(), uid, TokenType::Embedded);
    Ok(Json(json!({ "token": token })))
}

// ─── Generate magic link for test user ───────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestMagicLinkRequest {
    pub login_id: String,
    #[serde(rename = "URI")]
    pub uri: Option<String>,
}

pub async fn generate_magic_link_for_test_user(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<TestMagicLinkRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let users = state.users.read().await;
    let user = users.load(&req.login_id)?;
    // Emulator: skip test-user check — all users can use test magic links.
    // Real Descope enforces this, but the emulator relaxes it to support
    // test patterns that create users via regular mgmt API then login via magic link.
    let uid = user.user_id.clone();
    let email = user.email.clone().unwrap_or_default();
    drop(users);

    let token = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(token.clone(), uid, TokenType::Magic);
    tracing::info!(login_id = %req.login_id, token = %token, "🧪 Test magic link generated");

    let base = req.uri.as_deref().unwrap_or("http://localhost/verify");
    let link = format!("{base}?t={token}");

    Ok(Json(json!({
        "link": link,
        "token": token,
        "maskedEmail": email
    })))
}

// ─── Generate OTP for test user ──────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestOtpRequest {
    pub login_id: String,
    pub delivery_method: Option<String>,
}

pub async fn generate_otp_for_test_user(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<TestOtpRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let user_id = {
        let users = state.users.read().await;
        let user = users.load(&req.login_id)?;
        user.user_id.clone()
    };
    // Emulator: skip test-user check (see generate_magic_link_for_test_user)
    let code = crate::store::otp_store::generate_otp_code();
    state.otps.write().await.store(&user_id, code.clone());
    tracing::info!(login_id = %req.login_id, code = %code, "🧪 Test OTP generated");
    Ok(Json(json!({ "code": code, "loginId": req.login_id })))
}

// ─── User status (enable / disable) ──────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusUpdateRequest {
    pub login_id: String,
    pub status: String,
}

pub async fn status_update(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<StatusUpdateRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let mut users = state.users.write().await;
    let user = match req.status.as_str() {
        "enabled" => users.enable(&req.login_id)?,
        "disabled" => users.disable(&req.login_id)?,
        "invited" => users.set_status(&req.login_id, "invited")?,
        _ => {
            return Err(EmulatorError::Internal(format!(
                "unknown status: {}",
                req.status
            )))
        }
    };
    Ok(Json(json!({ "user": user.to_response() })))
}

// ─── Tenant remove ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantRemoveRequest {
    pub login_id: String,
    pub tenant_id: String,
}

pub async fn tenant_remove(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<TenantRemoveRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let mut users = state.users.write().await;
    let user = users.remove_tenant(&req.login_id, &req.tenant_id)?;
    Ok(Json(json!({ "user": user.to_response() })))
}

// ─── Tenant setRole ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantSetRoleRequest {
    pub login_id: String,
    pub tenant_id: String,
    pub role_names: Vec<String>,
}

pub async fn tenant_set_role(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<TenantSetRoleRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let mut users = state.users.write().await;
    let user = users.set_tenant_roles(&req.login_id, &req.tenant_id, req.role_names)?;
    Ok(Json(json!({ "user": user.to_response() })))
}

// ─── Add roles (append without replacing) ────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRolesRequest {
    pub login_id: String,
    pub role_names: Vec<String>,
}

pub async fn add_roles(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<AddRolesRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let mut users = state.users.write().await;
    let user = users.add_roles(&req.login_id, req.role_names)?;
    Ok(Json(json!({ "user": user.to_response() })))
}

// ─── Update picture ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePictureRequest {
    pub login_id: String,
    pub picture: String,
}

pub async fn update_picture(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<UpdatePictureRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let mut users = state.users.write().await;
    let user = users.update_picture(&req.login_id, req.picture)?;
    Ok(Json(json!({ "user": user.to_response() })))
}

// ─── Update custom attribute ─────────────────────────────────────────────────

/// Descope's API uses `attribute` (not `attributeKey`) for the custom attribute key.
/// Both `attribute` and `attributeKey` are accepted for backwards compatibility.
#[derive(Deserialize)]
pub struct UpdateCustomAttributeRequest {
    #[serde(alias = "loginId")]
    pub login_id: String,
    /// Descope sends `attribute` in its API (SDK sends `attributeKey`).
    #[serde(alias = "attributeKey")]
    pub attribute: String,
    #[serde(rename = "attributeValue")]
    pub attribute_value: Value,
}

pub async fn update_custom_attribute(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<UpdateCustomAttributeRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let mut users = state.users.write().await;
    let user = users.update_custom_attribute(&req.login_id, req.attribute, req.attribute_value)?;
    Ok(Json(json!({ "user": user.to_response() })))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::EmulatorConfig,
        state::EmulatorState,
        store::user_store::new_user_id,
        types::{User, UserTenant},
    };

    async fn make_state() -> EmulatorState {
        let config = EmulatorConfig::default();
        EmulatorState::new(&config).await.unwrap()
    }

    fn make_headers(state: &EmulatorState) -> axum::http::HeaderMap {
        let mut h = axum::http::HeaderMap::new();
        let val = format!(
            "Bearer {}:{}",
            state.config.project_id, state.config.management_key
        );
        h.insert("Authorization", val.parse().unwrap());
        h
    }

    async fn insert_user(state: &EmulatorState, login_id: &str) -> String {
        let uid = new_user_id();
        let mut u = User::default();
        u.user_id = uid.clone();
        u.login_ids = vec![login_id.to_string()];
        u.email = Some(login_id.to_string());
        u.status = "enabled".into();
        u.created_time = 0;
        state.users.write().await.insert(u).unwrap();
        uid
    }

    async fn insert_test_user(state: &EmulatorState, login_id: &str) -> String {
        let uid = new_user_id();
        let mut u = User::default();
        u.user_id = uid.clone();
        u.login_ids = vec![login_id.to_string()];
        u.email = Some(login_id.to_string());
        u.status = "enabled".into();
        u._is_test_user = true;
        u.created_time = 0;
        state.users.write().await.insert(u).unwrap();
        uid
    }

    // ─── generate_otp_for_test_user ───

    #[tokio::test]
    async fn generate_otp_for_test_user_returns_code() {
        let state = make_state().await;
        insert_test_user(&state, "testuser@test.com").await;
        let result = generate_otp_for_test_user(
            State(state.clone()),
            make_headers(&state),
            PermissiveJson(TestOtpRequest {
                login_id: "testuser@test.com".into(),
                delivery_method: Some("email".into()),
            }),
        )
        .await
        .unwrap();
        let code = result["code"].as_str().unwrap();
        assert_eq!(code.len(), 6);
        assert_eq!(result["loginId"].as_str().unwrap(), "testuser@test.com");
    }

    #[tokio::test]
    async fn generate_otp_for_non_test_user_succeeds_in_emulator() {
        let state = make_state().await;
        insert_user(&state, "regular@test.com").await;
        // Emulator relaxes the test-user check — OTP generation works for all users
        let result = generate_otp_for_test_user(
            State(state.clone()),
            make_headers(&state),
            PermissiveJson(TestOtpRequest {
                login_id: "regular@test.com".into(),
                delivery_method: None,
            }),
        )
        .await
        .unwrap();
        let code = result["code"].as_str().unwrap();
        assert_eq!(code.len(), 6);
    }

    // ─── create_test idempotency ───

    #[tokio::test]
    async fn create_test_user_is_idempotent_on_duplicate_login_id() {
        let state = make_state().await;
        let req = || {
            PermissiveJson(CreateUserRequest {
                login_id: Some("dup@test.com".into()),
                email: Some("dup@test.com".into()),
                test: true,
                ..Default::default()
            })
        };
        // First create succeeds.
        create_test(State(state.clone()), make_headers(&state), req())
            .await
            .unwrap();
        // Re-creating the same loginId (a Playwright retry re-running e2e setup,
        // where the prior attempt's user still lives in the emulator) must succeed
        // and return the existing user rather than erroring UserAlreadyExists.
        let second = create_test(State(state.clone()), make_headers(&state), req())
            .await
            .unwrap();
        assert_eq!(second["user"]["email"].as_str().unwrap(), "dup@test.com");
    }

    // Why: on the seed path, the backend stores a user's `uid` custom attribute as
    //      the descopeId and resolves admins via findById(descopeId)/loadByUserId.
    //      If the emulator assigns its own random userId, that lookup misses and
    //      admin bootstrap aborts with "Unable to find and validate existing admin
    //      user" (e2e shards — the admin permissions modal never opens). The seeded
    //      userId must equal the provided uid.
    // Decision: with use_uid_as_user_id=true, a non-empty customAttributes.uid
    //           becomes the userId so descopeId and userId stay aligned.
    #[tokio::test]
    async fn seed_path_uses_uid_custom_attribute_as_user_id() {
        let state = make_state().await;
        let mut attrs = std::collections::HashMap::new();
        attrs.insert("uid".to_string(), serde_json::json!("canonical-uid-123"));
        let resp = create_user_impl(
            &state,
            CreateUserRequest {
                login_id: Some("admin@test.com".into()),
                email: Some("admin@test.com".into()),
                custom_attributes: Some(attrs),
                ..Default::default()
            },
            false,
            true, // seed path
        )
        .await
        .unwrap();
        assert_eq!(
            resp["user"]["userId"].as_str().unwrap(),
            "canonical-uid-123"
        );
    }

    // Why: runtime createTestUser must NOT adopt the uid as the userId. The
    //      backend's createTestUser sets dto.descopeId = resp.userId, so the two
    //      already agree; overriding the userId with the uid broke the support
    //      change-password / cOTP flow.
    // Decision: with use_uid_as_user_id=false (the create/create_test routes),
    //           the userId is generated regardless of any provided uid.
    #[tokio::test]
    async fn runtime_create_generates_user_id_ignoring_uid() {
        let state = make_state().await;
        let mut attrs = std::collections::HashMap::new();
        attrs.insert("uid".to_string(), serde_json::json!("runtime-uid-xyz"));
        let resp = create_test(
            State(state.clone()),
            make_headers(&state),
            PermissiveJson(CreateUserRequest {
                login_id: Some("runtime@test.com".into()),
                email: Some("runtime@test.com".into()),
                custom_attributes: Some(attrs),
                ..Default::default()
            }),
        )
        .await
        .unwrap();
        assert_ne!(resp["user"]["userId"].as_str().unwrap(), "runtime-uid-xyz");
    }

    // ─── status_update ───

    #[tokio::test]
    async fn status_update_disables_then_enables_user() {
        let state = make_state().await;
        insert_user(&state, "kirk@test.com").await;

        let result = status_update(
            State(state.clone()),
            make_headers(&state),
            PermissiveJson(StatusUpdateRequest {
                login_id: "kirk@test.com".into(),
                status: "disabled".into(),
            }),
        )
        .await
        .unwrap();
        assert_eq!(result["user"]["status"].as_str().unwrap(), "disabled");

        let result = status_update(
            State(state.clone()),
            make_headers(&state),
            PermissiveJson(StatusUpdateRequest {
                login_id: "kirk@test.com".into(),
                status: "enabled".into(),
            }),
        )
        .await
        .unwrap();
        assert_eq!(result["user"]["status"].as_str().unwrap(), "enabled");
    }

    #[tokio::test]
    async fn status_update_unknown_user_returns_not_found() {
        let state = make_state().await;
        let err = status_update(
            State(state.clone()),
            make_headers(&state),
            PermissiveJson(StatusUpdateRequest {
                login_id: "ghost@test.com".into(),
                status: "disabled".into(),
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::UserNotFound));
    }

    // ─── tenant_remove ───

    #[tokio::test]
    async fn tenant_remove_removes_entry() {
        let state = make_state().await;
        insert_user(&state, "leo@test.com").await;
        state
            .users
            .write()
            .await
            .add_tenant(
                "leo@test.com",
                UserTenant {
                    tenant_id: "t1".into(),
                    tenant_name: "Corp".into(),
                    role_names: vec![],
                },
            )
            .unwrap();
        let result = tenant_remove(
            State(state.clone()),
            make_headers(&state),
            PermissiveJson(TenantRemoveRequest {
                login_id: "leo@test.com".into(),
                tenant_id: "t1".into(),
            }),
        )
        .await
        .unwrap();
        let tenants = result["user"]["userTenants"].as_array().unwrap();
        assert!(tenants.is_empty());
    }

    #[tokio::test]
    async fn tenant_remove_is_idempotent() {
        let state = make_state().await;
        insert_user(&state, "mia@test.com").await;
        // Call remove on a tenant the user is not in — should succeed (idempotent)
        let _ = tenant_remove(
            State(state.clone()),
            make_headers(&state),
            PermissiveJson(TenantRemoveRequest {
                login_id: "mia@test.com".into(),
                tenant_id: "nonexistent".into(),
            }),
        )
        .await
        .unwrap();
    }

    // ─── tenant_set_role ───

    #[tokio::test]
    async fn tenant_set_role_replaces_roles() {
        let state = make_state().await;
        insert_user(&state, "noah@test.com").await;
        state
            .users
            .write()
            .await
            .add_tenant(
                "noah@test.com",
                UserTenant {
                    tenant_id: "t2".into(),
                    tenant_name: "Corp".into(),
                    role_names: vec!["viewer".into()],
                },
            )
            .unwrap();
        let result = tenant_set_role(
            State(state.clone()),
            make_headers(&state),
            PermissiveJson(TenantSetRoleRequest {
                login_id: "noah@test.com".into(),
                tenant_id: "t2".into(),
                role_names: vec!["admin".into()],
            }),
        )
        .await
        .unwrap();
        let roles = &result["user"]["userTenants"][0]["roleNames"];
        assert_eq!(roles.as_array().unwrap()[0].as_str().unwrap(), "admin");
    }

    #[tokio::test]
    async fn tenant_set_role_fails_if_not_in_tenant() {
        let state = make_state().await;
        insert_user(&state, "olive@test.com").await;
        let err = tenant_set_role(
            State(state.clone()),
            make_headers(&state),
            PermissiveJson(TenantSetRoleRequest {
                login_id: "olive@test.com".into(),
                tenant_id: "missing".into(),
                role_names: vec![],
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::TenantNotFound));
    }
}

// ─── Update name ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNameRequest {
    pub login_id: String,
    pub name: String,
}

pub async fn update_name(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<UpdateNameRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let mut users = state.users.write().await;
    let user = users.patch(
        &req.login_id,
        UserPatch {
            name: Some(req.name),
            ..Default::default()
        },
    )?;
    Ok(Json(json!({ "user": user.to_response() })))
}

// ─── Update phone ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePhoneFieldRequest {
    pub login_id: String,
    pub phone: String,
    /// Real Descope's updatePhone sets the phone's verified status. Honor it so
    /// callers can provision an unverified phone (e.g. e2e phone-verification
    /// flows). Absent → leave verified_phone unchanged.
    pub verified: Option<bool>,
}

pub async fn update_phone_field(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<UpdatePhoneFieldRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let mut users = state.users.write().await;
    let user = users.patch(
        &req.login_id,
        UserPatch {
            phone: Some(req.phone),
            verified_phone: req.verified,
            ..Default::default()
        },
    )?;
    Ok(Json(json!({ "user": user.to_response() })))
}

// ─── Update loginId ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLoginIdRequest {
    pub login_id: String,
    pub new_login_id: String,
}

pub async fn update_login_id(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<UpdateLoginIdRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let mut users = state.users.write().await;
    let user = users.update_login_id(&req.login_id, &req.new_login_id)?;
    Ok(Json(json!({ "user": user.to_response() })))
}

// ─── Set/Remove global roles ──────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RolesRequest {
    pub login_id: String,
    pub role_names: Vec<String>,
}

pub async fn set_roles(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<RolesRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let mut users = state.users.write().await;
    let user = users.set_roles(&req.login_id, req.role_names)?;
    Ok(Json(json!({ "user": user.to_response() })))
}

pub async fn remove_roles(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<RolesRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let mut users = state.users.write().await;
    let user = users.remove_roles(&req.login_id, &req.role_names)?;
    Ok(Json(json!({ "user": user.to_response() })))
}

// ─── Batch Create ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCreateRequest {
    pub users: Vec<CreateUserRequest>,
}

pub async fn create_batch(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<BatchCreateRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let mut created = vec![];
    for user_req in req.users {
        let r = create_user_impl(&state, user_req, false, false).await?;
        // create_user_impl returns Json(json!({"user": {...}})) — extract the inner user object
        // so createdUsers is an array of flat user objects (what the Node SDK expects)
        let user_val = r.0["user"].clone();
        created.push(user_val);
    }
    Ok(Json(json!({ "createdUsers": created, "failedUsers": [] })))
}

// ─── Batch Delete ─────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteRequest {
    // SDK sends userIds (user IDs, not login IDs)
    pub user_ids: Option<Vec<String>>,
    // Legacy: login_ids
    pub login_ids: Option<Vec<String>>,
}

pub async fn delete_batch(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<BatchDeleteRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let mut users = state.users.write().await;
    if let Some(ref user_ids) = req.user_ids {
        for user_id in user_ids {
            users.delete_by_user_id(user_id);
        }
    } else if let Some(ref login_ids) = req.login_ids {
        for login_id in login_ids {
            users.delete_by_login_id(login_id);
        }
    }
    Ok(Json(json!({ "ok": true })))
}

// ─── Force Logout ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceLogoutRequest {
    pub login_id: String,
}

pub async fn force_logout(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<ForceLogoutRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let users = state.users.read().await;
    let user = users.load(&req.login_id)?;
    let uid = user.user_id.clone();
    drop(users);
    let ts = now() + 1; // +1 so iat < revoked_at holds for tokens issued in the same second
    state
        .user_revocations
        .write()
        .await
        .entry(uid)
        .and_modify(|v| {
            if ts > *v {
                *v = ts;
            }
        })
        .or_insert(ts);
    Ok(Json(json!({ "ok": true })))
}

// ─── Password Expire (stub) ───────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordExpireRequest {
    pub login_id: String,
}

pub async fn password_expire(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<PasswordExpireRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state.users.read().await.load(&req.login_id)?;
    Ok(Json(json!({ "ok": true })))
}

// ─── Set Temporary Password ───────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTemporaryPasswordRequest {
    pub login_id: String,
    pub password: String,
}

pub async fn set_temporary_password(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<SetTemporaryPasswordRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    // Enforce password policy
    {
        let cfg = state.auth_method_config.read().await;
        let policy = &cfg.config.password;
        if (req.password.len() as u32) < policy.min_length {
            return Err(EmulatorError::PasswordPolicy(format!(
                "Password must be at least {} characters",
                policy.min_length
            )));
        }
    }
    let hash = {
        let pwd = req.password.clone();
        tokio::task::spawn_blocking(move || {
            bcrypt::hash(&pwd, 10).map_err(|e| EmulatorError::Internal(e.to_string()))
        })
        .await
        .map_err(|e| EmulatorError::Internal(e.to_string()))??
    };
    state
        .users
        .write()
        .await
        .set_password(&req.login_id, hash)?;
    Ok(Json(json!({ "ok": true })))
}

// ─── Generate Enchanted Link for Test User ────────────────────────────────────

pub async fn generate_enchanted_link_for_test_user(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<TestMagicLinkRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let users = state.users.read().await;
    let user = users.load(&req.login_id)?;
    // Emulator: skip test-user check (see generate_magic_link_for_test_user)
    let uid = user.user_id.clone();
    let email = user.email.clone().unwrap_or_default();
    drop(users);

    let token = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(token.clone(), uid, crate::types::TokenType::Magic);
    tracing::info!(login_id = %req.login_id, token = %token, "🧪 Test enchanted link generated");

    let base = req.uri.as_deref().unwrap_or("http://localhost/verify");
    let link = format!("{base}?t={token}");

    Ok(Json(json!({
        "link": link,
        "token": token,
        "maskedEmail": email
    })))
}
