use chrono::Local;
use fs2::FileExt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub const SCHEMA_VERSION: u32 = 1;
pub const MANIFEST: &str = ".momonogi.json";
pub const LOCK: &str = ".momonogi.lock";
pub const INDEX: &str = "MEMORY.md";
pub const TYPES: [&str; 4] = ["user", "feedback", "project", "reference"];
pub const DEFAULT_GLOBAL_ROOT: &str = "~/.local/share/momonogi/store";
const DEFAULT_HARD_LINES: usize = 200;
const DEFAULT_HARD_BYTES: usize = 25 * 1024;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self(value.to_string())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub store_id: String,
    pub writers: Vec<String>,
    #[serde(default)]
    pub readers: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Note {
    pub fields: BTreeMap<String, String>,
    pub order: Vec<String>,
    pub body: String,
    pub text: String,
}

pub struct StoreLock {
    file: File,
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

pub fn expand_path(value: impl AsRef<Path>) -> PathBuf {
    let path = value.as_ref();
    let text = path.to_string_lossy();
    if text == "~" || text.starts_with("~/") || text.starts_with("~\\") {
        return match std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
            Some(home) => {
                PathBuf::from(home).join(text[2.min(text.len())..].trim_start_matches(['/', '\\']))
            }
            None => path.to_path_buf(),
        };
    }
    path.to_path_buf()
}

pub fn lock_store(root: &Path) -> Result<StoreLock> {
    fs::create_dir_all(root)?;
    let path = root.join(LOCK);
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|error| {
            Error(format!(
                "cannot open store lock {}: {error}",
                path.display()
            ))
        })?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match file.try_lock_exclusive() {
            Ok(()) => return Ok(StoreLock { file }),
            Err(error) if Instant::now() < deadline => {
                let _ = error;
                thread::sleep(Duration::from_millis(25));
            }
            Err(_) => {
                return Err(Error(format!(
                    "timed out waiting for store lock: {}",
                    path.display()
                )));
            }
        }
    }
}

fn replace_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    #[cfg(not(windows))]
    {
        fs::rename(source, destination)
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
        };
        let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
        let destination: Vec<u16> = destination
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        let result = unsafe {
            MoveFileExW(
                source.as_ptr(),
                destination.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        };
        if result == 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

pub fn atomic_write(path: &Path, data: &[u8], mode: u32) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| Error(format!("path has no parent: {}", path.display())))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".momonogi-{}.tmp", Uuid::new_v4().simple()));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(mode))?;
        }
        let _ = mode;
        file.write_all(data)?;
        file.sync_all()?;
        drop(file);
        replace_file(&temporary, path)?;
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub fn sha(data: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(data.as_ref()))
}

pub fn parse_note(text: &str) -> Result<Note> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.first().is_none_or(|line| line.trim() != "---") {
        return Err(Error("note must start with frontmatter".into()));
    }
    let end = lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, line)| line.trim() == "---")
        .map(|(index, _)| index)
        .ok_or_else(|| Error("frontmatter has no closing fence".into()))?;
    let mut fields = BTreeMap::new();
    let mut order = Vec::new();
    for raw in &lines[1..end] {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, raw_value) = line.split_once(':').ok_or_else(|| {
            Error(format!(
                "malformed frontmatter line: {:?}",
                &line[..line.len().min(80)]
            ))
        })?;
        let key = key.trim().to_owned();
        if fields.contains_key(&key) {
            return Err(Error(format!("duplicate frontmatter key: {key}")));
        }
        let mut value = raw_value.trim().to_owned();
        if value.len() >= 2 {
            let bytes = value.as_bytes();
            if (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
                || (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            {
                value = value[1..value.len() - 1]
                    .replace("\\\"", "\"")
                    .replace("\\\\", "\\");
            }
        }
        order.push(key.clone());
        fields.insert(key, value);
    }
    let mut body = lines[end + 1..]
        .join("\n")
        .trim_start_matches('\n')
        .to_owned();
    if text.ends_with('\n') && !body.is_empty() {
        body.push('\n');
    }
    validate(&fields, &body)?;
    Ok(Note {
        fields,
        order,
        body,
        text: text.to_owned(),
    })
}

fn quote(value: &str) -> String {
    let simple = value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "_./|@+-".contains(ch));
    if simple && !value.is_empty() {
        value.to_owned()
    } else {
        serde_json::to_string(value).expect("string serialization")
    }
}

