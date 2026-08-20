use clap::{Args, Parser, Subcommand, ValueEnum};
use momonogi::{configure, lifecycle, logo, store};
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use store::{Error, Note, Result};
use uuid::Uuid;

#[derive(Parser)]
#[command(
    name = "momo",
    version,
    about = "Concurrent-safe Momonogi shared memory"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init(InitArgs),
    Migrate(MigrateArgs),
    Get(GetArgs),
    List(ListArgs),
    Put(PutArgs),
    Archive(ArchiveArgs),
    Access(AccessArgs),
    Reindex(WriterRoot),
    Doctor(Root),
    Logo,
    Configure(ConfigureArgs),
    Hook(HookArgs),
    Sync(SyncArgs),
}

#[derive(Args)]
struct Root {
    root: PathBuf,
}
#[derive(Args)]
struct WriterRoot {
    root: PathBuf,
    #[arg(long)]
    agent: String,
}

#[derive(Args)]
struct InitArgs {
    root: PathBuf,
    #[arg(long)]
    store_id: String,
    #[arg(long, required = true)]
    writer: Vec<String>,
    #[arg(long)]
    reader: Vec<String>,
    #[arg(long)]
    force: bool,
    #[arg(long = "by", requires = "force")]
    actor: Option<String>,
    #[arg(long, requires = "force")]
    if_match: Option<String>,
}

#[derive(Args)]
struct MigrateArgs {
    root: PathBuf,
    #[arg(long, default_value = "global")]
    store_id: String,
    #[arg(long)]
    agent: String,
    #[arg(long, default_values = ["codex", "claude-code"])]
    writer: Vec<String>,
    #[arg(long, default_values = ["opencode", "openclaw"])]
    reader: Vec<String>,
}

#[derive(Args)]
struct GetArgs {
    root: PathBuf,
    slug: String,
    #[arg(long)]
    content: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum NoteType {
    User,
    Feedback,
    Project,
    Reference,
}
impl NoteType {
    fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Feedback => "feedback",
            Self::Project => "project",
            Self::Reference => "reference",
        }
    }
}

#[derive(Args)]
struct ListArgs {
    #[arg(default_value = store::DEFAULT_GLOBAL_ROOT)]
    root: PathBuf,
    #[arg(long = "type")]
    kinds: Vec<NoteType>,
    #[arg(long)]
    include_archived: bool,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    no_logo: bool,
}

#[derive(Args)]
struct PutArgs {
    root: PathBuf,
    file: PathBuf,
    #[arg(long)]
    agent: String,
    #[arg(long)]
    slug: Option<String>,
    #[arg(long)]
    if_match: Option<String>,
}

#[derive(Args)]
struct ArchiveArgs {
    root: PathBuf,
    slug: String,
    #[arg(long)]
    agent: String,
    #[arg(long)]
    if_match: String,
}

#[derive(Args)]
struct AccessArgs {
    #[command(subcommand)]
    command: AccessCommand,
}

#[derive(Subcommand)]
enum AccessCommand {
    List(AccessListArgs),
    #[command(alias = "set")]
    Grant(AccessGrantArgs),
    Revoke(AccessRevokeArgs),
}

