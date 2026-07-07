use crate::state::EmulatorState;
use axum::http::{header, Method};
use axum::{
    response::Html,
    routing::{delete, get, patch, post},
    Json, Router,
};
use tower_http::cors::{AllowOrigin, CorsLayer};
#[cfg(feature = "dev-ui")]
use tower_http::services::{ServeDir, ServeFile};

pub fn build_router(state: EmulatorState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request())
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ORIGIN,
            header::COOKIE,
            header::HeaderName::from_static("x-descope-project-id"),
            header::HeaderName::from_static("x-descope-sdk-name"),
            header::HeaderName::from_static("x-descope-sdk-version"),
            header::HeaderName::from_static("x-descope-sdk-session-id"),
        ]);

    let router = Router::new()
        // ── Lifecycle ────────────────────────────────────────────────
        .route("/health", get(crate::routes::emulator::health))
        .route("/emulator/reset", post(crate::routes::emulator::reset))
        // ── Emulator escape hatches ───────────────────────────────────
        .route(
            "/emulator/otp/:login_id",
            get(crate::routes::emulator::get_otp),
        )
        .route(
            "/emulator/tenant",
            post(crate::routes::emulator::create_tenant),
        )
        .route(
            "/emulator/seed/users",
            post(crate::routes::emulator::seed_users),
        )
        // ── JWKS ─────────────────────────────────────────────────────
        .route(
            "/.well-known/jwks.json",
            get(crate::routes::jwks::jwks_handler),
        )
        .route(
            "/v2/keys/:project_id",
            get(crate::routes::jwks::jwks_handler),
        )
        // ── Auth: Password ────────────────────────────────────────────
        .route(
            "/v1/auth/password/signup",
            post(crate::routes::auth::password::signup),
        )
        .route(
            "/v1/auth/password/signin",
            post(crate::routes::auth::password::signin),
        )
        .route(
            "/v1/auth/password/replace",
            post(crate::routes::auth::password::replace),
        )
        .route(
            "/v1/auth/password/reset",
            post(crate::routes::auth::password::send_reset),
        )
        .route(
            "/v1/auth/password/update",
            post(crate::routes::auth::password::update_password),
        )
        .route(
            "/v1/auth/password/policy",
            get(crate::routes::auth::password::policy),
        )
        // ── Flow runtime (descope-wc) ─────────────────────────────────
        .route("/v1/flow/start", post(crate::routes::auth::flow::start))
        .route("/v2/flow/start", post(crate::routes::auth::flow::start))
        .route("/v1/flow/next", post(crate::routes::auth::flow::next))
        .route("/v2/flow/next", post(crate::routes::auth::flow::next))
        .route("/pages/*rest", get(crate::routes::flow_assets::serve))
        // ── Auth: Magic Link ──────────────────────────────────────────
        .route(
            "/v1/auth/magiclink/signup/email",
            post(crate::routes::auth::magic_link::signup_email),
        )
        .route(
            "/v1/auth/magiclink/signin/email",
            post(crate::routes::auth::magic_link::signin_email),
        )
        .route(
            "/v1/auth/magiclink/verify",
            post(crate::routes::auth::magic_link::verify),
        )
        .route(
            "/v1/auth/magiclink/update/email",
            post(crate::routes::auth::magic_link::update_email),
        )
        .route(
            "/v1/auth/magiclink/signup/sms",
            post(crate::routes::auth::magic_link::signup_sms),
        )
        .route(
            "/v1/auth/magiclink/signin/sms",
            post(crate::routes::auth::magic_link::signin_sms),
        )
        .route(
            "/v1/auth/magiclink/signup-in/email",
            post(crate::routes::auth::magic_link::signup_in_email),
        )
        .route(
            "/v1/auth/magiclink/signup-in/sms",
            post(crate::routes::auth::magic_link::signup_in_sms),
        )
        .route(
            "/v1/auth/magiclink/update/phone/sms",
            post(crate::routes::auth::magic_link::update_phone_sms),
        )
        // ── Auth: SAML/SSO ────────────────────────────────────────────
        .route(
            "/v1/auth/saml/start",
            post(crate::routes::auth::saml::start),
        )
        .route(
            "/v1/auth/saml/authorize",
            post(crate::routes::auth::saml::start).get(crate::routes::auth::saml::start_get),
        )
        .route(
            "/v1/auth/saml/exchange",
            post(crate::routes::auth::saml::exchange),
        )
        .route(
            "/v1/auth/sso/authorize",
            post(crate::routes::auth::saml::start),
        )
        .route(
            "/v1/auth/sso/exchange",
            post(crate::routes::auth::saml::exchange),
        )
        // ── Auth: OTP ─────────────────────────────────────────────────
        .route(
            "/v1/auth/otp/signup/email",
            post(crate::routes::auth::otp::signup_email),
        )
        .route(
            "/v1/auth/otp/signup/sms",
            post(crate::routes::auth::otp::signup_phone_sms),
        )
        .route(
            "/v1/auth/otp/signin/email",
            post(crate::routes::auth::otp::signin_email),
        )
        .route(
            "/v1/auth/otp/signin/sms",
            post(crate::routes::auth::otp::signin_phone_sms),
        )
        .route(
            "/v1/auth/otp/verify/email",
            post(crate::routes::auth::otp::verify_email),
        )
        .route(
            "/v1/auth/otp/verify/sms",
            post(crate::routes::auth::otp::verify_phone_sms),
        )
        .route(
            "/v1/auth/otp/update/phone/sms",
            post(crate::routes::auth::otp::update_phone),
        )
        .route(
            "/v1/auth/otp/signup-in/email",
            post(crate::routes::auth::otp::signup_in_email),
        )
        .route(
            "/v1/auth/otp/signup-in/sms",
            post(crate::routes::auth::otp::signup_in_phone_sms),
        )
        // ── Session ───────────────────────────────────────────────────
        .route(
            "/v1/auth/refresh",
            post(crate::routes::auth::session::refresh),
        )
        // The @descope/react-sdk calls /v1/auth/try-refresh (not /v1/auth/refresh)
        // during session initialization. Alias it to the same handler.
        .route(
            "/v1/auth/try-refresh",
            post(crate::routes::auth::session::refresh),
        )
        .route(
            "/v1/auth/logout",
            post(crate::routes::auth::session::logout),
        )
        .route(
            "/v1/auth/logoutall",
            post(crate::routes::auth::session::logout_all),
        )
        .route("/v1/auth/me", get(crate::routes::auth::session::me))
        .route(
            "/v1/auth/me/history",
            get(crate::routes::auth::session::me_history),
        )
        .route(
            "/v1/auth/validate",
            post(crate::routes::auth::session::validate),
        )
        .route(
            "/v1/auth/tenant/select",
            post(crate::routes::auth::session::tenant_select),
        )
        // ── Mgmt: User ────────────────────────────────────────────────
        .route(
            "/v1/mgmt/user/create",
            post(crate::routes::mgmt::user::create),
        )
        .route(
            "/v1/mgmt/user/create/test",
            post(crate::routes::mgmt::user::create_test),
        )
        .route(
            "/v1/mgmt/user",
            get(crate::routes::mgmt::user::load).delete(crate::routes::mgmt::user::delete_user),
        )
        .route(
            "/v1/mgmt/user/userid",
            get(crate::routes::mgmt::user::load_by_user_id)
                .delete(crate::routes::mgmt::user::delete_by_user_id),
        )
        .route(
            "/v1/mgmt/user/search",
            post(crate::routes::mgmt::user::search),
        )
        // v2 alias — Node SDK sends to /v2/mgmt/user/search
        .route(
            "/v2/mgmt/user/search",
            post(crate::routes::mgmt::user::search),
        )
        // ── Mgmt: Project ─────────────────────────────────────────────
        // SDK `management.project.exportSnapshot()` posts here.
        .route(
            "/v1/mgmt/project/snapshot/export",
            post(crate::routes::mgmt::project::export_snapshot),
        )
        .route(
            "/v1/mgmt/user/update",
            post(crate::routes::mgmt::user::update),
        )
        .route(
            "/v1/mgmt/user/patch",
            patch(crate::routes::mgmt::user::user_patch),
        )
        .route(
            "/v1/mgmt/user/update/email",
            post(crate::routes::mgmt::user::update_email),
        )
        .route(
            "/v1/mgmt/user/password/set/active",
            post(crate::routes::mgmt::user::set_active_password),
        )
        .route(
            "/v1/mgmt/user/test/delete/all",
            delete(crate::routes::mgmt::user::delete_all_test_users),
        )
        .route(
            "/v1/mgmt/user/tenant/add",
            post(crate::routes::mgmt::user::add_tenant),
        )
        .route(
            "/v1/mgmt/user/tenant/remove",
            post(crate::routes::mgmt::user::tenant_remove),
        )
        .route(
            "/v1/mgmt/user/tenant/setRole",
            post(crate::routes::mgmt::user::tenant_set_role),
        )
        // Descope's real SDK paths for user-tenant mutations carry an `/update/`
        // segment (POST /v1/mgmt/user/update/tenant/add|remove). The bare
        // `/tenant/...` routes above are kept for the admin UI and existing
        // callers; these aliases let the official @descope SDK reach the handlers
        // (without them, addTenant 405s with an empty body → the SDK throws
        // "Unexpected end of JSON input").
        .route(
            "/v1/mgmt/user/update/tenant/add",
            post(crate::routes::mgmt::user::add_tenant),
        )
        .route(
            "/v1/mgmt/user/update/tenant/remove",
            post(crate::routes::mgmt::user::tenant_remove),
        )
        .route(
            "/v1/mgmt/user/status",
            post(crate::routes::mgmt::user::status_update),
        )
        .route(
            "/v1/mgmt/user/update/status",
            post(crate::routes::mgmt::user::status_update),
        )
        // SDK calls POST /v1/mgmt/user/delete (not DELETE /v1/mgmt/user)
        .route(
            "/v1/mgmt/user/delete",
            post(crate::routes::mgmt::user::delete_user_by_login_id_post),
        )
        .route(
            "/v1/mgmt/user/embeddedlink",
            post(crate::routes::mgmt::user::generate_embedded_link),
        )
        // Node SDK alias — uses /v1/mgmt/user/signin/embeddedlink
        .route(
            "/v1/mgmt/user/signin/embeddedlink",
            post(crate::routes::mgmt::user::generate_embedded_link),
        )
        // ── Mgmt: Tests ───────────────────────────────────────────────
        .route(
            "/v1/mgmt/tests/generate/magiclink",
            post(crate::routes::mgmt::user::generate_magic_link_for_test_user),
        )
        .route(
            "/v1/mgmt/tests/generate/otp",
            post(crate::routes::mgmt::user::generate_otp_for_test_user),
        )
        .route(
            "/v1/mgmt/tests/generate/enchantedlink",
            post(crate::routes::mgmt::user::generate_enchanted_link_for_test_user),
        )
        // ── Mgmt: User (extended) ─────────────────────────────────────
        .route(
            "/v1/mgmt/user/update/name",
            post(crate::routes::mgmt::user::update_name),
        )
        .route(
            "/v1/mgmt/user/update/phone",
            post(crate::routes::mgmt::user::update_phone_field),
        )
        .route(
            "/v1/mgmt/user/update/loginid",
            post(crate::routes::mgmt::user::update_login_id),
        )
        .route(
            "/v1/mgmt/user/update/role/set",
            post(crate::routes::mgmt::user::set_roles),
        )
        .route(
            "/v1/mgmt/user/update/role/remove",
            post(crate::routes::mgmt::user::remove_roles),
        )
        .route(
            "/v1/mgmt/user/update/role/add",
            post(crate::routes::mgmt::user::add_roles),
        )
        // v2 aliases — the official @descope SDK sends project-role mutations to
        // /v2/mgmt/user/update/role/* (addRoles → /v2/.../role/add). Without these the
        // v2 request 404/405s with an empty body and the SDK throws "Unexpected end of
        // JSON input" — the same class of bug as the user-tenant aliases above.
        .route(
            "/v2/mgmt/user/update/role/add",
            post(crate::routes::mgmt::user::add_roles),
        )
        .route(
            "/v2/mgmt/user/update/role/set",
            post(crate::routes::mgmt::user::set_roles),
        )
        .route(
            "/v2/mgmt/user/update/role/remove",
            post(crate::routes::mgmt::user::remove_roles),
        )
        .route(
            "/v1/mgmt/user/update/picture",
            post(crate::routes::mgmt::user::update_picture),
        )
        .route(
            "/v1/mgmt/user/update/customAttribute",
            post(crate::routes::mgmt::user::update_custom_attribute),
        )
        .route(
            "/v1/mgmt/user/create/batch",
            post(crate::routes::mgmt::user::create_batch),
        )
        .route(
            "/v1/mgmt/user/delete/batch",
            post(crate::routes::mgmt::user::delete_batch),
        )
        .route(
            "/v1/mgmt/user/logout",
            post(crate::routes::mgmt::user::force_logout),
        )
        .route(
            "/v1/mgmt/user/password/expire",
            post(crate::routes::mgmt::user::password_expire),
        )
        .route(
            "/v1/mgmt/user/password/set/temporary",
            post(crate::routes::mgmt::user::set_temporary_password),
        )
        // ── Mgmt: JWT ─────────────────────────────────────────────────
        .route(
            "/v1/mgmt/jwt/update",
            post(crate::routes::mgmt::jwt::update),
        )
        // ── Mgmt: Permissions ─────────────────────────────────────────
        .route(
            "/v1/mgmt/authz/permission",
            post(crate::routes::mgmt::permissions::create_permission),
        )
        .route(
            "/v1/mgmt/authz/permission/all",
            get(crate::routes::mgmt::permissions::load_all_permissions),
        )
        .route(
            "/v1/mgmt/authz/permission/update",
            post(crate::routes::mgmt::permissions::update_permission),
        )
        .route(
            "/v1/mgmt/authz/permission/delete",
            post(crate::routes::mgmt::permissions::delete_permission),
        )
        // ── Mgmt: Roles ───────────────────────────────────────────────
        .route(
            "/v1/mgmt/authz/role",
            post(crate::routes::mgmt::roles::create_role),
        )
        .route(
            "/v1/mgmt/authz/role/all",
            get(crate::routes::mgmt::roles::load_all_roles),
        )
        .route(
            "/v1/mgmt/authz/role/update",
            post(crate::routes::mgmt::roles::update_role),
        )
        .route(
            "/v1/mgmt/authz/role/delete",
            post(crate::routes::mgmt::roles::delete_role),
        )
        // ── Mgmt: Auth Method Config ──────────────────────────────────
        .route(
            "/v1/mgmt/config/auth-methods",
            get(crate::routes::mgmt::auth_method_config::get_auth_methods)
                .put(crate::routes::mgmt::auth_method_config::put_auth_methods),
        )
        // ── Mgmt: JWT Templates ───────────────────────────────────────
        .route(
            "/v1/mgmt/jwt/template",
            post(crate::routes::mgmt::jwt_templates::create_template),
        )
        .route(
            "/v1/mgmt/jwt/template/all",
            get(crate::routes::mgmt::jwt_templates::load_all_templates),
        )
        .route(
            "/v1/mgmt/jwt/template/update",
            post(crate::routes::mgmt::jwt_templates::update_template),
        )
        .route(
            "/v1/mgmt/jwt/template/delete",
            post(crate::routes::mgmt::jwt_templates::delete_template),
        )
        .route(
            "/v1/mgmt/jwt/template/set-active",
            post(crate::routes::mgmt::jwt_templates::set_active_template),
        )
        .route(
            "/v1/mgmt/jwt/template/active",
            get(crate::routes::mgmt::jwt_templates::get_active_template),
        )
        // ── Mgmt: Connectors ──────────────────────────────────────────
        .route(
            "/v1/mgmt/connector",
            post(crate::routes::mgmt::connectors::create_connector),
        )
        .route(
            "/v1/mgmt/connector/all",
            get(crate::routes::mgmt::connectors::load_all_connectors),
        )
        .route(
            "/v1/mgmt/connector/update",
            post(crate::routes::mgmt::connectors::update_connector),
        )
        .route(
            "/v1/mgmt/connector/delete",
            post(crate::routes::mgmt::connectors::delete_connector),
        )
        // ── Mgmt: IdP Emulators ──────────────────────────────────────
        .route("/v1/mgmt/idp", post(crate::routes::mgmt::idp::create_idp))
        .route(
            "/v1/mgmt/idp/all",
            get(crate::routes::mgmt::idp::load_all_idps),
        )
        .route(
            "/v1/mgmt/idp/update",
            post(crate::routes::mgmt::idp::update_idp),
        )
        .route(
            "/v1/mgmt/idp/delete",
            post(crate::routes::mgmt::idp::delete_idp),
        )
        // ── Mgmt: Custom Attributes ───────────────────────────────────
        .route(
            "/v1/mgmt/user/attribute",
            post(crate::routes::mgmt::custom_attributes::create_attribute),
        )
        .route(
            "/v1/mgmt/user/attribute/all",
            get(crate::routes::mgmt::custom_attributes::load_all_attributes),
        )
        .route(
            "/v1/mgmt/user/attribute/delete",
            post(crate::routes::mgmt::custom_attributes::delete_attribute),
        )
        // ── Mgmt: Access Keys ─────────────────────────────────────────
        .route(
            "/v1/mgmt/accesskey",
            post(crate::routes::mgmt::access_keys::create_access_key),
        )
        .route(
            "/v1/mgmt/accesskey/all",
            get(crate::routes::mgmt::access_keys::load_all_access_keys),
        )
        .route(
            "/v1/mgmt/accesskey/update",
            post(crate::routes::mgmt::access_keys::update_access_key),
        )
        .route(
            "/v1/mgmt/accesskey/delete",
            post(crate::routes::mgmt::access_keys::delete_access_key),
        )
        .route(
            "/v1/mgmt/accesskey/disable",
            post(crate::routes::mgmt::access_keys::disable_access_key),
        )
        // ── Emulator: Snapshot ────────────────────────────────────────
        .route(
            "/emulator/snapshot",
            get(crate::routes::emulator::snapshot::export)
                .post(crate::routes::emulator::snapshot::import),
        )
        .route(
            "/emulator/otps",
            get(crate::routes::emulator::snapshot::list_otps),
        )
        // ── Emulator: IdP OIDC ───────────────────────────────────────
        .route(
            "/emulator/idp/:idp_id/.well-known/openid-configuration",
            get(crate::routes::emulator::idp_oidc::discovery),
        )
        .route(
            "/emulator/idp/:idp_id/jwks",
            get(crate::routes::emulator::idp_oidc::jwks),
        )
        .route(
            "/emulator/idp/:idp_id/authorize",
            get(crate::routes::emulator::idp_oidc::authorize),
        )
        .route(
            "/emulator/idp/:idp_id/token",
            post(crate::routes::emulator::idp_oidc::token),
        )
        .route(
            "/emulator/idp/callback",
            get(crate::routes::emulator::idp_oidc::callback),
        )
        // ── Emulator: IdP SAML ───────────────────────────────────────
        .route(
            "/emulator/idp/:idp_id/metadata",
            get(crate::routes::emulator::idp_saml::metadata),
        )
        .route(
            "/emulator/idp/:idp_id/sso",
            get(crate::routes::emulator::idp_saml::sso),
        )
        .route(
            "/emulator/idp/saml/acs",
            post(crate::routes::emulator::idp_saml::saml_acs),
        )
        // ── Mgmt: Tenant ──────────────────────────────────────────────
        .route(
            "/v1/mgmt/tenant/all",
            get(crate::routes::mgmt::tenant::load_all),
        )
        .route(
            "/v1/mgmt/tenant/create",
            post(crate::routes::mgmt::tenant::create),
        )
        .route(
            "/v1/mgmt/tenant/update",
            post(crate::routes::mgmt::tenant::update),
        )
        .route(
            "/v1/mgmt/tenant",
            get(crate::routes::mgmt::tenant::load)
                .delete(crate::routes::mgmt::tenant::delete_tenant),
        )
        // Node SDK sends POST /v1/mgmt/tenant/delete with body {id, cascade}
        .route(
            "/v1/mgmt/tenant/delete",
            post(crate::routes::mgmt::tenant::delete_tenant_post),
        )
        .route(
            "/v1/mgmt/tenant/search",
            post(crate::routes::mgmt::tenant::search),
        )
        .layer(cors)
        .with_state(state)
        // ── OpenAPI / Swagger UI ──────────────────────────────────────
        .route("/openapi.json", get(openapi_json))
        .route("/docs", get(swagger_ui_html));

    // ── UI Serving ─────────────────────────────────────────────────
    // Default: embedded assets (baked into binary at compile time)
    // Feature `dev-ui`: filesystem serving from apps/ui/dist/ for hot-reload
    #[cfg(feature = "dev-ui")]
    let router = router.nest_service(
        "/",
        ServeDir::new("apps/ui/dist").fallback(ServeFile::new("apps/ui/dist/index.html")),
    );
    #[cfg(not(feature = "dev-ui"))]
    let router = router.merge(crate::embedded_ui::embedded_ui_router());

    // Outermost layer, added after the UI merge so every route — API,
    // emulator, docs, and UI assets — produces a request log line.
    let log_cfg = crate::request_log::BodyLogConfig::from_env();
    router.layer(axum::middleware::from_fn(move |req, next| {
        crate::request_log::log_request(log_cfg, req, next)
    }))
}

