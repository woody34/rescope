/// rescope-wasm: WASM bridge for the Rescope emulator.
///
/// Exposes two entry points to JavaScript:
/// - `init(config_json)` — initialize the emulator with a config
/// - `handle_request(request_json)` → response_json — process an API request

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

use rescope_core::{
    error::EmulatorError,
    store::{
        access_key_store::{AccessKeyStore, TenantRoleBinding},
        auth_method_config::{AuthMethodConfig, AuthMethodConfigStore},
        connector_store::{ConnectorStore, ConnectorType},
        custom_attribute_store::{CustomAttributeStore, AttributeType, AttributePermissions},
        idp_store::{IdpStore, IdpEmulator},
        jwt_template_store::{JwtTemplate, JwtTemplateStore},
        otp_store::{OtpStore, generate_otp_code},
        permission_store::PermissionStore,
        revocation_store::RevocationStore,
        role_store::RoleStore,
        tenant_store::TenantStore,
        token_store::TokenStore,
        user_store::{UserStore, UserUpdate},
    },
    types::{User, UserTenant},
};

// ── Request / Response types for the JS boundary ─────────────────────────────

#[derive(Debug, Deserialize)]
pub struct WasmRequest {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WasmResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl WasmResponse {
    fn json(status: u16, body: impl Serialize) -> Self {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        Self {
            status,
            headers,
            body: serde_json::to_string(&body).unwrap_or_default(),
        }
    }

    fn from_error(err: EmulatorError) -> Self {
        let (status, _, _) = err.status_and_code();
        let body = err.to_json_body();
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        Self { status, headers, body }
    }