#[derive(Args)]
struct AccessListArgs {
    #[arg(default_value = store::DEFAULT_GLOBAL_ROOT)]
    root: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum AccessRole {
    Writer,
    Reader,
}

#[derive(Args)]
struct AccessGrantArgs {
    root: PathBuf,
    agent: String,
    #[arg(long, value_enum)]
    role: AccessRole,
    #[arg(long = "by")]
    actor: String,
    #[arg(long)]
    if_match: String,
}

#[derive(Args)]
struct AccessRevokeArgs {
    root: PathBuf,
    agent: String,
    #[arg(long = "by")]
    actor: String,
    #[arg(long)]
    if_match: String,
}

#[derive(Args)]
struct ConfigureArgs {
    #[arg(long, value_enum, required = true)]
    host: Vec<configure::Host>,
    #[arg(long)]
    openclaw_workspace: Vec<PathBuf>,
    #[arg(long)]
    codex_project: Vec<PathBuf>,
    #[arg(long, default_value = store::DEFAULT_GLOBAL_ROOT)]
    memory_root: PathBuf,
    #[arg(long, default_value = "explicit", value_parser = ["explicit", "assisted"])]
    hook_mode: String,
    #[arg(long)]
    no_hooks: bool,
}

#[derive(Args)]
struct HookArgs {
    #[arg(long)]
    memory_root: PathBuf,
    #[arg(long, default_value = "explicit", value_parser = ["explicit", "assisted"])]
    mode: String,
}

#[derive(Args)]
struct SyncArgs {
    #[command(subcommand)]
    command: SyncCommand,
}

#[derive(Subcommand)]
enum SyncCommand {
    Status {
        root: PathBuf,
        #[arg(long)]
        session_id: Option<String>,
    },
    Mark {
        root: PathBuf,
        #[arg(long)]
        session_id: String,
    },
}

fn canonical_existing(path: &Path) -> Result<PathBuf> {
    store::expand_path(path)
        .canonicalize()
        .map_err(|error| Error(format!("{}: {error}", path.display())))
}

fn command_init(args: InitArgs) -> Result<()> {
    let root = store::expand_path(args.root);
    fs::create_dir_all(&root)?;
    let root = root.canonicalize()?;
    let _lock = store::lock_store(&root)?;
    let existing = root.join(store::MANIFEST).exists();
    if existing && !args.force {
        return Err(Error(
            "manifest already exists (use --force to replace it)".into(),
        ));
    }
    let previous = if existing {
        let loaded = store::load_manifest(&root)?;
        let actor = args
            .actor
            .as_deref()
            .ok_or_else(|| Error("replacing a manifest requires --by".into()))?;
        store::assert_writer(&loaded.manifest, actor)?;
        let expected = args
            .if_match
            .as_deref()
            .ok_or_else(|| Error("replacing a manifest requires --if-match".into()))?;
        check_manifest_etag(&loaded.etag, expected)?;
        Some(loaded.manifest)
    } else {
        None
    };
    let mut manifest = store::Manifest {
        schema_version: store::SCHEMA_VERSION,
        store_id: args.store_id,
        revision: 1,
        writers: store::unique_sorted(&args.writer),
        readers: store::unique_sorted(&args.reader),
        updated_by: None,
        updated_at: None,
    };
    if let Some(previous) = previous {
        manifest.revision = previous.revision;
        store::touch_manifest(
            &mut manifest,
            args.actor.as_deref().expect("validated replacement actor"),
        );
    }
    store::write_manifest(&root, &manifest)?;
    let (lines, bytes, _) = store::write_index(&root, &store::notes(&root)?)?;
    println!(
        "[momo] initialized {}: {lines} lines / {bytes} bytes",
        root.display()
    );
    Ok(())
}

fn command_migrate(args: MigrateArgs) -> Result<()> {
    let root = canonical_existing(&args.root)?;
    if root.join(store::MANIFEST).exists() {
        store::assert_writer(&store::read_manifest(&root)?, &args.agent)?;
    } else if !args.writer.contains(&args.agent) {
        return Err(Error(
            "migration agent must be one of the configured writers".into(),
        ));
    }
    let _lock = store::lock_store(&root)?;
    if !root.join(store::MANIFEST).exists() {
        let manifest = store::Manifest {
            schema_version: store::SCHEMA_VERSION,
            store_id: args.store_id,
            revision: 1,
            writers: store::unique_sorted(&args.writer),
            readers: store::unique_sorted(&args.reader),
            updated_by: None,
            updated_at: None,
        };
        store::write_manifest(&root, &manifest)?;
    }
    store::assert_writer(&store::read_manifest(&root)?, &args.agent)?;
    let mut notes = store::notes(&root)?;
    let mut changed = 0;
    for (filename, note) in &mut notes {
        let before = note.fields.clone();
        note.fields
            .entry("id".into())
            .or_insert_with(|| format!("mem_{}", Uuid::new_v4().simple()));
        note.fields
            .entry("status".into())
            .or_insert_with(|| "active".into());
        note.fields
            .entry("revision".into())
            .or_insert_with(|| "1".into());
        note.fields
            .entry("created_by".into())
            .or_insert_with(|| args.agent.clone());
        note.fields
            .entry("updated_by".into())
            .or_insert_with(|| args.agent.clone());
        if note.fields != before {
            note.text = store::render_note(note);
            store::atomic_write(&root.join(filename), note.text.as_bytes(), 0o600)?;
            changed += 1;
        }
    }
    let (lines, bytes, index_changed) = store::write_index(&root, &notes)?;
    println!(
        "[momo] migrated {changed} notes; index_changed={index_changed}; {lines} lines / {bytes} bytes"
    );
    Ok(())
}

fn valid_slug(slug: &str) -> bool {
    slug.ends_with(".md")
        && !slug.eq_ignore_ascii_case(store::INDEX)
        && !slug.contains(['/', '\\'])
        && slug
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
        && slug
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || "._-".contains(ch))
}

