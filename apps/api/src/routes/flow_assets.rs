//! Static flow-runtime assets served for the `descope-wc` web component.
//!
//! The widget fetches its screen config and per-screen HTML from
//! `<base-static-url>/pages/<projectId>/<version>/<file>`. We embed the three
//! assets at compile time and serve them for ANY project id / version path
//! segment, so the emulator answers regardless of which project the SDK targets.

use axum::{
    body::Body,
    http::{header, Response, StatusCode, Uri},
};

const CONFIG_JSON: &str = include_str!("../../assets/flow/config.json");
const SIGN_IN_HTML: &str = include_str!("../../assets/flow/signIn.html");
const SIGN_IN_PASSWORD_HTML: &str = include_str!("../../assets/flow/signInPassword.html");

/// GET /pages/*rest — serve embedded flow assets by the request path tail.
///
/// Matching ignores the projectId/version path segments and keys off the file
/// name only:
///
/// * `.../config.json`          → config.json (application/json)
/// * `.../signInPassword.html`  → password screen (text/html)
/// * `.../signIn.html`          → email screen (text/html)
///
/// Anything else → 404.
pub async fn serve(uri: Uri) -> Response<Body> {
    let path = uri.path();

    // Order matters: signInPassword.html also ends with "Password.html", but
    // signIn.html would NOT match it — still, check the more specific one first.
    let (body, content_type) = if path.ends_with("config.json") {
        (CONFIG_JSON, "application/json")
    } else if path.ends_with("signInPassword.html") {
        (SIGN_IN_PASSWORD_HTML, "text/html; charset=utf-8")
    } else if path.ends_with("signIn.html") {
        (SIGN_IN_HTML, "text/html; charset=utf-8")
    } else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("flow asset not found"))
            .unwrap();
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap()
}
