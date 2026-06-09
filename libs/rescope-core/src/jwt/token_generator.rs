use crate::{
    jwt::key_manager::KeyManager,
    store::{
        jwt_template_store::{evaluate_template_claims, JwtTemplate},
        role_store::RoleStore,
    },
    types::{RefreshClaims, SessionClaims, TenantClaim, User},
};
use anyhow::{Context, Result};
use jsonwebtoken::{encode, Algorithm, Header};
use std::{
    collections::HashSet,
    time::{SystemTime, UNIX_EPOCH},
};

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_secs()
}

/// Resolve the de-duplicated union of permission names for a given set of role names.
/// Unknown role names are silently ignored (roles may be deleted after assignment).
fn resolve_permissions(role_names: &[String], roles: &RoleStore) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<String> = Vec::new();
    for rn in role_names {
        if let Ok(role) = roles.load(rn.as_str()) {
            for p in &role.permission_names {
                if seen.insert(p.clone()) {
                    out.push(p.clone());
                }
            }
        }
    }
    out
}

/// Build the tenants map from the user's tenant memberships.
/// Each entry carries roleNames, permissionNames (resolved from roles), and tenantName.
fn build_tenants(user: &User, roles: &RoleStore) -> std::collections::HashMap<String, TenantClaim> {
    user.user_tenants
        .iter()
        .map(|t| {
            let permission_names = resolve_permissions(&t.role_names, roles);
            (
                t.tenant_id.clone(),
                TenantClaim {
                    role_names: t.role_names.clone(),
                    permission_names,
                    tenant_name: t.tenant_name.clone(),
                },
            )
        })
        .collect()
}

