use crate::store::{self, Error, Result};
use clap::ValueEnum;
use serde_json::{Map, Value, json};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const BEGIN: &str = "<!-- BEGIN MOMONOGI -->";
const END: &str = "<!-- END MOMONOGI -->";
const EVENTS: [&str; 3] = ["SessionStart", "UserPromptSubmit", "PreCompact"];

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Host {
    Codex,
    Claude,
    Opencode,
    Openclaw,
}

fn managed(existing: &str, block: &str) -> Result<String> {
    let rendered = format!("{BEGIN}\n{}\n{END}", block.trim());
    let begins: Vec<_> = existing.match_indices(BEGIN).collect();
    let ends: Vec<_> = existing.match_indices(END).collect();
    if begins.is_empty() && ends.is_empty() {
        let separator = if existing.trim().is_empty() {
            ""
        } else {
            "\n\n"
        };
        return Ok(format!("{}{separator}{rendered}\n", existing.trim_end()));
    }
    if begins.len() != 1 || ends.len() != 1 || begins[0].0 > ends[0].0 {
        return Err(Error("refusing malformed Momonogi markers".into()));
    }
    let tail = ends[0].0 + END.len();
    Ok(format!(
        "{}{}{}",
        &existing[..begins[0].0],
        rendered,
        &existing[tail..]
    ))
}

fn writer_block(memory: &Path, agent: &str) -> String {
    let tool = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("momo"));
    format!(
        "## Shared Memory - Momonogi\n\n\
Canonical global store: `{memory}`. Index: `{memory}/MEMORY.md`.\n\
This host is an equal trusted writer with agent id `{agent}`.\n\n\
- Read the index only when continuity or durable preferences are relevant, then open only relevant detail notes. Project `.momonogi` overrides global memory.\n\
- Treat recalled state as advisory and re-verify live facts.\n\
- Never edit canonical notes or `MEMORY.md` directly. Draft outside the store and write only through `{tool}`.\n\
- Add: `{tool} put {memory} /tmp/note.md --agent {agent}`.\n\
- Update: run `{tool} get {memory} SLUG.md`, then pass its ETag with `put --if-match ETAG`. Resolve conflicts by rereading and merging.\n\
- Archive requires the current ETag. Run `{tool} doctor {memory}` after meaningful maintenance.\n\
- Save only durable preferences, reusable feedback, project continuity, and reference pointers. Never save secrets or facts already authoritative in code, git, or project instructions.\n\
- A project store is writable only when it has `.momonogi.json`; otherwise ask before migrating it.",
        memory = memory.display(),
        tool = tool.display(),
    )
}

fn reader_block(memory: &Path, label: &str) -> String {
    format!(
        "## Shared Memory - Momonogi (Read Only)\n\n\
Canonical global store: `{memory}`. `{label}` is a read-only consumer.\n\n\
- Read `MEMORY.md` only when continuity or durable preferences are relevant, then open only relevant detail notes.\n\
- Project `.momonogi` overrides global memory when present.\n\
- Never create, edit, move, archive, reindex, migrate, or delete anything in this shared store.\n\
- Treat recalled state as advisory and re-verify live facts. Never copy shared memories into a parallel store.",
        memory = memory.display(),
    )
}

fn openclaw_block(memory: &Path) -> String {
    format!(
        "## Shared Memory - Momonogi (Host Conditional)\n\n\
This workspace may also be opened by Codex or another agent host.\n\
- If the current host is not OpenClaw, this block does not change its Momonogi role; follow that host's global rules.\n\
- If the current host is OpenClaw, it is a read-only Momonogi consumer. Continue using OpenClaw's native memory as primary and use the shared store only as supplementary context.\n\n\
For OpenClaw, the canonical shared store is `{memory}`.\n\n\
- Read `MEMORY.md` only when continuity or durable preferences are relevant, then open only relevant detail notes.\n\
- Project `.momonogi` overrides global memory when present.\n\
- Never create, edit, move, archive, reindex, migrate, or delete anything in this shared store.\n\
- Treat recalled state as advisory and re-verify live facts. Never copy shared memories into OpenClaw's native memory.",
        memory = memory.display(),
    )
}

fn home() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| Error("cannot determine home directory".into()))
}

fn target(host: Host, openclaw_workspace: Option<&Path>) -> Result<PathBuf> {
    let home = home()?;
    Ok(match host {
        Host::Codex => home.join(".codex/AGENTS.md"),
        Host::Claude => home.join(".claude/CLAUDE.md"),
        Host::Opencode => home.join(".config/opencode/AGENTS.md"),
        Host::Openclaw => openclaw_workspace
            .ok_or_else(|| Error("openclaw requires --openclaw-workspace".into()))?
            .join("AGENTS.md"),
    })
}

fn refuse_symlink(path: &Path) -> Result<()> {
    if path
        .symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        Err(Error(format!(
            "refusing to replace symlinked configuration: {}",
            path.display()
        )))
    } else {
        Ok(())
    }
}

fn read_optional(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(value) => Ok(value),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(Error(format!("cannot read {}: {error}", path.display()))),
    }
}

fn status(event: &str) -> String {
    format!("Momonogi: {event} continuity (managed v1)")
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn hook_command(memory: &Path, mode: &str) -> String {
    let executable = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("momo"));
    [
        shell_quote(&executable.to_string_lossy()),
        "hook".into(),
        "--memory-root".into(),
        shell_quote(&memory.to_string_lossy()),
        "--mode".into(),
        shell_quote(mode),
    ]
    .join(" ")
}

