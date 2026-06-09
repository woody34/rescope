use axum::{
    body::Body,
    http::{header, Response, StatusCode, Uri},
    routing::get,
    Router,
};
use rust_embed::Embed;

/// Embedded UI assets from `apps/ui/dist/`.
///
/// In debug builds, `rust-embed` reads from the filesystem automatically
/// (its default behavior when `debug-embed` feature is not set), so Vite
/// dev-server output is picked up without recompilation.
///
/// In release builds, assets are baked into the binary at compile time.
#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../ui/dist"]
struct UiAssets;

/// Build a router that serves embedded UI assets with SPA fallback.
///
/// This is used by `server.rs` for routes that do not match any API endpoint.
pub fn embedded_ui_router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new().fallback(get(serve_embedded_asset))
}

async fn serve_embedded_asset(uri: Uri) -> Response<Body> {
    let path = uri.path().trim_start_matches('/');

    // Try the exact path first
    if let Some(content) = UiAssets::get(path) {
        return build_response(path, &content.data);
    }

    // SPA fallback: serve index.html for any unmatched path
    match UiAssets::get("index.html") {
        Some(content) => build_response("index.html", &content.data),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("UI assets not found"))
            .unwrap(),
    }
}

fn build_response(path: &str, data: &[u8]) -> Response<Body> {
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .body(Body::from(data.to_vec()))
        .unwrap()
}
