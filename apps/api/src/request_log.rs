//! Per-request terminal logging middleware.
//!
//! Emits exactly one line per HTTP request, on completion, under the
//! `rescope::http` target so it is visible with the default env filter
//! (`rescope=info`). Lines carry a category tag derived from the path:
//!
//! ```text
//! INFO  rescope::http: [API] POST /v1/auth/otp/verify/email 401 2ms body={"loginId":"x"} resp={"errorCode":"E061102"}…
//! DEBUG rescope::http: [UI] GET /assets/index-Bx2.js 200 0ms
//! ```
//!
//! `[API]`/`[EMU]` log at INFO (except `/health`, which harnesses poll);
//! `[UI]`/`[DOCS]` log at DEBUG. Request bodies are captured only for
//! `[API]`/`[EMU]` requests with text-like content types, truncated to a
//! cap; response bodies only for error statuses (>= 400). Headers are
//! never logged. Secrets in bodies are shown deliberately — this is a
//! dev-only emulator and seeing what an SDK sent is the point.

use axum::body::{Body, Bytes};
use axum::extract::Request;
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use futures_util::StreamExt;
use std::time::Instant;

/// Extra bytes read past the cap so we can tell "exactly cap" from
/// "keeps going" without buffering unbounded input.
const TRUNCATION_SLACK: usize = 1;

const DEFAULT_BODY_MAX: usize = 256;

// ── Classification ───────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Api,
    Emu,
    Docs,
    Ui,
}

impl Category {
    fn tag(self) -> &'static str {
        match self {
            Category::Api => "[API]",
            Category::Emu => "[EMU]",
            Category::Docs => "[DOCS]",
            Category::Ui => "[UI]",
        }
    }
}

/// Classify a request path into its log category by prefix.
pub fn classify(path: &str) -> Category {
    if path.starts_with("/v1/")
        || path.starts_with("/v2/keys/")
        || path.starts_with("/.well-known/")
        || path.starts_with("/oauth/")
    {
        Category::Api
    } else if path.starts_with("/emulator/") || path == "/health" {
        Category::Emu
    } else if path == "/docs" || path == "/openapi.json" {
        Category::Docs
    } else {
        Category::Ui
    }
}

/// `[UI]`/`[DOCS]` traffic logs at DEBUG, as does `/health` — test
/// harnesses poll it and it would drum at INFO.
fn logs_at_debug(category: Category, path: &str) -> bool {
    matches!(category, Category::Ui | Category::Docs) || path == "/health"
}

// ── Configuration ────────────────────────────────────────────────────

/// Body-logging knobs, read once at router build time (not per request).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyLogConfig {
    pub enabled: bool,
    pub max: usize,
}

impl Default for BodyLogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max: DEFAULT_BODY_MAX,
        }
    }
}

impl BodyLogConfig {
    pub fn from_env() -> Self {
        Self::from_vars(
            std::env::var("RESCOPE_LOG_BODY").ok().as_deref(),
            std::env::var("RESCOPE_LOG_BODY_MAX").ok().as_deref(),
        )
    }

    /// `RESCOPE_LOG_BODY=0` disables body logging; anything else (or
    /// unset) leaves it on. `RESCOPE_LOG_BODY_MAX` overrides the cap;
    /// unparseable or zero values fall back to the default.
    fn from_vars(body: Option<&str>, max: Option<&str>) -> Self {
        Self {
            enabled: body != Some("0"),
            max: max
                .and_then(|v| v.parse::<usize>().ok())
                .filter(|&n| n > 0)
                .unwrap_or(DEFAULT_BODY_MAX),
        }
    }
}

// ── Body formatting ──────────────────────────────────────────────────

/// Only text-like payloads are ever logged; binary and multipart bodies
/// are not even buffered.
fn is_textual(content_type: Option<&str>) -> bool {
    let Some(ct) = content_type else { return false };
    let ct = ct
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    ct == "application/json"
        || ct == "application/x-www-form-urlencoded"
        || ct.starts_with("text/")
        || ct.ends_with("+json")
}

fn content_type(headers: &HeaderMap) -> Option<&str> {
    headers.get(header::CONTENT_TYPE)?.to_str().ok()
}