async fn openapi_json() -> Json<serde_json::Value> {
    Json(crate::openapi::build_openapi_spec())
}

async fn swagger_ui_html() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html>
<head>
  <title>Rescope API Docs</title>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
  <style>
    body { margin: 0; background: #1a1a2e; }
    .swagger-ui .topbar { display: none; }
  </style>
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js" crossorigin></script>
  <script>
    SwaggerUIBundle({
      url: '/openapi.json',
      dom_id: '#swagger-ui',
      presets: [SwaggerUIBundle.presets.apis, SwaggerUIBundle.SwaggerUIStandalonePreset],
      layout: 'BaseLayout',
      deepLinking: true,
      defaultModelsExpandDepth: -1,
    });
  </script>
</body>
</html>"#,
    )
}

#[cfg(test)]
mod tests {
    use super::build_router;
    use crate::config::EmulatorConfig;
    use crate::state::EmulatorState;
    use axum_test::TestServer;
    use serde_json::json;

    /// Why: the official @descope SDK calls user-tenant mutations at the
    ///      `/v1/mgmt/user/update/tenant/add` path (note the `/update/`
    ///      segment). The emulator only registered the bare
    ///      `/v1/mgmt/user/tenant/add`, so the SDK's `addTenant` hit a 405 with
    ///      an empty body and threw "Unexpected end of JSON input" — breaking any
    ///      flow that adds a user to a tenant (e.g. inviting an anonymous signer).
    /// Decision: drive the real router via the SDK path end-to-end so the alias
    ///      route cannot silently regress.
    #[tokio::test]
    async fn add_tenant_reachable_at_descope_sdk_update_path() {
        let state = EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap();
        let auth = format!(
            "{}:{}",
            state.config.project_id, state.config.management_key
        );
        let server = TestServer::new(build_router(state)).unwrap();

        server
            .post("/v1/mgmt/user/create")
            .authorization_bearer(&auth)
            .json(&json!({ "loginId": "tenant-route@example.com", "email": "tenant-route@example.com" }))
            .await
            .assert_status_ok();

        server
            .post("/v1/mgmt/user/update/tenant/add")
            .authorization_bearer(&auth)
            .json(&json!({ "loginId": "tenant-route@example.com", "tenantId": "t-route-1" }))
            .await
            .assert_status_ok();
    }

