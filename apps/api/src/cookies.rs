use axum::http::header::SET_COOKIE;
use axum::http::HeaderMap;

/// Build Set-Cookie headers for DS (session) and DSR (refresh) cookies.
///
/// Both cookies use `SameSite=None; Secure` so they are sent on cross-origin
/// requests — e.g. a browser app on `http://localhost:4200` calling this
/// emulator on `http://localhost:4600`. `Secure` is mandatory alongside
/// `SameSite=None`; browsers treat `http://localhost` as a secure context, so
/// the attribute is honored over plain HTTP there.
///
/// The DS (session) cookie is intentionally NOT `HttpOnly`, matching cloud
/// Descope: the web SDK's `getSessionToken()` reads the session JWT straight
/// from `document.cookie`, so the session survives a full-page reload without a
/// re-login. The DSR (refresh) cookie stays `HttpOnly`
/// — JS never reads it; it rides along on the credentialed refresh request.
pub fn build_auth_cookies(session_jwt: &str, refresh_jwt: &str, session_ttl: u64) -> HeaderMap {
    let mut headers = HeaderMap::new();

    let ds = format!(
        "DS={}; SameSite=None; Secure; Path=/; Max-Age={}",
        session_jwt, session_ttl
    );
    let dsr = format!(
        "DSR={}; HttpOnly; SameSite=None; Secure; Path=/",
        refresh_jwt
    );

    headers.append(SET_COOKIE, ds.parse().expect("valid DS cookie"));
    headers.append(SET_COOKIE, dsr.parse().expect("valid DSR cookie"));
    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_ds_and_dsr_cookies() {
        let headers = build_auth_cookies("session.tok", "refresh.tok", 3600);
        let cookies: Vec<&str> = headers
            .get_all(SET_COOKIE)
            .into_iter()
            .map(|v| v.to_str().unwrap())
            .collect();
        assert_eq!(cookies.len(), 2);
        assert!(cookies.iter().any(|c| c.starts_with("DS=")));
        assert!(cookies.iter().any(|c| c.starts_with("DSR=")));
    }

    #[test]
    fn ds_cookie_is_js_readable_and_cross_origin() {
        let headers = build_auth_cookies("s", "r", 3600);
        let ds = headers
            .get_all(SET_COOKIE)
            .into_iter()
            .map(|v| v.to_str().unwrap().to_string())
            .find(|c| c.starts_with("DS="))
            .unwrap();
        // DS must NOT be HttpOnly — the web SDK reads the session token from
        // document.cookie (getSessionToken), exactly as it does against cloud.
        assert!(!ds.contains("HttpOnly"));
        assert!(ds.contains("SameSite=None"));
        assert!(ds.contains("Secure"));
        assert!(ds.contains("Path=/"));
    }

    #[test]
    fn dsr_cookie_is_cross_origin_capable() {
        let headers = build_auth_cookies("s", "r", 3600);
        let dsr = headers
            .get_all(SET_COOKIE)
            .into_iter()
            .map(|v| v.to_str().unwrap().to_string())
            .find(|c| c.starts_with("DSR="))
            .unwrap();
        assert!(dsr.contains("HttpOnly"));
        assert!(dsr.contains("SameSite=None"));
        assert!(dsr.contains("Secure"));
    }
}
