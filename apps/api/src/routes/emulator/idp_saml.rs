/// SAML Identity Provider emulator endpoints.
///
/// GET  /emulator/idp/:idp_id/metadata → EntityDescriptor XML
/// GET  /emulator/idp/:idp_id/sso      → user picker / SAML Response
use axum::{
    extract::{Path, Query, State},
    http::header,
    response::{Html, IntoResponse, Response},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

use crate::{
    error::EmulatorError,
    routes::emulator::sso_provision::ResumeParams,
    sso_jit::{
        normalize_email, provision_sso_user_for_tenant, resolve_sso_tenant, SsoProvisionInput,
    },
    state::EmulatorState,
    store::{idp_store::IdpProtocol, token_store::generate_token},
    types::TokenType,
};

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_secs()
}

/// Simple UTC ISO 8601 timestamp without external chrono dependency.
fn utc_iso(secs: u64) -> String {
    // Manual conversion: days/hours/minutes/seconds
    let s = secs;
    let days = s / 86400;
    let rem = s % 86400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let sec = rem % 60;

    // Days since epoch → year/month/day (simplified Gregorian)
    let mut y: i64 = 1970;
    let mut d = days as i64;
    loop {
        let ydays = if is_leap(y) { 366 } else { 365 };
        if d < ydays {
            break;
        }
        d -= ydays;
        y += 1;
    }
    let leap = is_leap(y);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut mo = 1u32;
    for md in &month_days {
        if d < *md {
            break;
        }
        d -= md;
        mo += 1;
    }
    let day = d + 1;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, mo, day, h, m, sec
    )
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

// ─── X.509 Certificate Generation ───────────────────────────────────────────

/// Generate a self-signed X.509 certificate from an RSA private key PEM.
pub fn generate_x509_cert(private_pem: &str) -> Result<(String, String), EmulatorError> {
    use rcgen::{CertificateParams, KeyPair};

    let key_pair = KeyPair::from_pem(private_pem)
        .map_err(|e| EmulatorError::Internal(format!("Failed to parse IdP key: {e}")))?;

    let mut params = CertificateParams::default();
    params.distinguished_name.push(
        rcgen::DnType::CommonName,
        rcgen::DnValue::Utf8String("Rescope IdP Emulator".into()),
    );
    params.distinguished_name.push(
        rcgen::DnType::OrganizationName,
        rcgen::DnValue::Utf8String("Rescope".into()),
    );

    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| EmulatorError::Internal(format!("Failed to generate X.509 cert: {e}")))?;

    let cert_pem = cert.pem();
    // Extract just the base64 content (no headers)
    let cert_der_b64 = cert_pem
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect::<Vec<_>>()
        .join("");

    Ok((cert_pem, cert_der_b64))
}

// ─── Metadata ────────────────────────────────────────────────────────────────

pub async fn metadata(
    State(state): State<EmulatorState>,
    Path(idp_id): Path<String>,
) -> Result<Response, EmulatorError> {
    let idp = state.idp_emulators.read().await.load(&idp_id)?.clone();
    if idp.protocol != IdpProtocol::Saml {
        return Err(EmulatorError::ValidationError(
            "IdP is not configured for SAML".into(),
        ));
    }

    let idp_km = state.idp_keys.read().await.clone();
    let (_, cert_b64) = generate_x509_cert(&idp_km.private_pem)?;

    let base = format!(
        "http://localhost:{}/emulator/idp/{}",
        state.config.port, idp_id
    );

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata"
                     entityID="{entity_id}">
  <md:IDPSSODescriptor WantAuthnRequestsSigned="false"
                       protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <md:KeyDescriptor use="signing">
      <ds:KeyInfo xmlns:ds="http://www.w3.org/2000/09/xmldsig#">
        <ds:X509Data>
          <ds:X509Certificate>{cert}</ds:X509Certificate>
        </ds:X509Data>
      </ds:KeyInfo>
    </md:KeyDescriptor>
    <md:SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect"
                            Location="{sso_url}"/>
  </md:IDPSSODescriptor>
</md:EntityDescriptor>"#,
        entity_id = xml_escape(&base),
        cert = cert_b64,
        sso_url = xml_escape(&format!("{}/sso", base)),
    );

    Ok(([(header::CONTENT_TYPE, "application/xml")], xml).into_response())
}

