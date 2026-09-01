//! Interactive configuration screen for just-in-time SSO provisioning.
//!
//! GET  /emulator/sso/provision → form for a not-yet-existing SSO user
//! POST /emulator/sso/provision → provision the user, then resume the sign-in
//!
//! Every SSO surface that meets an unknown login ID can hand the browser to this
//! screen instead of failing. It resumes the sign-in in one of two ways:
//!
//! * `mode=code` — mint a SAML exchange code and redirect to the application's
//!   redirect URL, exactly as `/v1/auth/saml/start` would have done for a user
//!   that already existed.
//! * `mode=continue` — bounce back to the identity-provider endpoint that sent
//!   us here, with the new address as `login_id`, so it can carry on issuing an
//!   assertion or authorization code.

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use std::collections::HashMap;

use crate::{
    error::EmulatorError,
    sso_jit::{
        display_name_from_email, normalize_email, provision_sso_user, resolve_sso_tenant,
        SsoProvisionInput,
    },
    state::EmulatorState,
    store::token_store::generate_token,
    types::{Tenant, TokenType},
};

/// How the sign-in resumes once the user exists.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResumeMode {
    /// Mint a SAML exchange code and send the browser to the application.
    Code,
    /// Return to the IdP endpoint that referred us, carrying `login_id`.
    Continue,
}

impl ResumeMode {
    fn parse(raw: Option<&str>) -> Self {
        match raw {
            Some("continue") => Self::Continue,
            _ => Self::Code,
        }
    }
}

/// Parameters that survive the round trip through the form, describing where the
/// sign-in came from and where it resumes.
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResumeParams {
    pub email: Option<String>,
    pub tenant: Option<String>,
    pub mode: Option<String>,
    /// Application URL to redirect to with `?code=` (mode=code).
    #[serde(alias = "redirectURL", alias = "redirect_url")]
    pub redirect_url: Option<String>,
    /// IdP URL to return to with `login_id` appended (mode=continue).
    #[serde(alias = "continue_url")]
    pub continue_url: Option<String>,
}

impl ResumeParams {
    /// Build the query string that carries this sign-in into the screen.
    pub fn to_query(&self) -> String {
        let mut pairs: Vec<(&str, &str)> = Vec::new();
        if let Some(v) = self.email.as_deref() {
            pairs.push(("email", v));
        }
        if let Some(v) = self.tenant.as_deref() {
            pairs.push(("tenant", v));
        }
        if let Some(v) = self.mode.as_deref() {
            pairs.push(("mode", v));
        }
        if let Some(v) = self.redirect_url.as_deref() {
            pairs.push(("redirectUrl", v));
        }
        if let Some(v) = self.continue_url.as_deref() {
            pairs.push(("continueUrl", v));
        }
        pairs
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// Absolute URL of the configuration screen for this sign-in.
    pub fn to_url(&self, port: u16) -> String {
        format!(
            "http://localhost:{}/emulator/sso/provision?{}",
            port,
            self.to_query()
        )
    }
}

// ─── GET ─────────────────────────────────────────────────────────────────────

pub async fn show(
    State(state): State<EmulatorState>,
    Query(params): Query<ResumeParams>,
) -> Result<Response, EmulatorError> {
    let tenant = lookup_tenant(&state, &params).await?;
    Ok(Html(render_form(&state, &params, tenant.as_ref(), None).await).into_response())
}

// ─── POST ────────────────────────────────────────────────────────────────────

/// The posted form, as name/value pairs. Parsed by hand rather than through a
/// struct because the roles checkboxes repeat one key, and
/// `serde_urlencoded` collapses repeats into a single value.
struct ProvisionForm {
    pairs: Vec<(String, String)>,
}

impl ProvisionForm {
    fn parse(body: &str) -> Result<Self, EmulatorError> {
        let pairs: Vec<(String, String)> = serde_urlencoded::from_str(body)
            .map_err(|e| EmulatorError::ValidationError(format!("Invalid form body: {e}")))?;
        Ok(Self { pairs })
    }

