//! Runs flows exported from the Descope console.
//!
//! `DESCOPE_EMULATOR_FLOWS_DIR` names a directory of flow exports (`<flowId>.json`,
//! as the console's Export writes them) and the screens Descope serves for them
//! (`screens/<flowId>/<screenId>.html`, fetched from
//! `https://static.descope.com/pages/<projectId>/v2-beta/<flowId>/<screenId>.html`).
//! The emulator walks the export's task graph the way Descope does: a screen waits
//! for the widget's next submit, a condition picks a branch from the flow context,
//! and an automated task runs an action against the emulator's own stores.
//!
//! The actions are the ones a sign-in flow uses: password, email and SMS codes,
//! email links, passkeys (WebAuthn), SSO, a risk score tests can set, and user
//! property updates. Any other action fails the flow with an error that names
//! it, so a flow that needs more says so instead of misbehaving. Scriptlets
//! cannot run here (there is no JavaScript engine); they leave the form as it is.

use crate::{
    auth_policy::AuthPolicyGuard,
    cookies::build_auth_cookies,
    jwt::token_generator::{generate_refresh_jwt, generate_session_jwt},
    routes::auth::{
        otp::{masked_email, masked_phone},
        password::build_jwt_response,
        saml::{start_impl as saml_start, SamlStartRequest},
    },
    state::EmulatorState,
    store::{
        flow_store::{FlowRun, FlowStatus, SentLink},
        otp_store::generate_otp_code,
        token_store::generate_token,
        user_store::{new_user_id, UserPatch},
        webauthn_store::Ceremony,
    },
    types::{AuthType, TokenType, User},
};
use axum::http::HeaderMap;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

/// What the password screens say when a sign-in is refused, without saying which part was wrong.
const PASSWORD_REFUSED: &str = "Incorrect email, username or password.";
const CODE_REFUSED: &str = "Incorrect code. Check the code and try again.";
const NO_PASSKEY: &str = "No passkey is registered for this user on this device.";
const PASSKEY_REFUSED: &str =
    "The passkey could not be verified. Try again or sign in another way.";

/// More steps than any real flow takes between two screens; stops a cycle in a broken export.
const MAX_STEPS: usize = 200;

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ─── Loading ────────────────────────────────────────────────────────────────

/// One exported flow.
#[derive(Debug, Clone)]
pub struct FlowDef {
    pub id: String,
    start_task: String,
    tasks: HashMap<String, Value>,
    pub components_version: Option<String>,
}

impl FlowDef {
    /// Parse a console export. `None` for a JSON file that is not a flow (a theme).
    pub fn from_export(export: &Value) -> Option<Self> {
        let id = export.get("flowId")?.as_str()?.to_string();
        let contents = export.get("contents")?;
        let tasks: HashMap<String, Value> = contents
            .get("tasks")?
            .as_object()?
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let start_task = contents
            .get("startTask")
            .and_then(Value::as_str)
            .unwrap_or("start")
            .to_string();
        let components_version = export
            .pointer("/metadata/componentsVersion")
            .and_then(Value::as_str)
            .map(str::to_string);
        Some(Self {
            id,
            start_task,
            tasks,
            components_version,
        })
    }

    fn task(&self, id: &str) -> Option<&Value> {
        self.tasks.get(id)
    }

    /// The first screen, which the widget may draw before it starts the flow.
    fn start_screen(&self) -> Option<&Value> {
        let mut id = self.start_task.as_str();
        for _ in 0..4 {
            let task = self.task(id)?;
            if task["type"] == "screen" {
                return Some(task);
            }
            id = task.pointer("/next/rules/0/taskId")?.as_str()?;
        }
        None
    }

    /// This flow's entry in `config.json`.
    pub fn config_entry(&self) -> Value {
        match self.start_screen() {
            Some(screen) => json!({
                "startScreenId": format!("{}/{}", self.id, screen["screenId"].as_str().unwrap_or_default()),
                "startScreenName": screen["name"],
                "version": 1
            }),
            None => json!({ "version": 1 }),
        }
    }
}

/// The exported flows the emulator runs, and where their screens live.
#[derive(Debug, Default)]
pub struct FlowLibrary {
    flows: HashMap<String, FlowDef>,
    dir: Option<PathBuf>,
}

impl FlowLibrary {
    /// Load every flow export in `dir`. Files that are not flows are skipped;
    /// a flow file that cannot be read or parsed is an error, so a broken export
    /// fails the start instead of the first sign-in.
    pub fn load(dir: Option<&str>) -> anyhow::Result<Self> {
        let Some(dir) = dir else {
            return Ok(Self::default());
        };
        let dir = PathBuf::from(dir);
        let mut flows = HashMap::new();
        for entry in std::fs::read_dir(&dir).map_err(|e| {
            anyhow::anyhow!(
                "cannot read DESCOPE_EMULATOR_FLOWS_DIR {}: {e}",
                dir.display()
            )
        })? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let text = std::fs::read_to_string(&path)?;
            let export: Value = serde_json::from_str(&text)
                .map_err(|e| anyhow::anyhow!("{} is not JSON: {e}", path.display()))?;
            if let Some(flow) = FlowDef::from_export(&export) {
                tracing::info!(flow = %flow.id, file = %path.display(), "Loaded flow export");
                flows.insert(flow.id.clone(), flow);
            }
        }
        Ok(Self {
            flows,
            dir: Some(dir),
        })
    }

    pub fn from_flows(flows: Vec<FlowDef>, dir: Option<PathBuf>) -> Self {
        Self {
            flows: flows.into_iter().map(|f| (f.id.clone(), f)).collect(),
            dir,
        }
    }

    pub fn get(&self, flow_id: &str) -> Option<&FlowDef> {
        self.flows.get(flow_id)
    }

    pub fn flows(&self) -> impl Iterator<Item = &FlowDef> {
        self.flows.values()
    }

    /// The HTML of `<flowId>/<screenId>`, read on every request so a refreshed
    /// screen shows on the next page load.
    pub fn screen_html(&self, flow_id: &str, screen_id: &str) -> Option<String> {
        let dir = self.dir.as_ref()?;
        self.flows.get(flow_id)?;
        let safe = |s: &str| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        };
        if !safe(flow_id) || !safe(screen_id) {
            return None;
        }
        std::fs::read_to_string(screen_path(dir, flow_id, screen_id)).ok()
    }
}

fn screen_path(dir: &Path, flow_id: &str, screen_id: &str) -> PathBuf {
    dir.join("screens")
        .join(flow_id)
        .join(format!("{screen_id}.html"))
}

// ─── Envelopes ──────────────────────────────────────────────────────────────

