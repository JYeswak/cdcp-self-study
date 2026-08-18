//! Track-stamp validation.
//!
//! The stamp is deliberately data-shaped: a manifest names a track's bank,
//! objectives, citation-only corpus, integrity pins, honesty row, rights row,
//! and learner surfaces.  This module does not know any track's topics.  It checks
//! the contract that a future track can instantiate with its own data.
#![forbid(unsafe_code)]

use cdcp_core::{canonical_json, sha256_hex};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{load_toml, CheckError, Violation};

const STAMP_ID: &str = "track-stamp-v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrackManifest {
    stamp: String,
    schema_version: u32,
    id: String,
    title: String,
    item_count_floor: usize,
    declared_item_count: usize,
    bank_dir: String,
    objectives: String,
    corpus: String,
    notes: String,
    goldens: String,
    overlap_bank_dir: String,
    learner_page: String,
    learner_data: String,
    hub: String,
    learner_href: String,
    map: String,
    map_marker: String,
    scope_source_ids: Vec<String>,
    forbidden_language: Vec<String>,
    honesty: TrackHonesty,
    rights: TrackRights,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrackHonesty {
    banner: String,
    credential_claim: String,
    claim_marker: String,
    study_signal: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrackRights {
    status: String,
    source_class: String,
    capture: String,
    redistribution: String,
    ai_ingestion: String,
    reviewed_at: String,
    review_note: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectiveFile {
    schema_version: u32,
    objective: Vec<TrackObjective>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrackObjective {
    id: String,
    topic: String,
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusFile {
    schema_version: u32,
    source: Vec<TrackSource>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrackSource {
    id: String,
    title: String,
    publisher: String,
    url: String,
    rights: String,
    capture: String,
    redistribution: String,
    ai_ingestion: String,
    review_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrackItem {
    id: String,
    stem: String,
    choices: Vec<String>,
    correct: String,
    explanation: String,
    item_class: String,
    objective_ids: Vec<String>,
    source_ids: Vec<String>,
    work: String,
    source_class: String,
    status: String,
}

#[derive(Debug, Clone)]
pub struct TrackCheck {
    pub violations: Vec<Violation>,
    pub receipt: String,
}

#[derive(Debug, Clone)]
struct LoadedItem {
    item: TrackItem,
}

/// Discover one manifest per immediate child of `tracks/`.
pub fn check_discovered(root: &Path) -> Result<(Vec<Violation>, Vec<String>), CheckError> {
    let tracks = root.join("tracks");
    if !tracks.is_dir() {
        let violation = Violation::new(
            "track_directory_missing",
            format!(
                "{}: tracks/ is mandatory for the track-stamp gate; track discovery was not run",
                tracks.display()
            ),
        );
        let receipt = format!(
            "track-stamp: ERROR not-run — mandatory tracks/ directory absent: {}",
            tracks.display()
        );
        return Ok((vec![violation], vec![receipt]));
    }

    let mut manifests = Vec::new();
    for entry in fs::read_dir(&tracks)
        .map_err(|e| CheckError::Msg(format!("read {}: {e}", tracks.display())))?
    {
        let entry = entry.map_err(|e| CheckError::Msg(format!("read track entry: {e}")))?;
        let path = entry.path();
        if path.is_dir() && path.join("manifest.toml").is_file() {
            manifests.push(path.join("manifest.toml"));
        }
    }
    manifests.sort();

    let mut violations = Vec::new();
    let mut receipts = Vec::new();
    if manifests.is_empty() {
        violations.push(Violation::new(
            "track_no_manifests",
            "tracks/ exists but has no immediate child manifest.toml (empty track set = ERROR)",
        ));
        return Ok((violations, receipts));
    }

    for manifest in manifests {
        let checked = check_one(root, &manifest)?;
        receipts.push(checked.receipt);
        violations.extend(checked.violations);
    }
    Ok((violations, receipts))
}

/// Check one manifest. This is also the entry point used by the CLI's
/// `--track-check` mode and by the planted self-tests.
pub fn check_one(root: &Path, manifest_path: &Path) -> Result<TrackCheck, CheckError> {
    let manifest: TrackManifest = load_toml(manifest_path)?;
    let mut violations = Vec::new();

    if manifest.stamp != STAMP_ID {
        violations.push(Violation::new(
            "track_stamp_version",
            format!(
                "{}: stamp {:?} != {STAMP_ID}",
                manifest_path.display(),
                manifest.stamp
            ),
        ));
    }
    if manifest.schema_version != 1 {
        violations.push(Violation::new(
            "track_manifest_schema",
            format!(
                "{}: schema_version {} != 1",
                manifest.id, manifest.schema_version
            ),
        ));
    }
    for (label, value) in [
        ("id", &manifest.id),
        ("title", &manifest.title),
        ("map_marker", &manifest.map_marker),
    ] {
        if value.trim().is_empty() {
            violations.push(Violation::new(
                "track_manifest_empty",
                format!("{}: manifest {label} is empty", manifest_path.display()),
            ));
        }
    }
    if manifest.item_count_floor == 0 {
        violations.push(Violation::new(
            "track_floor_empty",
            format!(
                "{}: item_count_floor=0 (empty floor certifies nothing)",
                manifest.id
            ),
        ));
    }
    if manifest.declared_item_count == 0 {
        violations.push(Violation::new(
            "track_declared_count_empty",
            format!("{}: declared_item_count=0", manifest.id),
        ));
    }
    if manifest.scope_source_ids.is_empty() {
        violations.push(Violation::new(
            "track_scope_sources_empty",
            format!("{}: scope_source_ids is empty", manifest.id),
        ));
    }
    if manifest.forbidden_language.is_empty() {
        violations.push(Violation::new(
            "track_forbidden_language_empty",
            format!("{}: forbidden_language is empty", manifest.id),
        ));
    }

    validate_honesty(&manifest, &mut violations);
    validate_rights(&manifest, &mut violations);

    let objectives = load_optional::<ObjectiveFile>(
        root,
        &manifest.objectives,
        "track objectives",
        &mut violations,
    )?;
    let corpus =
        load_optional::<CorpusFile>(root, &manifest.corpus, "track corpus", &mut violations)?;
    let loaded_items = load_items(root, &manifest.bank_dir, &mut violations)?;

    let objective_map = objectives
        .as_ref()
        .map(|file| validate_objectives(file, &manifest.id, &mut violations))
        .unwrap_or_default();
    let source_map = corpus
        .as_ref()
        .map(|file| validate_sources(file, &manifest.id, &mut violations))
        .unwrap_or_default();
    for source_id in &manifest.scope_source_ids {
        if !source_map.contains_key(source_id) {
            violations.push(Violation::new(
                "track_scope_source_unresolved",
                format!(
                    "{}: scope_source_ids cites unknown source {source_id}",
                    manifest.id
                ),
            ));
        }
    }

    let mut ids = BTreeSet::new();
    let mut approved_count = 0usize;
    for loaded in &loaded_items {
        let item = &loaded.item;
        if !ids.insert(item.id.clone()) {
            violations.push(Violation::new(
                "track_duplicate_item_id",
                format!("{}: duplicate item id {}", manifest.id, item.id),
            ));
        }
        if !item.id.starts_with(&format!("{}-", manifest.id)) {
            violations.push(Violation::new(
                "track_item_id_scope",
                format!(
                    "{}: item {} is not scoped to track id",
                    manifest.id, item.id
                ),
            ));
        }
        if item.status == "approved" {
            approved_count += 1;
        } else {
            violations.push(Violation::new(
                "track_item_not_approved",
                format!(
                    "{}: {} has status {:?}; track banks require approved rows",
                    manifest.id, item.id, item.status
                ),
            ));
        }
        if item.source_class != "original" {
            violations.push(Violation::new(
                "track_item_not_original",
                format!("{}: {} source_class must be original", manifest.id, item.id),
            ));
        }
        if item.stem.trim().is_empty() {
            violations.push(Violation::new(
                "track_item_empty_stem",
                format!("{}: {} has empty stem", manifest.id, item.id),
            ));
        }
        if item.choices.len() != 4 || item.choices.iter().any(|choice| choice.trim().is_empty()) {
            violations.push(Violation::new(
                "track_item_choices",
                format!(
                    "{}: {} must have four non-empty choices",
                    manifest.id, item.id
                ),
            ));
        }
        if !matches!(item.correct.as_str(), "A" | "B" | "C" | "D") {
            violations.push(Violation::new(
                "track_item_correct",
                format!(
                    "{}: {} correct {:?} is not A-D",
                    manifest.id, item.id, item.correct
                ),
            ));
        }
        if item.explanation.trim().chars().count() < 20 {
            violations.push(Violation::new(
                "track_item_explanation",
                format!(
                    "{}: {} explanation is shorter than 20 characters",
                    manifest.id, item.id
                ),
            ));
        }
        if item.objective_ids.is_empty() {
            violations.push(Violation::new(
                "track_item_no_objective",
                format!("{}: {} has no objective_ids", manifest.id, item.id),
            ));
        }
        for objective_id in &item.objective_ids {
            if !objective_map.contains_key(objective_id) {
                violations.push(Violation::new(
                    "track_item_unresolved_objective",
                    format!(
                        "{}: {} cites unknown objective {objective_id}",
                        manifest.id, item.id
                    ),
                ));
            }
        }
        if item.source_ids.is_empty() {
            violations.push(Violation::new(
                "track_item_no_source",
                format!("{}: {} has no source_ids", manifest.id, item.id),
            ));
        }
        for source_id in &item.source_ids {
            if !source_map.contains_key(source_id) {
                violations.push(Violation::new(
                    "track_item_unresolved_source",
                    format!(
                        "{}: {} cites unknown source {source_id}",
                        manifest.id, item.id
                    ),
                ));
            }
        }
        validate_item_class(item, &manifest.id, &mut violations);
        check_forbidden_language(
            &manifest.forbidden_language,
            &format!("{} item {}", manifest.id, item.id),
            &item_text(item),
            &mut violations,
        );
    }

    if loaded_items.is_empty() {
        violations.push(Violation::new(
            "track_bank_empty",
            format!(
                "{}: bank contains zero TOML items (empty bank = ERROR)",
                manifest.id
            ),
        ));
    }
    if approved_count < manifest.item_count_floor {
        violations.push(Violation::new(
            "track_bank_below_floor",
            format!(
                "{}: approved items {} < named floor {}",
                manifest.id, approved_count, manifest.item_count_floor
            ),
        ));
    }
    if loaded_items.len() != manifest.declared_item_count {
        violations.push(Violation::new(
            "track_declared_count_mismatch",
            format!(
                "{}: declared_item_count {} != scanned files {}",
                manifest.id,
                manifest.declared_item_count,
                loaded_items.len()
            ),
        ));
    }

    let items: Vec<TrackItem> = loaded_items.iter().map(|row| row.item.clone()).collect();
    let bank_hash = track_bank_hash(&items)?;
    check_goldens(root, &manifest, &items, &bank_hash, &mut violations)?;
    check_surfaces(root, &manifest, &items, &mut violations)?;
    check_overlap(root, &manifest, &items, &mut violations)?;

    let receipt = format!(
        "track-stamp {}: bank={} approved={} floor={} objectives={} sources={} overlap_id={} overlap_stem={} bank_hash={} learner={}",
        manifest.id,
        loaded_items.len(),
        approved_count,
        manifest.item_count_floor,
        objective_map.len(),
        source_map.len(),
        overlap_count(root, &manifest.overlap_bank_dir, &items, true),
        overlap_count(root, &manifest.overlap_bank_dir, &items, false),
        bank_hash,
        manifest.learner_page
    );

    Ok(TrackCheck {
        violations,
        receipt,
    })
}

fn validate_honesty(manifest: &TrackManifest, violations: &mut Vec<Violation>) {
    let banner = manifest.honesty.banner.trim();
    let lower = banner.to_ascii_lowercase();
    if banner.is_empty()
        || !lower.contains("does not grant")
        || (!lower.contains("credential") && !lower.contains("certif"))
    {
        violations.push(Violation::new(
            "track_honesty_banner",
            format!(
                "{}: honesty.banner must explicitly deny credentials/certification",
                manifest.id
            ),
        ));
    }
    if manifest.honesty.credential_claim != "forbidden" {
        violations.push(Violation::new(
            "track_honesty_claim",
            format!(
                "{}: honesty.credential_claim must be forbidden",
                manifest.id
            ),
        ));
    }
    if manifest.honesty.claim_marker.trim().is_empty()
        || !manifest.honesty.claim_marker.contains("claim:")
    {
        violations.push(Violation::new(
            "track_honesty_marker",
            format!(
                "{}: honesty.claim_marker is empty or not a claim marker",
                manifest.id
            ),
        ));
    }
    if manifest.honesty.study_signal.trim().is_empty()
        || manifest
            .honesty
            .study_signal
            .to_ascii_lowercase()
            .contains("credential")
    {
        violations.push(Violation::new(
            "track_honesty_study_signal",
            format!(
                "{}: honesty.study_signal must be explicit and non-credential",
                manifest.id
            ),
        ));
    }
}

fn validate_rights(manifest: &TrackManifest, violations: &mut Vec<Violation>) {
    let row = &manifest.rights;
    for (label, value) in [
        ("status", &row.status),
        ("source_class", &row.source_class),
        ("capture", &row.capture),
        ("redistribution", &row.redistribution),
        ("ai_ingestion", &row.ai_ingestion),
        ("reviewed_at", &row.reviewed_at),
        ("review_note", &row.review_note),
    ] {
        if value.trim().is_empty()
            || matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "default" | "inherited"
            )
        {
            violations.push(Violation::new(
                "track_rights_empty",
                format!("{}: rights.{label} is empty/default/inherited", manifest.id),
            ));
        }
    }
    if row.capture != "citation-only" {
        violations.push(Violation::new(
            "track_rights_capture",
            format!("{}: rights.capture must be citation-only", manifest.id),
        ));
    }
    if row.source_class != "original" {
        violations.push(Violation::new(
            "track_rights_source_class",
            format!(
                "{}: rights.source_class must state original item prose",
                manifest.id
            ),
        ));
    }
}

fn load_optional<T: for<'de> Deserialize<'de>>(
    root: &Path,
    relative: &str,
    label: &str,
    violations: &mut Vec<Violation>,
) -> Result<Option<T>, CheckError> {
    if relative.trim().is_empty() {
        violations.push(Violation::new(
            "track_path_empty",
            format!("{label} path is empty"),
        ));
        return Ok(None);
    }
    let path = root.join(relative);
    if !path.is_file() {
        violations.push(Violation::new(
            "track_path_missing",
            format!("{label} missing: {}", path.display()),
        ));
        return Ok(None);
    }
    Ok(Some(load_toml(&path)?))
}

fn load_items(
    root: &Path,
    relative: &str,
    violations: &mut Vec<Violation>,
) -> Result<Vec<LoadedItem>, CheckError> {
    if relative.trim().is_empty() {
        violations.push(Violation::new("track_bank_path_empty", "bank_dir is empty"));
        return Ok(Vec::new());
    }
    let dir = root.join(relative);
    if !dir.is_dir() {
        violations.push(Violation::new(
            "track_bank_missing",
            format!("track bank missing: {}", dir.display()),
        ));
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in
        fs::read_dir(&dir).map_err(|e| CheckError::Msg(format!("read {}: {e}", dir.display())))?
    {
        let entry = entry.map_err(|e| CheckError::Msg(format!("read bank entry: {e}")))?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("toml") {
            paths.push(path);
        }
    }
    paths.sort();
    let mut out = Vec::new();
    for path in paths {
        let item: TrackItem = load_toml(&path)?;
        out.push(LoadedItem { item });
    }
    Ok(out)
}

fn validate_objectives(
    file: &ObjectiveFile,
    track_id: &str,
    violations: &mut Vec<Violation>,
) -> BTreeMap<String, ()> {
    let mut out = BTreeMap::new();
    if file.schema_version != 1 {
        violations.push(Violation::new(
            "track_objectives_schema",
            format!(
                "{}: objectives schema_version {} != 1",
                track_id, file.schema_version
            ),
        ));
    }
    if file.objective.is_empty() {
        violations.push(Violation::new(
            "track_objectives_empty",
            format!("{}: objective set is empty", track_id),
        ));
    }
    for objective in &file.objective {
        if objective.id.trim().is_empty()
            || objective.topic.trim().is_empty()
            || objective.text.trim().is_empty()
        {
            violations.push(Violation::new(
                "track_objective_empty",
                format!("{}: objective row has an empty field", track_id),
            ));
        }
        if out.insert(objective.id.clone(), ()).is_some() {
            violations.push(Violation::new(
                "track_duplicate_objective",
                format!("{}: duplicate objective {}", track_id, objective.id),
            ));
        }
    }
    out
}

fn validate_sources(
    file: &CorpusFile,
    track_id: &str,
    violations: &mut Vec<Violation>,
) -> BTreeMap<String, ()> {
    let mut out = BTreeMap::new();
    if file.schema_version != 1 {
        violations.push(Violation::new(
            "track_corpus_schema",
            format!(
                "{}: corpus schema_version {} != 1",
                track_id, file.schema_version
            ),
        ));
    }
    if file.source.is_empty() {
        violations.push(Violation::new(
            "track_corpus_empty",
            format!("{}: citation corpus is empty", track_id),
        ));
    }
    for source in &file.source {
        if source.id.trim().is_empty()
            || source.title.trim().is_empty()
            || source.publisher.trim().is_empty()
            || source.review_note.trim().is_empty()
            || source.rights.trim().is_empty()
            || source.capture.trim().is_empty()
            || source.redistribution.trim().is_empty()
            || source.ai_ingestion.trim().is_empty()
        {
            violations.push(Violation::new(
                "track_source_rights_empty",
                format!(
                    "{}: source {} has an empty metadata/rights field",
                    track_id, source.id
                ),
            ));
        }
        if !source.url.starts_with("https://") {
            violations.push(Violation::new(
                "track_source_url",
                format!(
                    "{}: source {} is not an HTTPS public citation",
                    track_id, source.id
                ),
            ));
        }
        if source.capture != "citation-only" {
            violations.push(Violation::new(
                "track_source_capture",
                format!(
                    "{}: source {} capture must be citation-only",
                    track_id, source.id
                ),
            ));
        }
        if out.insert(source.id.clone(), ()).is_some() {
            violations.push(Violation::new(
                "track_duplicate_source",
                format!("{}: duplicate source {}", track_id, source.id),
            ));
        }
    }
    out
}

fn validate_item_class(item: &TrackItem, track_id: &str, violations: &mut Vec<Violation>) {
    let work = item.work.trim();
    match item.item_class.as_str() {
        "calculation" => {
            let has_digit = work.chars().any(|c| c.is_ascii_digit());
            let has_operator = work.chars().any(|c| "=×*/+-".contains(c));
            if work.is_empty() || !has_digit || !has_operator {
                violations.push(Violation::new(
                    "track_item_calculation_work",
                    format!(
                        "{}: {} calculation item lacks numeric/operator work",
                        track_id, item.id
                    ),
                ));
            }
        }
        "one-line-defect" => {
            let lower = work.to_ascii_lowercase();
            if work.is_empty()
                || !(lower.contains("defect")
                    || lower.contains("reject")
                    || lower.contains("must")
                    || work.contains("→")
                    || work.contains("->"))
            {
                violations.push(Violation::new(
                    "track_item_defect_work",
                    format!(
                        "{}: {} one-line-defect item lacks a defect/rejection cue",
                        track_id, item.id
                    ),
                ));
            }
        }
        other => violations.push(Violation::new(
            "track_item_class",
            format!(
                "{}: {} item_class {:?} is not calculation or one-line-defect",
                track_id, item.id, other
            ),
        )),
    }
}

fn item_text(item: &TrackItem) -> String {
    let mut text = format!(
        "{} {} {} {}",
        item.stem, item.explanation, item.work, item.item_class
    );
    for choice in &item.choices {
        text.push(' ');
        text.push_str(choice);
    }
    text
}

fn check_forbidden_language(
    phrases: &[String],
    context: &str,
    text: &str,
    violations: &mut Vec<Violation>,
) {
    let lower = text.to_ascii_lowercase();
    for phrase in phrases {
        let needle = phrase.trim().to_ascii_lowercase();
        if !needle.is_empty() && lower.contains(&needle) {
            violations.push(Violation::new(
                "track_forbidden_language",
                format!("{context}: forbidden phrase {:?} found", phrase),
            ));
        }
    }
}

fn check_goldens(
    root: &Path,
    manifest: &TrackManifest,
    items: &[TrackItem],
    bank_hash: &str,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let dir = root.join(&manifest.goldens);
    let hash_path = dir.join("bank_hash.txt");
    let ids_path = dir.join("item_ids.txt");
    let expected_hash = match fs::read_to_string(&hash_path) {
        Ok(text) => text.trim().to_string(),
        Err(_) => {
            violations.push(Violation::new(
                "track_golden_missing",
                format!("{}: missing {}", manifest.id, hash_path.display()),
            ));
            String::new()
        }
    };
    if !expected_hash.is_empty() && expected_hash != bank_hash {
        violations.push(Violation::new(
            "track_golden_bank_hash",
            format!(
                "{}: bank_hash golden {} != computed {}",
                manifest.id, expected_hash, bank_hash
            ),
        ));
    }

    let mut ids: Vec<&str> = items.iter().map(|item| item.id.as_str()).collect();
    ids.sort_unstable();
    let expected_ids = ids.join("\n");
    match fs::read_to_string(&ids_path) {
        Ok(text) if text.trim() == expected_ids => {}
        Ok(text) => violations.push(Violation::new(
            "track_golden_item_ids",
            format!(
                "{}: item_ids golden differs (got {} bytes)",
                manifest.id,
                text.len()
            ),
        )),
        Err(_) => violations.push(Violation::new(
            "track_golden_missing",
            format!("{}: missing {}", manifest.id, ids_path.display()),
        )),
    }
    Ok(())
}

fn check_surfaces(
    root: &Path,
    manifest: &TrackManifest,
    items: &[TrackItem],
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let page_path = root.join(&manifest.learner_page);
    let page = read_surface(&page_path, &manifest.id, "learner page", violations);
    if let Some(text) = page {
        if !text.contains(&manifest.honesty.banner) {
            violations.push(Violation::new(
                "track_honesty_not_rendered",
                format!(
                    "{}: learner page does not contain the manifest honesty banner",
                    manifest.id
                ),
            ));
        }
        if !text.contains(&manifest.honesty.claim_marker) {
            violations.push(Violation::new(
                "track_honesty_marker_missing",
                format!(
                    "{}: learner page does not contain honesty claim marker",
                    manifest.id
                ),
            ));
        }
        check_forbidden_language(
            &manifest.forbidden_language,
            &format!("{} learner page", manifest.id),
            &text,
            violations,
        );
    }

    let data_path = root.join(&manifest.learner_data);
    if let Some(text) = read_surface(&data_path, &manifest.id, "learner data", violations) {
        for item in items {
            if !text.contains(&item.id) || !text.contains(&item.stem) {
                violations.push(Violation::new(
                    "track_learner_data_drift",
                    format!(
                        "{}: learner data does not contain item {} and its stem",
                        manifest.id, item.id
                    ),
                ));
            }
        }
    }

    let hub_path = root.join(&manifest.hub);
    if let Some(text) = read_surface(&hub_path, &manifest.id, "hub", violations) {
        if !text.contains(&manifest.learner_href) {
            violations.push(Violation::new(
                "track_not_in_hub",
                format!(
                    "{}: hub {} lacks link {}",
                    manifest.id, manifest.hub, manifest.learner_href
                ),
            ));
        }
    }

    let map_path = root.join(&manifest.map);
    if let Some(text) = read_surface(&map_path, &manifest.id, "map", violations) {
        if !text.contains(&manifest.map_marker) {
            violations.push(Violation::new(
                "track_not_in_map",
                format!(
                    "{}: map {} lacks marker {:?}",
                    manifest.id, manifest.map, manifest.map_marker
                ),
            ));
        }
    }

    let notes_path = root.join(&manifest.notes);
    if let Some(text) = read_surface(&notes_path, &manifest.id, "track notes", violations) {
        if text.trim().is_empty() {
            violations.push(Violation::new(
                "track_notes_empty",
                format!("{}: notes file is empty", manifest.id),
            ));
        }
        check_forbidden_language(
            &manifest.forbidden_language,
            &format!("{} track notes", manifest.id),
            &text,
            violations,
        );
    }
    Ok(())
}

fn read_surface(
    path: &Path,
    track_id: &str,
    label: &str,
    violations: &mut Vec<Violation>,
) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(text) => Some(text),
        Err(_) => {
            violations.push(Violation::new(
                "track_surface_missing",
                format!("{}: {label} missing: {}", track_id, path.display()),
            ));
            None
        }
    }
}

fn normalise_stem(stem: &str) -> String {
    stem.split_whitespace()
        .map(|word| word.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

fn stem_hash(stem: &str) -> String {
    sha256_hex(normalise_stem(stem).as_bytes())
}

fn load_reference_bank(
    root: &Path,
    relative: &str,
) -> Result<(BTreeSet<String>, BTreeSet<String>), CheckError> {
    let dir = root.join(relative);
    let mut ids = BTreeSet::new();
    let mut stems = BTreeSet::new();
    if !dir.is_dir() {
        return Ok((ids, stems));
    }
    for entry in fs::read_dir(&dir)
        .map_err(|e| CheckError::Msg(format!("read reference bank {}: {e}", dir.display())))?
    {
        let path = entry
            .map_err(|e| CheckError::Msg(format!("read reference bank entry: {e}")))?
            .path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|e| CheckError::Msg(format!("read {}: {e}", path.display())))?;
        let value: toml::Value = toml::from_str(&text)
            .map_err(|e| CheckError::Msg(format!("parse {}: {e}", path.display())))?;
        if let Some(id) = value.get("id").and_then(toml::Value::as_str) {
            ids.insert(id.to_string());
        }
        if let Some(stem) = value.get("stem").and_then(toml::Value::as_str) {
            stems.insert(stem_hash(stem));
        }
    }
    Ok((ids, stems))
}

fn overlap_count(root: &Path, relative: &str, items: &[TrackItem], ids: bool) -> usize {
    let Ok((reference_ids, reference_stems)) = load_reference_bank(root, relative) else {
        return 0;
    };
    if ids {
        items
            .iter()
            .filter(|item| reference_ids.contains(&item.id))
            .count()
    } else {
        items
            .iter()
            .filter(|item| reference_stems.contains(&stem_hash(&item.stem)))
            .count()
    }
}

fn check_overlap(
    root: &Path,
    manifest: &TrackManifest,
    items: &[TrackItem],
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let reference_dir = root.join(&manifest.overlap_bank_dir);
    if !reference_dir.is_dir() {
        violations.push(Violation::new(
            "track_overlap_bank_missing",
            format!(
                "{}: overlap bank {} is missing",
                manifest.id, manifest.overlap_bank_dir
            ),
        ));
        return Ok(());
    }
    let (reference_ids, reference_stems) = load_reference_bank(root, &manifest.overlap_bank_dir)?;
    let ids: Vec<String> = items
        .iter()
        .filter(|item| reference_ids.contains(&item.id))
        .map(|item| item.id.clone())
        .collect();
    let stems: Vec<String> = items
        .iter()
        .filter(|item| reference_stems.contains(&stem_hash(&item.stem)))
        .map(|item| item.id.clone())
        .collect();
    if !ids.is_empty() || !stems.is_empty() {
        violations.push(Violation::new(
            "track_overlap",
            format!(
                "{}: overlap by id={ids:?}, normalized stem hash={stems:?}",
                manifest.id
            ),
        ));
    }
    Ok(())
}

fn track_bank_hash(items: &[TrackItem]) -> Result<String, CheckError> {
    let mut sorted = items.to_vec();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    let bytes = canonical_json(&sorted)
        .map_err(|e| CheckError::Msg(format!("track bank canonical JSON: {e}")))?;
    Ok(sha256_hex(&bytes))
}

/// Run the planted known-bads that make the track stamp falsifiable.
pub fn run_selftest() -> Result<(), CheckError> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| CheckError::Msg(format!("track selftest clock: {e}")))?
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "cdcp-track-selftest-{}-{nonce}",
        std::process::id()
    ));
    let result = run_selftest_at(&root);
    let _ = fs::remove_dir_all(&root);
    result
}

fn run_selftest_at(root: &Path) -> Result<(), CheckError> {
    let track = root.join("tracks/fixture");
    fs::create_dir_all(track.join("bank"))
        .map_err(|e| CheckError::Msg(format!("track selftest mkdir bank: {e}")))?;
    fs::create_dir_all(track.join("corpus"))
        .map_err(|e| CheckError::Msg(format!("track selftest mkdir corpus: {e}")))?;
    fs::create_dir_all(track.join("goldens"))
        .map_err(|e| CheckError::Msg(format!("track selftest mkdir goldens: {e}")))?;
    fs::create_dir_all(root.join("bank/items"))
        .map_err(|e| CheckError::Msg(format!("track selftest mkdir reference bank: {e}")))?;
    fs::create_dir_all(root.join("web"))
        .map_err(|e| CheckError::Msg(format!("track selftest mkdir web: {e}")))?;
    fs::create_dir_all(root.join("docs"))
        .map_err(|e| CheckError::Msg(format!("track selftest mkdir docs: {e}")))?;

    fs::write(
        root.join("bank/items/reference.toml"),
        "id = \"reference-item\"\nstem = \"A reference-only item\"\n",
    )
    .map_err(|e| CheckError::Msg(format!("track selftest reference item: {e}")))?;
    fs::write(track.join("objectives.toml"), "schema_version = 1\n\n[[objective]]\nid = \"fixture-objective\"\ntopic = \"fixture\"\ntext = \"Compute a fixture quantity.\"\n")
        .map_err(|e| CheckError::Msg(format!("track selftest objectives: {e}")))?;
    fs::write(track.join("corpus/sources.toml"), "schema_version = 1\n\n[[source]]\nid = \"fixture-source\"\ntitle = \"Fixture public citation\"\npublisher = \"Fixture publisher\"\nurl = \"https://example.com/fixture\"\nrights = \"publisher-retains-copyright\"\ncapture = \"citation-only\"\nredistribution = \"not-licensed\"\nai_ingestion = \"metadata-only\"\nreview_note = \"URL metadata only; no body retained.\"\n")
        .map_err(|e| CheckError::Msg(format!("track selftest corpus: {e}")))?;
    fs::write(
        track.join("notes.md"),
        "# Fixture\n\nOriginal study note.\n",
    )
    .map_err(|e| CheckError::Msg(format!("track selftest notes: {e}")))?;
    fs::write(
        root.join("web/track.html"),
        "Fixture study only; this track does not grant any credential. [[claim:track-honesty]]\n",
    )
    .map_err(|e| CheckError::Msg(format!("track selftest page: {e}")))?;
    fs::write(
        root.join("web/track-data.json"),
        "{\"items\":[{\"id\":\"fixture-calculation\",\"stem\":\"A fixture gives 2 units and asks for a total.\"}]}\n",
    )
    .map_err(|e| CheckError::Msg(format!("track selftest learner data: {e}")))?;
    fs::write(root.join("web/index.html"), "track.html\n")
        .map_err(|e| CheckError::Msg(format!("track selftest hub: {e}")))?;
    fs::write(root.join("docs/map.md"), "fixture\n")
        .map_err(|e| CheckError::Msg(format!("track selftest map: {e}")))?;
    fs::write(
        track.join("manifest.toml"),
        "stamp = \"track-stamp-v1\"\nschema_version = 1\nid = \"fixture\"\ntitle = \"Fixture\"\nitem_count_floor = 1\ndeclared_item_count = 1\nbank_dir = \"tracks/fixture/bank\"\nobjectives = \"tracks/fixture/objectives.toml\"\ncorpus = \"tracks/fixture/corpus/sources.toml\"\nnotes = \"tracks/fixture/notes.md\"\ngoldens = \"tracks/fixture/goldens\"\noverlap_bank_dir = \"bank/items\"\nlearner_page = \"web/track.html\"\nlearner_data = \"web/track-data.json\"\nhub = \"web/index.html\"\nlearner_href = \"track.html\"\nmap = \"docs/map.md\"\nmap_marker = \"fixture\"\nscope_source_ids = [\"fixture-source\"]\nforbidden_language = [\"fixture certified\"]\n\n[honesty]\nbanner = \"Fixture study only; this track does not grant any credential.\"\ncredential_claim = \"forbidden\"\nclaim_marker = \"[[claim:track-honesty]]\"\nstudy_signal = \"study-only\"\n\n[rights]\nstatus = \"reviewed\"\nsource_class = \"original\"\ncapture = \"citation-only\"\nredistribution = \"not-licensed\"\nai_ingestion = \"metadata-only\"\nreviewed_at = \"2026-08-17\"\nreview_note = \"Original item prose; citation metadata only; no external body retained.\"\n",
    )
    .map_err(|e| CheckError::Msg(format!("track selftest manifest: {e}")))?;

    write_fixture_item(&track.join("bank/fixture.toml"), "calculation")?;
    let loaded = load_items(root, "tracks/fixture/bank", &mut Vec::new())?;
    let items: Vec<TrackItem> = loaded.iter().map(|row| row.item.clone()).collect();
    fs::write(
        track.join("goldens/bank_hash.txt"),
        format!("{}\n", track_bank_hash(&items)?),
    )
    .map_err(|e| CheckError::Msg(format!("track selftest golden hash: {e}")))?;
    fs::write(track.join("goldens/item_ids.txt"), "fixture-calculation\n")
        .map_err(|e| CheckError::Msg(format!("track selftest golden ids: {e}")))?;

    let manifest = track.join("manifest.toml");
    let good = check_one(root, &manifest)?;
    if !good.violations.is_empty() {
        return Err(CheckError::Msg(format!(
            "known-good fixture unexpectedly RED: {}",
            good.violations
                .iter()
                .map(|v| v.code.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }

    fs::remove_file(track.join("bank/fixture.toml"))
        .map_err(|e| CheckError::Msg(format!("track selftest empty-bank plant: {e}")))?;
    let empty = check_one(root, &manifest)?;
    if !empty
        .violations
        .iter()
        .any(|v| v.code == "track_bank_empty")
        || !empty
            .violations
            .iter()
            .any(|v| v.code == "track_bank_below_floor")
    {
        return Err(CheckError::Msg(
            "empty-bank plant did not reach both bank-empty and floor RED paths".into(),
        ));
    }

    write_fixture_item(&track.join("bank/fixture.toml"), "name-recall")?;
    let recall = check_one(root, &manifest)?;
    if !recall
        .violations
        .iter()
        .any(|v| v.code == "track_item_class")
    {
        return Err(CheckError::Msg(
            "name-recall plant was accepted; item-class gate is vacuous".into(),
        ));
    }

    let absent_root = root.join("absent-tracks-root");
    fs::create_dir_all(&absent_root)
        .map_err(|e| CheckError::Msg(format!("track selftest absent-tracks root: {e}")))?;
    let (absent, absent_receipts) = check_discovered(&absent_root)?;
    if !absent.iter().any(|v| v.code == "track_directory_missing")
        || absent_receipts.len() != 1
        || !absent_receipts[0].contains("ERROR not-run")
    {
        return Err(CheckError::Msg(
            "absent-tracks plant did not produce a distinct not-run receipt and RED violation"
                .into(),
        ));
    }

    println!("INJECTIONS=4 SUITE=track-selftest");
    println!(
        "track-selftest: known-good=GREEN empty-bank=RED[track_bank_empty,track_bank_below_floor] name-recall=RED[track_item_class] absent-tracks=RED[track_directory_missing]"
    );
    Ok(())
}

fn write_fixture_item(path: &Path, item_class: &str) -> Result<(), CheckError> {
    let text = format!(
        "id = \"fixture-calculation\"\nstem = \"A fixture gives 2 units and asks for a total.\"\nchoices = [\"1\", \"2\", \"3\", \"4\"]\ncorrect = \"D\"\nexplanation = \"The worked arithmetic is two plus two, so the selected result is four.\"\nitem_class = \"{item_class}\"\nobjective_ids = [\"fixture-objective\"]\nsource_ids = [\"fixture-source\"]\nwork = \"2 + 2 = 4\"\nsource_class = \"original\"\nstatus = \"approved\"\n"
    );
    fs::write(path, text).map_err(|e| CheckError::Msg(format!("track selftest item: {e}")))
}

/// Run one manifest and print a compact result for human/operator use.
pub fn run_track_check(root: &Path, manifest: &Path) -> Result<(), CheckError> {
    let checked = check_one(root, manifest)?;
    println!("{}", checked.receipt);
    if checked.violations.is_empty() {
        println!("track-check: GREEN");
        Ok(())
    } else {
        for violation in &checked.violations {
            eprintln!("track-check: {} — {}", violation.code, violation.message);
        }
        Err(CheckError::Msg(format!(
            "track-check: RED ({} violation(s))",
            checked.violations.len()
        )))
    }
}

/// Public wrapper used by the binary's planted selftest mode.
pub fn run_track_selftest() -> Result<(), CheckError> {
    run_selftest()
}