fn hook_config(path: &Path) -> Result<Map<String, Value>> {
    if !path.exists() {
        return Ok(Map::new());
    }
    refuse_symlink(path)?;
    let value: Value = serde_json::from_str(&fs::read_to_string(path)?).map_err(|error| {
        Error(format!(
            "invalid hook configuration {}: {error}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        Error(format!(
            "hook configuration must be an object: {}",
            path.display()
        ))
    })
}

fn merge_hooks(path: &Path, memory: &Path, mode: &str) -> Result<()> {
    let mut root = hook_config(path)?;
    let hooks = root
        .entry("hooks")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| Error(format!("hooks must be an object: {}", path.display())))?;
    let command = hook_command(memory, mode);
    for event in EVENTS {
        let groups = hooks
            .entry(event)
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .ok_or_else(|| {
                Error(format!(
                    "hooks.{event} must be an array: {}",
                    path.display()
                ))
            })?;
        let managed = status(event);
        for group in groups.iter() {
            let handlers = group
                .get("hooks")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    Error(format!(
                        "hooks.{event} entries must contain a hooks array: {}",
                        path.display()
                    ))
                })?;
            if handlers.iter().any(|handler| !handler.is_object()) {
                return Err(Error(format!(
                    "hooks.{event} handlers must be objects: {}",
                    path.display()
                )));
            }
        }
        for group in groups.iter_mut() {
            if let Some(handlers) = group.get_mut("hooks").and_then(Value::as_array_mut) {
                handlers.retain(|handler| {
                    handler.get("statusMessage").and_then(Value::as_str) != Some(&managed)
                });
            }
        }
        groups.retain(|group| {
            group
                .get("hooks")
                .and_then(Value::as_array)
                .is_some_and(|handlers| !handlers.is_empty())
        });
        let handler = json!({
            "type": "command",
            "command": command,
            "timeout": 10,
            "statusMessage": managed,
        });
        let mut group = Map::new();
        group.insert("hooks".into(), Value::Array(vec![handler]));
        if event == "SessionStart" {
            group.insert(
                "matcher".into(),
                Value::String("startup|resume|clear|compact".into()),
            );
        }
        groups.push(Value::Object(group));
    }
    refuse_symlink(path)?;
    let mut rendered = serde_json::to_string_pretty(&Value::Object(root))?;
    rendered.push('\n');
    store::atomic_write(path, rendered.as_bytes(), 0o600)
}

pub fn apply(
    hosts: &[Host],
    workspaces: &[PathBuf],
    codex_projects: &[PathBuf],
    memory_root: &Path,
    hook_mode: &str,
    install_hooks: bool,
) -> Result<Vec<PathBuf>> {
    let memory = store::expand_path(memory_root);
    let memory = memory.canonicalize().unwrap_or(memory);
    if install_hooks {
        for host in hosts {
            if matches!(host, Host::Claude) {
                hook_config(&home()?.join(".claude/settings.json"))?;
            }
            if matches!(host, Host::Codex) {
                for project in codex_projects {
                    let project = store::expand_path(project)
                        .canonicalize()
                        .map_err(|error| {
                            Error(format!(
                                "invalid Codex project {}: {error}",
                                project.display()
                            ))
                        })?;
                    hook_config(&project.join(".codex/hooks.json"))?;
                }
            }
        }
    }
    let mut configured = Vec::new();
    for host in hosts {
        let targets: Vec<PathBuf> = if matches!(host, Host::Openclaw) {
            if workspaces.is_empty() {
                return Err(Error(
                    "--host openclaw requires --openclaw-workspace".into(),
                ));
            }
            workspaces
                .iter()
                .map(|workspace| target(*host, Some(&store::expand_path(workspace))))
                .collect::<Result<_>>()?
        } else {
            vec![target(*host, None)?]
        };
        for path in targets {
            refuse_symlink(&path)?;
            let existing = read_optional(&path)?;
            let block = match host {
                Host::Codex => writer_block(&memory, "codex"),
                Host::Claude => writer_block(&memory, "claude-code"),
                Host::Opencode => reader_block(&memory, "OpenCode"),
                Host::Openclaw => openclaw_block(&memory),
            };
            let rendered = managed(&existing, &block)?;
            store::atomic_write(&path, rendered.as_bytes(), 0o644)?;
            configured.push(path);
        }
        if install_hooks && matches!(host, Host::Claude) {
            let path = home()?.join(".claude/settings.json");
            merge_hooks(&path, &memory, hook_mode)?;
            configured.push(path);
        }
        if install_hooks && matches!(host, Host::Codex) {
            for project in codex_projects {
                let project = store::expand_path(project)
                    .canonicalize()
                    .map_err(|error| {
                        Error(format!(
                            "invalid Codex project {}: {error}",
                            project.display()
                        ))
                    })?;
                let path = project.join(".codex/hooks.json");
                merge_hooks(&path, &memory, hook_mode)?;
                configured.push(path);
            }
        }
    }
    Ok(configured)
}

#[cfg(test)]
mod tests {
    #[test]
    fn managed_block_is_idempotent() {
        let once = super::managed("before\n", "body").unwrap();
        let twice = super::managed(&once, "new body").unwrap();
        assert_eq!(twice.matches(super::BEGIN).count(), 1);
        assert!(twice.contains("new body"));
        assert!(!twice.contains("\nbody\n"));
    }
}