/// Collapse a captured body to a single trimmed line and truncate it to
/// `cap` bytes, appending `…` when anything was cut (here or upstream).
fn format_body(bytes: &[u8], cap: usize, source_truncated: bool) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut line = String::with_capacity(text.len().min(cap));
    let mut in_whitespace = false;
    for ch in text.trim().chars() {
        if ch.is_whitespace() {
            if !in_whitespace {
                line.push(' ');
            }
            in_whitespace = true;
        } else {
            line.push(ch);
            in_whitespace = false;
        }
    }

    let mut truncated = source_truncated;
    if line.len() > cap {
        let mut end = cap;
        while !line.is_char_boundary(end) {
            end -= 1;
        }
        line.truncate(end);
        truncated = true;
    }
    if truncated {
        line.push('…');
    }
    line
}

/// The one place a log line is assembled. Deliberately takes no header
/// map: header values (Authorization in particular) cannot leak into
/// logs because they never reach this function.
fn build_line(
    category: Category,
    method: &Method,
    path_and_query: &str,
    status: StatusCode,
    latency_ms: u128,
    body: Option<&str>,
    resp: Option<&str>,
) -> String {
    let mut line = format!(
        "{} {} {} {} {}ms",
        category.tag(),
        method,
        path_and_query,
        status.as_u16(),
        latency_ms
    );
    if let Some(body) = body {
        line.push_str(" body=");
        line.push_str(body);
    }
    if let Some(resp) = resp {
        line.push_str(" resp=");
        line.push_str(resp);
    }
    line
}

// ── Bounded body capture ─────────────────────────────────────────────

enum ReadOutcome {
    /// The body ended within the bound; `chunks` hold it entirely.
    Complete,
    /// The bound was hit; the rest of the body is still in `stream`.
    HasMore,
    /// The body errored mid-read; the error must be replayed downstream.
    Errored(axum::Error),
}

/// Read up to `cap + TRUNCATION_SLACK` bytes of `body`, returning the
/// loggable prefix (≤ `cap` bytes), whether the body continued past the
/// cap, and a reconstructed `Body` that yields the original bytes — all
/// of them — to the inner handler. Memory held here is bounded by the
/// cap plus one in-flight chunk, no matter how large the body is.
async fn buffer_prefix(body: Body, cap: usize) -> (Bytes, bool, Body) {
    let bound = cap + TRUNCATION_SLACK;
    let mut stream = body.into_data_stream();
    let mut chunks: Vec<Bytes> = Vec::new();
    let mut total: usize = 0;

    let outcome = loop {
        if total >= bound {
            break ReadOutcome::HasMore;
        }
        match stream.next().await {
            None => break ReadOutcome::Complete,
            Some(Ok(chunk)) => {
                total += chunk.len();
                chunks.push(chunk);
            }
            Some(Err(err)) => break ReadOutcome::Errored(err),
        }
    };

    let mut prefix = Vec::with_capacity(total.min(cap));
    for chunk in &chunks {
        let room = cap - prefix.len();
        if room == 0 {
            break;
        }
        prefix.extend_from_slice(&chunk[..chunk.len().min(room)]);
    }
    let truncated = total > cap;

    let body = match outcome {
        ReadOutcome::Complete => Body::from(chunks.concat()),
        ReadOutcome::HasMore => {
            let replay = futures_util::stream::iter(chunks.into_iter().map(Ok::<_, axum::Error>));
            Body::from_stream(replay.chain(stream))
        }
        ReadOutcome::Errored(err) => {
            let items = chunks
                .into_iter()
                .map(Ok::<_, axum::Error>)
                .chain(std::iter::once(Err(err)));
            Body::from_stream(futures_util::stream::iter(items))
        }
    };

    (Bytes::from(prefix), truncated, body)
}

// ── Middleware ───────────────────────────────────────────────────────

