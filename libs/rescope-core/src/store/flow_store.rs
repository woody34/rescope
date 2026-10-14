use std::collections::HashMap;
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

/// TTL for a flow execution before it's swept as expired (seconds).
const FLOW_TTL_SECS: u64 = 600;

#[cfg(test)]
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_secs()
}

/// Lifecycle state of a single flow execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowStatus {
    /// Waiting for the next user input (a screen is shown).
    Waiting,
    /// Flow finished successfully (auth minted).
    Completed,
    /// Flow terminated with an error.
    Failed,
}

/// A single in-flight `sign-up-or-in-passwords` execution.
///
/// `step` tracks which of the two screens the runtime last rendered:
/// `1` = email screen (`signIn`), `2` = password screen (`signInPassword`).
#[derive(Debug, Clone)]
pub struct FlowExecution {
    pub execution_id: String,
    pub flow_id: String,
    pub status: FlowStatus,
    /// 1 = email screen, 2 = password screen.
    pub step: u8,
    pub screen_id: String,
    pub login_id: Option<String>,
    pub created_at: u64,
}

/// An execution of a flow loaded from a Descope export (see the API crate's
/// `flow_engine`). `task_id` is the task waiting for input: a screen, or an
/// automated task that finishes a step the browser started (a passkey, a link).
/// `context` is the flow context the export's conditions and templates read.
#[derive(Debug, Clone)]
pub struct FlowRun {
    pub execution_id: String,
    pub flow_id: String,
    pub task_id: String,
    pub context: serde_json::Value,
    pub status: FlowStatus,
    pub created_at: u64,
}

/// The last sign-in link a flow sent to a login id, for tests to open.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SentLink {
    pub token: String,
    pub link: String,
    pub execution_id: String,
}

/// In-memory store of flow executions, keyed by executionId.
#[derive(Default)]
pub struct FlowStore {
    execs: HashMap<String, FlowExecution>,
    runs: HashMap<String, FlowRun>,
    sent_links: HashMap<String, SentLink>,
    risk_scores: HashMap<String, f64>,
    default_risk_score: Option<f64>,
}