// ─── SSO Endpoint ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SsoParams {
    /// Base64-encoded, deflated AuthnRequest (ignored by emulator, just passed through)
    #[serde(rename = "SAMLRequest")]
    pub saml_request: Option<String>,
    #[serde(rename = "RelayState")]
    pub relay_state: Option<String>,
    /// Programmatic test helper: auto-select this user
    pub login_id: Option<String>,
}

pub async fn sso(
    State(state): State<EmulatorState>,
    Path(idp_id): Path<String>,
    Query(params): Query<SsoParams>,
) -> Result<Response, EmulatorError> {
    let idp = state.idp_emulators.read().await.load(&idp_id)?.clone();
    if idp.protocol != IdpProtocol::Saml {
        return Err(EmulatorError::ValidationError(
            "IdP is not configured for SAML".into(),
        ));
    }

    let relay_state = params.relay_state.clone().unwrap_or_default();

    // Determine ACS URL from tenant's SAML config
    let acs_url = {
        let tenants = state.tenants.read().await;
        let tenant = tenants.load(&idp.tenant_id)?;
        tenant
            .saml_config
            .as_ref()
            .and_then(|c| c.acs_url.clone())
            .unwrap_or_else(|| {
                format!(
                    "http://localhost:{}/emulator/idp/saml/acs",
                    state.config.port
                )
            })
    };

    // If login_id is provided (programmatic, or a return trip from the
    // configuration screen), generate SAML Response immediately.
    if let Some(login_id) = params.login_id {
        let user = match state.users.read().await.load(&login_id) {
            Ok(user) => Some(user.clone()),
            Err(_) => None,
        };
        // An identity provider is authoritative for whoever it asserts, so an
        // address this emulator has not seen is provisioned into the IdP's
        // tenant rather than rejected.
        let user = match user {
            Some(user) => user,
            None => {
                provision_sso_user_for_tenant(
                    &state,
                    &idp.tenant_id,
                    SsoProvisionInput::new(login_id.clone()),
                )
                .await?
            }
        };

        let base = format!(
            "http://localhost:{}/emulator/idp/{}",
            state.config.port, idp_id
        );

        let saml_response = generate_saml_response(&user, &base, &acs_url, &idp.attribute_mapping)?;

        let saml_response_b64 = STANDARD.encode(&saml_response);

        // Return auto-submit form that POSTs to ACS
        let html = auto_submit_form(&acs_url, &saml_response_b64, &relay_state);
        return Ok(Html(html).into_response());
    }

    // Browser mode: show user picker with auto-submit buttons
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

    // Anyone on this tenant can sign in, whether or not they exist yet — the
    // configuration screen creates them and returns here with `login_id`.
    let new_user_url = {
        let continue_url = format!(
            "http://localhost:{}/emulator/idp/{}/sso?RelayState={}",
            state.config.port,
            idp_id,
            urlencoding::encode(&relay_state)
        );
        ResumeParams {
            email: None,
            tenant: Some(idp.tenant_id.clone()),
            mode: Some("continue".into()),
            redirect_url: None,
            continue_url: Some(continue_url),
        }
        .to_url(state.config.port)
    };

    let user_rows = if users.is_empty() {
        "<tr><td colspan=\"3\" style=\"text-align:center;padding:2rem;color:#888;\">No users yet — sign in as a new one below</td></tr>".to_string()
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
                        <td><a class="btn" href="/emulator/idp/{idp_id}/sso?RelayState={relay_state}&login_id={login_id}">Login</a></td>
                    </tr>"#,
                    name = xml_escape(name),
                    email = xml_escape(email),
                    idp_id = idp_id,
                    relay_state = urlencoding::encode(&relay_state),
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
  .badge{{display:inline-block;background:#da3633;color:#fff;font-size:.7rem;padding:.15rem .5rem;border-radius:4px;margin-left:.5rem;vertical-align:middle}}
  table{{width:100%;border-collapse:collapse}}
  th{{text-align:left;color:#8b949e;font-size:.75rem;text-transform:uppercase;padding:.5rem;border-bottom:1px solid #30363d}}
  td{{padding:.75rem .5rem;border-bottom:1px solid #21262d}}
  .btn{{display:inline-block;background:#39d353;color:#0d1117;padding:.4rem 1rem;border-radius:6px;text-decoration:none;font-size:.85rem;font-weight:600}}
  .btn:hover{{background:#2ea043}}
  .new-user{{display:inline-block;margin-top:1.25rem;color:#58a6ff;text-decoration:none;font-size:.85rem}}
  .new-user:hover{{text-decoration:underline}}
</style>
</head>
<body>
<div class="card">
  <h1>⚡ Rescope IdP Emulator <span class="badge">SAML</span></h1>
  <div class="subtitle">{display_name} — Select a user to sign in</div>
  <table>
    <thead><tr><th>Name</th><th>Email</th><th></th></tr></thead>
    <tbody>{user_rows}</tbody>
  </table>
  <a class="new-user" href="{new_user_url}">+ Sign in as a new user</a>
</div>
</body>
</html>"#,
        display_name = xml_escape(&idp.display_name),
        user_rows = user_rows,
        new_user_url = xml_escape(&new_user_url),
    );

    Ok(Html(html).into_response())
}

// ─── SAML ACS Callback (SP-side) ────────────────────────────────────────────

/// Receives a SAML Response POST from the IdP, extracts the user info,
/// generates an SP code, and redirects to the app.
pub async fn saml_acs(
    State(state): State<EmulatorState>,
    body: String,
) -> Result<Response, EmulatorError> {
    // Parse form body: SAMLResponse + RelayState
    let form: HashMap<String, String> = serde_urlencoded::from_str(&body)
        .map_err(|e| EmulatorError::ValidationError(format!("Invalid form body: {e}")))?;

    let saml_response_b64 = form
        .get("SAMLResponse")
        .ok_or(EmulatorError::ValidationError(
            "Missing SAMLResponse".into(),
        ))?;
    let relay_state = form.get("RelayState").cloned().unwrap_or_default();

    // Decode SAML Response
    let saml_xml = String::from_utf8(
        STANDARD
            .decode(saml_response_b64)
            .map_err(|e| EmulatorError::ValidationError(format!("Invalid base64: {e}")))?,
    )
    .map_err(|e| EmulatorError::ValidationError(format!("Invalid UTF-8: {e}")))?;

    // Extract NameID from the XML (simple substring extraction)
    let name_id = extract_xml_value(&saml_xml, "saml:NameID").ok_or(
        EmulatorError::ValidationError("Missing NameID in SAML Response".into()),
    )?;

    // Find the user by login_id (NameID). An assertion for someone the emulator
    // has never seen provisions them into the SSO tenant that owns their
    // domain — the same just-in-time behavior a real deployment has.
    let known_user_id = {
        let users = state.users.read().await;
        users.load(&name_id).ok().map(|u| u.user_id.clone())
    };
    let user_id = match known_user_id {
        Some(user_id) => user_id,
        None => {
            let email = normalize_email(&name_id).ok_or(EmulatorError::UserNotFound)?;
            let tenant = resolve_sso_tenant(&state, &email)
                .await
                .map_err(|_| EmulatorError::UserNotFound)?;
            crate::sso_jit::provision_sso_user(&state, &tenant, SsoProvisionInput::new(email))
                .await?
                .user_id
        }
    };

    // Generate SP code
    let sp_code = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(sp_code.clone(), user_id, TokenType::Saml);

    // Redirect to app
    let url = if relay_state.is_empty() {
        format!("?code={}", sp_code)
    } else {
        format!("{}?code={}", relay_state, sp_code)
    };

    Ok(axum::response::Redirect::to(&url).into_response())
}

// ─── SAML Response Generation ────────────────────────────────────────────────

fn generate_saml_response(
    user: &crate::types::User,
    issuer: &str,
    acs_url: &str,
    attribute_mapping: &HashMap<String, String>,
) -> Result<String, EmulatorError> {
    let now = now_secs();
    let response_id = format!("_resp_{}", Uuid::new_v4().as_simple());
    let assertion_id = format!("_assert_{}", Uuid::new_v4().as_simple());
    let issue_instant = utc_iso(now);
    let not_before = utc_iso(now.saturating_sub(300)); // 5 min grace
    let not_on_or_after = utc_iso(now + 3600);
    let authn_instant = utc_iso(now);

    let login_id = user
        .login_ids
        .first()
        .cloned()
        .unwrap_or_else(|| user.user_id.clone());

    // Build attributes from mapping
    let user_json = serde_json::to_value(user).unwrap_or_default();
    let mut attributes = String::new();
    for (saml_name, user_field) in attribute_mapping {
        let value = resolve_user_field(&user_json, user_field);
        if let Some(s) = value.as_str() {
            attributes.push_str(&format!(
                r#"      <saml:Attribute Name="{name}" NameFormat="urn:oasis:names:tc:SAML:2.0:attrname-format:basic">
        <saml:AttributeValue>{value}</saml:AttributeValue>
      </saml:Attribute>
"#,
                name = xml_escape(saml_name),
                value = xml_escape(s),
            ));
        }
    }

    // Default email attribute if not in mapping
    if !attribute_mapping.contains_key("email") {
        if let Some(ref email) = user.email {
            attributes.push_str(&format!(
                r#"      <saml:Attribute Name="email" NameFormat="urn:oasis:names:tc:SAML:2.0:attrname-format:basic">
        <saml:AttributeValue>{}</saml:AttributeValue>
      </saml:Attribute>
"#,
                xml_escape(email),
            ));
        }
    }

    let xml = format!(
        r#"<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
                xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
                ID="{response_id}"
                Version="2.0"
                IssueInstant="{issue_instant}"
                Destination="{acs_url}"
                InResponseTo="_emulator_authn_request">
  <samlp:Status>
    <samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success"/>
  </samlp:Status>
  <saml:Assertion ID="{assertion_id}"
                  Version="2.0"
                  IssueInstant="{issue_instant}">
    <saml:Issuer>{issuer}</saml:Issuer>
    <saml:Subject>
      <saml:NameID Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress">{name_id}</saml:NameID>
      <saml:SubjectConfirmation Method="urn:oasis:names:tc:SAML:2.0:cm:bearer">
        <saml:SubjectConfirmationData NotOnOrAfter="{not_on_or_after}"
                                      Recipient="{acs_url}"
                                      InResponseTo="_emulator_authn_request"/>
      </saml:SubjectConfirmation>
    </saml:Subject>
    <saml:Conditions NotBefore="{not_before}" NotOnOrAfter="{not_on_or_after}">
      <saml:AudienceRestriction>
        <saml:Audience>{acs_url}</saml:Audience>
      </saml:AudienceRestriction>
    </saml:Conditions>
    <saml:AuthnStatement AuthnInstant="{authn_instant}"
                         SessionIndex="{assertion_id}">
      <saml:AuthnContext>
        <saml:AuthnContextClassRef>urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport</saml:AuthnContextClassRef>
      </saml:AuthnContext>
    </saml:AuthnStatement>
    <saml:AttributeStatement>
{attributes}    </saml:AttributeStatement>
  </saml:Assertion>
</samlp:Response>"#,
        response_id = xml_escape(&response_id),
        assertion_id = xml_escape(&assertion_id),
        issue_instant = issue_instant,
        acs_url = xml_escape(acs_url),
        issuer = xml_escape(issuer),
        name_id = xml_escape(&login_id),
        not_before = not_before,
        not_on_or_after = not_on_or_after,
        authn_instant = authn_instant,
        attributes = attributes,
    );

    Ok(xml)
}

/// Generate an auto-submit HTML form that POSTs SAMLResponse + RelayState to the ACS URL.
fn auto_submit_form(acs_url: &str, saml_response_b64: &str, relay_state: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<body onload="document.forms[0].submit()">
<form method="POST" action="{acs_url}">
  <input type="hidden" name="SAMLResponse" value="{saml_response}" />
  <input type="hidden" name="RelayState" value="{relay_state}" />
  <noscript><button type="submit">Submit</button></noscript>
</form>
</body>
</html>"#,
        acs_url = xml_escape(acs_url),
        saml_response = saml_response_b64,
        relay_state = xml_escape(relay_state),
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Extract text content of a simple XML element by tag name.
fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    if let Some(start_idx) = xml.find(&open) {
        // Find the > after the opening tag (skip attributes)
        let after_tag = &xml[start_idx + open.len()..];
        if let Some(gt_pos) = after_tag.find('>') {
            let content_start = start_idx + open.len() + gt_pos + 1;
            if let Some(end_idx) = xml[content_start..].find(&close) {
                let value = &xml[content_start..content_start + end_idx];
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Resolve a dotted path like "user.email" against a JSON value.
fn resolve_user_field(user_json: &Value, path: &str) -> Value {
    let path = path.strip_prefix("user.").unwrap_or(path);
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = user_json;
    for part in &parts {
        match current.get(part).or_else(|| {
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
        store::idp_store::IdpEmulator,
        types::{AuthType, Tenant, User},
    };
    use axum::extract::{Path as AxumPath, Query as AxumQuery, State as AxumState};

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
            .idp_emulators
            .write()
            .await
            .insert(IdpEmulator {
                id: "mock-idp".into(),
                protocol: IdpProtocol::Saml,
                display_name: "Mock IdP".into(),
                tenant_id: "acme".into(),
                attribute_mapping: HashMap::new(),
            })
            .unwrap();
        state
    }

    async fn body_text(resp: Response) -> String {
        String::from_utf8(
            axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap()
    }

    fn sso_params(login_id: Option<&str>) -> SsoParams {
        SsoParams {
            saml_request: None,
            relay_state: Some("http://localhost:4200/cb".into()),
            login_id: login_id.map(str::to_string),
        }
    }

    /// Why: naming an address the emulator has never seen used to fail the whole
    ///      sign-in, so SSO only worked for pre-seeded users.
    /// Decision: the IdP is authoritative for its tenant, so provision the user
    ///      and go on to issue the assertion.
    #[tokio::test]
    async fn sso_provisions_an_unknown_login_id_and_still_asserts() {
        let state = make_state().await;

        let resp = sso(
            AxumState(state.clone()),
            AxumPath("mock-idp".into()),
            AxumQuery(sso_params(Some("newcomer@acme.test"))),
        )
        .await
        .unwrap();

        let html = body_text(resp).await;
        assert!(html.contains("SAMLResponse"));
        assert!(html.contains("onload"));

        let users = state.users.read().await;
        let user = users.load("newcomer@acme.test").expect("provisioned");
        assert_eq!(user.status, "enabled");
        assert_eq!(user.user_tenants[0].tenant_id, "acme");
    }

    /// Why: the picker only listed users that already existed, leaving a browser
    ///      sign-in no way to become somebody new.
    /// Decision: link to the configuration screen, which returns here with the
    ///      new address as `login_id`.
    #[tokio::test]
    async fn sso_picker_offers_a_new_user_entry_point() {
        let state = make_state().await;

        let resp = sso(
            AxumState(state),
            AxumPath("mock-idp".into()),
            AxumQuery(sso_params(None)),
        )
        .await
        .unwrap();

        let html = body_text(resp).await;
        assert!(html.contains("Sign in as a new user"));
        assert!(html.contains("/emulator/sso/provision"));
        assert!(html.contains("mode=continue"));
    }

    /// Why: an assertion posted straight to the consumer (the shape a real IdP
    ///      sends) is the other way an unknown user arrives, and it failed the
    ///      same way.
    /// Decision: provision from the NameID, into the tenant owning its domain.
    #[tokio::test]
    async fn acs_provisions_a_user_named_only_by_the_assertion() {
        let state = make_state().await;
        let assertion = generate_saml_response(
            &User {
                user_id: "asserted".into(),
                login_ids: vec!["asserted@acme.test".into()],
                email: Some("asserted@acme.test".into()),
                ..Default::default()
            },
            "http://idp",
            "http://acs",
            &HashMap::new(),
        )
        .unwrap();
        let body = serde_urlencoded::to_string([
            ("SAMLResponse", STANDARD.encode(&assertion)),
            ("RelayState", "http://localhost:4200/cb".into()),
        ])
        .unwrap();

        let resp = saml_acs(AxumState(state.clone()), body).await.unwrap();

        let location = resp
            .headers()
            .get(axum::http::header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.starts_with("http://localhost:4200/cb?code="));

        let users = state.users.read().await;
        assert!(users.load("asserted@acme.test").is_ok());
    }

    /// Why: provisioning is scoped to SSO tenants — an assertion for a domain no
    ///      tenant claims is not something the emulator can place.
    /// Decision: keep rejecting it rather than inventing a tenant.
    #[tokio::test]
    async fn acs_rejects_an_assertion_for_an_unclaimed_domain() {
        let state = make_state().await;
        let assertion = generate_saml_response(
            &User {
                user_id: "stranger".into(),
                login_ids: vec!["stranger@elsewhere.test".into()],
                ..Default::default()
            },
            "http://idp",
            "http://acs",
            &HashMap::new(),
        )
        .unwrap();
        let body =
            serde_urlencoded::to_string([("SAMLResponse", STANDARD.encode(&assertion))]).unwrap();

        let err = saml_acs(AxumState(state), body).await.unwrap_err();
        assert!(matches!(err, EmulatorError::UserNotFound));
    }

    #[test]
    fn utc_iso_formats_correctly() {
        // 2024-01-01T00:00:00Z
        assert_eq!(utc_iso(1704067200), "2024-01-01T00:00:00Z");
    }

    #[test]
    fn xml_escape_handles_special_chars() {
        assert_eq!(xml_escape("<a&b>"), "&lt;a&amp;b&gt;");
        assert_eq!(xml_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn extract_xml_value_works() {
        let xml = r#"<saml:NameID Format="email">alice@acme.com</saml:NameID>"#;
        assert_eq!(
            extract_xml_value(xml, "saml:NameID"),
            Some("alice@acme.com".into())
        );
    }

    #[test]
    fn extract_xml_value_missing_tag() {
        let xml = "<foo>bar</foo>";
        assert_eq!(extract_xml_value(xml, "baz"), None);
    }

    #[test]
    fn generate_saml_response_contains_required_elements() {
        let user = User {
            user_id: "u1".into(),
            login_ids: vec!["alice@acme.com".into()],
            email: Some("alice@acme.com".into()),
            name: Some("Alice".into()),
            ..Default::default()
        };
        let mapping = HashMap::from([
            ("email".into(), "user.email".into()),
            ("firstName".into(), "user.name".into()),
        ]);

        let xml = generate_saml_response(&user, "http://idp", "http://acs", &mapping).unwrap();

        assert!(xml.contains("samlp:Response"));
        assert!(xml.contains("saml:Assertion"));
        assert!(xml.contains("saml:Issuer"));
        assert!(xml.contains("alice@acme.com")); // NameID
        assert!(xml.contains("saml:AuthnStatement"));
        assert!(xml.contains("saml:AttributeStatement"));
        assert!(xml.contains(r#"Name="email""#));
        assert!(xml.contains(r#"Name="firstName""#));
    }

    #[test]
    fn generate_saml_response_default_email_attribute() {
        let user = User {
            user_id: "u1".into(),
            login_ids: vec!["alice@acme.com".into()],
            email: Some("alice@acme.com".into()),
            ..Default::default()
        };
        // Empty mapping → email attribute should be auto-added
        let xml =
            generate_saml_response(&user, "http://idp", "http://acs", &HashMap::new()).unwrap();
        assert!(xml.contains(r#"Name="email""#));
        assert!(xml.contains("alice@acme.com"));
    }

    #[test]
    fn auto_submit_form_contains_fields() {
        let html = auto_submit_form("http://acs", "base64data", "relay123");
        assert!(html.contains("http://acs"));
        assert!(html.contains("base64data"));
        assert!(html.contains("relay123"));
        assert!(html.contains("SAMLResponse"));
        assert!(html.contains("RelayState"));
        assert!(html.contains("onload"));
    }

    #[test]
    fn generate_x509_cert_works() {
        let km = crate::jwt::key_manager::KeyManager::generate().unwrap();
        let (pem, b64) = generate_x509_cert(&km.private_pem).unwrap();
        assert!(pem.starts_with("-----BEGIN CERTIFICATE-----"));
        assert!(!b64.is_empty());
        // b64 should be pure base64 without headers
        assert!(!b64.contains("-----"));
    }
}
