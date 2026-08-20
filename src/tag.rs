use crate::store::{self, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TagAction {
    Add,
    Remove,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TagCount {
    pub name: String,
    pub count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagMutation {
    pub changed: bool,
    pub slug: String,
    pub tag: String,
    pub tags: Vec<String>,
    pub revision: u64,
    pub etag: String,
    pub index_lines: usize,
    pub index_bytes: usize,
}

pub fn normalize_tag(value: &str) -> Result<String> {
    let mut normalized = String::new();
    for character in value.trim().chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
        } else if character.is_ascii_whitespace() || character == '-' {
            if !normalized.is_empty() && !normalized.ends_with('-') {
                normalized.push('-');
            }
        } else if matches!(character, '.' | '_') {
            normalized.push(character);
        } else {
            return Err(Error(format!(
                "invalid tag {value:?}; use letters, digits, spaces, '.', '_' or '-'"
            )));
        }
    }
    while normalized.ends_with('-') {
        normalized.pop();
    }
    if normalized.is_empty()
        || normalized.len() > 32
        || !normalized.as_bytes()[0].is_ascii_alphanumeric()
    {
        return Err(Error(format!(
            "invalid tag {value:?}; normalized tags must be 1-32 characters and start with a letter or digit"
        )));
    }
    Ok(normalized)
}

pub fn tags_from_fields(fields: &BTreeMap<String, String>) -> Result<Vec<String>> {
    let mut tags = BTreeSet::new();
    if let Some(value) = fields.get("tags") {
        for tag in value.split(',') {
            if !tag.trim().is_empty() {
                tags.insert(normalize_tag(tag)?);
            }
        }
    }
    Ok(tags.into_iter().collect())
}

pub fn list_tags(root: &Path) -> Result<Vec<TagCount>> {
    let mut counts = BTreeMap::<String, usize>::new();
    for note in store::notes(root)?.into_values() {
        for tag in tags_from_fields(&note.fields)? {
            *counts.entry(tag).or_default() += 1;
        }
    }
    Ok(counts
        .into_iter()
        .map(|(name, count)| TagCount { name, count })
        .collect())
}

pub fn change_tag(
    root: &Path,
    slug: &str,
    value: &str,
    action: TagAction,
    actor: &str,
    if_match: &str,
) -> Result<TagMutation> {
    if !store::valid_slug(slug) {
        return Err(Error(format!("unsafe or invalid note filename: {slug}")));
    }
    let tag = normalize_tag(value)?;
    store::assert_writer(&store::read_manifest(root)?, actor)?;
    let _lock = store::lock_store(root)?;
    store::assert_writer(&store::read_manifest(root)?, actor)?;
    let mut notes = store::notes(root)?;
    let note = notes
        .get_mut(slug)
        .ok_or_else(|| Error(format!("note not found: {slug}")))?;
    let actual = store::sha(note.text.as_bytes());
    if actual != if_match {
        return Err(Error(format!(
            "etag conflict for {slug}: expected {if_match}, current {actual}"
        )));
    }

    let mut tags = tags_from_fields(&note.fields)?;
    let changed = match action {
        TagAction::Add if !tags.contains(&tag) => {
            tags.push(tag.clone());
            tags.sort();
            true
        }
        TagAction::Remove if tags.contains(&tag) => {
            tags.retain(|current| current != &tag);
            true
        }
        TagAction::Add | TagAction::Remove => false,
    };
    if !changed {
        let revision = note
            .fields
            .get("revision")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        let index = store::index_text(&notes);
        return Ok(TagMutation {
            changed: false,
            slug: slug.into(),
            tag,
            tags,
            revision,
            etag: actual,
            index_lines: index.lines().count(),
            index_bytes: index.len(),
        });
    }

    if tags.is_empty() {
        note.fields.remove("tags");
    } else {
        note.fields.insert("tags".into(), tags.join(", "));
    }
    let revision = note
        .fields
        .get("revision")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0)
        + 1;
    note.fields.insert("revision".into(), revision.to_string());
    note.fields.insert("updated".into(), store::today());
    note.fields.insert("updated_by".into(), actor.into());
    note.text = store::render_note(note);
    let etag = store::sha(note.text.as_bytes());
    let text = note.text.clone();
    let candidate = store::index_text(&notes);
    let (index_lines, index_bytes) = store::check_index(&candidate)?;
    store::atomic_write(&root.join(slug), text.as_bytes(), 0o600)?;
    store::atomic_write(&root.join(store::INDEX), candidate.as_bytes(), 0o644)?;
    Ok(TagMutation {
        changed: true,
        slug: slug.into(),
        tag,
        tags,
        revision,
        etag,
        index_lines,
        index_bytes,
    })
}