pub fn render_note(note: &Note) -> String {
    const PREFERRED: [&str; 13] = [
        "id",
        "name",
        "description",
        "type",
        "scope",
        "status",
        "created",
        "updated",
        "revision",
        "created_by",
        "updated_by",
        "source",
        "last_verified",
    ];
    let mut keys = Vec::new();
    for key in PREFERRED
        .iter()
        .map(|value| value.to_string())
        .chain(note.order.iter().cloned())
        .chain(note.fields.keys().cloned())
    {
        if note.fields.get(&key).is_some_and(|value| !value.is_empty()) && !keys.contains(&key) {
            keys.push(key);
        }
    }
    let mut result = String::from("---\n");
    for key in keys {
        result.push_str(&format!("{key}: {}\n", quote(&note.fields[&key])));
    }
    result.push_str("---\n\n");
    result.push_str(&note.body);
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

pub fn validate(fields: &BTreeMap<String, String>, body: &str) -> Result<()> {
    for key in ["name", "description", "type", "created", "updated"] {
        if fields.get(key).is_none_or(String::is_empty) {
            return Err(Error(format!("missing required field: {key}")));
        }
    }
    if !TYPES.contains(&fields["type"].as_str()) {
        return Err(Error(format!("invalid type: {}", fields["type"])));
    }
    if fields
        .get("scope")
        .is_some_and(|scope| !scope.is_empty() && scope != "global" && scope != "repo")
    {
        return Err(Error(format!("invalid scope: {}", fields["scope"])));
    }
    for key in ["name", "description"] {
        if fields[key].contains('\n') {
            return Err(Error(format!("{key} must be one line")));
        }
    }
    if matches!(fields["type"].as_str(), "feedback" | "project") {
        let why =
            Regex::new(r"(?im)^\s*(?:[*#>-]+\s*)?\*{0,2}Why\*{0,2}\s*[:：]").expect("valid regex");
        let how = Regex::new(r"(?im)^\s*(?:[*#>-]+\s*)?\*{0,2}How to apply\*{0,2}\s*[:：]")
            .expect("valid regex");
        if !why.is_match(body) {
            return Err(Error(format!("{} note requires Why:", fields["type"])));
        }
        if !how.is_match(body) {
            return Err(Error(format!(
                "{} note requires How to apply:",
                fields["type"]
            )));
        }
    }
    Ok(())
}

pub fn read_manifest(root: &Path) -> Result<Manifest> {
    let path = root.join(MANIFEST);
    if !path.is_file() {
        return Err(Error(
            "missing store manifest; run init or migrate first".into(),
        ));
    }
    let data = fs::read(&path)?;
    let manifest: Manifest = serde_json::from_slice(&data)
        .map_err(|error| Error(format!("cannot read manifest: {error}")))?;
    if manifest.schema_version != SCHEMA_VERSION {
        return Err(Error("unsupported manifest schema".into()));
    }
    Ok(manifest)
}

pub fn assert_writer(manifest: &Manifest, agent: &str) -> Result<()> {
    if manifest.writers.iter().any(|writer| writer == agent) {
        Ok(())
    } else {
        Err(Error(format!("agent {agent:?} is not a configured writer")))
    }
}

fn valid_slug(value: &str) -> bool {
    let bytes = value.as_bytes();
    value.ends_with(".md")
        && !value.eq_ignore_ascii_case(INDEX)
        && !value.contains(['/', '\\'])
        && bytes
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

pub fn notes(root: &Path) -> Result<BTreeMap<String, Note>> {
    let mut result = BTreeMap::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.extension().and_then(|value| value.to_str()) != Some("md") || name == INDEX {
            continue;
        }
        if path.is_symlink() || !valid_slug(&name) {
            return Err(Error(format!("unsafe or invalid note filename: {name}")));
        }
        let text = fs::read_to_string(&path)?;
        result.insert(name, parse_note(&text)?);
    }
    Ok(result)
}

fn escape_label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

pub fn index_text(notes: &BTreeMap<String, Note>) -> String {
    let mut lines = vec![
        "# Memory Index".into(),
        "".into(),
        "> Pointers only — the actual content lives in the linked files, never here.".into(),
        "> Generated by momonogi; do not edit by hand.".into(),
        "> Soft cap 150 lines / 20 KB. Hard cap 200 lines / 25 KB.".into(),
        "".into(),
    ];
    for kind in TYPES {
        let mut members: Vec<_> = notes
            .iter()
            .filter(|(_, note)| note.fields["type"] == kind)
            .collect();
        members.sort_by_key(|(filename, note)| {
            (note.fields["name"].to_lowercase(), (*filename).clone())
        });
        if members.is_empty() {
            continue;
        }
        lines.push(format!("## {kind}"));
        lines.push(String::new());
        for (filename, note) in members {
            lines.push(format!(
                "- [{}]({filename}) — {}",
                escape_label(&note.fields["name"]),
                note.fields["description"].replace('\n', " ")
            ));
        }
        lines.push(String::new());
    }
    format!("{}\n", lines.join("\n").trim_end())
}

pub fn check_index(text: &str) -> Result<(usize, usize)> {
    let hard_lines = std::env::var("MOMONOGI_HARD")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_HARD_LINES);
    let hard_bytes = std::env::var("MOMONOGI_HARD_BYTES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_HARD_BYTES);
    let lines = text.lines().count();
    let bytes = text.len();
    if lines > hard_lines || bytes > hard_bytes {
        return Err(Error(format!(
            "generated index exceeds hard cap: {lines} lines / {bytes} bytes"
        )));
    }
    Ok((lines, bytes))
}

pub fn write_index(root: &Path, notes: &BTreeMap<String, Note>) -> Result<(usize, usize, bool)> {
    let text = index_text(notes);
    let (lines, bytes) = check_index(&text)?;
    let path = root.join(INDEX);
    let changed = fs::read_to_string(&path).ok().as_deref() != Some(&text);
    if changed {
        atomic_write(&path, text.as_bytes(), 0o644)?;
    }
    Ok((lines, bytes, changed))
}

pub fn today() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

pub fn unique_sorted(values: &[String]) -> Vec<String> {
    values
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn duplicate_names(notes: &BTreeMap<String, Note>) -> Vec<String> {
    let mut seen = HashSet::new();
    notes
        .values()
        .filter_map(|note| {
            let value = note.fields["name"].to_lowercase();
            if seen.insert(value.clone()) {
                None
            } else {
                Some(value)
            }
        })
        .collect()
}

pub fn read_all(path: &Path) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    File::open(path)?.read_to_end(&mut data)?;
    Ok(data)
}
