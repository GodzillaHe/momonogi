use crate::store::{self, Error, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const STATE: &str = ".momonogi-state.json";
const MAX_SESSIONS: usize = 128;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct State {
    #[serde(default)]
    sessions: BTreeMap<String, Session>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Session {
    dirty: bool,
    needs_reconcile: bool,
    mode: String,
    last_event: String,
    #[serde(default)]
    updated_at: i64,
}

fn state_path(root: &Path) -> std::path::PathBuf {
    root.join(STATE)
}

fn load(root: &Path) -> Result<State> {
    let path = state_path(root);
    if !path.exists() {
        return Ok(State::default());
    }
    serde_json::from_slice(&fs::read(path)?).map_err(Into::into)
}

fn save(root: &Path, state: &State) -> Result<()> {
    let mut text = serde_json::to_string_pretty(state)?;
    text.push('\n');
    store::atomic_write(&state_path(root), text.as_bytes(), 0o600)
}

fn event_name(event: &Map<String, Value>) -> Result<&str> {
    ["hook_event_name", "hookEventName", "event"]
        .iter()
        .find_map(|key| event.get(*key).and_then(Value::as_str))
        .ok_or_else(|| Error("hook_event_name is missing".into()))
}

fn session_id(event: &Map<String, Value>) -> Result<&str> {
    let value = event
        .get("session_id")
        .and_then(Value::as_str)
        .ok_or_else(|| Error("session_id is missing".into()))?;
    if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        Err(Error("session_id is invalid".into()))
    } else {
        Ok(value)
    }
}

fn allow() -> Value {
    json!({"continue": true, "suppressOutput": true})
}
fn warning(message: String) -> Value {
    json!({"continue": true, "suppressOutput": false, "systemMessage": message})
}

fn prune(state: &mut State, current: &str) {
    if state.sessions.len() <= MAX_SESSIONS {
        return;
    }
    let mut oldest: Vec<_> = state
        .sessions
        .iter()
        .filter(|(id, _)| id.as_str() != current)
        .map(|(id, session)| (session.updated_at, id.clone()))
        .collect();
    oldest.sort();
    for (_, id) in oldest {
        if state.sessions.len() <= MAX_SESSIONS {
            break;
        }
        state.sessions.remove(&id);
    }
}

pub fn handle(root: &Path, mode: &str, input: &str) -> Result<Value> {
    let root = store::expand_path(root).canonicalize()?;
    let event: Map<String, Value> = serde_json::from_str(input)
        .map_err(|error| Error(format!("invalid hook JSON: {error}")))?;
    let name = event_name(&event)?.to_owned();
    let id = session_id(&event)?.to_owned();
    let _lock = store::lock_store(&root)?;
    let mut state = load(&root)?;
    let session = state.sessions.entry(id.clone()).or_insert_with(|| Session {
        dirty: false,
        needs_reconcile: false,
        mode: mode.into(),
        last_event: "SessionStart".into(),
        updated_at: Utc::now().timestamp(),
    });
    session.mode = mode.into();
    session.last_event = name.clone();
    session.updated_at = Utc::now().timestamp();
    let output = match name.as_str() {
        "SessionStart" => {
            let status = if session.needs_reconcile {
                "A previous transition needs reconciliation."
            } else if session.dirty {
                "This session has unsynced durable work."
            } else {
                "No unsynced work is recorded."
            };
            let context = format!(
                "Momonogi continuity protocol. Memory index: {}/MEMORY.md\nRead the index only when continuity or durable preferences are relevant, then open only the needed notes. {status}\nAfter a real continuity sync, run: momo sync mark {} --session-id {}",
                root.display(),
                root.display(),
                id
            );
            json!({"continue": true, "suppressOutput": true, "hookSpecificOutput": {"hookEventName": "SessionStart", "additionalContext": context}})
        }
        "UserPromptSubmit" => {
            if event
                .get("prompt")
                .and_then(Value::as_str)
                .is_some_and(|prompt| {
                    matches!(
                        prompt.split_whitespace().next(),
                        Some("/compact" | "/clear")
                    )
                })
            {
                allow()
            } else {
                session.dirty = true;
                allow()
            }
        }
        "PreCompact" => {
            let trigger = event
                .get("trigger")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_lowercase();
            if trigger == "manual" && (session.dirty || session.needs_reconcile) {
                json!({"continue": false, "stopReason": format!("Momonogi blocked manual compaction because this session has unsynced work. Sync it, run `momo sync mark {} --session-id {}`, then retry.", root.display(), id)})
            } else if trigger != "manual" && (session.dirty || session.needs_reconcile) {
                session.needs_reconcile = true;
                warning("Momonogi: compaction is continuing with unsynced work; reconcile after compaction.".into())
            } else {
                allow()
            }
        }
        _ => warning(format!(
            "Momonogi hook received unsupported event {name:?}; no memory was changed."
        )),
    };
    prune(&mut state, &id);
    save(&root, &state)?;
    Ok(output)
}

pub fn mark(root: &Path, id: &str) -> Result<()> {
    let root = store::expand_path(root).canonicalize()?;
    let _lock = store::lock_store(&root)?;
    let index = fs::read_to_string(root.join(store::INDEX))
        .map_err(|_| Error("MEMORY.md is missing".into()))?;
    store::check_index(&index)?;
    let mut state = load(&root)?;
    let session = state
        .sessions
        .get_mut(id)
        .ok_or_else(|| Error(format!("unknown session id: {id}")))?;
    session.dirty = false;
    session.needs_reconcile = false;
    session.last_event = "mark-synced".into();
    save(&root, &state)
}

pub fn status(root: &Path, id: Option<&str>) -> Result<Value> {
    let root = store::expand_path(root).canonicalize()?;
    let state = load(&root)?;
    if let Some(id) = id {
        Ok(serde_json::to_value(state.sessions.get(id).ok_or_else(
            || Error(format!("unknown session id: {id}")),
        )?)?)
    } else {
        Ok(serde_json::to_value(state)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_state_is_bounded_and_keeps_current() {
        let mut state = State::default();
        for index in 0..=MAX_SESSIONS {
            state.sessions.insert(
                format!("s{index}"),
                Session {
                    dirty: false,
                    needs_reconcile: false,
                    mode: "explicit".into(),
                    last_event: "SessionStart".into(),
                    updated_at: index as i64,
                },
            );
        }
        prune(&mut state, "s0");
        assert_eq!(state.sessions.len(), MAX_SESSIONS);
        assert!(state.sessions.contains_key("s0"));
        assert!(!state.sessions.contains_key("s1"));
    }
}