fn envelope(execution_id: &str, step_id: &str, status: &str) -> Value {
    json!({
        "executionId": execution_id,
        "stepId": step_id,
        "status": status,
        "action": "",
        "screen": null,
        "redirect": { "url": "", "isPopup": false },
        "webauthn": { "transactionId": "", "options": "" },
        "authInfo": null,
        "error": null,
        "lastAuth": null,
        "stepName": "",
        "samlIdpResponse": null,
        "openInNewTabUrl": "",
        "runnerLogs": [],
        "nativeResponse": null,
        "output": {}
    })
}

pub fn failed(execution_id: &str, code: &str, description: &str) -> Value {
    let mut env = envelope(execution_id, "", "failed");
    env["error"] =
        json!({ "errorCode": code, "errorDescription": description, "errorMessage": description });
    env
}

// ─── Context ────────────────────────────────────────────────────────────────

/// Read `a.b.c` from a JSON object; `Null` when any part is missing.
fn get_path<'a>(value: &'a Value, path: &str) -> &'a Value {
    path.split('.')
        .try_fold(value, |v, key| v.get(key))
        .unwrap_or(&Value::Null)
}

/// Write `a.b.c` into a JSON object, creating objects on the way.
fn set_path(value: &mut Value, path: &str, new_value: Value) {
    let mut cursor = value;
    let mut parts = path.split('.').peekable();
    while let Some(key) = parts.next() {
        if !cursor.is_object() {
            *cursor = json!({});
        }
        let obj = cursor.as_object_mut().expect("just made an object");
        if parts.peek().is_none() {
            obj.insert(key.to_string(), new_value);
            return;
        }
        cursor = obj.entry(key.to_string()).or_insert_with(|| json!({}));
    }
}