impl FlowStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert (or overwrite) a flow execution.
    pub fn insert(&mut self, exec: FlowExecution) {
        self.execs.insert(exec.execution_id.clone(), exec);
    }

    /// Non-destructive read of an execution.
    pub fn get(&self, id: &str) -> Option<&FlowExecution> {
        self.execs.get(id)
    }

    /// Mutable access to an execution.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut FlowExecution> {
        self.execs.get_mut(id)
    }

    /// Advance an execution from the email screen to the password screen,
    /// recording the login_id the user submitted. No-op if the id is unknown.
    pub fn advance_to_password(&mut self, id: &str, login_id: String) {
        if let Some(exec) = self.execs.get_mut(id) {
            exec.step = 2;
            exec.screen_id = "signInPassword".to_string();
            exec.login_id = Some(login_id);
            exec.status = FlowStatus::Waiting;
        }
    }

    /// Mark an execution completed. No-op if the id is unknown.
    pub fn complete(&mut self, id: &str) {
        if let Some(exec) = self.execs.get_mut(id) {
            exec.status = FlowStatus::Completed;
        }
    }

    /// Mark an execution failed. No-op if the id is unknown.
    pub fn fail(&mut self, id: &str) {
        if let Some(exec) = self.execs.get_mut(id) {
            exec.status = FlowStatus::Failed;
        }
    }

    /// Clear the `sign-up-or-in-passwords` executions.
    ///
    /// Runs of exported flows, sent links and risk scores are kept, for the
    /// reason `/emulator/reset` keeps magic-link tokens: parallel test shards
    /// share one emulator, and one shard's reset must not end a sign-in another
    /// shard is part way through. Runs expire on their own; tests clear risk
    /// scores with `DELETE /emulator/risk`.
    pub fn reset(&mut self) {
        self.execs.clear();
    }

    /// Drop executions older than the TTL relative to `now`.
    pub fn sweep_expired(&mut self, now: u64) {
        self.execs
            .retain(|_, e| now.saturating_sub(e.created_at) < FLOW_TTL_SECS);
        self.runs
            .retain(|_, r| now.saturating_sub(r.created_at) < FLOW_TTL_SECS);
    }

    pub fn insert_run(&mut self, run: FlowRun) {
        self.runs.insert(run.execution_id.clone(), run);
    }

    pub fn run(&self, id: &str) -> Option<&FlowRun> {
        self.runs.get(id)
    }

    pub fn run_mut(&mut self, id: &str) -> Option<&mut FlowRun> {
        self.runs.get_mut(id)
    }

    /// Record the sign-in link a flow sent to `login_id`, replacing an older one.
    pub fn record_sent_link(&mut self, login_id: &str, link: SentLink) {
        self.sent_links.insert(login_id.to_lowercase(), link);
    }

    pub fn sent_link(&self, login_id: &str) -> Option<&SentLink> {
        self.sent_links.get(&login_id.to_lowercase())
    }

    /// Set the score the flow's risk step reports for `login_id`, or for every
    /// sign-in when `login_id` is `None`.
    pub fn set_risk_score(&mut self, login_id: Option<&str>, score: f64) {
        match login_id {
            Some(id) => {
                self.risk_scores.insert(id.to_lowercase(), score);
            }
            None => self.default_risk_score = Some(score),
        }
    }

    /// The score the risk step reports: the login id's, else the default, else 0.
    pub fn risk_score(&self, login_ids: &[String]) -> f64 {
        login_ids
            .iter()
            .find_map(|id| self.risk_scores.get(&id.to_lowercase()).copied())
            .or(self.default_risk_score)
            .unwrap_or(0.0)
    }

    pub fn clear_risk_scores(&mut self) {
        self.risk_scores.clear();
        self.default_risk_score = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exec(id: &str) -> FlowExecution {
        FlowExecution {
            execution_id: id.to_string(),
            flow_id: "sign-up-or-in-passwords".to_string(),
            status: FlowStatus::Waiting,
            step: 1,
            screen_id: "signIn".to_string(),
            login_id: None,
            created_at: now_secs(),
        }
    }

    #[test]
    fn insert_and_get() {
        let mut store = FlowStore::new();
        store.insert(exec("e1"));
        assert_eq!(store.get("e1").unwrap().step, 1);
        assert_eq!(store.get("e1").unwrap().screen_id, "signIn");
        assert!(store.get("missing").is_none());
    }

    #[test]
    fn advance_sets_step_and_login() {
        let mut store = FlowStore::new();
        store.insert(exec("e1"));
        store.advance_to_password("e1", "u@example.com".into());
        let e = store.get("e1").unwrap();
        assert_eq!(e.step, 2);
        assert_eq!(e.screen_id, "signInPassword");
        assert_eq!(e.login_id.as_deref(), Some("u@example.com"));
        assert_eq!(e.status, FlowStatus::Waiting);
    }

    #[test]
    fn complete_and_fail() {
        let mut store = FlowStore::new();
        store.insert(exec("e1"));
        store.complete("e1");
        assert_eq!(store.get("e1").unwrap().status, FlowStatus::Completed);
        store.fail("e1");
        assert_eq!(store.get("e1").unwrap().status, FlowStatus::Failed);
    }

    #[test]
    fn reset_clears() {
        let mut store = FlowStore::new();
        store.insert(exec("e1"));
        store.reset();
        assert!(store.get("e1").is_none());
    }

    #[test]
    fn a_login_ids_risk_score_beats_the_default() {
        let mut store = FlowStore::new();
        assert_eq!(store.risk_score(&["a@x.test".into()]), 0.0);

        store.set_risk_score(None, 0.2);
        store.set_risk_score(Some("A@x.test"), 0.9);

        assert_eq!(store.risk_score(&["a@x.test".into()]), 0.9);
        assert_eq!(store.risk_score(&["b@x.test".into()]), 0.2);
        store.clear_risk_scores();
        assert_eq!(store.risk_score(&["a@x.test".into()]), 0.0);
    }

    #[test]
    fn reset_keeps_runs_of_exported_flows() {
        let mut store = FlowStore::new();
        store.insert_run(FlowRun {
            execution_id: "f|#|1".into(),
            flow_id: "f".into(),
            task_id: "0".into(),
            context: serde_json::json!({}),
            status: FlowStatus::Waiting,
            created_at: now_secs(),
        });
        store.reset();
        assert!(store.run("f|#|1").is_some());
    }

    #[test]
    fn sweep_drops_expired() {
        let mut store = FlowStore::new();
        let mut old = exec("old");
        old.created_at = 0;
        store.insert(old);
        store.insert(exec("fresh"));
        store.sweep_expired(FLOW_TTL_SECS + 1);
        assert!(store.get("old").is_none());
        assert!(store.get("fresh").is_some());
    }
}
