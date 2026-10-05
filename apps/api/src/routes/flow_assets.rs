//! Static flow-runtime assets served for the `descope-wc` web component.
//!
//! The widget fetches its screen config and per-screen HTML from
//! `<base-static-url>/pages/<projectId>/<version>/<file>`. We embed the three
//! assets at compile time and serve them for ANY project id / version path
//! segment, so the emulator answers regardless of which project the SDK targets.
//!
//! The theme is not embedded: `DESCOPE_EMULATOR_THEME_FILE` points at a
//! compiled theme (what Descope serves at `.../v2-beta/theme.json`), read on
//! every request so edits show on the next page load.

use axum::{
    body::Body,
    extract::State,
    http::{header, Response, StatusCode, Uri},
};

use crate::state::EmulatorState;

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
/// * `.../signIn.html`          → sign-in screen: email or username (text/html)
/// * `.../theme.json`           → the configured theme file, 404 when none is set
///
/// Anything else → 404.
pub async fn serve(State(state): State<EmulatorState>, uri: Uri) -> Response<Body> {
    let path = uri.path();

    if path.ends_with("theme.json") {
        return serve_theme(state.config.theme_file.as_deref());
    }

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

/// Serve the theme file verbatim. A console style export (`{"theme": {"styles":
/// ...}}`) holds editor tokens that Descope compiles server-side into the CSS
/// strings the widget injects; the emulator does not reimplement that compiler,
/// so it refuses the export instead of rendering an unstyled flow.
fn serve_theme(theme_file: Option<&str>) -> Response<Body> {
    let Some(theme_file) = theme_file else {
        return plain(StatusCode::NOT_FOUND, "flow asset not found".to_string());
    };

    let body = match std::fs::read_to_string(theme_file) {
        Ok(body) => body,
        Err(err) => {
            return plain(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("cannot read DESCOPE_EMULATOR_THEME_FILE {theme_file}: {err}"),
            )
        }
    };

    let is_console_export = serde_json::from_str::<serde_json::Value>(&body)
        .map(|theme| theme.pointer("/theme/styles").is_some())
        .unwrap_or(false);
    if is_console_export {
        return plain(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "{theme_file} is a console style export; point DESCOPE_EMULATOR_THEME_FILE at \
                 the compiled theme from https://static.descope.com/pages/<projectId>/v2-beta/theme.json"
            ),
        );
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap()
}

fn plain(status: StatusCode, message: String) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(message))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use crate::config::EmulatorConfig;
    use crate::server::build_router;
    use crate::state::EmulatorState;
    use axum_test::TestServer;
    use std::io::Write;

    async fn server_with_theme(theme_file: Option<String>) -> TestServer {
        let config = EmulatorConfig {
            theme_file,
            ..EmulatorConfig::default()
        };
        let state = EmulatorState::new(&config).await.unwrap();
        TestServer::new(build_router(state)).unwrap()
    }

    fn theme_file(contents: &str) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(contents.as_bytes()).unwrap();
        file
    }

    #[tokio::test]
    async fn sign_in_screen_labels_the_field_above_it_with_a_full_width_continue() {
        let server = server_with_theme(None).await;

        let html = server.get("/pages/PROJ/v2-beta/signIn.html").await.text();

        assert!(html.contains(r#"label="Email or Username""#));
        assert!(html.contains(r#"label-type="static""#));
        assert!(html.contains(r#"full-width="true" id="Ppb_65tyyn""#));
        assert_eq!(html.matches("<descope-button").count(), 1);
    }

    #[tokio::test]
    async fn password_screen_titles_the_step_and_signs_in_with_a_full_width_button() {
        let server = server_with_theme(None).await;

        let html = server
            .get("/pages/PROJ/v2-beta/signInPassword.html")
            .await
            .text();

        assert!(html.contains(r#"variant="h3">Enter your password</descope-text>"#));
        assert!(html.contains(r#"label="Password""#));
        assert!(html.contains(r#"full-width="true" id="Ppb_65tyyp""#));
        assert!(html.contains(">Sign in</descope-button>"));
        assert_eq!(html.matches("<descope-button").count(), 1);
    }

    #[tokio::test]
    async fn sign_in_screens_will_not_submit_an_empty_field() {
        let server = server_with_theme(None).await;

        for (screen, field) in [("signIn", "email"), ("signInPassword", "password")] {
            let html = server
                .get(&format!("/pages/PROJ/v2-beta/{screen}.html"))
                .await
                .text();
            let input = html
                .split("<descope-text-field")
                .nth(1)
                .and_then(|rest| rest.split('>').next())
                .unwrap();

            assert!(input.contains(&format!(r#"name="{field}""#)));
            assert!(
                input.contains(r#"required="true""#),
                "{screen} lets an empty {field} through"
            );
        }
    }

    const COMPILED_THEME: &str = r#"{"light":{"globals":"[data-theme=light]{--descope-colors-primary-main:#131340}","components":{}},"dark":{"globals":"","components":{}}}"#;

    #[tokio::test]
    async fn without_a_theme_file_the_theme_is_not_found() {
        let server = server_with_theme(None).await;

        server
            .get("/pages/PROJ/v2-beta/theme.json")
            .await
            .assert_status_not_found();
    }

    #[tokio::test]
    async fn serves_the_configured_theme_file_as_theme_json() {
        let file = theme_file(COMPILED_THEME);
        let server = server_with_theme(Some(file.path().display().to_string())).await;

        let response = server.get("/pages/PROJ/v2-beta/theme.json").await;

        response.assert_status_ok();
        response.assert_header("content-type", "application/json");
        assert_eq!(response.text(), COMPILED_THEME);
    }

    #[tokio::test]
    async fn serves_edits_to_the_theme_file_without_a_restart() {
        let file = theme_file(COMPILED_THEME);
        let server = server_with_theme(Some(file.path().display().to_string())).await;
        server
            .get("/pages/PROJ/v2-beta/theme.json")
            .await
            .assert_status_ok();

        let edited = COMPILED_THEME.replace("#131340", "#1D4ED8");
        std::fs::write(file.path(), &edited).unwrap();

        assert_eq!(
            server.get("/pages/PROJ/v2-beta/theme.json").await.text(),
            edited
        );
    }

    #[tokio::test]
    async fn refuses_a_style_exported_from_the_descope_console() {
        let file = theme_file(
            r#"{"theme":{"componentsVersion":"2.2.10","styles":{"light":{"globals":{},"components":{}}}}}"#,
        );
        let server = server_with_theme(Some(file.path().display().to_string())).await;

        let response = server.get("/pages/PROJ/v2-beta/theme.json").await;

        response.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        assert!(response.text().contains("console style export"));
    }

    #[tokio::test]
    async fn reports_a_theme_file_that_cannot_be_read() {
        let server = server_with_theme(Some("/nonexistent/theme.json".to_string())).await;

        let response = server.get("/pages/PROJ/v2-beta/theme.json").await;

        response.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        assert!(response.text().contains("/nonexistent/theme.json"));
    }
}