    fn ok() -> Self {
        Self::json(200, serde_json::json!({}))
    }
}

// ── Global state ─────────────────────────────────────────────────────────────

struct EmulatorState {
    users: UserStore,
    tenants: TenantStore,
    #[allow(dead_code)] tokens: TokenStore,
    #[allow(dead_code)] revoked: RevocationStore,
    otps: OtpStore,
    permissions: PermissionStore,
    roles: RoleStore,
    auth_method_config: AuthMethodConfigStore,
    jwt_templates: JwtTemplateStore,
    connectors: ConnectorStore,
    custom_attributes: CustomAttributeStore,
    access_keys: AccessKeyStore,
    idp_emulators: IdpStore,
}

impl EmulatorState {
    fn new() -> Self {
        Self {
            users: UserStore::new(),
            tenants: TenantStore::new(),
            tokens: TokenStore::new(),
            revoked: RevocationStore::new(),
            otps: OtpStore::new(),
            permissions: PermissionStore::new(),
            roles: RoleStore::new(),
            auth_method_config: AuthMethodConfigStore::new(),
            jwt_templates: JwtTemplateStore::new(),
            connectors: ConnectorStore::new(),
            custom_attributes: CustomAttributeStore::new(),
            access_keys: AccessKeyStore::new(),
            idp_emulators: IdpStore::new(),
        }
    }
}

static STATE: RwLock<Option<EmulatorState>> = RwLock::new(None);

// ── Exported WASM API ──────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn init(config_json: &str) -> Result<(), JsValue> {
    let _config: serde_json::Value = serde_json::from_str(config_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid config: {e}")))?;
    *STATE.write().unwrap() = Some(EmulatorState::new());
    Ok(())
}

#[wasm_bindgen]
pub fn handle_request(request_json: &str) -> String {
    let req: WasmRequest = match serde_json::from_str(request_json) {
        Ok(r) => r,
        Err(e) => {
            let resp = WasmResponse::json(400, serde_json::json!({"error": format!("Invalid request: {e}")}));
            return serde_json::to_string(&resp).unwrap();
        }
    };
    let response = route(&req);
    serde_json::to_string(&response).unwrap()
}

// ── Router ─────────────────────────────────────────────────────────────────────

fn route(req: &WasmRequest) -> WasmResponse {
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/health") => WasmResponse::json(200, serde_json::json!({"status": "ok", "wasm": true})),

        // Users
        ("POST", "/v1/mgmt/user/create") => handle_create_user(req),
        ("POST", "/v1/mgmt/user/delete") => handle_delete_user(req),
        ("POST", "/v1/mgmt/user/search") | ("POST", "/v2/mgmt/user/search") => handle_search_users(req),
        ("POST", "/v1/mgmt/user/update") => handle_update_user(req),
        ("POST", "/v1/mgmt/user/patch") => handle_update_user(req),
        ("POST", "/v1/mgmt/user/update/status") => handle_set_status(req),

        // Tenants
        ("POST", "/v1/mgmt/tenant/create") => handle_create_tenant(req),
        ("POST", "/v1/mgmt/tenant/delete") => handle_delete_tenant(req),
        ("GET", "/v1/mgmt/tenant/all") => handle_list_tenants(),
        ("POST", "/v1/mgmt/tenant/search") => handle_list_tenants(),
        ("POST", "/v1/mgmt/tenant/update") => handle_update_tenant(req),

        // Permissions
        ("GET", "/v1/mgmt/authz/permission/all") => handle_list_permissions(),
        ("POST", "/v1/mgmt/authz/permission") => handle_create_permission(req),
        ("POST", "/v1/mgmt/authz/permission/delete") => handle_delete_permission(req),
        ("POST", "/v1/mgmt/authz/permission/update") => handle_update_permission(req),

        // Roles
        ("GET", "/v1/mgmt/authz/role/all") => handle_list_roles(),
        ("POST", "/v1/mgmt/authz/role") => handle_create_role(req),
        ("POST", "/v1/mgmt/authz/role/delete") => handle_delete_role(req),
        ("POST", "/v1/mgmt/authz/role/update") => handle_update_role(req),

        // Auth methods
        ("GET", "/v1/mgmt/config/auth-methods") => handle_get_auth_methods(),
        ("PUT", "/v1/mgmt/config/auth-methods") => handle_update_auth_methods(req),

        // JWT Templates
        ("GET", "/v1/mgmt/jwt/template/all") => handle_list_jwt_templates(),
        ("GET", "/v1/mgmt/jwt/template/active") => handle_get_active_jwt_template(),
        ("POST", "/v1/mgmt/jwt/template") => handle_create_jwt_template(req),
        ("POST", "/v1/mgmt/jwt/template/delete") => handle_delete_jwt_template(req),

        // Connectors
        ("GET", "/v1/mgmt/connector/all") => handle_list_connectors(),
        ("POST", "/v1/mgmt/connector") => handle_create_connector(req),
        ("POST", "/v1/mgmt/connector/delete") => handle_delete_connector(req),

        // Custom Attributes
        ("GET", "/v1/mgmt/user/attribute/all") => handle_list_custom_attributes(),
        ("POST", "/v1/mgmt/user/attribute") => handle_create_custom_attribute(req),
        ("POST", "/v1/mgmt/user/attribute/delete") => handle_delete_custom_attribute(req),

        // Access Keys
        ("GET", "/v1/mgmt/accesskey/all") => handle_list_access_keys(),
        ("POST", "/v1/mgmt/accesskey") => handle_create_access_key(req),
        ("POST", "/v1/mgmt/accesskey/delete") => handle_delete_access_key(req),

        // IdP Emulators
        ("GET", "/v1/mgmt/idp/all") => handle_list_idps(),
        ("POST", "/v1/mgmt/idp") => handle_create_idp(req),
        ("POST", "/v1/mgmt/idp/delete") => handle_delete_idp(req),

        // Auth — password
        ("POST", "/v1/auth/password/signin") => handle_password_signin(req),
        ("POST", "/v1/auth/password/signup") => handle_password_signup(req),

        // Auth — OTP
        ("POST", "/v1/auth/otp/signin/email") => handle_otp_signin_email(req),
        ("POST", "/v1/auth/otp/verify/email") => handle_otp_verify_email(req),

        // Auth — Magic Link
        ("POST", "/v1/auth/magiclink/signin/email") => handle_magiclink_signin_email(req),
        ("POST", "/v1/auth/magiclink/verify") => handle_magiclink_verify(req),

        // Batch user create (used by seed)
        ("POST", "/v1/mgmt/user/create/batch") => handle_create_users_batch(req),

        // Emulator controls
        ("POST", "/emulator/reset") => handle_reset(),
        ("GET", "/emulator/snapshot") => handle_snapshot(),
        ("GET", "/emulator/otps") => handle_get_otps(),

        // Emulator OTP retrieval by loginId (for demo UI)
        ("GET", path) if path.starts_with("/emulator/otp/") => {
            let login_id = path.trim_start_matches("/emulator/otp/");
            handle_get_otp_for_login(login_id)
        }

        _ => WasmResponse::json(404, serde_json::json!({"error": format!("Not found: {} {}", req.method, req.path)})),
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn parse_body<T: serde::de::DeserializeOwned>(req: &WasmRequest) -> Result<T, WasmResponse> {
    let body = req.body.as_deref().unwrap_or("{}");
    serde_json::from_str(body).map_err(|e|
        WasmResponse::json(400, serde_json::json!({"error": format!("Invalid body: {e}")}))
    )
}

fn with_state<F, R>(f: F) -> R where F: FnOnce(&EmulatorState) -> R {
    let guard = STATE.read().unwrap();
    f(guard.as_ref().expect("Call init() first"))
}

fn with_state_mut<F, R>(f: F) -> R where F: FnOnce(&mut EmulatorState) -> R {
    let mut guard = STATE.write().unwrap();
    f(guard.as_mut().expect("Call init() first"))
}

// ── User handlers ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateUserReq {
    #[serde(default)] login_ids: Vec<String>,
    #[serde(default)] login_id: Option<String>,
    #[serde(default)] email: Option<String>,
    #[serde(default)] phone: Option<String>,
    #[serde(default)] name: Option<String>,
    #[serde(default)] role_names: Vec<String>,
    #[serde(default)] user_tenants: Vec<UserTenant>,
}

fn handle_create_user(req: &WasmRequest) -> WasmResponse {
    let body: CreateUserReq = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    let login_ids = if !body.login_ids.is_empty() { body.login_ids }
        else if let Some(lid) = body.login_id { vec![lid] }
        else { return WasmResponse::json(400, serde_json::json!({"error": "loginId or loginIds required"})) };

    with_state_mut(|state| {
        let user = User {
            user_id: uuid::Uuid::new_v4().to_string(),
            login_ids,
            email: body.email,
            phone: body.phone,
            name: body.name,
            role_names: body.role_names,
            user_tenants: body.user_tenants,
            status: "enabled".to_string(),
            ..Default::default()
        };
        match state.users.insert(user.clone()) {
            Ok(()) => WasmResponse::json(200, serde_json::json!({"user": user.to_response()})),
            Err(e) => WasmResponse::from_error(e),
        }
    })
}

fn handle_delete_user(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { login_id: String }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| { state.users.delete_by_login_id(&body.login_id); WasmResponse::ok() })
}

