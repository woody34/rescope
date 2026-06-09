use crate::{
    error::EmulatorError,
    jwt::key_manager::KeyManager,
    types::{RefreshClaims, SessionClaims},
};
use jsonwebtoken::{decode, Algorithm, Validation};

pub fn validate_session_jwt(km: &KeyManager, token: &str) -> Result<SessionClaims, EmulatorError> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;

    decode::<SessionClaims>(token, &km.decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|e| {
            if e.kind() == &jsonwebtoken::errors::ErrorKind::ExpiredSignature {
                EmulatorError::TokenExpired
            } else {
                EmulatorError::InvalidToken
            }
        })
}

pub fn validate_refresh_jwt(km: &KeyManager, token: &str) -> Result<RefreshClaims, EmulatorError> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;

    decode::<RefreshClaims>(token, &km.decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|e| {
            if e.kind() == &jsonwebtoken::errors::ErrorKind::ExpiredSignature {
                EmulatorError::TokenExpired
            } else {
                EmulatorError::InvalidToken
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::token_generator::{generate_refresh_jwt, generate_session_jwt};
    use crate::store::role_store::RoleStore;
    use crate::types::User;

    fn km() -> std::sync::Arc<KeyManager> {
        KeyManager::generate().unwrap()
    }

    fn user() -> User {
        let mut u = User::default();
        u.user_id = "u1".into();
        u
    }

    fn roles() -> RoleStore {
        RoleStore::default()
    }

    #[test]
    fn valid_session_token_passes() {
        let km = km();
        let jwt = generate_session_jwt(&km, &user(), "proj", 3600, None, &roles(), "otp").unwrap();
        let claims = validate_session_jwt(&km, &jwt).unwrap();
        assert_eq!(claims.sub, "u1");
    }

    #[test]
    fn expired_session_token_is_rejected() {
        let km = km();
        let mut header = jsonwebtoken::Header::new(Algorithm::RS256);
        header.kid = Some(km.kid.clone());
        let claims = crate::types::SessionClaims {
            sub: "u1".into(),
            iss: "proj".into(),
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
            descope_id: "u1".into(),
            tenants: std::collections::HashMap::new(),
        };
        let jwt = jsonwebtoken::encode(&header, &claims, &km.encoding_key).unwrap();
        let err = validate_session_jwt(&km, &jwt).unwrap_err();
        assert!(matches!(err, EmulatorError::TokenExpired));
    }

    #[test]
    fn wrong_key_session_token_is_rejected() {
        let km1 = km();
        let km2 = km();
        let jwt = generate_session_jwt(&km1, &user(), "proj", 3600, None, &roles(), "otp").unwrap();
        let err = validate_session_jwt(&km2, &jwt).unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidToken));
    }

    #[test]
    fn valid_refresh_token_passes() {
        let km = km();
        let jwt = generate_refresh_jwt(&km, "u1", "proj", 86400).unwrap();
        let claims = validate_refresh_jwt(&km, &jwt).unwrap();
        assert_eq!(claims.sub, "u1");
    }

    #[test]
    fn expired_refresh_token_is_rejected() {
        let km = km();
        let mut header = jsonwebtoken::Header::new(Algorithm::RS256);
        header.kid = Some(km.kid.clone());
        let claims = crate::types::RefreshClaims {
            sub: "u1".into(),
            iss: "proj".into(),
            exp: 1,
            iat: 1,
            drn: "DSR".into(),
        };
        let jwt = jsonwebtoken::encode(&header, &claims, &km.encoding_key).unwrap();
        let err = validate_refresh_jwt(&km, &jwt).unwrap_err();
        assert!(matches!(err, EmulatorError::TokenExpired));
    }

    #[test]
    fn garbage_token_is_rejected() {
        let km = km();
        let err = validate_session_jwt(&km, "not.a.jwt").unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidToken));
    }
}