fn command_get(args: GetArgs) -> Result<()> {
    let root = canonical_existing(&args.root)?;
    let path = root.join(&args.slug);
    if !valid_slug(&args.slug) || !path.is_file() || path.is_symlink() {
        return Err(Error(format!(
            "note not found or unsafe slug: {}",
            args.slug
        )));
    }
    let data = store::read_all(&path)?;
    if args.content {
        std::io::stdout().write_all(&data)?;
    } else {
        println!(
            "{}",
            serde_json::to_string(&json!({"etag": store::sha(&data), "slug": args.slug}))?
        );
    }
    Ok(())
}

fn list_rows(root: &Path, archived: bool) -> Result<Vec<(String, Note, bool)>> {
    let mut rows: Vec<_> = store::notes(root)?
        .into_iter()
        .map(|(slug, note)| (slug, note, false))
        .collect();
    if archived {
        let archive = root.join("archive");
        if archive.is_dir() {
            for entry in fs::read_dir(archive)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) == Some("md") {
                    let slug = entry.file_name().to_string_lossy().into_owned();
                    if path.is_symlink() || !valid_slug(&slug) {
                        return Err(Error(format!(
                            "unsafe or invalid archived note filename: {slug}"
                        )));
                    }
                    rows.push((slug, store::parse_note(&fs::read_to_string(path)?)?, true));
                }
            }
        }
    }
    Ok(rows)
}