fn handle_search_users(_req: &WasmRequest) -> WasmResponse {
    with_state(|state| {
        let users: Vec<_> = state.users.all_users().into_iter().map(|u| u.to_response()).collect();
        WasmResponse::json(200, serde_json::json!({"users": users}))
    })
}

fn handle_update_user(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { login_id: String, #[serde(default)] email: Option<String>,
               #[serde(default)] phone: Option<String>, #[serde(default)] name: Option<String>,
               #[serde(default)] role_names: Option<Vec<String>> }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        match state.users.update(&body.login_id, UserUpdate {
            email: body.email, phone: body.phone, name: body.name,
            role_names: body.role_names, ..UserUpdate::default()
        }) {
            Ok(u) => WasmResponse::json(200, serde_json::json!({"user": u.to_response()})),
            Err(e) => WasmResponse::from_error(e),
        }
    })
}

fn handle_set_status(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { login_id: String, status: String }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        match state.users.set_status(&body.login_id, &body.status) {
            Ok(u) => WasmResponse::json(200, serde_json::json!({"user": u.to_response()})),
            Err(e) => WasmResponse::from_error(e),
        }
    })
}

// ── Tenant handlers ──────────────────────────────────────────────────────────

fn handle_create_tenant(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { name: String, #[serde(default)] id: Option<String>,
               #[serde(default)] self_provisioning_domains: Vec<String> }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        match state.tenants.create(body.id, body.name, body.self_provisioning_domains) {
            Ok(id) => WasmResponse::json(200, serde_json::json!({"id": id})),
            Err(e) => WasmResponse::from_error(e),
        }
    })
}