    /// Why: the official @descope SDK calls `addRoles` at the **v2** path
    ///      `/v2/mgmt/user/update/role/add`; the emulator only registered the v1
    ///      route, so the SDK hit a 404/405 with an empty body and threw "Unexpected
    ///      end of JSON input" — blocking any SDK-driven project-role assignment.
    /// Decision: drive the real router via the SDK's v2 path end-to-end so the alias
    ///      cannot silently regress, and assert the roles actually land on the user.
    #[tokio::test]
    async fn add_roles_reachable_at_descope_sdk_v2_path() {
        let state = EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap();
        let auth = format!(
            "{}:{}",
            state.config.project_id, state.config.management_key
        );
        let server = TestServer::new(build_router(state)).unwrap();

        server
            .post("/v1/mgmt/user/create")
            .authorization_bearer(&auth)
            .json(&json!({ "loginId": "role-route@test.com", "email": "role-route@test.com" }))
            .await
            .assert_status_ok();

        let resp = server
            .post("/v2/mgmt/user/update/role/add")
            .authorization_bearer(&auth)
            .json(&json!({ "loginId": "role-route@test.com", "roleNames": ["admin", "editor"] }))
            .await;
        resp.assert_status_ok();
        let body = resp.json::<serde_json::Value>();
        let roles = body["user"]["roleNames"].as_array().unwrap();
        assert!(roles.iter().any(|r| r == "admin"));
        assert!(roles.iter().any(|r| r == "editor"));
    }
}