fn command_list(args: ListArgs) -> Result<()> {
    let root = canonical_existing(&args.root)?;
    let kinds: Vec<_> = args.kinds.iter().map(|kind| kind.as_str()).collect();
    let mut rows = list_rows(&root, args.include_archived)?;
    if !kinds.is_empty() {
        rows.retain(|(_, note, _)| kinds.contains(&note.fields["type"].as_str()));
    }
    rows.sort_by_key(|(slug, note, archived)| {
        (
            store::TYPES
                .iter()
                .position(|kind| kind == &note.fields["type"])
                .unwrap_or(99),
            *archived,
            note.fields["name"].to_lowercase(),
            slug.clone(),
        )
    });
    if args.json {
        let payload: Vec<_> = rows.iter().map(|(slug, note, archived)| json!({"archived": archived, "description": note.fields["description"], "etag": store::sha(note.text.as_bytes()), "name": note.fields["name"], "revision": note.fields.get("revision").and_then(|value| value.parse::<u64>().ok()).unwrap_or(0), "scope": note.fields.get("scope"), "slug": slug, "status": if *archived { "archived" } else { note.fields.get("status").map(String::as_str).unwrap_or("active") }, "type": note.fields["type"], "updated": note.fields["updated"], "updated_by": note.fields.get("updated_by")})).collect();
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    if !args.no_logo {
        println!("{}\n", logo::render());
    }
    let active = rows.iter().filter(|(_, _, archived)| !archived).count();
    let archived = rows.iter().filter(|(_, _, archived)| *archived).count();
    let suffix = if args.include_archived {
        format!(" + {archived} archived")
    } else {
        String::new()
    };
    println!("[momo] {active} active{suffix} in {}", root.display());
    let mut current = String::new();
    for (slug, note, archived) in rows {
        if note.fields["type"] != current {
            current = note.fields["type"].clone();
            println!("\n[{current}]");
        }
        let mut metadata = vec![note.fields["updated"].clone()];
        if let Some(value) = note.fields.get("revision") {
            metadata.push(format!("r{value}"));
        }
        if let Some(value) = note.fields.get("updated_by") {
            metadata.push(format!("by {value}"));
        }
        if archived {
            metadata.push("archived".into());
        }
        println!(
            "- {slug} | {} — {} ({})",
            note.fields["name"],
            note.fields["description"],
            metadata.join(", ")
        );
    }
    Ok(())
}

fn command_put(args: PutArgs) -> Result<()> {
    let root = canonical_existing(&args.root)?;
    let source = canonical_existing(&args.file)?;
    if source.parent() == Some(&root) || source.starts_with(&root) {
        return Err(Error(
            "source note must be outside the canonical store".into(),
        ));
    }
    let slug = args.slug.unwrap_or_else(|| {
        source
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    });
    if !valid_slug(&slug) {
        return Err(Error(
            "slug must be a lowercase safe .md filename and cannot be memory.md".into(),
        ));
    }
    let mut incoming = store::parse_note(&fs::read_to_string(source)?)?;
    store::assert_writer(&store::read_manifest(&root)?, &args.agent)?;
    let _lock = store::lock_store(&root)?;
    store::assert_writer(&store::read_manifest(&root)?, &args.agent)?;
    let mut notes = store::notes(&root)?;
    if let Some(current) = notes.get(&slug) {
        let actual = store::sha(current.text.as_bytes());
        let Some(expected) = args.if_match else {
            return Err(Error(format!(
                "updating an existing note requires --if-match {actual}"
            )));
        };
        if expected != actual {
            return Err(Error(format!(
                "etag conflict for {slug}: expected {expected}, current {actual}"
            )));
        }
        incoming.fields.insert(
            "id".into(),
            current
                .fields
                .get("id")
                .cloned()
                .unwrap_or_else(|| format!("mem_{}", Uuid::new_v4().simple())),
        );
        incoming
            .fields
            .insert("created".into(), current.fields["created"].clone());
        incoming.fields.insert(
            "created_by".into(),
            current
                .fields
                .get("created_by")
                .cloned()
                .unwrap_or_else(|| args.agent.clone()),
        );
        incoming.fields.insert(
            "revision".into(),
            (current
                .fields
                .get("revision")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0)
                + 1)
            .to_string(),
        );
    } else {
        if args.if_match.is_some() {
            return Err(Error("--if-match supplied for a new note".into()));
        }
        if notes
            .values()
            .any(|note| note.fields["name"].eq_ignore_ascii_case(&incoming.fields["name"]))
        {
            return Err(Error("memory name already exists".into()));
        }
        incoming
            .fields
            .entry("id".into())
            .or_insert_with(|| format!("mem_{}", Uuid::new_v4().simple()));
        incoming
            .fields
            .insert("created_by".into(), args.agent.clone());
        incoming.fields.insert("revision".into(), "1".into());
    }
    incoming.fields.insert("updated".into(), store::today());
    incoming.fields.insert("updated_by".into(), args.agent);
    incoming
        .fields
        .entry("status".into())
        .or_insert_with(|| "active".into());
    incoming.text = store::render_note(&incoming);
    notes.insert(slug.clone(), incoming.clone());
    let candidate = store::index_text(&notes);
    let (lines, bytes) = store::check_index(&candidate)?;
    store::atomic_write(&root.join(&slug), incoming.text.as_bytes(), 0o600)?;
    store::atomic_write(&root.join(store::INDEX), candidate.as_bytes(), 0o644)?;
    println!(
        "{}",
        serde_json::to_string(
            &json!({"etag": store::sha(incoming.text.as_bytes()), "index_bytes": bytes, "index_lines": lines, "revision": incoming.fields["revision"].parse::<u64>().unwrap_or(0), "slug": slug})
        )?
    );
    Ok(())
}

fn command_archive(args: ArchiveArgs) -> Result<()> {
    let root = canonical_existing(&args.root)?;
    store::assert_writer(&store::read_manifest(&root)?, &args.agent)?;
    let _lock = store::lock_store(&root)?;
    store::assert_writer(&store::read_manifest(&root)?, &args.agent)?;
    let mut notes = store::notes(&root)?;
    let mut note = notes
        .remove(&args.slug)
        .ok_or_else(|| Error(format!("note not found: {}", args.slug)))?;
    let actual = store::sha(note.text.as_bytes());
    if args.if_match != actual {
        return Err(Error(format!(
            "etag conflict for {}: current {actual}",
            args.slug
        )));
    }
    let archive = root.join("archive");
    fs::create_dir_all(&archive)?;
    let mut destination = archive.join(&args.slug);
    let mut collision = 0_u32;
    while destination.exists() {
        collision += 1;
        let suffix = if collision == 1 {
            store::today()
        } else {
            format!("{}-{collision}", store::today())
        };
        destination = archive.join(format!("{}-{suffix}.md", args.slug.trim_end_matches(".md"),));
    }
    fs::rename(root.join(&args.slug), &destination)?;
    note.fields.insert("status".into(), "archived".into());
    note.fields.insert("updated".into(), store::today());
    note.fields.insert("updated_by".into(), args.agent);
    note.fields.insert(
        "revision".into(),
        (note
            .fields
            .get("revision")
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0)
            + 1)
        .to_string(),
    );
    note.text = store::render_note(&note);
    store::atomic_write(&destination, note.text.as_bytes(), 0o600)?;
    let (lines, bytes, _) = store::write_index(&root, &notes)?;
    println!(
        "[momo] archived {} -> {}; {lines} lines / {bytes} bytes",
        args.slug,
        destination
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );
    Ok(())
}