fn handle_delete_tenant(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] struct B { id: String }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| { state.tenants.delete_tenant(&body.id); WasmResponse::ok() })
}

fn handle_list_tenants() -> WasmResponse {
    with_state(|state| WasmResponse::json(200, serde_json::json!({"tenants": state.tenants.load_all()})))
}

fn handle_update_tenant(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { id: String, #[serde(default)] name: Option<String>,
               #[serde(default)] self_provisioning_domains: Option<Vec<String>> }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        match state.tenants.update(&body.id, body.name, body.self_provisioning_domains, None, None, None, None, None, None, None) {
            Ok(()) => WasmResponse::ok(),
            Err(e) => WasmResponse::from_error(e),
        }
    })
}

// ── Permission handlers ──────────────────────────────────────────────────────

fn handle_list_permissions() -> WasmResponse {
    with_state(|state| WasmResponse::json(200, serde_json::json!({"permissions": state.permissions.load_all()})))
}

fn handle_create_permission(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] struct B { name: String, #[serde(default)] description: Option<String> }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        match state.permissions.create(body.name, body.description.unwrap_or_default()) {
            Ok(p) => WasmResponse::json(200, serde_json::json!({"permission": p})),
            Err(e) => WasmResponse::from_error(e),
        }
    })
}

fn handle_delete_permission(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] struct B { name: String }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| { match state.permissions.delete(&body.name) { Ok(()) => WasmResponse::ok(), Err(e) => WasmResponse::from_error(e) } })
}

fn handle_update_permission(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { name: String, #[serde(default)] new_name: Option<String>, #[serde(default)] description: Option<String> }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        match state.permissions.update(&body.name, body.new_name, body.description) { Ok(()) => WasmResponse::ok(), Err(e) => WasmResponse::from_error(e) }
    })
}

// ── Role handlers ────────────────────────────────────────────────────────────

fn handle_list_roles() -> WasmResponse {
    with_state(|state| WasmResponse::json(200, serde_json::json!({"roles": state.roles.load_all()})))
}

fn handle_create_role(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { name: String, #[serde(default)] description: Option<String>, #[serde(default)] permission_names: Vec<String> }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        match state.roles.create(body.name, body.description.unwrap_or_default(), body.permission_names) {
            Ok(r) => WasmResponse::json(200, serde_json::json!({"role": r})),
            Err(e) => WasmResponse::from_error(e),
        }
    })
}

fn handle_delete_role(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] struct B { name: String }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| { match state.roles.delete(&body.name) { Ok(()) => WasmResponse::ok(), Err(e) => WasmResponse::from_error(e) } })
}

fn handle_update_role(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { name: String, #[serde(default)] new_name: Option<String>,
               #[serde(default)] description: Option<String>,
               #[serde(default)] permission_names: Option<Vec<String>> }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        match state.roles.update(&body.name, body.new_name, body.description, body.permission_names, None, None) {
            Ok(()) => WasmResponse::ok(), Err(e) => WasmResponse::from_error(e),
        }
    })
}

// ── Auth Method Config ───────────────────────────────────────────────────────

fn handle_get_auth_methods() -> WasmResponse {
    with_state(|state| WasmResponse::json(200, serde_json::json!({"authMethods": state.auth_method_config.get()})))
}

fn handle_update_auth_methods(req: &WasmRequest) -> WasmResponse {
    let body: AuthMethodConfig = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| { state.auth_method_config.replace(body); WasmResponse::ok() })
}

// ── JWT Templates ────────────────────────────────────────────────────────────

fn handle_list_jwt_templates() -> WasmResponse {
    with_state(|state| WasmResponse::json(200, serde_json::json!({"templates": state.jwt_templates.load_all()})))
}

fn handle_get_active_jwt_template() -> WasmResponse {
    with_state(|state| WasmResponse::json(200, serde_json::json!({"template": state.jwt_templates.active()})))
}