    /// First non-empty value for a key.
    fn get(&self, key: &str) -> Option<String> {
        self.pairs
            .iter()
            .find(|(k, v)| k == key && !v.trim().is_empty())
            .map(|(_, v)| v.clone())
    }

    /// Every non-empty value for a key — the roles checkboxes and the free-text
    /// roles field share the `roles` name so they merge naturally.
    fn get_all(&self, key: &str) -> Vec<String> {
        self.pairs
            .iter()
            .filter(|(k, v)| k == key && !v.trim().is_empty())
            .map(|(_, v)| v.clone())
            .collect()
    }
}

pub async fn submit(
    State(state): State<EmulatorState>,
    Query(query): Query<ResumeParams>,
    body: String,
) -> Result<Response, EmulatorError> {
    let form = ProvisionForm::parse(&body)?;

    // The form posts the routing params back as hidden inputs; a hand-rolled
    // POST may instead leave them on the query string.
    let resume = ResumeParams {
        email: form.get("email").or(query.email),
        tenant: form.get("tenant").or(query.tenant),
        mode: form.get("mode").or(query.mode),
        redirect_url: form
            .get("redirectUrl")
            .or(form.get("redirectURL"))
            .or(query.redirect_url),
        continue_url: form.get("continueUrl").or(query.continue_url),
    };

    let email = match resume.email.as_deref().and_then(normalize_email) {
        Some(email) => email,
        None => return re_render(&state, &resume, "Enter a valid email address.").await,
    };

    let tenant = match resolve_target_tenant(&state, &resume, &email).await {
        Ok(tenant) => tenant,
        Err(message) => return re_render(&state, &resume, &message).await,
    };

    let custom_attributes = match parse_custom_attributes(form.get("customAttributes").as_deref()) {
        Ok(attrs) => attrs,
        Err(message) => return re_render(&state, &resume, &message).await,
    };

    let input = SsoProvisionInput {
        email: email.clone(),
        name: form.get("name"),
        given_name: form.get("givenName"),
        family_name: form.get("familyName"),
        role_names: parse_roles(&form.get_all("roles")),
        custom_attributes,
    };

    let user = match provision_sso_user(&state, &tenant, input).await {
        Ok(user) => user,
        Err(e) => return re_render(&state, &resume, &e.to_string()).await,
    };

    match ResumeMode::parse(resume.mode.as_deref()) {
        ResumeMode::Continue => {
            let target = resume
                .continue_url
                .as_deref()
                .filter(|v| !v.is_empty())
                .ok_or_else(|| {
                    EmulatorError::ValidationError(
                        "continueUrl is required for mode=continue".into(),
                    )
                })?;
            Ok(Redirect::to(&append_query(target, "login_id", &email)).into_response())
        }
        ResumeMode::Code => {
            let code = generate_token();
            state
                .tokens
                .write()
                .await
                .insert(code.clone(), user.user_id.clone(), TokenType::Saml);
            let target = resume.redirect_url.clone().unwrap_or_default();
            Ok(Redirect::to(&append_query(&target, "code", &code)).into_response())
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Append a query parameter, respecting any query string already on the URL.
/// Emulator-only: the target is whatever the application asked to be sent back
/// to, exactly as the pre-existing SAML start handler already trusted it.
fn append_query(url: &str, key: &str, value: &str) -> String {
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}{key}={}", urlencoding::encode(value))
}

/// Flatten the submitted role values — one per checked checkbox, plus a
/// comma- or newline-separated free-text field — into a de-duplicated list.
fn parse_roles(raw: &[String]) -> Vec<String> {
    raw.iter()
        .flat_map(|value| value.split([',', '\n']))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .fold(Vec::new(), |mut acc, role| {
            if !acc.contains(&role) {
                acc.push(role);
            }
            acc
        })
}

fn parse_custom_attributes(
    raw: Option<&str>,
) -> Result<HashMap<String, serde_json::Value>, String> {
    let text = raw.unwrap_or_default().trim();
    if text.is_empty() {
        return Ok(HashMap::new());
    }
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(serde_json::Value::Object(map)) => Ok(map.into_iter().collect()),
        Ok(_) => Err("Custom attributes must be a JSON object.".into()),
        Err(e) => Err(format!("Custom attributes are not valid JSON: {e}")),
    }
}

/// The tenant this sign-in belongs to, resolved from the explicit tenant id when
/// present and otherwise from the email domain.
async fn resolve_target_tenant(
    state: &EmulatorState,
    params: &ResumeParams,
    email: &str,
) -> Result<Tenant, String> {
    if let Some(id) = params.tenant.as_deref().filter(|v| !v.is_empty()) {
        let tenant = state
            .tenants
            .read()
            .await
            .load(id)
            .cloned()
            .map_err(|_| format!("Unknown tenant `{id}`."))?;
        let domains = tenant.all_domains().cloned().collect::<Vec<_>>();
        if !domains.is_empty() && !tenant.owns_email(email) {
            return Err(format!(
                "{email} is not on a domain owned by this tenant ({}).",
                domains.join(", ")
            ));
        }
        return Ok(tenant);
    }
    resolve_sso_tenant(state, email)
        .await
        .map_err(|_| format!("No SSO tenant owns the domain of {email}."))
}

async fn lookup_tenant(
    state: &EmulatorState,
    params: &ResumeParams,
) -> Result<Option<Tenant>, EmulatorError> {
    if let Some(id) = params.tenant.as_deref().filter(|v| !v.is_empty()) {
        return Ok(state.tenants.read().await.load(id).ok().cloned());
    }
    if let Some(email) = params.email.as_deref().and_then(normalize_email) {
        return Ok(resolve_sso_tenant(state, &email).await.ok());
    }
    Ok(None)
}

async fn re_render(
    state: &EmulatorState,
    params: &ResumeParams,
    error: &str,
) -> Result<Response, EmulatorError> {
    let tenant = lookup_tenant(state, params).await?;
    Ok(Html(render_form(state, params, tenant.as_ref(), Some(error)).await).into_response())
}

// ─── Rendering ───────────────────────────────────────────────────────────────

async fn render_form(
    state: &EmulatorState,
    params: &ResumeParams,
    tenant: Option<&Tenant>,
    error: Option<&str>,
) -> String {
    let email = params
        .email
        .as_deref()
        .and_then(normalize_email)
        .unwrap_or_default();
    let suggested_name = if email.is_empty() {
        String::new()
    } else {
        display_name_from_email(&email)
    };

    let tenant_label = tenant
        .map(|t| {
            let domains = t.all_domains().cloned().collect::<Vec<_>>().join(", ");
            if domains.is_empty() {
                t.name.clone()
            } else {
                format!("{} — {}", t.name, domains)
            }
        })
        .unwrap_or_else(|| "Resolved from the email domain".to_string());

    let jit_defaults = tenant.map(|t| t.sso_jit()).unwrap_or_default();
    let known_roles: Vec<String> = {
        let roles = state.roles.read().await;
        let mut names: Vec<String> = roles.load_all().iter().map(|r| r.name.clone()).collect();
        names.sort();
        names
    };
    let preselected: Vec<String> = if jit_defaults.default_role_names.is_empty() {
        state.roles.read().await.default_role_names()
    } else {
        jit_defaults.default_role_names.clone()
    };

    let role_checkboxes = if known_roles.is_empty() {
        r#"<p class="hint">No roles defined in this project — use the field below to name one anyway.</p>"#.to_string()
    } else {
        known_roles
            .iter()
            .map(|name| {
                format!(
                    r#"<label class="check"><input type="checkbox" name="roles" value="{value}"{checked}> {label}</label>"#,
                    value = html_escape(name),
                    label = html_escape(name),
                    checked = if preselected.contains(name) {
                        " checked"
                    } else {
                        ""
                    },
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let error_banner = error
        .map(|e| format!(r#"<div class="error">{}</div>"#, html_escape(e)))
        .unwrap_or_default();

    let hidden = [
        ("mode", params.mode.as_deref().unwrap_or("code")),
        ("tenant", params.tenant.as_deref().unwrap_or("")),
        ("redirectUrl", params.redirect_url.as_deref().unwrap_or("")),
        ("continueUrl", params.continue_url.as_deref().unwrap_or("")),
    ]
    .iter()
    .map(|(k, v)| {
        format!(
            r#"<input type="hidden" name="{k}" value="{v}">"#,
            k = k,
            v = html_escape(v)
        )
    })
    .collect::<Vec<_>>()
    .join("\n");

    let email_readonly = if params.email.is_some() {
        " readonly"
    } else {
        ""
    };
    let custom_attributes = if jit_defaults.default_custom_attributes.is_empty() {
        String::new()
    } else {
        serde_json::to_string_pretty(&jit_defaults.default_custom_attributes).unwrap_or_default()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Rescope — Configure SSO user</title>
<style>
  *{{margin:0;padding:0;box-sizing:border-box}}
  body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#0d1117;color:#e6edf3;min-height:100vh;display:flex;align-items:center;justify-content:center;padding:2rem}}
  .card{{background:#161b22;border:1px solid #30363d;border-radius:12px;padding:2rem;max-width:560px;width:100%}}
  h1{{font-size:1.1rem;color:#39d353;margin-bottom:.35rem}}
  .subtitle{{color:#8b949e;font-size:.85rem;margin-bottom:1.5rem}}
  .error{{background:#3d1418;border:1px solid #da3633;color:#ffa198;padding:.6rem .8rem;border-radius:6px;font-size:.8rem;margin-bottom:1rem}}
  label.field{{display:block;margin-bottom:1rem}}
  label.field > span{{display:block;color:#8b949e;font-size:.72rem;text-transform:uppercase;letter-spacing:.04em;margin-bottom:.35rem}}
  input[type=text],input[type=email],textarea{{width:100%;background:#0d1117;border:1px solid #30363d;border-radius:6px;color:#e6edf3;padding:.5rem .65rem;font-size:.9rem;font-family:inherit}}
  textarea{{min-height:5rem;font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:.8rem}}
  input[readonly]{{color:#8b949e}}
  .row{{display:flex;gap:1rem}}
  .row > label.field{{flex:1}}
  fieldset{{border:1px solid #30363d;border-radius:6px;padding:.75rem .85rem;margin-bottom:1rem}}
  legend{{color:#8b949e;font-size:.72rem;text-transform:uppercase;letter-spacing:.04em;padding:0 .35rem}}
  .check{{display:inline-flex;align-items:center;gap:.35rem;font-size:.85rem;margin:.2rem .9rem .2rem 0}}
  .hint{{color:#8b949e;font-size:.75rem;margin-top:.35rem}}
  button{{background:#39d353;color:#0d1117;border:0;padding:.55rem 1.2rem;border-radius:6px;font-size:.9rem;font-weight:600;cursor:pointer}}
  button:hover{{background:#2ea043}}
</style>
</head>
<body>
<div class="card">
  <h1>⚡ Rescope — First SSO sign-in</h1>
  <div class="subtitle">{tenant_label}</div>
  {error_banner}
  <form method="POST">
    {hidden}
    <label class="field">
      <span>Email</span>
      <input type="email" name="email" value="{email}" placeholder="you@example.test" required{email_readonly}>
    </label>
    <label class="field">
      <span>Display name</span>
      <input type="text" name="name" value="{suggested_name}" placeholder="Optional">
    </label>
    <div class="row">
      <label class="field">
        <span>Given name</span>
        <input type="text" name="givenName" placeholder="Optional">
      </label>
      <label class="field">
        <span>Family name</span>
        <input type="text" name="familyName" placeholder="Optional">
      </label>
    </div>
    <fieldset>
      <legend>Roles</legend>
      {role_checkboxes}
      <label class="field" style="margin-top:.6rem;margin-bottom:0">
        <span>Additional roles (comma separated)</span>
        <input type="text" name="roles" placeholder="e.g. Auditor, Reviewer">
      </label>
    </fieldset>
    <label class="field">
      <span>Custom attributes (JSON object)</span>
      <textarea name="customAttributes" placeholder="{{}}">{custom_attributes}</textarea>
    </label>
    <button type="submit">Create user and continue</button>
  </form>
</div>
</body>
</html>"#,
        tenant_label = html_escape(&tenant_label),
        error_banner = error_banner,
        hidden = hidden,
        email = html_escape(&email),
        email_readonly = email_readonly,
        suggested_name = html_escape(&suggested_name),
        role_checkboxes = role_checkboxes,
        custom_attributes = html_escape(&custom_attributes),
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::EmulatorConfig, types::AuthType};
    use axum::http::StatusCode;

    async fn make_state() -> EmulatorState {
        let state = EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap();
        state.tenants.write().await.insert(Tenant {
            id: "acme".into(),
            name: "Acme".into(),
            domains: vec!["acme.test".into()],
            auth_type: AuthType::Saml,
            ..Default::default()
        });
        state
    }

    fn form_body(pairs: &[(&str, &str)]) -> String {
        serde_urlencoded::to_string(pairs).unwrap()
    }

    fn location(resp: &Response) -> String {
        resp.headers()
            .get(axum::http::header::LOCATION)
            .expect("redirect")
            .to_str()
            .unwrap()
            .to_string()
    }

    /// Why: the screen is reached mid-redirect, so it has to render the address
    ///      the sign-in was started with rather than asking for it again.
    /// Decision: prefill and lock the email, and surface the owning tenant.
    #[tokio::test]
    async fn get_prefills_the_email_and_names_the_tenant() {
        let state = make_state().await;
        let params = ResumeParams {
            email: Some("New.Person@acme.test".into()),
            ..Default::default()
        };

        let resp = show(State(state), Query(params)).await.unwrap();
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();

        assert!(body.contains(r#"value="new.person@acme.test""#));
        assert!(body.contains("New Person"));
        assert!(body.contains("Acme"));
        assert!(body.contains("acme.test"));
    }

    /// Why: this is how a browser SSO sign-in finishes — the application expects
    ///      the same `?code=` redirect it would have got for an existing user.
    /// Decision: provision, mint a SAML code, and redirect to the app.
    #[tokio::test]
    async fn submitting_in_code_mode_provisions_and_redirects_with_a_code() {
        let state = make_state().await;
        let body = form_body(&[
            ("email", "brand.new@acme.test"),
            ("mode", "code"),
            ("redirectUrl", "http://localhost:4200/login/sso"),
            ("name", "Brand New"),
            ("roles", "Reviewer, Auditor"),
            ("customAttributes", r#"{"department":"ops"}"#),
        ]);

        let resp = submit(State(state.clone()), Query(ResumeParams::default()), body)
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        let target = location(&resp);
        assert!(target.starts_with("http://localhost:4200/login/sso?code="));

        let users = state.users.read().await;
        let user = users.load("brand.new@acme.test").expect("provisioned");
        assert_eq!(user.name.as_deref(), Some("Brand New"));
        assert_eq!(user.role_names, vec!["Reviewer", "Auditor"]);
        assert_eq!(user.custom_attributes["department"], "ops");
        assert_eq!(user.status, "enabled");
    }

    /// Why: the minted code has to be redeemable by `/v1/auth/saml/exchange`,
    ///      otherwise the screen produces a dead end instead of a session.
    /// Decision: assert the token store resolves the code to the new user.
    #[tokio::test]
    async fn the_minted_code_resolves_to_the_provisioned_user() {
        let state = make_state().await;
        let body = form_body(&[
            ("email", "exchangeable@acme.test"),
            ("redirectUrl", "http://localhost:4200/cb"),
        ]);

        let resp = submit(State(state.clone()), Query(ResumeParams::default()), body)
            .await
            .unwrap();
        let code = location(&resp).split("code=").nth(1).unwrap().to_string();

        let entry = state.tokens.write().await.consume(&code).unwrap();
        let users = state.users.read().await;
        let user = users.load_by_user_id(&entry.user_id).unwrap();
        assert_eq!(user.email.as_deref(), Some("exchangeable@acme.test"));
    }

    /// Why: the IdP pickers send the browser here and need it back, with the new
    ///      address, to carry on issuing an assertion or authorization code.
    /// Decision: append `login_id` to the referring URL, preserving its query.
    #[tokio::test]
    async fn submitting_in_continue_mode_returns_to_the_idp_with_the_login_id() {
        let state = make_state().await;
        let body = form_body(&[
            ("email", "picker@acme.test"),
            ("mode", "continue"),
            (
                "continueUrl",
                "http://localhost:4600/emulator/idp/IDP1/sso?RelayState=abc",
            ),
        ]);

        let resp = submit(State(state), Query(ResumeParams::default()), body)
            .await
            .unwrap();

        assert_eq!(
            location(&resp),
            "http://localhost:4600/emulator/idp/IDP1/sso?RelayState=abc&login_id=picker%40acme.test"
        );
    }

    /// Why: a mistyped address must not strand the person on a raw JSON error —
    ///      they are in a browser, mid-sign-in.
    /// Decision: re-render the form with the reason, and create nothing.
    #[tokio::test]
    async fn a_domain_the_tenant_does_not_own_re_renders_with_an_error() {
        let state = make_state().await;
        let body = form_body(&[
            ("email", "outsider@elsewhere.test"),
            ("tenant", "acme"),
            ("redirectUrl", "http://localhost:4200/cb"),
        ]);

        let resp = submit(State(state.clone()), Query(ResumeParams::default()), body)
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let html = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(html.contains("is not on a domain owned by this tenant"));
        assert!(state.users.read().await.all_users().is_empty());
    }

    /// Why: the custom-attributes field is free-text JSON, and a typo there
    ///      should be correctable rather than fatal.
    /// Decision: re-render with the parse error instead of provisioning.
    #[tokio::test]
    async fn malformed_custom_attributes_re_render_with_an_error() {
        let state = make_state().await;
        let body = form_body(&[
            ("email", "typo@acme.test"),
            ("redirectUrl", "http://localhost:4200/cb"),
            ("customAttributes", "{not json"),
        ]);

        let resp = submit(State(state.clone()), Query(ResumeParams::default()), body)
            .await
            .unwrap();

        let html = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(html.contains("not valid JSON"));
        assert!(state.users.read().await.all_users().is_empty());
    }

    /// Why: the checkboxes and the free-text field both post as `roles`, so the
    ///      handler receives several values and must merge them.
    /// Decision: split on both separators and drop duplicates, preserving order.
    #[test]
    fn roles_parse_from_checkboxes_and_free_text_without_duplicates() {
        let raw = vec![
            "Admin".to_string(),
            "Reviewer".to_string(),
            "Admin, Auditor".to_string(),
        ];
        assert_eq!(
            parse_roles(&raw),
            vec![
                "Admin".to_string(),
                "Reviewer".to_string(),
                "Auditor".to_string()
            ]
        );
        assert!(parse_roles(&[]).is_empty());
    }

    #[test]
    fn append_query_respects_an_existing_query_string() {
        assert_eq!(
            append_query("http://x/y", "code", "a b"),
            "http://x/y?code=a%20b"
        );
        assert_eq!(
            append_query("http://x/y?z=1", "code", "abc"),
            "http://x/y?z=1&code=abc"
        );
    }
}
