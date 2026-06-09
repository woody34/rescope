use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum EmulatorError {
    #[error("user not found")]
    UserNotFound,

    #[error("user already exists")]
    UserAlreadyExists,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("invalid token")]
    InvalidToken,

    #[error("token expired")]
    TokenExpired,

    #[error("tenant not found")]
    TenantNotFound,

    #[error("tenant already exists")]
    TenantAlreadyExists,

    #[error("user is not configured for SSO")]
    NotSsoUser,

    #[error("user is not a test user")]
    NotTestUser,

    #[error("user is disabled")]
    UserDisabled,

    #[error("missing or invalid authorization header")]
    Unauthorized,

    #[error("too many requests")]
    TooManyRequests,

    #[error("auth method is disabled")]
    AuthMethodDisabled,

    #[error("permission not found")]
    PermissionNotFound,

    #[error("permission already exists")]
    PermissionAlreadyExists,

    #[error("role not found")]
    RoleNotFound,

    #[error("role already exists")]
    RoleAlreadyExists,

    #[error("JWT template not found")]
    JwtTemplateNotFound,

    #[error("JWT template already exists")]
    JwtTemplateAlreadyExists,

    #[error("connector not found")]
    ConnectorNotFound,

    #[error("custom attribute not found")]
    CustomAttributeNotFound,

    #[error("custom attribute already exists")]
    CustomAttributeAlreadyExists,

    #[error("access key not found")]
    AccessKeyNotFound,

    #[error("IdP emulator not found")]
    IdpNotFound,

    #[error("password policy violation: {0}")]
    PasswordPolicy(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("validation error: {0}")]
    ValidationError(String),
}

impl EmulatorError {
    /// Returns (HTTP status code, Descope error code, human description).
    /// This is framework-agnostic — the API layer converts to axum Response.
    pub fn status_and_code(&self) -> (u16, &'static str, &'static str) {
        match self {
            Self::UserNotFound => (400, "E112102", "User not found"),
            Self::UserAlreadyExists => (400, "E062108", "User already exists"),
            Self::InvalidCredentials => (401, "E011003", "Invalid credentials"),
            Self::InvalidToken => (401, "E011003", "Invalid token"),
            Self::TokenExpired => (401, "E011003", "Token expired"),
            Self::TenantNotFound => (400, "E062108", "Could not find tenant"),
            Self::TenantAlreadyExists => (400, "E062108", "Tenant already exists"),
            Self::NotSsoUser => (400, "E062108", "User is not configured for SSO"),
            Self::NotTestUser => (400, "E062108", "User is not a test user"),
            Self::UserDisabled => (403, "E011006", "User is disabled"),
            Self::Unauthorized => (401, "E011004", "Missing or invalid authorization"),
            Self::TooManyRequests => (429, "E011005", "Too many requests"),
            Self::AuthMethodDisabled => (403, "E011006", "Auth method is disabled"),
            Self::PermissionNotFound => (400, "E062108", "Permission not found"),
            Self::PermissionAlreadyExists => (409, "E062109", "Permission already exists"),
            Self::RoleNotFound => (400, "E062108", "Role not found"),
            Self::RoleAlreadyExists => (409, "E062109", "Role already exists"),
            Self::JwtTemplateNotFound => (400, "E062108", "JWT template not found"),
            Self::JwtTemplateAlreadyExists => (409, "E062109", "JWT template already exists"),
            Self::ConnectorNotFound => (400, "E062108", "Connector not found"),
            Self::CustomAttributeNotFound => (400, "E062108", "Custom attribute not found"),
            Self::CustomAttributeAlreadyExists => {
                (409, "E062109", "Custom attribute already exists")
            }
            Self::AccessKeyNotFound => (400, "E062108", "Access key not found"),
            Self::IdpNotFound => (400, "E062108", "IdP emulator not found"),
            Self::PasswordPolicy(_) => (400, "E062108", "Password policy violation"),
            Self::Internal(_) => (500, "E000000", "Internal error"),
            Self::ValidationError(_) => (400, "E062110", "Validation error"),
        }
    }

    /// Serialize the error as a JSON body string (framework-agnostic).
    pub fn to_json_body(&self) -> String {
        let (_, error_code, description) = self.status_and_code();
        json!({
            "errorCode": error_code,
            "errorDescription": description,
        })
        .to_string()
    }
}

// ── Optional axum integration ────────────────────────────────────────────────
#[cfg(feature = "axum")]
mod axum_impl {
    use super::EmulatorError;
    use axum::{
        http::StatusCode,
        response::{IntoResponse, Response},
        Json,
    };
    use serde_json::json;

    impl IntoResponse for EmulatorError {
        fn into_response(self) -> Response {
            let (status_u16, error_code, description) = self.status_and_code();
            let status =
                StatusCode::from_u16(status_u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            // Match real Descope error envelope: only errorCode + errorDescription
            let body = json!({
                "errorCode": error_code,
                "errorDescription": description,
            });
            (status, Json(body)).into_response()
        }
    }
}