fn handle_create_jwt_template(req: &WasmRequest) -> WasmResponse {
    let body: JwtTemplate = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        match state.jwt_templates.create(body) {
            Ok(t) => WasmResponse::json(200, serde_json::json!({"template": t})),
            Err(e) => WasmResponse::from_error(e),
        }
    })
}

fn handle_delete_jwt_template(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] struct B { id: String }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| { match state.jwt_templates.delete(&body.id) { Ok(()) => WasmResponse::ok(), Err(e) => WasmResponse::from_error(e) } })
}

// ── Connectors ───────────────────────────────────────────────────────────────

fn handle_list_connectors() -> WasmResponse {
    with_state(|state| WasmResponse::json(200, serde_json::json!({"connectors": state.connectors.load_all()})))
}

fn handle_create_connector(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { name: String, #[serde(rename = "type")] connector_type: ConnectorType, #[serde(default)] config: serde_json::Value }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        match state.connectors.create(body.name, body.connector_type, body.config) {
            Ok(c) => WasmResponse::json(200, serde_json::json!({"connector": c})),
            Err(e) => WasmResponse::from_error(e),
        }
    })
}

fn handle_delete_connector(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] struct B { id: String }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| { match state.connectors.delete(&body.id) { Ok(()) => WasmResponse::ok(), Err(e) => WasmResponse::from_error(e) } })
}

// ── Custom Attributes ────────────────────────────────────────────────────────

fn handle_list_custom_attributes() -> WasmResponse {
    with_state(|state| WasmResponse::json(200, serde_json::json!({"attributes": state.custom_attributes.load_all()})))
}

fn handle_create_custom_attribute(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { name: String, machine_name: String, attribute_type: AttributeType, permissions: AttributePermissions }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        match state.custom_attributes.create(body.name, body.machine_name, body.attribute_type, body.permissions) {
            Ok(a) => WasmResponse::json(200, serde_json::json!({"attribute": a})),
            Err(e) => WasmResponse::from_error(e),
        }
    })
}

fn handle_delete_custom_attribute(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { machine_name: String }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| { match state.custom_attributes.delete(&body.machine_name) { Ok(()) => WasmResponse::ok(), Err(e) => WasmResponse::from_error(e) } })
}

// ── Access Keys ──────────────────────────────────────────────────────────────

fn handle_list_access_keys() -> WasmResponse {
    with_state(|state| WasmResponse::json(200, serde_json::json!({"keys": state.access_keys.load_all()})))
}

fn handle_create_access_key(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { name: String, #[serde(default)] expire_time: Option<u64>,
               #[serde(default)] permitted_ips: Vec<String>,
               #[serde(default)] role_names: Vec<String>,
               #[serde(default)] key_tenants: Vec<TenantRoleBinding> }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        match state.access_keys.create(body.name, body.expire_time, body.permitted_ips, body.role_names, body.key_tenants, "playground".into()) {
            Ok((key, cleartext)) => WasmResponse::json(200, serde_json::json!({"key": key, "cleartext": cleartext})),
            Err(e) => WasmResponse::from_error(e),
        }
    })
}

fn handle_delete_access_key(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] struct B { id: String }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| { match state.access_keys.delete(&body.id) { Ok(()) => WasmResponse::ok(), Err(e) => WasmResponse::from_error(e) } })
}

// ── IdP Emulators ────────────────────────────────────────────────────────────

fn handle_list_idps() -> WasmResponse {
    with_state(|state| WasmResponse::json(200, serde_json::json!({"idps": state.idp_emulators.list()})))
}

fn handle_create_idp(req: &WasmRequest) -> WasmResponse {
    let body: IdpEmulator = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        match state.idp_emulators.insert(body) {
            Ok(idp) => WasmResponse::json(200, serde_json::json!({"idp": idp})),
            Err(e) => WasmResponse::from_error(e),
        }
    })
}

fn handle_delete_idp(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] struct B { id: String }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| { match state.idp_emulators.delete(&body.id) { Ok(()) => WasmResponse::ok(), Err(e) => WasmResponse::from_error(e) } })
}

// ── Emulator controls ────────────────────────────────────────────────────────

fn handle_reset() -> WasmResponse {
    *STATE.write().unwrap() = Some(EmulatorState::new());
    WasmResponse::ok()
}

