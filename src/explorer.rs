use crate::registry::{self, StoreKind};
use crate::store::{self, Error, Note, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoreSource {
    pub kind: StoreKind,
    pub path: PathBuf,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveFilter {
    Active,
    Archived,
    #[default]
    All,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFilter {
    #[serde(default)]
    pub search: String,
    #[serde(default)]
    pub memory_types: Vec<String>,
    #[serde(default)]
    pub statuses: Vec<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub archive: ArchiveFilter,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySummary {
    pub store_id: String,
    pub store_kind: StoreKind,
    pub store_path: String,
    pub slug: String,
    pub archived: bool,
    pub name: String,
    pub description: String,
    pub memory_type: String,
    pub scope: String,
    pub status: String,
    pub updated: String,
    pub revision: u64,
    pub tags: Vec<String>,
    pub etag: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryIssue {
    pub store_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct MemoryIndex {
    pub notes: Vec<MemorySummary>,
    pub issues: Vec<MemoryIssue>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryDetail {
    pub summary: MemorySummary,
    pub body: String,
    pub content: String,
}

pub fn registry_sources(registry_path: &Path, global_root: &Path) -> Result<Vec<StoreSource>> {
    let registry = registry::load_registry(registry_path)?;
    let mut sources = Vec::with_capacity(registry.projects.len() + 1);
    sources.push(StoreSource {
        kind: StoreKind::Global,
        path: store::expand_path(global_root),
    });
    sources.extend(registry.projects.into_iter().map(|path| StoreSource {
        kind: StoreKind::Project,
        path: PathBuf::from(path),
    }));
    Ok(sources)
}

fn tags(note: &Note) -> Vec<String> {
    let mut tags: Vec<_> = note
        .fields
        .get("tags")
        .into_iter()
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_lowercase)
        .collect();
    tags.sort();
    tags.dedup();
    tags
}

fn summary(
    source: &StoreSource,
    store_id: &str,
    slug: String,
    note: &Note,
    archived: bool,
) -> MemorySummary {
    let scope = note.fields.get("scope").cloned().unwrap_or_else(|| {
        if source.kind == StoreKind::Global {
            "global".into()
        } else {
            "repo".into()
        }
    });
    MemorySummary {
        store_id: store_id.into(),
        store_kind: source.kind,
        store_path: source.path.to_string_lossy().into_owned(),
        slug,
        archived,
        name: note.fields["name"].clone(),
        description: note.fields["description"].clone(),
        memory_type: note.fields["type"].clone(),
        scope,
        status: if archived {
            "archived".into()
        } else {
            note.fields
                .get("status")
                .cloned()
                .unwrap_or_else(|| "active".into())
        },
        updated: note.fields["updated"].clone(),
        revision: note
            .fields
            .get("revision")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0),
        tags: tags(note),
        etag: store::sha(note.text.as_bytes()),
    }
}

pub fn filter_memories(notes: &[MemorySummary], filter: &MemoryFilter) -> Vec<MemorySummary> {
    let search = filter.search.trim().to_lowercase();
    notes
        .iter()
        .filter(|note| match filter.archive {
            ArchiveFilter::Active => !note.archived,
            ArchiveFilter::Archived => note.archived,
            ArchiveFilter::All => true,
        })
        .filter(|note| {
            filter.memory_types.is_empty()
                || filter
                    .memory_types
                    .iter()
                    .any(|value| value == &note.memory_type)
        })
        .filter(|note| {
            filter.statuses.is_empty() || filter.statuses.iter().any(|value| value == &note.status)
        })
        .filter(|note| {
            filter.scopes.is_empty() || filter.scopes.iter().any(|value| value == &note.scope)
        })
        .filter(|note| {
            search.is_empty()
                || [
                    note.name.as_str(),
                    note.description.as_str(),
                    note.slug.as_str(),
                    note.store_id.as_str(),
                ]
                .iter()
                .any(|value| value.to_lowercase().contains(&search))
                || note.tags.iter().any(|tag| tag.contains(&search))
        })
        .cloned()
        .collect()
}

fn scan_directory(
    source: &StoreSource,
    store_id: &str,
    directory: &Path,
    archived: bool,
    index: &mut MemoryIndex,
) -> Result<()> {
    if !directory.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let slug = entry.file_name().to_string_lossy().into_owned();
        if path.extension().and_then(|value| value.to_str()) != Some("md")
            || (!archived && slug.eq_ignore_ascii_case(store::INDEX))
        {
            continue;
        }
        if path.is_symlink() || !store::valid_slug(&slug) {
            index.issues.push(MemoryIssue {
                store_path: source.path.to_string_lossy().into_owned(),
                slug: Some(slug),
                message: "unsafe or invalid note filename".into(),
            });
            continue;
        }
        let parsed = fs::read_to_string(&path)
            .map_err(Error::from)
            .and_then(|text| store::parse_note(&text));
        match parsed {
            Ok(note) => index
                .notes
                .push(summary(source, store_id, slug, &note, archived)),
            Err(error) => index.issues.push(MemoryIssue {
                store_path: source.path.to_string_lossy().into_owned(),
                slug: Some(slug),
                message: error.to_string(),
            }),
        }
    }
    Ok(())
}

pub fn index_memories(sources: &[StoreSource], filter: &MemoryFilter) -> MemoryIndex {
    let mut index = MemoryIndex::default();
    for source in sources {
        let loaded = match store::load_manifest(&source.path) {
            Ok(loaded) => loaded,
            Err(error) => {
                index.issues.push(MemoryIssue {
                    store_path: source.path.to_string_lossy().into_owned(),
                    slug: None,
                    message: error.to_string(),
                });
                continue;
            }
        };
        if let Err(error) = scan_directory(
            source,
            &loaded.manifest.store_id,
            &source.path,
            false,
            &mut index,
        ) {
            index.issues.push(MemoryIssue {
                store_path: source.path.to_string_lossy().into_owned(),
                slug: None,
                message: error.to_string(),
            });
        }
        if let Err(error) = scan_directory(
            source,
            &loaded.manifest.store_id,
            &source.path.join("archive"),
            true,
            &mut index,
        ) {
            index.issues.push(MemoryIssue {
                store_path: source.path.to_string_lossy().into_owned(),
                slug: None,
                message: error.to_string(),
            });
        }
    }
    index.notes = filter_memories(&index.notes, filter);
    index.notes.sort_by(|left, right| {
        let left_kind = if left.store_kind == StoreKind::Global {
            0
        } else {
            1
        };
        let right_kind = if right.store_kind == StoreKind::Global {
            0
        } else {
            1
        };
        (
            left_kind,
            &left.store_id,
            left.archived,
            &left.name,
            &left.slug,
        )
            .cmp(&(
                right_kind,
                &right.store_id,
                right.archived,
                &right.name,
                &right.slug,
            ))
    });
    index
}

pub fn read_memory(source: &StoreSource, slug: &str, archived: bool) -> Result<MemoryDetail> {
    if !store::valid_slug(slug) {
        return Err(Error(format!("unsafe or invalid note filename: {slug}")));
    }
    let loaded = store::load_manifest(&source.path)?;
    let path = if archived {
        source.path.join("archive").join(slug)
    } else {
        source.path.join(slug)
    };
    if !path.is_file() || path.is_symlink() {
        return Err(Error(format!("note not found: {slug}")));
    }
    let content = fs::read_to_string(path)?;
    let note = store::parse_note(&content)?;
    Ok(MemoryDetail {
        summary: summary(
            source,
            &loaded.manifest.store_id,
            slug.into(),
            &note,
            archived,
        ),
        body: note.body,
        content,
    })
}
