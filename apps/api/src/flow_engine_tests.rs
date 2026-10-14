//! The flow engine end to end, through `/v2/flow/*`, on the `sign-in-anyway`
//! fixture flow in `assets/test-flows` (shaped like a console export).

use crate::config::EmulatorConfig;
use crate::jwt::token_validator::validate_session_jwt;
use crate::server::build_router;
use crate::state::EmulatorState;
use crate::store::webauthn_store::test_support::SoftAuthenticator;
use crate::types::UserTenant;
use axum_test::TestServer;
use serde_json::{json, Value};

const FLOW: &str = "sign-in-anyway";
const PASSWORD: &str = "Secret123!";
const ORIGIN: &str = "http://localhost:4200";

fn flows_dir() -> String {
    format!("{}/assets/test-flows", env!("CARGO_MANIFEST_DIR"))
}

async fn setup() -> (TestServer, EmulatorState) {
    let config = EmulatorConfig {
        flows_dir: Some(flows_dir()),
        ..EmulatorConfig::default()
    };
    let state = EmulatorState::new(&config).await.unwrap();
    let server = TestServer::new(build_router(state.clone())).unwrap();
    (server, state)
}

async fn sign_up(server: &TestServer, login_id: &str) {
    server
        .post("/v1/auth/password/signup")
        .json(&json!({ "loginId": login_id, "password": PASSWORD, "user": { "email": login_id } }))
        .await
        .assert_status_ok();
}

/// One browser's run of the flow.
struct Browser<'a> {
    server: &'a TestServer,
    execution_id: String,
    step_id: String,
}

impl<'a> Browser<'a> {
    /// The widget draws the welcome screen itself and sends its submit with the start.
    async fn start(server: &'a TestServer, email: &str, webauthn_support: bool) -> (Self, Value) {
        let body = server
            .post("/v2/flow/start")
            .json(&json!({
                "flowId": FLOW,
                "options": {
                    "redirectUrl": format!("{ORIGIN}/login"),
                    "deviceInfo": { "webAuthnSupport": webauthn_support }
                },
                "interactionId": "continue",
                "input": { "email": email, "origin": ORIGIN }
            }))
            .await
            .json::<Value>();
        let browser = Self {
            server,
            execution_id: body["executionId"].as_str().unwrap_or_default().to_string(),
            step_id: body["stepId"].as_str().unwrap_or_default().to_string(),
        };
        (browser, body)
    }

    async fn next(&mut self, interaction: &str, input: Value) -> Value {
        let response = self
            .server
            .post("/v2/flow/next")
            .json(&json!({
                "executionId": self.execution_id,
                "stepId": self.step_id,
                "interactionId": interaction,
                "input": input
            }))
            .await;
        response.assert_status_ok();
        let body = response.json::<Value>();
        if let Some(step) = body["stepId"].as_str().filter(|s| !s.is_empty()) {
            self.step_id = step.to_string();
        }
        body
    }
}

fn screen_of(body: &Value) -> &str {
    body["screen"]["id"].as_str().unwrap_or_default()
}

fn screen(id: &str) -> String {
    format!("{FLOW}/{id}")
}

async fn last_sign_in_method(state: &EmulatorState, login_id: &str) -> Value {
    state
        .users
        .read()
        .await
        .load(login_id)
        .unwrap()
        .custom_attributes["lastSignInMethod"]
        .clone()
}

async fn amr(state: &EmulatorState, body: &Value) -> Vec<String> {
    let jwt = body["authInfo"]["sessionJwt"].as_str().unwrap();
    validate_session_jwt(&*state.km().await, jwt).unwrap().amr
}

#[tokio::test]
async fn config_json_starts_an_exported_flow_on_its_first_screen() {
    let (server, _) = setup().await;

    let config = server
        .get("/pages/P1/v2-beta/config.json")
        .await
        .json::<Value>();

    assert_eq!(config["flows"][FLOW]["startScreenId"], screen("SCwelcome"));
    assert_eq!(
        config["flows"]["sign-up-or-in-passwords"]["startScreenId"],
        "signIn"
    );
    assert_eq!(config["componentsVersion"], "3.20.4");
}