fn handle_snapshot() -> WasmResponse {
    with_state(|state| {
        WasmResponse::json(200, serde_json::json!({
            "users": state.users.all_users().into_iter().map(|u| u.to_response()).collect::<Vec<_>>(),
            "tenants": state.tenants.load_all(),
            "permissions": state.permissions.load_all(),
            "roles": state.roles.load_all(),
            "authMethodConfig": state.auth_method_config.get(),
            "jwtTemplates": state.jwt_templates.load_all(),
            "connectors": state.connectors.load_all(),
            "customAttributes": state.custom_attributes.load_all(),
            "accessKeys": state.access_keys.load_all(),
        }))
    })
}

fn handle_get_otps() -> WasmResponse {
    with_state(|state| WasmResponse::json(200, serde_json::json!({"otps": state.otps.list_all()})))
}

// ── Auth helpers ─────────────────────────────────────────────────────────────

/// Build a fake-but-structurally-valid session response for the emulator.
fn fake_session(user_id: &str, email: &str) -> serde_json::Value {
    // Header.Payload.Signature — all base64url fakes for demo purposes only
    let header = "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImRlbW8ifQ";
    let payload = base64_encode(&serde_json::json!({
        "sub": user_id,
        "email": email,
        "iss": "rescope-demo",
        "iat": 0,
        "exp": 9999999999_u64,
    }).to_string());
    let token = format!("{header}.{payload}.DEMO_SIGNATURE");
    serde_json::json!({
        "sessionJwt": token,
        "refreshJwt": token,
        "user": { "userId": user_id, "email": email },
        "firstSeen": false,
    })
}

fn base64_encode(s: &str) -> String {
    // Simple base64url without padding for demo JWTs
    let bytes = s.as_bytes();
    let encoded = bytes.iter().fold(String::new(), |mut acc, b| {
        acc.push_str(&format!("{:02x}", b));
        acc
    });
    encoded
}

// ── Auth handlers ─────────────────────────────────────────────────────────────

fn handle_password_signin(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { login_id: String, password: String }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state(|state| {
        match state.users.load(&body.login_id) {
            Ok(user) => {
                let _ = body.password; // accept any password in emulator
                let email = user.email.clone().unwrap_or_else(|| body.login_id.clone());
                WasmResponse::json(200, fake_session(&user.user_id, &email))
            }
            Err(_) => WasmResponse::json(400, serde_json::json!({"errorCode": "E111001", "errorDescription": "User not found"}))
        }
    })
}

fn handle_password_signup(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { login_id: String, #[serde(default)] email: Option<String>, #[allow(dead_code)] #[serde(default)] password: Option<String> }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    let email = body.email.clone().unwrap_or_else(|| body.login_id.clone());
    with_state_mut(|state| {
        let user = User {
            user_id: uuid::Uuid::new_v4().to_string(),
            login_ids: vec![body.login_id.clone()],
            email: Some(email.clone()),
            status: "enabled".to_string(),
            ..Default::default()
        };
        let _ = state.users.insert(user.clone());
        WasmResponse::json(200, fake_session(&user.user_id, &email))
    })
}

fn handle_otp_signin_email(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { login_id: String }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        let user_id = match state.users.load(&body.login_id) {
            Ok(u) => u.user_id.clone(),
            Err(_) => {
                let u = User {
                    user_id: uuid::Uuid::new_v4().to_string(),
                    login_ids: vec![body.login_id.clone()],
                    email: Some(body.login_id.clone()),
                    status: "enabled".to_string(),
                    ..Default::default()
                };
                let uid = u.user_id.clone();
                let _ = state.users.insert(u);
                uid
            }
        };
        let code = generate_otp_code();
        state.otps.store(&user_id, code);
        WasmResponse::json(200, serde_json::json!({"maskedEmail": "***@example.com", "pendingRefJwt": "demo"}))
    })
}