fn print_access(loaded: &store::LoadedManifest, changed: bool, json_output: bool) -> Result<()> {
    let payload = json!({
        "changed": changed,
        "etag": loaded.etag,
        "readers": loaded.manifest.readers,
        "revision": loaded.manifest.revision,
        "store_id": loaded.manifest.store_id,
        "updated_at": loaded.manifest.updated_at,
        "updated_by": loaded.manifest.updated_by,
        "writers": loaded.manifest.writers,
    });
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!(
            "[momo] access revision={} etag={} store={}",
            loaded.manifest.revision, loaded.etag, loaded.manifest.store_id
        );
        println!("writers: {}", loaded.manifest.writers.join(", "));
        println!("readers: {}", loaded.manifest.readers.join(", "));
    }
    Ok(())
}

fn check_manifest_etag(actual: &str, expected: &str) -> Result<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(Error(format!(
            "manifest etag conflict: expected {expected}, current {actual}"
        )))
    }
}

fn command_access(args: AccessArgs) -> Result<()> {
    match args.command {
        AccessCommand::List(args) => {
            let root = canonical_existing(&args.root)?;
            let loaded = store::load_manifest(&root)?;
            print_access(&loaded, false, args.json)
        }
        AccessCommand::Grant(args) => {
            if !store::valid_agent_id(&args.agent) {
                return Err(Error(format!("invalid agent id: {:?}", args.agent)));
            }
            let root = canonical_existing(&args.root)?;
            let _lock = store::lock_store(&root)?;
            let mut loaded = store::load_manifest(&root)?;
            store::assert_writer(&loaded.manifest, &args.actor)?;
            check_manifest_etag(&loaded.etag, &args.if_match)?;
            let desired = match args.role {
                AccessRole::Writer => store::AccessRole::Writer,
                AccessRole::Reader => store::AccessRole::Reader,
            };
            if store::manifest_role(&loaded.manifest, &args.agent) == Some(desired) {
                return print_access(&loaded, false, true);
            }
            if desired == store::AccessRole::Reader
                && loaded.manifest.writers.len() == 1
                && loaded.manifest.writers[0] == args.agent
            {
                return Err(Error("cannot downgrade the final writer".into()));
            }
            loaded.manifest.writers.retain(|agent| agent != &args.agent);
            loaded.manifest.readers.retain(|agent| agent != &args.agent);
            match desired {
                store::AccessRole::Writer => loaded.manifest.writers.push(args.agent),
                store::AccessRole::Reader => loaded.manifest.readers.push(args.agent),
            }
            loaded.manifest.writers.sort();
            loaded.manifest.readers.sort();
            store::touch_manifest(&mut loaded.manifest, &args.actor);
            loaded.etag = store::write_manifest(&root, &loaded.manifest)?;
            print_access(&loaded, true, true)
        }
        AccessCommand::Revoke(args) => {
            if !store::valid_agent_id(&args.agent) {
                return Err(Error(format!("invalid agent id: {:?}", args.agent)));
            }
            let root = canonical_existing(&args.root)?;
            let _lock = store::lock_store(&root)?;
            let mut loaded = store::load_manifest(&root)?;
            store::assert_writer(&loaded.manifest, &args.actor)?;
            check_manifest_etag(&loaded.etag, &args.if_match)?;
            let current = store::manifest_role(&loaded.manifest, &args.agent);
            if current.is_none() {
                return print_access(&loaded, false, true);
            }
            if current == Some(store::AccessRole::Writer) && loaded.manifest.writers.len() == 1 {
                return Err(Error("cannot revoke the final writer".into()));
            }
            loaded.manifest.writers.retain(|agent| agent != &args.agent);
            loaded.manifest.readers.retain(|agent| agent != &args.agent);
            store::touch_manifest(&mut loaded.manifest, &args.actor);
            loaded.etag = store::write_manifest(&root, &loaded.manifest)?;
            print_access(&loaded, true, true)
        }
    }
}