pub fn generate_session_jwt(
    km: &KeyManager,
    user: &User,
    project_id: &str,
    ttl: u64,
    template: Option<&JwtTemplate>,
    roles: &RoleStore,
    method: &str,
) -> Result<String> {
    // If a template is active, evaluate its claims and merge them in.
    if let Some(tmpl) = template {
        let user_json = serde_json::to_value(user).unwrap_or_default();
        let extra = evaluate_template_claims(tmpl, &user_json);
        return generate_session_jwt_with_extra(km, user, project_id, ttl, &extra, roles, method);
    }

    let iat = now();
    let exp = iat + ttl;

    let attrs = &user.custom_attributes;
    let str_attr = |k: &str| {
        attrs
            .get(k)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let bool_attr = |k: &str| attrs.get(k).and_then(|v| v.as_bool()).unwrap_or(false);

    let tenants = build_tenants(user, roles);
    // Global permissions = union of permissions from all global roles.
    let global_permissions = resolve_permissions(&user.role_names, roles);
    let _ = global_permissions; // available but not embedded as top-level (Descope doesn't add global perms to JWT)

    let claims = SessionClaims {
        sub: user.user_id.clone(),
        iss: project_id.to_string(),
        exp,
        iat,
        drn: "DS".to_string(),
        amr: vec![method.to_string()],
        roles: user.role_names.clone(),
        email: user.email.clone().unwrap_or_default(),
        email_verified: user.verified_email,
        phone: user.phone.clone().unwrap_or_default(),
        phone_verified: user.verified_phone,
        name: user.name.clone().unwrap_or_default(),
        username: str_attr("username"),
        company: str_attr("company"),
        uid: str_attr("uid"),
        photo_url: user.picture.clone().unwrap_or_default(),
        is_super: bool_attr("super"),
        support: bool_attr("support"),
        descope_id: user.user_id.clone(),
        tenants,
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(km.kid.clone());

    encode(&header, &claims, &km.encoding_key).context("Failed to sign session JWT")
}

/// Generate a session JWT with additional top-level claims (e.g., `dct` for tenant selection).
pub fn generate_session_jwt_with_extra(
    km: &KeyManager,
    user: &User,
    project_id: &str,
    ttl: u64,
    extra: &std::collections::HashMap<String, serde_json::Value>,
    roles: &RoleStore,
    method: &str,
) -> Result<String> {
    let iat = now();
    let exp = iat + ttl;

    let attrs = &user.custom_attributes;
    let str_attr = |k: &str| {
        attrs
            .get(k)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let bool_attr = |k: &str| attrs.get(k).and_then(|v| v.as_bool()).unwrap_or(false);

    let tenants = build_tenants(user, roles);

    let base = SessionClaims {
        sub: user.user_id.clone(),
        iss: project_id.to_string(),
        exp,
        iat,
        drn: "DS".to_string(),
        amr: vec![method.to_string()],
        roles: user.role_names.clone(),
        email: user.email.clone().unwrap_or_default(),
        email_verified: user.verified_email,
        phone: user.phone.clone().unwrap_or_default(),
        phone_verified: user.verified_phone,
        name: user.name.clone().unwrap_or_default(),
        username: str_attr("username"),
        company: str_attr("company"),
        uid: str_attr("uid"),
        photo_url: user.picture.clone().unwrap_or_default(),
        is_super: bool_attr("super"),
        support: bool_attr("support"),
        descope_id: user.user_id.clone(),
        tenants,
    };

    // Merge base claims with extra claims as a JSON map
    let mut map = serde_json::to_value(&base)
        .context("Failed to serialize claims")?
        .as_object()
        .cloned()
        .unwrap_or_default();
    for (k, v) in extra {
        map.insert(k.clone(), v.clone());
    }

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(km.kid.clone());

    encode(&header, &serde_json::Value::Object(map), &km.encoding_key)
        .context("Failed to sign session JWT with extra claims")
}

pub fn generate_refresh_jwt(
    km: &KeyManager,
    user_id: &str,
    project_id: &str,
    ttl: u64,
) -> Result<String> {
    let iat = now();
    let exp = iat + ttl;

    let claims = RefreshClaims {
        sub: user_id.to_string(),
        iss: project_id.to_string(),
        exp,
        iat,
        drn: "DSR".to_string(),
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(km.kid.clone());

    encode(&header, &claims, &km.encoding_key).context("Failed to sign refresh JWT")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::token_validator::validate_session_jwt;
    use crate::types::User;
    use base64::Engine;

    fn test_km() -> std::sync::Arc<KeyManager> {
        KeyManager::generate().unwrap()
    }

    fn test_user() -> User {
        let mut user = User::default();
        user.user_id = "user-123".into();
        user.email = Some("alice@example.com".into());
        user.verified_email = true;
        user.name = Some("Alice".into());
        user
    }

    fn empty_roles() -> RoleStore {
        RoleStore::new()
    }

    #[test]
    fn session_jwt_has_correct_alg_and_kid() {
        let km = test_km();
        let jwt = generate_session_jwt(
            &km,
            &test_user(),
            "test-project",
            3600,
            None,
            &empty_roles(),
            "pwd",
        )
        .unwrap();
        let header = jsonwebtoken::decode_header(&jwt).unwrap();
        assert_eq!(header.alg, Algorithm::RS256);
        assert_eq!(header.kid.unwrap(), km.kid);
    }

    #[test]
    fn session_jwt_contains_required_claims() {
        let km = test_km();
        let jwt = generate_session_jwt(
            &km,
            &test_user(),
            "test-project",
            3600,
            None,
            &empty_roles(),
            "pwd",
        )
        .unwrap();
        let claims = validate_session_jwt(&km, &jwt).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.email, "alice@example.com");
        assert!(claims.email_verified);
        assert_eq!(claims.name, "Alice");
        assert_eq!(claims.iss, "test-project");
        assert_eq!(claims.drn, "DS");
        assert_eq!(claims.amr, vec!["pwd"]);
    }

    #[test]
    fn refresh_jwt_has_drn_dsr() {
        let km = test_km();
        let refresh = generate_refresh_jwt(&km, "user-123", "proj", 2_592_000).unwrap();
        let mut v = jsonwebtoken::Validation::new(Algorithm::RS256);
        v.validate_exp = false;
        let r_claims =
            jsonwebtoken::decode::<RefreshClaims>(&refresh, &km.decoding_key, &v).unwrap();
        assert_eq!(r_claims.claims.drn, "DSR");
    }

    #[test]
    fn refresh_jwt_has_longer_ttl_than_session() {
        let km = test_km();
        let session =
            generate_session_jwt(&km, &test_user(), "proj", 3600, None, &empty_roles(), "pwd")
                .unwrap();
        let refresh = generate_refresh_jwt(&km, "user-123", "proj", 2_592_000).unwrap();

        let mut v = jsonwebtoken::Validation::new(Algorithm::RS256);
        v.validate_exp = false;

        let s_claims =
            jsonwebtoken::decode::<SessionClaims>(&session, &km.decoding_key, &v).unwrap();
        let r_claims =
            jsonwebtoken::decode::<RefreshClaims>(&refresh, &km.decoding_key, &v).unwrap();

        assert!(r_claims.claims.exp > s_claims.claims.exp);
    }

    #[test]
    fn session_jwt_expired_is_rejected_by_validator() {
        let km = test_km();
        // Create a token with exp=1 (1970-01-01) directly
        let mut header = jsonwebtoken::Header::new(Algorithm::RS256);
        header.kid = Some(km.kid.clone());
        let claims = SessionClaims {
            sub: "u".into(),
            iss: "p".into(),
            exp: 1,
            iat: 1,
            drn: "DS".into(),
            amr: vec![],
            roles: vec![],
            email: "".into(),
            email_verified: false,
            phone: "".into(),
            phone_verified: false,
            name: "".into(),
            username: "".into(),
            company: "".into(),
            uid: "".into(),
            photo_url: "".into(),
            is_super: false,
            support: false,
            descope_id: "u".into(),
            tenants: Default::default(),
        };
        let jwt = jsonwebtoken::encode(&header, &claims, &km.encoding_key).unwrap();
        let result = validate_session_jwt(&km, &jwt);
        assert!(result.is_err());
    }

    #[test]
    fn tenant_permissions_resolved_from_role_store() {
        let km = test_km();
        let mut roles = RoleStore::new();
        roles
            .create(
                "admin".into(),
                "".into(),
                vec!["read".into(), "write".into()],
            )
            .unwrap();

        let mut user = test_user();
        user.user_tenants = vec![crate::types::UserTenant {
            tenant_id: "t1".into(),
            tenant_name: "Corp".into(),
            role_names: vec!["admin".into()],
        }];

        let jwt = generate_session_jwt(&km, &user, "proj", 3600, None, &roles, "email").unwrap();
        let payload: serde_json::Value = {
            let part = jwt.split('.').nth(1).unwrap();
            let decoded = String::from_utf8(
                base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .decode(part)
                    .unwrap(),
            )
            .unwrap();
            serde_json::from_str(&decoded).unwrap()
        };

        let perms = &payload["tenants"]["t1"]["permissionNames"];
        assert!(perms
            .as_array()
            .unwrap()
            .contains(&serde_json::Value::String("read".into())));
        assert!(perms
            .as_array()
            .unwrap()
            .contains(&serde_json::Value::String("write".into())));

        let tn = &payload["tenants"]["t1"]["tenantName"];
        assert_eq!(tn.as_str().unwrap(), "Corp");
    }
}