fn handle_otp_verify_email(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { login_id: String, code: String }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        let (user_id, email) = match state.users.load(&body.login_id) {
            Ok(u) => (u.user_id.clone(), u.email.clone().unwrap_or_else(|| body.login_id.clone())),
            Err(_) => return WasmResponse::json(400, serde_json::json!({"errorCode": "E111001", "errorDescription": "User not found"}))
        };
        match state.otps.consume(&user_id, &body.code) {
            Ok(()) => WasmResponse::json(200, fake_session(&user_id, &email)),
            Err(_) => WasmResponse::json(401, serde_json::json!({"errorCode": "E061102", "errorDescription": "Invalid OTP code"}))
        }
    })
}

fn handle_magiclink_signin_email(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct B { login_id: String, #[serde(default, rename = "URI")] uri: Option<String> }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        let user_id = match state.users.load(&body.login_id) {
            Ok(u) => u.user_id.clone(),
            Err(_) => {
                let u = User {
                    user_id: uuid::Uuid::new_v4().to_string(),
                    login_ids: vec![body.login_id.clone()],
                    email: Some(body.login_id.clone()),
                    status: "enabled".to_string(),
                    ..Default::default()
                };
                let uid = u.user_id.clone();
                let _ = state.users.insert(u);
                uid
            }
        };
        let token = generate_otp_code();
        state.otps.store(&user_id, token);
        let _ = body.uri;
        WasmResponse::json(200, serde_json::json!({"maskedEmail": "***@example.com", "pendingRefJwt": "demo"}))
    })
}

fn handle_magiclink_verify(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] struct B { token: String }
    let body: B = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        let otps = state.otps.list_all();
        let user_id = otps.iter()
            .find(|(_, code)| code.as_str() == body.token.as_str())
            .map(|(uid, _)| uid.clone());
        match user_id {
            Some(uid) => {
                let _ = state.otps.consume(&uid, &body.token);
                let (email, id) = state.users.load_by_user_id(&uid)
                    .map(|u| (u.email.clone().unwrap_or_default(), u.user_id.clone()))
                    .unwrap_or_else(|_| (String::new(), uid));
                WasmResponse::json(200, fake_session(&id, &email))
            }
            None => WasmResponse::json(401, serde_json::json!({"errorCode": "E062503", "errorDescription": "Invalid or expired magic link token"}))
        }
    })
}

fn handle_get_otp_for_login(login_id: &str) -> WasmResponse {
    with_state(|state| {
        match state.users.load(login_id) {
            Ok(user) => {
                match state.otps.peek(&user.user_id) {
                    Some(code) => WasmResponse::json(200, serde_json::json!({"code": code, "loginId": login_id})),
                    None => WasmResponse::json(404, serde_json::json!({"error": "No pending OTP for this user"}))
                }
            }
            Err(_) => WasmResponse::json(404, serde_json::json!({"error": "User not found"}))
        }
    })
}

// ── Batch user create ─────────────────────────────────────────────────────────

fn handle_create_users_batch(req: &WasmRequest) -> WasmResponse {
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct BatchReq { users: Vec<BatchUser> }
    #[derive(Deserialize)] #[serde(rename_all = "camelCase")]
    struct BatchUser {
        #[serde(default)] login_id: Option<String>,
        #[serde(default)] email: Option<String>,
        #[serde(default)] name: Option<String>,
        #[allow(dead_code)] #[serde(default)] password: Option<String>,
        #[serde(default)] role_names: Vec<String>,
        #[serde(default)] user_tenants: Vec<UserTenant>,
    }
    let body: BatchReq = match parse_body(req) { Ok(b) => b, Err(e) => return e };
    with_state_mut(|state| {
        let mut created = vec![];
        for bu in body.users {
            let login_id = bu.login_id.clone()
                .or_else(|| bu.email.clone())
                .unwrap_or_default();
            if login_id.is_empty() { continue; }
            let user = User {
                user_id: uuid::Uuid::new_v4().to_string(),
                login_ids: vec![login_id],
                email: bu.email,
                name: bu.name,
                role_names: bu.role_names,
                user_tenants: bu.user_tenants,
                status: "enabled".to_string(),
                ..Default::default()
            };
            let _ = bu.password;
            if state.users.insert(user.clone()).is_ok() {
                created.push(user.to_response());
            }
        }
        WasmResponse::json(200, serde_json::json!({"createdUsers": created, "failedUsers": []}))
    })
}