fn command_reindex(args: WriterRoot) -> Result<()> {
    let root = canonical_existing(&args.root)?;
    store::assert_writer(&store::read_manifest(&root)?, &args.agent)?;
    let _lock = store::lock_store(&root)?;
    store::assert_writer(&store::read_manifest(&root)?, &args.agent)?;
    let (lines, bytes, changed) = store::write_index(&root, &store::notes(&root)?)?;
    println!("[momo] reindexed changed={changed}; {lines} lines / {bytes} bytes");
    Ok(())
}

fn command_doctor(args: Root) -> Result<()> {
    let root = canonical_existing(&args.root)?;
    let manifest = store::read_manifest(&root)?;
    let notes = store::notes(&root)?;
    let expected = store::index_text(&notes);
    let mut issues = Vec::new();
    if fs::read_to_string(root.join(store::INDEX)).ok().as_deref() != Some(&expected) {
        issues.push("MEMORY.md does not match the canonical notes; run reindex".to_string());
    }
    for duplicate in store::duplicate_names(&notes) {
        issues.push(format!("duplicate memory name: {duplicate}"));
    }
    if manifest.writers.is_empty() {
        issues.push("manifest has no writers".into());
    }
    let (lines, bytes) = store::check_index(&expected)?;
    if issues.is_empty() {
        println!(
            "[momo] doctor: clean — index {lines} lines / {:.1} KB, {} note(s), no broken pointers, orphans, or schema errors.",
            bytes as f64 / 1024.0,
            notes.len()
        );
        Ok(())
    } else {
        for issue in &issues {
            eprintln!("issue: {issue}");
        }
        Err(Error(format!("doctor found {} issue(s)", issues.len())))
    }
}

fn run(command: Command) -> Result<()> {
    match command {
        Command::Init(args) => command_init(args),
        Command::Migrate(args) => command_migrate(args),
        Command::Get(args) => command_get(args),
        Command::List(args) => command_list(args),
        Command::Put(args) => command_put(args),
        Command::Archive(args) => command_archive(args),
        Command::Access(args) => command_access(args),
        Command::Reindex(args) => command_reindex(args),
        Command::Doctor(args) => command_doctor(args),
        Command::Logo => {
            println!("{}", logo::render());
            Ok(())
        }
        Command::Configure(args) => {
            let codex_without_hooks = !args.no_hooks
                && args
                    .host
                    .iter()
                    .any(|host| matches!(host, configure::Host::Codex))
                && args.codex_project.is_empty()
                && store::manifest_role(
                    &store::read_manifest(&canonical_existing(&args.memory_root)?)?,
                    configure::Host::Codex.agent_id(),
                ) == Some(store::AccessRole::Writer);
            for path in configure::apply(
                &args.host,
                &args.openclaw_workspace,
                &args.codex_project,
                &args.memory_root,
                &args.hook_mode,
                !args.no_hooks,
            )? {
                println!("[momo] configured {}", path.display());
            }
            if codex_without_hooks {
                eprintln!(
                    "[momo] Codex rules configured; lifecycle hooks are project-scoped, so pass --codex-project PATH for each project that should run them"
                );
            }
            Ok(())
        }
        Command::Hook(args) => {
            let mut input = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)?;
            println!(
                "{}",
                serde_json::to_string(&lifecycle::handle(&args.memory_root, &args.mode, &input)?)?
            );
            Ok(())
        }
        Command::Sync(args) => match args.command {
            SyncCommand::Status { root, session_id } => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&lifecycle::status(
                        &root,
                        session_id.as_deref()
                    )?)?
                );
                Ok(())
            }
            SyncCommand::Mark { root, session_id } => {
                lifecycle::mark(&root, &session_id)?;
                println!("[momo] session {session_id} marked synced");
                Ok(())
            }
        },
    }
}

fn main() -> ExitCode {
    match run(Cli::parse().command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("momo: error: {error}");
            ExitCode::from(2)
        }
    }
}