/// Axum middleware: wire with
/// `middleware::from_fn(move |req, next| request_log::log_request(cfg, req, next))`.
pub async fn log_request(cfg: BodyLogConfig, req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str().to_owned())
        .unwrap_or_else(|| req.uri().path().to_owned());
    let path = req.uri().path().to_owned();
    let category = classify(&path);

    let capture_request = cfg.enabled
        && matches!(category, Category::Api | Category::Emu)
        && is_textual(content_type(req.headers()));
    let (body_log, req) = if capture_request {
        let (parts, body) = req.into_parts();
        let (prefix, truncated, body) = buffer_prefix(body, cfg.max).await;
        let logged = (!prefix.is_empty()).then(|| format_body(&prefix, cfg.max, truncated));
        (logged, Request::from_parts(parts, body))
    } else {
        (None, req)
    };

    // Latency covers the inner service only, not our body buffering.
    let start = Instant::now();
    let response = next.run(req).await;
    let latency_ms = start.elapsed().as_millis();

    let status = response.status();
    let capture_response =
        cfg.enabled && status.as_u16() >= 400 && is_textual(content_type(response.headers()));
    let (resp_log, response) = if capture_response {
        let (parts, body) = response.into_parts();
        let (prefix, truncated, body) = buffer_prefix(body, cfg.max).await;
        let logged = (!prefix.is_empty()).then(|| format_body(&prefix, cfg.max, truncated));
        (logged, Response::from_parts(parts, body))
    } else {
        (None, response)
    };

    let line = build_line(
        category,
        &method,
        &path_and_query,
        status,
        latency_ms,
        body_log.as_deref(),
        resp_log.as_deref(),
    );
    if logs_at_debug(category, &path) {
        tracing::debug!(target: "rescope::http", "{line}");
    } else {
        tracing::info!(target: "rescope::http", "{line}");
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::routing::post;
    use axum::{middleware, Json, Router};
    use axum_test::TestServer;
    use serde_json::json;
    use std::io;
    use std::sync::{Arc, Mutex};

    // ── Classification ───────────────────────────────────────────────

    #[test]
    fn classify_api_prefixes() {
        assert_eq!(classify("/v1/auth/password/signin"), Category::Api);
        assert_eq!(classify("/v1/mgmt/user/create"), Category::Api);
        assert_eq!(classify("/v2/keys/emulator-project"), Category::Api);
        assert_eq!(classify("/.well-known/jwks.json"), Category::Api);
        assert_eq!(classify("/oauth/authorize"), Category::Api);
    }

    #[test]
    fn classify_emulator_and_health() {
        assert_eq!(classify("/emulator/reset"), Category::Emu);
        assert_eq!(classify("/emulator/otp/user@example.com"), Category::Emu);
        assert_eq!(classify("/health"), Category::Emu);
    }

    #[test]
    fn classify_docs() {
        assert_eq!(classify("/docs"), Category::Docs);
        assert_eq!(classify("/openapi.json"), Category::Docs);
    }

    #[test]
    fn classify_everything_else_is_ui() {
        assert_eq!(classify("/"), Category::Ui);
        assert_eq!(classify("/assets/index-Bx2.js"), Category::Ui);
        assert_eq!(classify("/users"), Category::Ui);
        // No trailing slash → served by the SPA fallback, so UI is correct.
        assert_eq!(classify("/v1"), Category::Ui);
    }

    #[test]
    fn debug_level_for_ui_docs_and_health() {
        assert!(logs_at_debug(Category::Ui, "/assets/app.js"));
        assert!(logs_at_debug(Category::Docs, "/docs"));
        assert!(logs_at_debug(Category::Emu, "/health"));
        assert!(!logs_at_debug(Category::Emu, "/emulator/reset"));
        assert!(!logs_at_debug(Category::Api, "/v1/auth/otp/verify/email"));
    }

    // ── Content-type gate ────────────────────────────────────────────

    #[test]
    fn textual_content_types_pass_gate() {
        assert!(is_textual(Some("application/json")));
        assert!(is_textual(Some("application/json; charset=utf-8")));
        assert!(is_textual(Some("application/x-www-form-urlencoded")));
        assert!(is_textual(Some("text/plain")));
        assert!(is_textual(Some("text/html; charset=utf-8")));
        assert!(is_textual(Some("application/problem+json")));
    }

    #[test]
    fn binary_and_missing_content_types_fail_gate() {
        assert!(!is_textual(Some("application/octet-stream")));
        assert!(!is_textual(Some("multipart/form-data; boundary=x")));
        assert!(!is_textual(Some("image/png")));
        assert!(!is_textual(None));
    }

    // ── Body formatting ──────────────────────────────────────────────

    #[test]
    fn multiline_body_collapses_to_one_line() {
        let pretty = "{\n  \"a\": 1,\n  \"b\": 2\n}\n";
        let line = format_body(pretty.as_bytes(), 256, false);
        assert!(!line.contains('\n'));
        assert_eq!(line, "{ \"a\": 1, \"b\": 2 }");
    }

    #[test]
    fn long_body_truncates_with_marker() {
        let body = "x".repeat(1000);
        let line = format_body(body.as_bytes(), 256, false);
        assert!(line.ends_with('…'));
        assert_eq!(line.trim_end_matches('…').len(), 256);
    }

    #[test]
    fn exact_cap_body_is_not_marked_truncated() {
        let body = "x".repeat(256);
        let line = format_body(body.as_bytes(), 256, false);
        assert!(!line.ends_with('…'));
        assert_eq!(line.len(), 256);
    }

    #[test]
    fn source_truncation_forces_marker() {
        let line = format_body(b"short", 256, true);
        assert_eq!(line, "short…");
    }

    #[test]
    fn truncation_respects_char_boundaries() {
        // 'é' is 2 bytes; a cap landing mid-char must back off, not panic.
        let body = "é".repeat(200);
        let line = format_body(body.as_bytes(), 255, false);
        assert!(line.ends_with('…'));
        assert!(line.trim_end_matches('…').len() <= 255);
    }

    // ── Config ───────────────────────────────────────────────────────

    #[test]
    fn config_defaults() {
        let cfg = BodyLogConfig::from_vars(None, None);
        assert_eq!(
            cfg,
            BodyLogConfig {
                enabled: true,
                max: 256
            }
        );
    }

    #[test]
    fn config_disable_and_custom_cap() {
        let cfg = BodyLogConfig::from_vars(Some("0"), Some("1024"));
        assert_eq!(
            cfg,
            BodyLogConfig {
                enabled: false,
                max: 1024
            }
        );
        let cfg = BodyLogConfig::from_vars(Some("1"), Some("garbage"));
        assert_eq!(
            cfg,
            BodyLogConfig {
                enabled: true,
                max: 256
            }
        );
        let cfg = BodyLogConfig::from_vars(None, Some("0"));
        assert_eq!(cfg.max, 256);
    }

    // ── Bounded body capture ─────────────────────────────────────────

    #[tokio::test]
    async fn small_body_roundtrips_completely() {
        let original = b"{\"loginId\":\"x@y.com\"}".to_vec();
        let (prefix, truncated, body) = buffer_prefix(Body::from(original.clone()), 256).await;
        assert!(!truncated);
        assert_eq!(&prefix[..], &original[..]);
        let replayed = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        assert_eq!(&replayed[..], &original[..]);
    }

    #[tokio::test]
    async fn oversized_multichunk_body_is_replayed_byte_identical() {
        let chunks = vec![
            Bytes::from("a".repeat(200)),
            Bytes::from("b".repeat(200)),
            Bytes::from("c".repeat(200)),
        ];
        let original: Vec<u8> = chunks.concat();
        let stream = futures_util::stream::iter(chunks.into_iter().map(Ok::<_, axum::Error>));
        let (prefix, truncated, body) = buffer_prefix(Body::from_stream(stream), 256).await;
        assert!(truncated);
        assert_eq!(prefix.len(), 256);
        let replayed = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        assert_eq!(&replayed[..], &original[..]);
    }

    // ── Behavior through the middleware (in-process) ─────────────────

    fn echo_router(cfg: BodyLogConfig) -> Router {
        Router::new()
            .route("/v1/echo", post(|body: String| async move { body }))
            .layer(middleware::from_fn(move |req, next| {
                log_request(cfg, req, next)
            }))
    }

    /// Why: spec "Body capture does not alter request handling" — the
    ///      middleware buffers and reconstructs the request body, and a
    ///      bug there would corrupt every JSON request to the emulator.
    /// Decision: drive an echo handler through the real middleware and
    ///      assert the handler saw the exact bytes the client sent.
    #[tokio::test]
    async fn json_request_reaches_handler_intact() {
        let payload = json!({"loginId": "x@y.com", "code": "123456"});
        let server = TestServer::new(echo_router(BodyLogConfig::default())).unwrap();
        let response = server.post("/v1/echo").json(&payload).await;
        response.assert_status_ok();
        assert_eq!(response.text(), serde_json::to_string(&payload).unwrap());
    }

    /// Why: bodies larger than the buffering bound take the chained
    ///      replay path (design D3) — the riskiest code in the module.
    /// Decision: compare status + echoed bytes against a baseline router
    ///      with no logging layer; they must be identical.
    #[tokio::test]
    async fn over_cap_request_matches_no_logging_baseline() {
        let big = "x".repeat(10_000);

        let logged = TestServer::new(echo_router(BodyLogConfig::default())).unwrap();
        let baseline = TestServer::new(
            Router::new().route("/v1/echo", post(|body: String| async move { body })),
        )
        .unwrap();

        let logged_res = logged
            .post("/v1/echo")
            .content_type("text/plain")
            .text(big.clone())
            .await;
        let baseline_res = baseline
            .post("/v1/echo")
            .content_type("text/plain")
            .text(big.clone())
            .await;

        assert_eq!(logged_res.status_code(), baseline_res.status_code());
        assert_eq!(logged_res.text(), baseline_res.text());
        assert_eq!(logged_res.text(), big);
    }

    // ── Log output assertions ────────────────────────────────────────

    #[derive(Clone, Default)]
    struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

    impl CaptureWriter {
        fn contents(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }
    }

    impl io::Write for CaptureWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CaptureWriter {
        type Writer = CaptureWriter;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    fn capture_logs() -> (CaptureWriter, tracing::subscriber::DefaultGuard) {
        let writer = CaptureWriter::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(writer.clone())
            .with_ansi(false)
            .with_max_level(tracing::Level::DEBUG)
            .finish();
        let guard = tracing::subscriber::set_default(subscriber);
        (writer, guard)
    }

    /// Why: spec "Authorization headers are never logged" — the mgmt key
    ///      rides in that header on every management call.
    /// Decision: capture real tracing output for a request carrying the
    ///      default mgmt credentials and assert the key never appears.
    #[tokio::test]
    async fn authorization_header_never_appears_in_logs() {
        let (writer, _guard) = capture_logs();
        let server = TestServer::new(echo_router(BodyLogConfig::default())).unwrap();
        server
            .post("/v1/echo")
            .authorization_bearer("emulator-project:emulator-key")
            .json(&json!({"loginId": "x@y.com"}))
            .await
            .assert_status_ok();

        let logs = writer.contents();
        assert!(logs.contains("[API] POST /v1/echo 200"));
        assert!(logs.contains("body={\"loginId\":\"x@y.com\"}"));
        assert!(!logs.contains("emulator-key"));
        assert!(!logs.contains("Authorization"));
    }

    /// Why: spec "Response bodies are logged only for error responses".
    /// Decision: one 400 route and one 200 route; only the 400 line may
    ///      carry a `resp=` segment.
    #[tokio::test]
    async fn response_body_logged_only_for_errors() {
        let (writer, _guard) = capture_logs();
        let router = Router::new()
            .route(
                "/v1/fail",
                post(|| async {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"errorCode": "E000001"})),
                    )
                }),
            )
            .route("/v1/ok", post(|| async { Json(json!({"ok": true})) }))
            .layer(middleware::from_fn(|req, next| {
                log_request(BodyLogConfig::default(), req, next)
            }));
        let server = TestServer::new(router).unwrap();

        server.post("/v1/fail").await.assert_status_bad_request();
        server.post("/v1/ok").await.assert_status_ok();

        let logs = writer.contents();
        assert!(logs.contains("[API] POST /v1/fail 400"));
        assert!(logs.contains("resp={\"errorCode\":\"E000001\"}"));
        let ok_line = logs
            .lines()
            .find(|l| l.contains("/v1/ok"))
            .expect("expected a log line for /v1/ok");
        assert!(!ok_line.contains("resp="));
    }

    /// Why: spec "Body logging is configurable" — `RESCOPE_LOG_BODY=0`
    ///      must suppress both `body=` and `resp=` segments.
    #[tokio::test]
    async fn disabled_body_logging_omits_body_and_resp() {
        let (writer, _guard) = capture_logs();
        let cfg = BodyLogConfig {
            enabled: false,
            max: 256,
        };
        let router = Router::new()
            .route(
                "/v1/fail",
                post(|| async {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"errorCode": "E000001"})),
                    )
                }),
            )
            .layer(middleware::from_fn(move |req, next| {
                log_request(cfg, req, next)
            }));
        let server = TestServer::new(router).unwrap();

        server
            .post("/v1/fail")
            .json(&json!({"loginId": "x@y.com"}))
            .await
            .assert_status_bad_request();

        let logs = writer.contents();
        assert!(logs.contains("[API] POST /v1/fail 400"));
        assert!(!logs.contains("body="));
        assert!(!logs.contains("resp="));
    }

    /// Why: spec scenario "Query string preserved".
    #[tokio::test]
    async fn query_string_appears_in_log_line() {
        let (writer, _guard) = capture_logs();
        let router = Router::new()
            .route("/emulator/otp/:id", axum::routing::get(|| async { "ok" }))
            .layer(middleware::from_fn(|req, next| {
                log_request(BodyLogConfig::default(), req, next)
            }));
        let server = TestServer::new(router).unwrap();

        server
            .get("/emulator/otp/user@example.com")
            .add_query_param("channel", "sms")
            .await
            .assert_status_ok();

        let logs = writer.contents();
        assert!(logs.contains("[EMU] GET /emulator/otp/user@example.com?channel=sms 200"));
    }
}