fn as_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// `{{path}}` placeholders filled from the context, as Descope templates do.
fn render(template: &str, ctx: &Value) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        match rest[start + 2..].find("}}") {
            Some(end) => {
                out.push_str(&as_text(get_path(
                    ctx,
                    rest[start + 2..start + 2 + end].trim(),
                )));
                rest = &rest[start + 2 + end + 2..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// An action argument: `{"type": "inline", "value": ...}` (templated when a string)
/// or `{"type": "context", "value": "<path>"}`.
fn argument(task: &Value, name: &str, ctx: &Value) -> Value {
    resolve_value(&task["arguments"][name], ctx)
}

fn resolve_value(arg: &Value, ctx: &Value) -> Value {
    match arg["type"].as_str() {
        Some("context") => get_path(ctx, arg["value"].as_str().unwrap_or_default()).clone(),
        _ => match &arg["value"] {
            Value::String(s) => Value::String(render(s, ctx)),
            other => other.clone(),
        },
    }
}

fn argument_text(task: &Value, name: &str, ctx: &Value) -> String {
    as_text(&argument(task, name, ctx))
}

/// What conditions read about a user (`user.*`, `unauthUser.*`, `authInfo.user.*`).
fn project_user(user: &User, has_passkey: bool) -> Value {
    json!({
        "userId": user.user_id,
        "loginIds": user.login_ids,
        "email": user.email.clone().unwrap_or_default(),
        "phone": user.phone.clone().unwrap_or_default(),
        "name": user.name.clone().unwrap_or_default(),
        "verifiedEmail": user.verified_email,
        "verifiedPhone": user.verified_phone,
        "tenantIds": user.user_tenants.iter().map(|t| t.tenant_id.clone()).collect::<Vec<_>>(),
        "customAttributes": user.custom_attributes,
        "status": user.status,
        "webauthn": has_passkey,
        "password": user.password,
    })
}

// ─── Runs ───────────────────────────────────────────────────────────────────

struct Run {
    id: String,
    task_id: String,
    ctx: Value,
    status: FlowStatus,
}

impl Run {
    fn auth_user_id(&self) -> Option<String> {
        self.ctx["auth"]["userId"].as_str().map(str::to_string)
    }

    fn login_id(&self) -> String {
        as_text(&self.ctx["form"]["email"]).trim().to_string()
    }

    fn sign_in(&mut self, user_id: &str, method: &str, first_seen: bool) {
        self.ctx["auth"] = json!({ "userId": user_id, "method": method, "firstSeen": first_seen });
    }
}

/// What one task did.
enum Outcome {
    /// Follow the task's rule for this interaction.
    Interaction(String),
    /// Stop and hand the widget this envelope; `await_task` takes the next submit.
    Wait { envelope: Value, await_task: String },
    /// Show the last screen again with this error.
    ScreenError(String),
    /// The flow finished.
    Done(HeaderMap, Value),
    /// The flow cannot continue.
    Fail(String, String),
}

/// Merge a screen submit into `form`. Inputs named `form.x` land at `form.x`.
fn merge_form(ctx: &mut Value, input: &Value) {
    let Some(fields) = input.as_object() else {
        return;
    };
    for (key, value) in fields {
        if matches!(
            key.as_str(),
            "origin"
                | "transactionId"
                | "response"
                | "failure"
                | "failureReason"
                | "failureMessage"
                | "token"
                | "exchangeCode"
                | "exchangeError"
                | "idpInitiated"
        ) {
            continue;
        }
        let path = key.strip_prefix("form.").unwrap_or(key);
        set_path(ctx, &format!("form.{path}"), value.clone());
    }
}

fn remember_origin(ctx: &mut Value, input: &Value) {
    if let Some(origin) = input.get("origin").and_then(Value::as_str) {
        ctx["origin"] = json!(origin);
    }
}

pub async fn start(state: &EmulatorState, def: &FlowDef, body: &Value) -> (HeaderMap, Value) {
    let input = body.get("input").cloned().unwrap_or_else(|| json!({}));
    let options = body.get("options").cloned().unwrap_or_else(|| json!({}));
    let mut ctx = json!({
        "form": {},
        "options": options,
        "deviceInfo": options.get("deviceInfo").cloned().unwrap_or_else(|| json!({})),
        "sentTo": {},
        "pending": {},
    });
    remember_origin(&mut ctx, &input);
    merge_form(&mut ctx, &input);

    let mut run = Run {
        id: format!("{}|#|{}", def.id, &generate_token()[..27]),
        task_id: def.start_task.clone(),
        ctx,
        status: FlowStatus::Waiting,
    };

    // config.json advertises the start screen, so the widget draws it itself and
    // sends its first submit with the start, as an interaction id plus the input.
    let interaction = body
        .get("interactionId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let result = match def.start_screen() {
        Some(screen) if !interaction.is_empty() => {
            let screen_id = screen["id"].as_str().unwrap_or_default().to_string();
            run.ctx["screenTask"] = json!(screen_id);
            match follow(screen, &interaction) {
                Some(next) => drive(state, def, &mut run, next, None).await,
                None => (
                    HeaderMap::new(),
                    failed(&run.id, "E102103", "Did not find next task"),
                ),
            }
        }
        _ => {
            let first = run.task_id.clone();
            drive(state, def, &mut run, first, None).await
        }
    };
    save(state, def, &run).await;
    result
}

pub async fn next(state: &EmulatorState, def: &FlowDef, body: &Value) -> (HeaderMap, Value) {
    let execution_id = body["executionId"].as_str().unwrap_or_default().to_string();
    let Some(stored) = state.flows.read().await.run(&execution_id).cloned() else {
        return (
            HeaderMap::new(),
            failed(&execution_id, "E102103", "Did not find next task"),
        );
    };
    let mut run = Run {
        id: stored.execution_id,
        task_id: stored.task_id,
        ctx: stored.context,
        status: stored.status,
    };
    let input = body.get("input").cloned().unwrap_or_else(|| json!({}));
    let interaction = body["interactionId"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let step = body["stepId"].as_str().unwrap_or_default().to_string();
    remember_origin(&mut run.ctx, &input);

    let result = if interaction == "polling" {
        poll(state, def, &mut run).await
    } else if run.status != FlowStatus::Waiting {
        (
            HeaderMap::new(),
            failed(&run.id, "E102103", "Did not find next task"),
        )
    } else if (input.get("token").is_some() || input.get("exchangeCode").is_some())
        && def.task(&step).is_some_and(|t| t["type"] == "automated")
    {
        // Back from an email link or an SSO round trip: the URL names the step that finishes it.
        drive(state, def, &mut run, step, Some(input)).await
    } else {
        let current = run.task_id.clone();
        match def.task(&current) {
            Some(task) if task["type"] == "screen" => {
                merge_form(&mut run.ctx, &input);
                match follow(task, &interaction) {
                    Some(next) => drive(state, def, &mut run, next, None).await,
                    None => (
                        HeaderMap::new(),
                        failed(&run.id, "E102103", "Did not find next task"),
                    ),
                }
            }
            Some(_) => drive(state, def, &mut run, current, Some(input)).await,
            None => (
                HeaderMap::new(),
                failed(&run.id, "E102103", "Did not find next task"),
            ),
        }
    };
    save(state, def, &run).await;
    result
}

/// A screen that waits on an email link polls. It moves on once the link was
/// opened; the tab that opened it ran the rest of the flow, so this one reports
/// where that run ended up.
async fn poll(state: &EmulatorState, def: &FlowDef, run: &mut Run) -> (HeaderMap, Value) {
    let verified = run.ctx["pending"]["magicLink"]["verified"] == true;
    if !verified {
        let mut env = envelope(&run.id, &run.task_id, "waiting");
        env["action"] = json!("poll");
        return (HeaderMap::new(), env);
    }
    if let Some(last) = run
        .ctx
        .get("lastEnvelope")
        .filter(|v| !v.is_null())
        .cloned()
    {
        return (HeaderMap::new(), last);
    }
    let current = run.task_id.clone();
    match def.task(&current).and_then(|t| follow(t, "polling")) {
        Some(next) => drive(state, def, run, next, None).await,
        None => (
            HeaderMap::new(),
            failed(&run.id, "E102103", "Did not find next task"),
        ),
    }
}

async fn save(state: &EmulatorState, def: &FlowDef, run: &Run) {
    let mut flows = state.flows.write().await;
    let created_at = flows
        .run(&run.id)
        .map(|r| r.created_at)
        .unwrap_or_else(now_secs);
    flows.sweep_expired(now_secs());
    flows.insert_run(FlowRun {
        execution_id: run.id.clone(),
        flow_id: def.id.clone(),
        task_id: run.task_id.clone(),
        context: run.ctx.clone(),
        status: run.status.clone(),
        created_at,
    });
}

/// The task a rule sends this interaction to. An action's own finish step
/// (`verifyStepId` and the like) is reached by resuming, never by a rule.
fn follow(task: &Value, interaction: &str) -> Option<String> {
    let own_steps: Vec<String> = ["verifyStepId", "finishStepId", "exchangeStepId"]
        .iter()
        .filter_map(|name| {
            task.pointer(&format!("/arguments/{name}/value"))
                .and_then(Value::as_str)
        })
        .map(str::to_string)
        .collect();
    task.pointer("/next/rules")?
        .as_array()?
        .iter()
        .filter(|rule| {
            interaction.is_empty()
                || rule["interactionId"].as_str().unwrap_or_default() == interaction
        })
        .filter_map(|rule| rule["taskId"].as_str())
        .find(|id| !own_steps.iter().any(|own| own == id))
        .map(str::to_string)
}

/// Walk the graph from `task_id` until a screen or an action stops it.
/// `input` goes to the first task only: what the widget sent with this request.
async fn drive(
    state: &EmulatorState,
    def: &FlowDef,
    run: &mut Run,
    mut task_id: String,
    mut input: Option<Value>,
) -> (HeaderMap, Value) {
    for _ in 0..MAX_STEPS {
        let Some(task) = def.task(&task_id).cloned() else {
            return (
                HeaderMap::new(),
                failed(
                    &run.id,
                    "E102103",
                    &format!("The flow has no task {task_id}"),
                ),
            );
        };
        let outcome = match task["type"].as_str() {
            Some("screen") => {
                run.task_id = task_id.clone();
                run.ctx["screenTask"] = json!(task_id);
                let env = screen_envelope(state, def, run, &task, None).await;
                remember(run, &env);
                return (HeaderMap::new(), env);
            }
            Some("condition") => Outcome::Interaction(evaluate_condition(state, run, &task).await),
            Some("automated") => {
                let step_input = input.take().unwrap_or_else(|| json!({}));
                perform(state, def, run, &task, &step_input).await
            }
            _ => Outcome::Interaction(String::new()),
        };
        match outcome {
            Outcome::Interaction(interaction) => match follow(&task, &interaction) {
                Some(next) => task_id = next,
                None => {
                    return (
                        HeaderMap::new(),
                        failed(
                            &run.id,
                            "E102103",
                            &format!("Task {task_id} has no rule for {interaction}"),
                        ),
                    )
                }
            },
            Outcome::Wait {
                envelope,
                await_task,
            } => {
                run.task_id = await_task;
                remember(run, &envelope);
                return (HeaderMap::new(), envelope);
            }
            Outcome::ScreenError(text) => {
                let screen_task = as_text(&run.ctx["screenTask"]);
                let Some(screen) = def.task(&screen_task).cloned() else {
                    return (HeaderMap::new(), failed(&run.id, "E102104", &text));
                };
                run.task_id = screen_task;
                let env = screen_envelope(state, def, run, &screen, Some(&text)).await;
                remember(run, &env);
                return (HeaderMap::new(), env);
            }
            Outcome::Done(headers, env) => {
                run.status = FlowStatus::Completed;
                remember(run, &env);
                return (headers, env);
            }
            Outcome::Fail(code, description) => {
                run.status = FlowStatus::Failed;
                let env = failed(&run.id, &code, &description);
                remember(run, &env);
                return (HeaderMap::new(), env);
            }
        }
    }
    (
        HeaderMap::new(),
        failed(&run.id, "E102105", "The flow did not reach a screen"),
    )
}

/// Kept so a tab polling the same run can be told where it ended up.
fn remember(run: &mut Run, env: &Value) {
    if run.ctx["pending"]["magicLink"]["verified"] == true {
        run.ctx["lastEnvelope"] = env.clone();
    }
}

async fn screen_envelope(
    state: &EmulatorState,
    def: &FlowDef,
    run: &Run,
    task: &Value,
    error: Option<&str>,
) -> Value {
    let task_id = task["id"].as_str().unwrap_or_default();
    let mut env = envelope(&run.id, task_id, "waiting");
    env["stepName"] = task["name"].clone();
    let full = full_context(state, run).await;
    let mut screen_state = json!({
        "componentsConfig": {
            "samlGroupMappings": {},
            "ssoApplications": {},
            "thirdPartyAppApproveScopes": {},
            "userRoles": {},
            "userSelectedTenant": {}
        },
        "form": run.ctx["form"],
        "inputs": run.ctx["form"],
        "sentTo": run.ctx["sentTo"],
        "user": full["user"],
        "project": { "name": "local" }
    });
    for key in task["contextKeys"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        set_path(&mut screen_state, key, get_path(&full, key).clone());
    }
    if let Some(text) = error {
        screen_state["errorText"] = json!(text);
        screen_state["errorType"] = json!("error");
    }
    env["screen"] = json!({
        "id": format!("{}/{}", def.id, task["screenId"].as_str().unwrap_or_default()),
        "state": screen_state
    });
    env
}

/// The run's context plus what conditions read live from the stores: the signed-in
/// user, the user the typed identifier names, and whether that address uses SSO.
async fn full_context(state: &EmulatorState, run: &Run) -> Value {
    let mut ctx = run.ctx.clone();
    let users = state.users.read().await;
    let webauthn = state.webauthn.read().await;

    let auth_user = run
        .auth_user_id()
        .and_then(|id| users.load_by_user_id(&id).ok())
        .map(|u| project_user(u, webauthn.has_credentials(&u.user_id)));
    let unauth_user = users
        .load(&run.login_id())
        .ok()
        .map(|u| project_user(u, webauthn.has_credentials(&u.user_id)));

    ctx["user"] = auth_user.clone().unwrap_or_else(|| json!({}));
    ctx["unauthUser"] = unauth_user.unwrap_or_else(|| json!({}));
    ctx["authInfo"] = json!({
        "firstSeen": run.ctx["auth"]["firstSeen"] == true,
        "user": auth_user.unwrap_or_else(|| json!({}))
    });
    drop(webauthn);
    drop(users);

    let login_id = run.login_id();
    ctx["ssoEnabled"] = json!(
        login_id.contains('@')
            && state
                .tenants
                .read()
                .await
                .find_by_email(&login_id)
                .is_ok_and(|t| matches!(t.auth_type, AuthType::Saml | AuthType::Oidc))
    );
    ctx
}

// ─── Conditions ─────────────────────────────────────────────────────────────

async fn evaluate_condition(state: &EmulatorState, run: &Run, task: &Value) -> String {
    let ctx = full_context(state, run).await;
    let mut otherwise = "ELSE".to_string();
    for condition in task["conditions"].as_array().into_iter().flatten() {
        let interaction = condition["interactionId"].as_str().unwrap_or_default();
        if interaction == "ELSE" {
            otherwise = interaction.to_string();
            continue;
        }
        let atoms = condition["atomicConditions"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if !atoms.is_empty() && atoms.iter().all(|atom| atom_holds(atom, &ctx)) {
            return interaction.to_string();
        }
    }
    otherwise
}

fn truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::String(s) => s.eq_ignore_ascii_case("true"),
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        _ => false,
    }
}

fn is_empty(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        Value::Array(a) => a.is_empty(),
        Value::Object(o) => o.is_empty(),
        _ => false,
    }
}

fn number(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str()?.trim().parse().ok())
}

fn looks_like_email(text: &str) -> bool {
    let Some((user, domain)) = text.trim().split_once('@') else {
        return false;
    };
    !user.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains('@')
}

fn atom_holds(atom: &Value, ctx: &Value) -> bool {
    let target = resolve_value(&atom["target"], ctx);
    let predicate = resolve_value(&atom["predicate"], ctx);
    let compare = |f: fn(f64, f64) -> bool| match (number(&target), number(&predicate)) {
        (Some(a), Some(b)) => f(a, b),
        _ => false,
    };
    match atom["operator"].as_str().unwrap_or_default() {
        "is-true" => truthy(&target),
        "is-false" => !truthy(&target),
        "equal" => as_text(&target) == as_text(&predicate),
        "not-equal" => as_text(&target) != as_text(&predicate),
        "contains" => as_text(&target).contains(&as_text(&predicate)),
        "greater-than" => compare(|a, b| a > b),
        "greater-than-or-equal" => compare(|a, b| a >= b),
        "less-than" => compare(|a, b| a < b),
        "less-than-or-equal" => compare(|a, b| a <= b),
        "empty" => is_empty(&target),
        "not-empty" => !is_empty(&target),
        "is-email" => looks_like_email(&as_text(&target)),
        "is-not-email" => !looks_like_email(&as_text(&target)),
        other => {
            tracing::warn!(
                operator = other,
                "The emulator does not evaluate this condition operator"
            );
            false
        }
    }
}

// ─── Actions ────────────────────────────────────────────────────────────────

/// Follow the task's rule for an error by name when the flow handles it,
/// otherwise show the screen again with `text`.
fn handled_error(task: &Value, interaction: &str, text: &str) -> Outcome {
    if follow(task, interaction).is_some() {
        Outcome::Interaction(interaction.to_string())
    } else {
        Outcome::ScreenError(text.to_string())
    }
}

async fn perform(
    state: &EmulatorState,
    def: &FlowDef,
    run: &mut Run,
    task: &Value,
    input: &Value,
) -> Outcome {
    let action = task["action"].as_str().unwrap_or_default();
    match action {
        "risk" => risk(state, run).await,
        "update-user-properties" => update_user_properties(state, run, task).await,
        "password-sign-in-v2" | "password-sign-in" => password_sign_in(state, run, task).await,
        "password-replace-v3" | "password-replace" => password_replace(state, run).await,
        "password-update" => password_update(state, run).await,
        "sign-in-otp-email" | "sign-up-or-in-otp-email" | "sign-up-otp-email" => {
            send_code(
                state,
                run,
                task,
                Channel::Email,
                action != "sign-in-otp-email",
            )
            .await
        }
        "sign-in-otp-sms" | "sign-up-or-in-otp-sms" | "sign-up-otp-sms" => {
            send_code(state, run, task, Channel::Sms, action != "sign-in-otp-sms").await
        }
        "verify-code-otp-email" => verify_code(state, run, Channel::Email).await,
        "verify-code-otp-sms" => verify_code(state, run, Channel::Sms).await,
        "sign-in-magic-linkv3-email"
        | "sign-in-magic-link-email"
        | "sign-up-or-in-magic-link-email"
        | "update-user-magic-link-email" => {
            send_link(
                state,
                def,
                run,
                task,
                action == "update-user-magic-link-email",
            )
            .await
        }
        "verify-magic-link" => verify_link(state, run, input).await,
        "webauthn-sign-up-start" | "webauthn-sign-in-start" | "webauthn-update-user-start" => {
            passkey_start(state, def, run, task, action).await
        }
        "webauthn-sign-up-finish" | "webauthn-sign-in-finish" | "webauthn-update-user-finish" => {
            passkey_finish(state, run, input, action).await
        }
        "saml-start" | "sso-start" => sso_start(state, def, run, task).await,
        "exchange-saml-code" | "exchange-sso-code" => sso_exchange(state, run, input).await,
        "scriptlet" => {
            tracing::warn!(task = %task["id"], "Scriptlets do not run in the emulator; the form is left as it is");
            Outcome::Interaction("success".into())
        }
        "logged-in" => logged_in(state, run).await,
        other => Outcome::Fail(
            "E102004".into(),
            format!(
                "The emulator does not run the {other} action (task {})",
                task["id"].as_str().unwrap_or_default()
            ),
        ),
    }
}

/// The ids the signed-in user, or else the typed identifier, signs in with.
async fn known_login_ids(state: &EmulatorState, run: &Run) -> Vec<String> {
    let users = state.users.read().await;
    if let Some(user) = run
        .auth_user_id()
        .and_then(|id| users.load_by_user_id(&id).ok())
    {
        let mut ids = user.login_ids.clone();
        ids.extend(user.email.clone());
        return ids;
    }
    vec![run.login_id()]
}

async fn risk(state: &EmulatorState, run: &mut Run) -> Outcome {
    let ids = known_login_ids(state, run).await;
    let score = state.flows.read().await.risk_score(&ids);
    run.ctx["riskInfo"] = json!({ "riskScore": score });
    Outcome::Interaction("success".into())
}

/// The user an action works on: the signed-in user, else the one the typed identifier names.
async fn target_user_id(state: &EmulatorState, run: &Run) -> Option<String> {
    if let Some(id) = run.auth_user_id() {
        return Some(id);
    }
    state
        .users
        .read()
        .await
        .load(&run.login_id())
        .ok()
        .map(|u| u.user_id.clone())
}

async fn update_user_properties(state: &EmulatorState, run: &mut Run, task: &Value) -> Outcome {
    let Some(user_id) = target_user_id(state, run).await else {
        return Outcome::Interaction("success".into());
    };
    let ctx = full_context(state, run).await;
    let mut definitions = task
        .pointer("/arguments/userPropertiesDefinitions/value")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    // Without definitions the step saves what the screen collected.
    if definitions.is_empty() {
        if let Some(name) = run.ctx["form"]["fullName"].as_str() {
            definitions.push(json!({ "key": "displayName", "type": "string", "value": name }));
        }
    }

    let mut users = state.users.write().await;
    let Ok(user) = users.load_by_user_id_mut(&user_id) else {
        return Outcome::Interaction("success".into());
    };
    for definition in definitions {
        let key = definition["key"].as_str().unwrap_or_default();
        let value = match definition["type"].as_str() {
            Some("inline") | Some("context") => resolve_value(&definition, &ctx),
            _ => definition["value"].clone(),
        };
        if let Some(attribute) = key.strip_prefix("customAttributes.") {
            user.custom_attributes.insert(attribute.to_string(), value);
        } else if matches!(key, "displayName" | "name") {
            user.name = Some(as_text(&value));
        } else if key == "givenName" {
            user.given_name = Some(as_text(&value));
        } else if key == "familyName" {
            user.family_name = Some(as_text(&value));
        }
    }
    Outcome::Interaction("success".into())
}

async fn password_sign_in(state: &EmulatorState, run: &mut Run, task: &Value) -> Outcome {
    let login_id = run.login_id();
    let password = as_text(&run.ctx["form"]["password"]);
    if AuthPolicyGuard::check_method_enabled(state, "password")
        .await
        .is_err()
        || AuthPolicyGuard::check_not_locked_out(state, &login_id)
            .await
            .is_err()
    {
        return Outcome::ScreenError(PASSWORD_REFUSED.into());
    }
    let found = state.users.read().await.load(&login_id).ok().map(|u| {
        (
            u.user_id.clone(),
            u._password_hash.clone(),
            u.status.clone(),
        )
    });
    let Some((user_id, Some(hash), status)) = found else {
        AuthPolicyGuard::record_failure(state, &login_id).await;
        return handled_error(task, "UserDoesNotExists", PASSWORD_REFUSED);
    };
    let valid = tokio::task::spawn_blocking(move || bcrypt::verify(&password, &hash))
        .await
        .ok()
        .and_then(Result::ok)
        .unwrap_or(false);
    if !valid || status == "disabled" {
        AuthPolicyGuard::record_failure(state, &login_id).await;
        return Outcome::ScreenError(PASSWORD_REFUSED.into());
    }
    AuthPolicyGuard::clear_failures(state, &login_id).await;
    run.sign_in(&user_id, "pwd", false);
    Outcome::Interaction("success".into())
}

async fn set_password(state: &EmulatorState, user_id: &str, password: String) -> Outcome {
    let hash = match tokio::task::spawn_blocking(move || bcrypt::hash(password, 4)).await {
        Ok(Ok(hash)) => hash,
        _ => return Outcome::ScreenError("The password could not be saved.".into()),
    };
    let mut users = state.users.write().await;
    match users.load_by_user_id_mut(user_id) {
        Ok(user) => {
            user._password_hash = Some(hash);
            user.password = true;
            Outcome::Interaction("success".into())
        }
        Err(_) => Outcome::ScreenError("The password could not be saved.".into()),
    }
}

fn new_password(run: &Run) -> String {
    ["newPassword_noPolicyOverrides", "newPassword"]
        .iter()
        .map(|key| as_text(&run.ctx["form"][*key]))
        .find(|p| !p.is_empty())
        .unwrap_or_default()
}

async fn password_replace(state: &EmulatorState, run: &mut Run) -> Outcome {
    let Some(user_id) = target_user_id(state, run).await else {
        return Outcome::ScreenError(PASSWORD_REFUSED.into());
    };
    let password = new_password(run);
    let outcome = set_password(state, &user_id, password).await;
    if run.auth_user_id().is_none() {
        run.sign_in(&user_id, "pwd", false);
    }
    outcome
}

async fn password_update(state: &EmulatorState, run: &mut Run) -> Outcome {
    let Some(user_id) = run.auth_user_id() else {
        return Outcome::ScreenError("Sign in before you set a password.".into());
    };
    let password = new_password(run);
    set_password(state, &user_id, password).await
}

#[derive(Clone, Copy)]
enum Channel {
    Email,
    Sms,
}

async fn send_code(
    state: &EmulatorState,
    run: &mut Run,
    task: &Value,
    channel: Channel,
    may_sign_up: bool,
) -> Outcome {
    let mut created = false;
    let user_id = match target_user_id(state, run).await {
        Some(id) => id,
        None if may_sign_up
            && matches!(channel, Channel::Email)
            && looks_like_email(&run.login_id()) =>
        {
            let email = run.login_id().to_lowercase();
            let user_id = new_user_id();
            let user = User {
                user_id: user_id.clone(),
                login_ids: vec![email.clone()],
                email: Some(email),
                status: "enabled".into(),
                created_time: now_secs(),
                ..Default::default()
            };
            if state.users.write().await.insert(user).is_err() {
                return Outcome::ScreenError("The account could not be created.".into());
            }
            created = true;
            user_id
        }
        None => {
            return handled_error(
                task,
                "UserDoesNotExists",
                "No account was found for this sign-in.",
            )
        }
    };

    let (email, phone) = {
        let users = state.users.read().await;
        match users.load_by_user_id(&user_id) {
            Ok(u) => (u.email.clone(), u.phone.clone()),
            Err(_) => (None, None),
        }
    };
    let code = generate_otp_code();
    state.otps.write().await.store(&user_id, code.clone());
    match channel {
        Channel::Email => {
            let to = email.unwrap_or_else(|| run.login_id());
            tracing::info!(to = %to, code = %code, "📧 Flow OTP generated");
            run.ctx["sentTo"]["maskedEmail"] = json!(masked_email(&to));
        }
        Channel::Sms => {
            let to = phone.unwrap_or_else(|| run.login_id());
            tracing::info!(to = %to, code = %code, "📱 Flow OTP generated");
            run.ctx["sentTo"]["maskedPhone"] = json!(masked_phone(&to));
        }
    }
    run.ctx["pending"]["code"] = json!({ "userId": user_id, "created": created });
    Outcome::Interaction("success".into())
}

async fn verify_code(state: &EmulatorState, run: &mut Run, channel: Channel) -> Outcome {
    let Some(user_id) = run.ctx["pending"]["code"]["userId"]
        .as_str()
        .map(str::to_string)
    else {
        return Outcome::ScreenError(CODE_REFUSED.into());
    };
    let code = as_text(&run.ctx["form"]["code"]);
    if state
        .otps
        .write()
        .await
        .consume(&user_id, code.trim())
        .is_err()
    {
        return Outcome::ScreenError(CODE_REFUSED.into());
    }
    let login_id = {
        let users = state.users.read().await;
        users
            .load_by_user_id(&user_id)
            .ok()
            .and_then(|u| u.login_ids.first().cloned())
    };
    if let Some(login_id) = login_id {
        let patch = match channel {
            Channel::Email => UserPatch {
                verified_email: Some(true),
                ..Default::default()
            },
            Channel::Sms => UserPatch {
                verified_phone: Some(true),
                ..Default::default()
            },
        };
        let _ = state.users.write().await.patch(&login_id, patch);
    }
    if run.auth_user_id().is_none() {
        let created = run.ctx["pending"]["code"]["created"] == true;
        let method = match channel {
            Channel::Email => "email",
            Channel::Sms => "sms",
        };
        run.sign_in(&user_id, method, created);
    }
    Outcome::Interaction("success".into())
}

/// Add the flow's resume marker and the token to the link's return URL.
fn link_url(base: &str, execution_id: &str, step_id: &str, token: &str) -> String {
    let separator = if base.contains('?') { '&' } else { '?' };
    format!(
        "{base}{separator}descope-login-flow={}&t={token}",
        urlencoding::encode(&format!("{execution_id}_{step_id}"))
    )
}

fn return_url(run: &Run, explicit: &str) -> String {
    [
        explicit.to_string(),
        as_text(&run.ctx["options"]["redirectUrl"]),
        as_text(&run.ctx["options"]["location"]),
    ]
    .into_iter()
    .find(|u| !u.is_empty())
    .map(|u| u.split(['?', '#']).next().unwrap_or_default().to_string())
    .unwrap_or_else(|| "http://localhost:4200/login".into())
}

async fn send_link(
    state: &EmulatorState,
    def: &FlowDef,
    run: &mut Run,
    task: &Value,
    update_user: bool,
) -> Outcome {
    let user_id = if update_user {
        run.auth_user_id()
    } else {
        target_user_id(state, run).await
    };
    let Some(user_id) = user_id else {
        return handled_error(
            task,
            "UserDoesNotExists",
            "No account was found for this sign-in.",
        );
    };
    let email = {
        let users = state.users.read().await;
        users
            .load_by_user_id(&user_id)
            .ok()
            .and_then(|u| u.email.clone())
    }
    .unwrap_or_else(|| run.login_id());

    let ctx = full_context(state, run).await;
    let verify_step = argument_text(task, "verifyStepId", &ctx);
    let token = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(token.clone(), user_id.clone(), TokenType::Magic);
    let link = link_url(
        &return_url(run, &argument_text(task, "uri", &ctx)),
        &run.id,
        &verify_step,
        &token,
    );
    tracing::info!(to = %email, link = %link, flow = %def.id, "📧 Flow sign-in link generated");

    let sent = SentLink {
        token: token.clone(),
        link,
        execution_id: run.id.clone(),
    };
    {
        let mut flows = state.flows.write().await;
        flows.record_sent_link(&email, sent.clone());
        flows.record_sent_link(&run.login_id(), sent);
    }
    run.ctx["sentTo"]["maskedEmail"] = json!(masked_email(&email));
    run.ctx["pending"]["magicLink"] =
        json!({ "token": token, "userId": user_id, "verified": false });
    Outcome::Interaction("success".into())
}

async fn verify_link(state: &EmulatorState, run: &mut Run, input: &Value) -> Outcome {
    let token = input["token"].as_str().unwrap_or_default();
    let pending = run.ctx["pending"]["magicLink"].clone();
    if token.is_empty() || pending["token"].as_str() != Some(token) {
        return Outcome::Fail("E062503".into(), "The sign-in link is not valid".into());
    }
    let Ok(entry) = state.tokens.write().await.consume(token) else {
        return Outcome::Fail(
            "E062503".into(),
            "The sign-in link has expired or was already used".into(),
        );
    };
    let login_id = {
        let users = state.users.read().await;
        users
            .load_by_user_id(&entry.user_id)
            .ok()
            .and_then(|u| u.login_ids.first().cloned())
    };
    if let Some(login_id) = login_id {
        let _ = state.users.write().await.patch(
            &login_id,
            UserPatch {
                verified_email: Some(true),
                ..Default::default()
            },
        );
    }
    run.ctx["pending"]["magicLink"]["verified"] = json!(true);
    if run.auth_user_id().is_none() {
        run.sign_in(&entry.user_id, "magiclink", false);
    }
    Outcome::Interaction("success".into())
}

/// The relying party is the page the widget runs on.
fn rp_id(run: &Run) -> String {
    let origin = [
        as_text(&run.ctx["origin"]),
        as_text(&run.ctx["options"]["location"]),
    ]
    .into_iter()
    .find(|o| !o.is_empty())
    .unwrap_or_else(|| "http://localhost".into());
    let without_scheme = origin.split("://").nth(1).unwrap_or(&origin);
    without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .split(':')
        .next()
        .unwrap_or("localhost")
        .to_string()
}

fn ceremony_of(action: &str) -> Ceremony {
    if action.starts_with("webauthn-sign-up") {
        Ceremony::SignUp
    } else if action.starts_with("webauthn-sign-in") {
        Ceremony::SignIn
    } else {
        Ceremony::Update
    }
}

async fn passkey_start(
    state: &EmulatorState,
    def: &FlowDef,
    run: &mut Run,
    task: &Value,
    action: &str,
) -> Outcome {
    let ceremony = ceremony_of(action);
    let ctx = full_context(state, run).await;
    let finish = argument_text(task, "finishStepId", &ctx);

    let (login_id, user_id, display_name) = match ceremony {
        Ceremony::SignUp => {
            if state.users.read().await.load(&run.login_id()).is_ok() {
                return handled_error(
                    task,
                    "UserAlreadyExists",
                    "An account already exists for this sign-in.",
                );
            }
            (
                run.login_id(),
                String::new(),
                as_text(&run.ctx["form"]["fullName"]),
            )
        }
        Ceremony::SignIn => {
            let users = state.users.read().await;
            let Ok(user) = users.load(&run.login_id()) else {
                return handled_error(task, "UserDoesNotExists", NO_PASSKEY);
            };
            if !state.webauthn.read().await.has_credentials(&user.user_id) {
                return Outcome::ScreenError(NO_PASSKEY.into());
            }
            (
                run.login_id(),
                user.user_id.clone(),
                user.name.clone().unwrap_or_default(),
            )
        }
        Ceremony::Update => {
            let Some(user_id) = run.auth_user_id() else {
                return Outcome::ScreenError("Sign in before you add biometrics.".into());
            };
            let users = state.users.read().await;
            let Ok(user) = users.load_by_user_id(&user_id) else {
                return Outcome::ScreenError("Sign in before you add biometrics.".into());
            };
            let named = argument_text(task, "passkeyLoginId", &ctx);
            let login_id = if named.is_empty() {
                user.login_ids.first().cloned().unwrap_or_default()
            } else {
                named
            };
            (
                login_id,
                user.user_id.clone(),
                user.name.clone().unwrap_or_default(),
            )
        }
    };

    let challenge: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
    let (transaction_id, options) = state.webauthn.write().await.begin(
        ceremony,
        &login_id,
        &user_id,
        &display_name,
        &rp_id(run),
        now_secs(),
        generate_token(),
        challenge,
    );
    run.ctx["pending"]["passkey"] = json!({ "transactionId": transaction_id, "loginId": login_id });

    let screen_task = as_text(&run.ctx["screenTask"]);
    let mut env = match def.task(&screen_task) {
        Some(screen) => screen_envelope(state, def, run, screen, None).await,
        None => envelope(&run.id, &finish, "waiting"),
    };
    env["stepId"] = json!(finish);
    env["action"] = json!(if ceremony == Ceremony::SignIn {
        "webauthnGet"
    } else {
        "webauthnCreate"
    });
    env["webauthn"] = json!({ "transactionId": transaction_id, "options": options });
    Outcome::Wait {
        envelope: env,
        await_task: finish,
    }
}

async fn passkey_finish(
    state: &EmulatorState,
    run: &mut Run,
    input: &Value,
    action: &str,
) -> Outcome {
    let ceremony = ceremony_of(action);
    if let Some(failure) = input["failure"].as_str().filter(|f| !f.is_empty()) {
        return Outcome::ScreenError(format!(
            "The passkey request was cancelled or not allowed ({failure})."
        ));
    }
    let transaction_id = input["transactionId"].as_str().unwrap_or_default();
    let response = input["response"].as_str().unwrap_or_default();

    let transaction = match state
        .webauthn
        .write()
        .await
        .take_transaction(transaction_id, ceremony)
    {
        Ok(t) => t,
        Err(_) => return Outcome::ScreenError(PASSKEY_REFUSED.into()),
    };
    match ceremony {
        Ceremony::SignIn => match state
            .webauthn
            .write()
            .await
            .finish_assertion(&transaction, response)
        {
            Ok(user_id) => {
                run.sign_in(&user_id, "webauthn", false);
                Outcome::Interaction("success".into())
            }
            Err(e) => {
                tracing::info!(error = %e, "Passkey sign-in refused");
                Outcome::ScreenError(PASSKEY_REFUSED.into())
            }
        },
        Ceremony::Update => {
            let Some(user_id) = run.auth_user_id() else {
                return Outcome::ScreenError(PASSKEY_REFUSED.into());
            };
            match state
                .webauthn
                .write()
                .await
                .finish_registration(&transaction, &user_id, response)
            {
                Ok(()) => Outcome::Interaction("success".into()),
                Err(e) => {
                    tracing::info!(error = %e, "Passkey registration refused");
                    Outcome::ScreenError(PASSKEY_REFUSED.into())
                }
            }
        }
        Ceremony::SignUp => {
            let user_id = new_user_id();
            let email = transaction.login_id.to_lowercase();
            // Check the response before the user exists, so a refused passkey leaves nothing behind.
            if let Err(e) =
                state
                    .webauthn
                    .write()
                    .await
                    .finish_registration(&transaction, &user_id, response)
            {
                tracing::info!(error = %e, "Passkey sign-up refused");
                return Outcome::ScreenError(PASSKEY_REFUSED.into());
            }
            let user = User {
                user_id: user_id.clone(),
                login_ids: vec![email.clone()],
                email: looks_like_email(&email).then_some(email),
                name: Some(as_text(&run.ctx["form"]["fullName"])).filter(|n| !n.is_empty()),
                status: "enabled".into(),
                created_time: now_secs(),
                ..Default::default()
            };
            if state.users.write().await.insert(user).is_err() {
                return Outcome::ScreenError("An account already exists for this sign-in.".into());
            }
            run.sign_in(&user_id, "webauthn", true);
            Outcome::Interaction("success".into())
        }
    }
}

async fn sso_start(state: &EmulatorState, def: &FlowDef, run: &mut Run, task: &Value) -> Outcome {
    let ctx = full_context(state, run).await;
    let hint = Some(argument_text(task, "loginHint", &ctx))
        .filter(|h| !h.is_empty())
        .unwrap_or_else(|| run.login_id());
    let exchange = argument_text(task, "exchangeStepId", &ctx);
    let back = link_url(
        &return_url(run, &argument_text(task, "redirectUrl", &ctx)),
        &run.id,
        &exchange,
        "",
    )
    .trim_end_matches("&t=")
    .to_string();
    let request = SamlStartRequest {
        tenant: hint.clone(),
        redirect_url: Some(back),
        login_hint: Some(hint),
        sso_jit_prompt: None,
    };
    let url = match saml_start(state.clone(), request).await {
        Ok(axum::Json(body)) => as_text(&body["url"]),
        Err(e) => return Outcome::ScreenError(format!("SSO sign-in could not start: {e}")),
    };
    let screen_task = as_text(&run.ctx["screenTask"]);
    let mut env = match def.task(&screen_task) {
        Some(screen) => screen_envelope(state, def, run, screen, None).await,
        None => envelope(&run.id, &exchange, "waiting"),
    };
    env["stepId"] = json!(exchange);
    env["action"] = json!("redirect");
    env["redirect"] = json!({ "url": url, "isPopup": false });
    Outcome::Wait {
        envelope: env,
        await_task: exchange,
    }
}

async fn sso_exchange(state: &EmulatorState, run: &mut Run, input: &Value) -> Outcome {
    let code = input["exchangeCode"].as_str().unwrap_or_default();
    let Ok(entry) = state.tokens.write().await.consume(code) else {
        return Outcome::Fail("E062503".into(), "The SSO code is not valid".into());
    };
    if entry.user_id.starts_with("tenant:") {
        return Outcome::Fail("E112102".into(), "User not found".into());
    }
    run.sign_in(&entry.user_id, "saml", false);
    Outcome::Interaction("success".into())
}

async fn logged_in(state: &EmulatorState, run: &mut Run) -> Outcome {
    let Some(user_id) = run.auth_user_id() else {
        return Outcome::Fail(
            "E102106".into(),
            "The flow ended without signing anyone in".into(),
        );
    };
    let method = as_text(&run.ctx["auth"]["method"]);
    let first_seen = run.ctx["auth"]["firstSeen"] == true;
    let has_passkey = state.webauthn.read().await.has_credentials(&user_id);

    let users = state.users.read().await;
    let Ok(user) = users.load_by_user_id(&user_id) else {
        return Outcome::Fail("E112102".into(), "User not found".into());
    };
    if user.status == "disabled" {
        return Outcome::Fail("E011006".into(), "User is disabled".into());
    }
    let km = state.km().await;
    let session = {
        let templates = state.jwt_templates.read().await;
        generate_session_jwt(
            &km,
            user,
            &state.config.project_id,
            state.config.session_ttl,
            templates.active(),
            &*state.roles.read().await,
            if method.is_empty() { "pwd" } else { &method },
        )
    };
    let refresh = generate_refresh_jwt(
        &km,
        &user.user_id,
        &state.config.project_id,
        state.config.refresh_ttl,
    );
    let (Ok(session), Ok(refresh)) = (session, refresh) else {
        return Outcome::Fail("E000000".into(), "The session could not be signed".into());
    };
    let mut user_resp = serde_json::to_value(user.to_response()).unwrap_or_else(|_| json!({}));
    user_resp["webauthn"] = json!(has_passkey);
    let mut auth_info = build_jwt_response(state, &user.user_id, &session, &refresh, user_resp);
    auth_info["firstSeen"] = json!(first_seen);
    drop(users);
    let _ = state.users.write().await.record_login_by_user_id(&user_id);

    let mut env = envelope(&run.id, &run.task_id, "completed");
    env["authInfo"] = auth_info;
    let cookies = build_auth_cookies(&session, &refresh, state.config.session_ttl);
    Outcome::Done(cookies, env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_fill_from_the_context() {
        let ctx =
            json!({ "form": { "email": "pat@example.com" }, "user": { "phone": "+15550100" } });
        assert_eq!(render("{{form.email}}", &ctx), "pat@example.com");
        assert_eq!(
            render("to {{user.phone}} and {{missing.key}}!", &ctx),
            "to +15550100 and !"
        );
    }

    #[test]
    fn conditions_read_the_operators_the_sign_in_flow_uses() {
        let ctx = json!({
            "riskInfo": { "riskScore": 0.7 },
            "unauthUser": { "webauthn": true, "tenantIds": [], "customAttributes": { "lastSignInMethod": "code" } },
            "form": { "email": "pat@example.com" },
            "user": { "email": "" }
        });
        let atom = |operator: &str, path: &str, value: Value| json!({ "operator": operator, "target": { "type": "context", "value": path }, "predicate": { "type": "inline", "value": value } });

        assert!(atom_holds(
            &atom("greater-than", "riskInfo.riskScore", json!(0.5)),
            &ctx
        ));
        assert!(!atom_holds(
            &atom("greater-than", "riskInfo.riskScore", json!(0.9)),
            &ctx
        ));
        assert!(atom_holds(
            &atom("is-true", "unauthUser.webauthn", json!("")),
            &ctx
        ));
        assert!(atom_holds(
            &atom("empty", "unauthUser.tenantIds", json!("")),
            &ctx
        ));
        assert!(atom_holds(&atom("empty", "user.email", json!("")), &ctx));
        assert!(!atom_holds(
            &atom("not-empty", "user.email", json!("")),
            &ctx
        ));
        assert!(atom_holds(
            &atom(
                "equal",
                "unauthUser.customAttributes.lastSignInMethod",
                json!("code")
            ),
            &ctx
        ));
        assert!(atom_holds(&atom("is-email", "form.email", json!("")), &ctx));
        assert!(!atom_holds(
            &atom(
                "is-email",
                "unauthUser.customAttributes.lastSignInMethod",
                json!("")
            ),
            &ctx
        ));
    }

    #[test]
    fn an_actions_own_finish_step_is_not_followed_as_a_rule() {
        let task = json!({
            "arguments": { "verifyStepId": { "type": "inline", "value": "24.end" } },
            "next": { "rules": [
                { "interactionId": "success", "taskId": "24.end" },
                { "interactionId": "success", "taskId": "25" }
            ] }
        });
        assert_eq!(follow(&task, "success").as_deref(), Some("25"));
    }

    #[test]
    fn the_relying_party_is_the_widgets_host() {
        let run = Run {
            id: "f|#|1".into(),
            task_id: "0".into(),
            ctx: json!({ "origin": "http://localhost:4200" }),
            status: FlowStatus::Waiting,
        };
        assert_eq!(rp_id(&run), "localhost");
    }

    #[test]
    fn a_link_names_the_run_and_step_it_resumes() {
        assert_eq!(
            link_url("http://localhost:4200/login", "f|#|abc", "24.end", "TOK"),
            "http://localhost:4200/login?descope-login-flow=f%7C%23%7Cabc_24.end&t=TOK"
        );
    }
}

#[cfg(test)]
#[path = "flow_engine_tests.rs"]
mod scenarios;