#[tokio::test]
async fn serves_the_exported_screens_and_nothing_outside_them() {
    let (server, _) = setup().await;

    let html = server
        .get(&format!("/pages/P1/v2-beta/{}.html", screen("SCwelcome")))
        .await;
    html.assert_status_ok();
    assert!(html.text().contains(r#"name="email""#));

    server
        .get(&format!("/pages/P1/v2-beta/{FLOW}/SCmissing.html"))
        .await
        .assert_status_not_found();
    server
        .get("/pages/P1/v2-beta/no-such-flow/SCwelcome.html")
        .await
        .assert_status_not_found();
}

#[tokio::test]
async fn a_json_file_that_is_not_a_flow_is_skipped() {
    let (_, state) = setup().await;
    assert!(state.flow_library.get(FLOW).is_some());
    assert_eq!(state.flow_library.flows().count(), 1);
}

#[tokio::test]
async fn a_first_sign_in_offers_the_methods_then_remembers_the_password() {
    let (server, state) = setup().await;
    sign_up(&server, "first@example.com").await;

    let (mut browser, chooser) = Browser::start(&server, "first@example.com", false).await;
    assert_eq!(chooser["status"], "waiting");
    assert!(browser.execution_id.starts_with("sign-in-anyway|#|"));
    assert_eq!(screen_of(&chooser), screen("SCchooser"));
    assert_eq!(
        chooser["screen"]["state"]["form"]["email"],
        "first@example.com"
    );

    let password_screen = browser.next("use-password", json!({})).await;
    assert_eq!(screen_of(&password_screen), screen("SCpassword"));
    let done = browser
        .next("pwSignInSubmit", json!({ "password": PASSWORD }))
        .await;

    assert_eq!(done["status"], "completed");
    assert_eq!(amr(&state, &done).await, vec!["pwd"]);
    assert_eq!(
        last_sign_in_method(&state, "first@example.com").await,
        "password"
    );

    let (_, again) = Browser::start(&server, "first@example.com", false).await;
    assert_eq!(screen_of(&again), screen("SCpassword"));
}

#[tokio::test]
async fn a_refused_password_shows_the_password_screen_with_why() {
    let (server, _) = setup().await;
    sign_up(&server, "refused@example.com").await;
    let (mut browser, _) = Browser::start(&server, "refused@example.com", false).await;
    browser.next("use-password", json!({})).await;

    let refused = browser
        .next("pwSignInSubmit", json!({ "password": "WrongOne!" }))
        .await;

    assert_eq!(refused["status"], "waiting");
    assert_eq!(screen_of(&refused), screen("SCpassword"));
    assert_eq!(
        refused["screen"]["state"]["errorText"],
        "Incorrect email, username or password."
    );
}

#[tokio::test]
async fn a_user_with_no_tenant_is_offered_biometrics_and_can_add_them() {
    let (server, state) = setup().await;
    sign_up(&server, "promote@example.com").await;
    let mut authenticator = SoftAuthenticator::new();

    let (mut browser, _) = Browser::start(&server, "promote@example.com", true).await;
    browser.next("use-password", json!({})).await;
    let promote = browser
        .next("pwSignInSubmit", json!({ "password": PASSWORD }))
        .await;
    assert_eq!(screen_of(&promote), screen("SCpromote"));

    let create = browser.next("add", json!({ "origin": ORIGIN })).await;
    assert_eq!(create["action"], "webauthnCreate");
    assert_eq!(create["stepId"], "23.end");
    assert!(
        create["screen"]["state"].is_object(),
        "descope-wc reads screen.state on every response"
    );
    let options = create["webauthn"]["options"].as_str().unwrap();
    let options_value: Value = serde_json::from_str(options).unwrap();
    assert_eq!(options_value["publicKey"]["rp"]["id"], "localhost");
    assert_eq!(
        options_value["publicKey"]["user"]["name"],
        "promote@example.com"
    );

    let done = browser
        .next(
            "submit",
            json!({ "transactionId": create["webauthn"]["transactionId"], "response": authenticator.create(options) }),
        )
        .await;

    assert_eq!(done["status"], "completed");
    assert_eq!(done["authInfo"]["user"]["webauthn"], true);
    let user_id = state
        .users
        .read()
        .await
        .load("promote@example.com")
        .unwrap()
        .user_id
        .clone();
    assert!(state.webauthn.read().await.has_credentials(&user_id));
}

#[tokio::test]
async fn not_now_signs_in_without_a_passkey() {
    let (server, _) = setup().await;
    sign_up(&server, "notnow@example.com").await;
    let (mut browser, _) = Browser::start(&server, "notnow@example.com", true).await;
    browser.next("use-password", json!({})).await;
    browser
        .next("pwSignInSubmit", json!({ "password": PASSWORD }))
        .await;

    let done = browser.next("not-now", json!({})).await;

    assert_eq!(done["status"], "completed");
    assert_eq!(done["authInfo"]["user"]["webauthn"], false);
}

#[tokio::test]
async fn a_cancelled_passkey_prompt_shows_the_promote_screen_again() {
    let (server, _) = setup().await;
    sign_up(&server, "cancel@example.com").await;
    let (mut browser, _) = Browser::start(&server, "cancel@example.com", true).await;
    browser.next("use-password", json!({})).await;
    browser
        .next("pwSignInSubmit", json!({ "password": PASSWORD }))
        .await;
    let create = browser.next("add", json!({ "origin": ORIGIN })).await;

    let cancelled = browser
        .next("submit", json!({ "transactionId": create["webauthn"]["transactionId"], "failure": "NotAllowedError" }))
        .await;

    assert_eq!(screen_of(&cancelled), screen("SCpromote"));
    assert!(cancelled["screen"]["state"]["errorText"]
        .as_str()
        .unwrap()
        .contains("cancelled"));
}

#[tokio::test]
async fn a_user_in_a_tenant_is_not_offered_biometrics() {
    let (server, state) = setup().await;
    sign_up(&server, "tenant@example.com").await;
    state
        .users
        .write()
        .await
        .add_tenant(
            "tenant@example.com",
            UserTenant {
                tenant_id: "T1".into(),
                ..Default::default()
            },
        )
        .unwrap();
    let (mut browser, _) = Browser::start(&server, "tenant@example.com", true).await;
    browser.next("use-password", json!({})).await;

    let done = browser
        .next("pwSignInSubmit", json!({ "password": PASSWORD }))
        .await;

    assert_eq!(done["status"], "completed");
}

/// Sign up, sign in with a password, and add a passkey from the promote screen.
async fn user_with_passkey(server: &TestServer, email: &str) -> SoftAuthenticator {
    sign_up(server, email).await;
    let mut authenticator = SoftAuthenticator::new();
    let (mut browser, _) = Browser::start(server, email, true).await;
    browser.next("use-password", json!({})).await;
    browser
        .next("pwSignInSubmit", json!({ "password": PASSWORD }))
        .await;
    let create = browser.next("add", json!({ "origin": ORIGIN })).await;
    let options = create["webauthn"]["options"].as_str().unwrap().to_string();
    let done = browser
        .next(
            "submit",
            json!({ "transactionId": create["webauthn"]["transactionId"], "response": authenticator.create(&options) }),
        )
        .await;
    assert_eq!(done["status"], "completed");
    authenticator
}

#[tokio::test]
async fn try_another_way_offers_biometrics_to_a_user_with_a_passkey() {
    let (server, _) = setup().await;
    user_with_passkey(&server, "another@example.com").await;

    let (mut browser, password_screen) = Browser::start(&server, "another@example.com", true).await;
    assert_eq!(screen_of(&password_screen), screen("SCpassword"));
    let chooser = browser.next("pwUseMagicLink", json!({})).await;

    assert_eq!(screen_of(&chooser), screen("SCchooserPasskey"));
}

#[tokio::test]
async fn biometrics_sign_in_and_are_remembered_as_the_method() {
    let (server, state) = setup().await;
    let mut authenticator = user_with_passkey(&server, "bio@example.com").await;
    let (mut browser, _) = Browser::start(&server, "bio@example.com", true).await;
    browser.next("pwUseMagicLink", json!({})).await;

    let get = browser
        .next("bio-signin", json!({ "origin": ORIGIN }))
        .await;
    assert_eq!(get["action"], "webauthnGet");
    let options = get["webauthn"]["options"].as_str().unwrap();
    let done = browser
        .next(
            "submit",
            json!({ "transactionId": get["webauthn"]["transactionId"], "response": authenticator.get(options) }),
        )
        .await;

    assert_eq!(done["status"], "completed");
    assert_eq!(amr(&state, &done).await, vec!["webauthn"]);
    assert_eq!(
        last_sign_in_method(&state, "bio@example.com").await,
        "passkey"
    );
    let (_, next_time) = Browser::start(&server, "bio@example.com", true).await;
    assert_eq!(screen_of(&next_time), screen("SCchooserPasskey"));
}

#[tokio::test]
async fn a_passkey_from_another_authenticator_does_not_sign_in() {
    let (server, _) = setup().await;
    user_with_passkey(&server, "impostor@example.com").await;
    let (mut browser, _) = Browser::start(&server, "impostor@example.com", true).await;
    browser.next("pwUseMagicLink", json!({})).await;
    let get = browser
        .next("bio-signin", json!({ "origin": ORIGIN }))
        .await;

    let mut impostor = SoftAuthenticator::new();
    let refused = browser
        .next(
            "submit",
            json!({
                "transactionId": get["webauthn"]["transactionId"],
                "response": impostor.get(get["webauthn"]["options"].as_str().unwrap())
            }),
        )
        .await;

    assert_eq!(refused["status"], "waiting");
    assert_eq!(screen_of(&refused), screen("SCchooserPasskey"));
    assert!(refused["screen"]["state"]["errorText"].as_str().is_some());
}

#[tokio::test]
async fn an_emailed_code_signs_in_and_a_wrong_one_says_so() {
    let (server, state) = setup().await;
    sign_up(&server, "code@example.com").await;
    let (mut browser, _) = Browser::start(&server, "code@example.com", false).await;

    let code_screen = browser.next("send-code", json!({})).await;
    assert_eq!(screen_of(&code_screen), screen("SCcode"));
    assert_eq!(
        code_screen["screen"]["state"]["sentTo"]["maskedEmail"],
        "co***@example.com"
    );
    let code = server
        .get("/emulator/otp/code@example.com")
        .await
        .json::<Value>()["code"]
        .as_str()
        .unwrap()
        .to_string();

    let wrong = browser
        .next("submit-code", json!({ "code": "000000x" }))
        .await;
    assert_eq!(screen_of(&wrong), screen("SCcode"));
    assert!(wrong["screen"]["state"]["errorText"].as_str().is_some());

    let done = browser.next("submit-code", json!({ "code": code })).await;
    assert_eq!(done["status"], "completed");
    assert_eq!(amr(&state, &done).await, vec!["email"]);
    assert_eq!(
        last_sign_in_method(&state, "code@example.com").await,
        "code"
    );
}

#[tokio::test]
async fn a_sign_in_link_resumes_the_flow_and_the_waiting_tab_follows() {
    let (server, state) = setup().await;
    sign_up(&server, "link@example.com").await;
    let (mut waiting_tab, _) = Browser::start(&server, "link@example.com", false).await;

    let sent = waiting_tab.next("send-link", json!({})).await;
    assert_eq!(screen_of(&sent), screen("SClinkSent"));
    let poll = waiting_tab.next("polling", json!({})).await;
    assert_eq!(poll["action"], "poll");

    let link = server
        .get("/emulator/flow/link/link@example.com")
        .await
        .json::<Value>();
    let url = link["link"].as_str().unwrap();
    assert!(
        url.starts_with("http://localhost:4200/login?descope-login-flow="),
        "{url}"
    );
    let token = link["token"].as_str().unwrap();

    // The link opens in a new tab: descope-wc resumes the run at the step the URL names.
    let opened = server
        .post("/v2/flow/next")
        .json(&json!({
            "executionId": waiting_tab.execution_id,
            "stepId": "24.end",
            "interactionId": "submit",
            "input": { "token": token }
        }))
        .await
        .json::<Value>();
    assert_eq!(opened["status"], "completed");
    assert_eq!(amr(&state, &opened).await, vec!["magiclink"]);
    assert_eq!(
        last_sign_in_method(&state, "link@example.com").await,
        "link"
    );

    let followed = waiting_tab.next("polling", json!({})).await;
    assert_eq!(followed["status"], "completed");
}

#[tokio::test]
async fn a_high_risk_score_asks_for_a_code_after_the_password() {
    let (server, _) = setup().await;
    sign_up(&server, "risky@example.com").await;
    server
        .post("/emulator/risk")
        .json(&json!({ "loginId": "risky@example.com", "score": 0.9 }))
        .await
        .assert_status_ok();
    let (mut browser, _) = Browser::start(&server, "risky@example.com", false).await;
    browser.next("use-password", json!({})).await;

    let second_factor = browser
        .next("pwSignInSubmit", json!({ "password": PASSWORD }))
        .await;
    assert_eq!(screen_of(&second_factor), screen("SCsecondFactor"));
    let code = server
        .get("/emulator/otp/risky@example.com")
        .await
        .json::<Value>()["code"]
        .as_str()
        .unwrap()
        .to_string();
    let done = browser.next("submit-code", json!({ "code": code })).await;
    assert_eq!(done["status"], "completed");

    server.delete("/emulator/risk").await.assert_status_ok();
    let (mut calm, _) = Browser::start(&server, "risky@example.com", false).await;
    let calm_done = calm
        .next("pwSignInSubmit", json!({ "password": PASSWORD }))
        .await;
    assert_eq!(calm_done["status"], "completed");
}

#[tokio::test]
async fn a_risk_score_must_be_between_zero_and_one() {
    let (server, _) = setup().await;
    server
        .post("/emulator/risk")
        .json(&json!({ "score": 4 }))
        .await
        .assert_status_bad_request();
}

#[tokio::test]
async fn an_action_the_emulator_does_not_run_fails_the_flow_by_name() {
    let def = crate::flow_engine::FlowDef::from_export(&json!({
        "flowId": "pushes",
        "contents": {
            "startTask": "1",
            "tasks": {
                "1": { "id": "1", "type": "automated", "action": "send-push-notification", "next": { "rules": [] } }
            }
        }
    }))
    .unwrap();
    let state = EmulatorState::new(&EmulatorConfig::default())
        .await
        .unwrap();

    let (_, env) = crate::flow_engine::start(&state, &def, &json!({ "flowId": "pushes" })).await;

    assert_eq!(env["status"], "failed");
    assert!(env["error"]["errorDescription"]
        .as_str()
        .unwrap()
        .contains("send-push-notification"));
}

#[tokio::test]
async fn a_reset_does_not_end_a_run_another_test_is_part_way_through() {
    let (server, _) = setup().await;
    sign_up(&server, "shard@example.com").await;
    let (mut browser, _) = Browser::start(&server, "shard@example.com", false).await;

    server.post("/emulator/reset").await.assert_status_ok();
    let next = browser.next("use-password", json!({})).await;

    assert_eq!(screen_of(&next), screen("SCpassword"));
}
