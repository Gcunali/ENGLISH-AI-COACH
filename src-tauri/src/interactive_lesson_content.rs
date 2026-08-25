use crate::{interactive_lesson::*, sha256};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

pub const MAX_PACKAGE_BYTES: u64 = 512 * 1024;
const MAX_STAGE_ITEMS: usize = 100;
const MAX_BLOCK_TEXT: usize = 2_000;

#[derive(Clone, Debug, Default)]
pub struct InteractiveLessonContentRegistry {
    lessons: BTreeMap<String, RegisteredLesson>,
    exact_versions: BTreeMap<(String, u32), RegisteredLesson>,
    #[cfg_attr(not(test), allow(dead_code))]
    invalid_packages: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawPackage {
    package_schema_version: u32,
    lesson_flow_version: u32,
    lesson_id: String,
    content_version: u32,
    publication_state: PublicationState,
    title: String,
    description: String,
    language: String,
    reference_locale: String,
    cefr_band: crate::placement::CefrBand,
    estimated_minutes: u32,
    objectives: Vec<String>,
    tags: Vec<String>,
    stages: Vec<RawStage>,
    assets: Vec<LessonAsset>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawStage {
    stage_id: String,
    stage_type: InteractiveStageType,
    stage_schema_version: u32,
    title: String,
    instructions: String,
    required: bool,
    payload: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TheoryPayload {
    blocks: Vec<TheoryBlock>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VocabularyPayload {
    items: Vec<VisualVocabularyItem>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ListeningPayload {
    segments: Vec<ListeningSegment>,
    reveal_text_after_first_play: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RepeatPayload {
    targets: Vec<RepeatTarget>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SpeakingCheckPayload {
    targets: Vec<SpeakingCheckTarget>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExerciseStagePayload {
    items: Vec<crate::interactive_exercise::RawExerciseItem>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GuidedConversationPayload {
    scenario: String,
    student_role: String,
    teacher_role: String,
    goal: String,
    target_vocabulary: Vec<String>,
    target_expressions: Vec<String>,
    minimum_student_turns: u32,
    recommended_student_turns: u32,
    maximum_student_turns: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyPayload {}

impl InteractiveLessonContentRegistry {
    pub fn load(root: PathBuf) -> Self {
        if !root.is_dir() {
            return Self::default();
        }
        let mut candidates: BTreeMap<(String, u32), Vec<RegisteredLesson>> = BTreeMap::new();
        let mut invalid = Vec::new();
        let mut directories = match fs::read_dir(&root) {
            Ok(values) => values.filter_map(Result::ok).collect::<Vec<_>>(),
            Err(_) => return Self::default(),
        };
        directories.sort_by_key(|entry| entry.file_name());
        for entry in directories {
            let path = entry.path();
            let metadata = match fs::symlink_metadata(&path) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                continue;
            }
            match load_package(&path) {
                Ok(lesson) => candidates
                    .entry((
                        lesson.package.lesson_id.clone(),
                        lesson.package.content_version,
                    ))
                    .or_default()
                    .push(lesson),
                Err(error) => invalid.push(format!(
                    "{}: {}",
                    entry.file_name().to_string_lossy(),
                    sanitize_error(&error)
                )),
            }
        }
        let mut lessons = BTreeMap::new();
        let mut exact_versions = BTreeMap::new();
        for ((lesson_id, content_version), versions) in candidates {
            if versions.len() != 1 {
                invalid.push(format!(
                    "{lesson_id}: duplicate lesson id and content version"
                ));
                continue;
            }
            let lesson = versions.into_iter().next().expect("one package");
            exact_versions.insert((lesson_id.clone(), content_version), lesson.clone());
            if !matches!(
                lesson.package.publication_state,
                PublicationState::Published
            ) {
                continue;
            }
            let replace = lessons
                .get(&lesson_id)
                .map(|existing: &RegisteredLesson| {
                    existing.package.content_version < lesson.package.content_version
                })
                .unwrap_or(true);
            if replace {
                lessons.insert(lesson_id, lesson);
            }
        }
        Self {
            lessons,
            exact_versions,
            invalid_packages: invalid,
        }
    }
    pub fn list(&self) -> Vec<RegisteredLesson> {
        self.lessons.values().cloned().collect()
    }
    pub fn get(&self, lesson_id: &str) -> Option<RegisteredLesson> {
        self.lessons.get(lesson_id).cloned()
    }
    pub fn get_exact(&self, lesson_id: &str, content_version: u32) -> Option<RegisteredLesson> {
        self.exact_versions
            .get(&(lesson_id.to_owned(), content_version))
            .cloned()
    }
    pub fn published_count(&self) -> usize {
        self.lessons.len()
    }
    #[cfg(test)]
    pub fn invalid_count(&self) -> usize {
        self.invalid_packages.len()
    }
}

fn load_package(directory: &Path) -> Result<RegisteredLesson, String> {
    let manifest = directory.join("lesson.json");
    let metadata = fs::metadata(&manifest).map_err(|_| "lesson.json is missing".to_owned())?;
    if metadata.len() > MAX_PACKAGE_BYTES {
        return Err("lesson.json exceeds 512 KB".into());
    }
    let raw: RawPackage = serde_json::from_slice(
        &fs::read(&manifest).map_err(|_| "lesson.json could not be read".to_owned())?,
    )
    .map_err(|error| format!("lesson.json is invalid: {error}"))?;
    validate_slug(&raw.lesson_id, "lessonId")?;
    if raw.package_schema_version != INTERACTIVE_LESSON_PACKAGE_SCHEMA_VERSION {
        return Err("unsupported packageSchemaVersion".into());
    }
    if raw.lesson_flow_version != INTERACTIVE_LESSON_FLOW_VERSION {
        return Err("unsupported lessonFlowVersion".into());
    }
    if raw.content_version == 0 {
        return Err("contentVersion must be at least 1".into());
    }
    bounded(&raw.title, 1, 100, "title")?;
    bounded(&raw.description, 1, 500, "description")?;
    if raw.language != "en" || raw.reference_locale != "en-US" {
        return Err("only en / en-US packages are supported".into());
    }
    if raw.estimated_minutes == 0 || raw.estimated_minutes > 480 {
        return Err("estimatedMinutes must be between 1 and 480".into());
    }
    if raw.objectives.len() > 5
        || raw.tags.len() > 8
        || raw.stages.is_empty()
        || raw.stages.len() > 20
    {
        return Err("package collection limit exceeded".into());
    }
    for objective in &raw.objectives {
        bounded(objective, 1, 180, "objective")?;
    }
    for tag in &raw.tags {
        bounded(tag, 1, 50, "tag")?;
    }
    let mut stage_ids = BTreeSet::new();
    let mut stage_types = BTreeSet::new();
    let mut last_order = None;
    let mut stages = Vec::with_capacity(raw.stages.len());
    for stage in raw.stages {
        validate_slug(&stage.stage_id, "stageId")?;
        if !stage_ids.insert(stage.stage_id.clone()) {
            return Err("duplicate stageId".into());
        }
        if !stage_types.insert(stage.stage_type) {
            return Err("a stage type may occur only once".into());
        }
        let order = InteractiveStageType::ORDER
            .iter()
            .position(|kind| *kind == stage.stage_type)
            .expect("known type");
        if last_order.is_some_and(|previous| order <= previous) {
            return Err("stages are not in canonical order".into());
        }
        last_order = Some(order);
        if stage.stage_schema_version != 1 {
            return Err(format!(
                "unsupported stage schema: {} v{}",
                stage.stage_type.as_str(),
                stage.stage_schema_version
            ));
        }
        bounded(&stage.title, 1, 100, "stage title")?;
        bounded(&stage.instructions, 1, 500, "stage instructions")?;
        let payload = parse_payload(stage.stage_type, stage.payload)?;
        if payload.stage_type() != stage.stage_type {
            return Err("stage type and payload do not match".into());
        }
        validate_payload(&payload)?;
        stages.push(InteractiveStage {
            stage_id: stage.stage_id,
            stage_type: stage.stage_type,
            stage_schema_version: stage.stage_schema_version,
            title: stage.title,
            instructions: stage.instructions,
            required: stage.required,
            payload,
        });
    }
    if let Some(index) = stages
        .iter()
        .position(|stage| stage.stage_type == InteractiveStageType::Analysis)
    {
        if index + 1 != stages.len() {
            return Err("analysis must be the last stage".into());
        }
    }
    let mut asset_ids = BTreeSet::new();
    for asset in &raw.assets {
        validate_slug(&asset.asset_id, "assetId")?;
        if !asset_ids.insert(asset.asset_id.clone()) {
            return Err("duplicate assetId".into());
        }
        validate_asset(
            directory,
            asset,
            matches!(raw.publication_state, PublicationState::Published),
        )?;
    }
    let asset_by_id: BTreeMap<_, _> = raw
        .assets
        .iter()
        .map(|asset| (asset.asset_id.as_str(), asset))
        .collect();
    for stage in &stages {
        match &stage.payload {
            StagePayload::VisualVocabulary { items } => {
                for item in items {
                    if let Some(id) = &item.image_asset_id {
                        require_asset(&asset_by_id, id, AssetType::Image)?;
                    }
                }
            }
            StagePayload::Listening { segments, .. } => {
                for segment in segments {
                    if let Some(id) = &segment.audio_asset_id {
                        require_asset(&asset_by_id, id, AssetType::Audio)?;
                    }
                }
            }
            StagePayload::Repeat { targets } => {
                for target in targets {
                    if let Some(id) = &target.reference_audio_asset_id {
                        require_asset(&asset_by_id, id, AssetType::Audio)?;
                    }
                }
            }
            _ => {}
        }
    }
    let asset_files = raw
        .assets
        .iter()
        .map(|asset| (asset.asset_id.clone(), directory.join(&asset.path)))
        .collect();
    let package = InteractiveLessonPackage {
        package_schema_version: raw.package_schema_version,
        lesson_flow_version: raw.lesson_flow_version,
        lesson_id: raw.lesson_id,
        content_version: raw.content_version,
        publication_state: raw.publication_state,
        title: raw.title,
        description: raw.description,
        language: raw.language,
        reference_locale: raw.reference_locale,
        cefr_band: raw.cefr_band,
        estimated_minutes: raw.estimated_minutes,
        objectives: raw.objectives,
        tags: raw.tags,
        stages,
        assets: raw.assets,
    };
    let canonical = serde_json::to_vec(&package).map_err(|error| error.to_string())?;
    Ok(RegisteredLesson {
        package,
        package_hash: sha256::bytes(&canonical),
        asset_files,
    })
}

fn require_asset(
    assets: &BTreeMap<&str, &LessonAsset>,
    id: &str,
    expected: AssetType,
) -> Result<(), String> {
    let asset = assets
        .get(id)
        .ok_or("stage references an undeclared asset")?;
    if std::mem::discriminant(&asset.r#type) != std::mem::discriminant(&expected) {
        return Err("stage references an asset of the wrong type".into());
    }
    Ok(())
}

fn parse_payload(kind: InteractiveStageType, value: Value) -> Result<StagePayload, String> {
    let invalid = |error: serde_json::Error| format!("invalid {} payload: {error}", kind.as_str());
    Ok(match kind {
        InteractiveStageType::Theory => {
            let value: TheoryPayload = serde_json::from_value(value).map_err(invalid)?;
            StagePayload::Theory {
                blocks: value.blocks,
            }
        }
        InteractiveStageType::VisualVocabulary => {
            let value: VocabularyPayload = serde_json::from_value(value).map_err(invalid)?;
            StagePayload::VisualVocabulary { items: value.items }
        }
        InteractiveStageType::Listening => {
            let value: ListeningPayload = serde_json::from_value(value).map_err(invalid)?;
            StagePayload::Listening {
                segments: value.segments,
                reveal_text_after_first_play: value.reveal_text_after_first_play,
            }
        }
        InteractiveStageType::Repeat => {
            let value: RepeatPayload = serde_json::from_value(value).map_err(invalid)?;
            StagePayload::Repeat {
                targets: value.targets,
            }
        }
        InteractiveStageType::SpeakingCheck => {
            let value: SpeakingCheckPayload = serde_json::from_value(value).map_err(invalid)?;
            StagePayload::SpeakingCheck {
                targets: value.targets,
            }
        }
        InteractiveStageType::Exercise => {
            let value: ExerciseStagePayload = serde_json::from_value(value).map_err(invalid)?;
            let items = value
                .items
                .into_iter()
                .map(crate::interactive_exercise::parse_item)
                .collect::<Result<Vec<_>, _>>()?;
            StagePayload::Exercise { items }
        }
        InteractiveStageType::GuidedConversation => {
            let value: GuidedConversationPayload =
                serde_json::from_value(value).map_err(invalid)?;
            StagePayload::GuidedConversation {
                scenario: value.scenario,
                student_role: value.student_role,
                teacher_role: value.teacher_role,
                goal: value.goal,
                target_vocabulary: value.target_vocabulary,
                target_expressions: value.target_expressions,
                minimum_student_turns: value.minimum_student_turns,
                recommended_student_turns: value.recommended_student_turns,
                maximum_student_turns: value.maximum_student_turns,
            }
        }
        InteractiveStageType::Analysis => {
            serde_json::from_value::<EmptyPayload>(value).map_err(invalid)?;
            StagePayload::Analysis {}
        }
    })
}

fn validate_payload(payload: &StagePayload) -> Result<(), String> {
    match payload {
        StagePayload::Theory { blocks } => {
            if blocks.is_empty() || blocks.len() > MAX_STAGE_ITEMS {
                return Err("theory blocks must contain 1..100 items".into());
            }
            for block in blocks {
                match block.r#type {
                    TheoryBlockType::Paragraph => {
                        bounded(
                            block.text.as_deref().unwrap_or(""),
                            1,
                            MAX_BLOCK_TEXT,
                            "paragraph",
                        )?;
                        if block.items.is_some() || block.english.is_some() || block.title.is_some()
                        {
                            return Err("paragraph has incompatible fields".into());
                        }
                    }
                    TheoryBlockType::BulletList => {
                        let items = block.items.as_ref().ok_or("bullet_list requires items")?;
                        if items.is_empty() || items.len() > 50 {
                            return Err("bullet_list requires 1..50 items".into());
                        }
                        for item in items {
                            bounded(item, 1, 500, "bullet")?;
                        }
                        if block.text.is_some() || block.english.is_some() || block.title.is_some()
                        {
                            return Err("bullet_list has incompatible fields".into());
                        }
                    }
                    TheoryBlockType::Example => {
                        bounded(
                            block.english.as_deref().unwrap_or(""),
                            1,
                            MAX_BLOCK_TEXT,
                            "example english",
                        )?;
                        if let Some(value) = &block.explanation {
                            bounded(value, 1, MAX_BLOCK_TEXT, "example explanation")?;
                        }
                        if block.text.is_some() || block.items.is_some() || block.title.is_some() {
                            return Err("example has incompatible fields".into());
                        }
                    }
                    TheoryBlockType::Callout => {
                        bounded(
                            block.text.as_deref().unwrap_or(""),
                            1,
                            MAX_BLOCK_TEXT,
                            "callout",
                        )?;
                        if let Some(value) = &block.title {
                            bounded(value, 1, 100, "callout title")?;
                        }
                        if block.items.is_some()
                            || block.english.is_some()
                            || block.explanation.is_some()
                        {
                            return Err("callout has incompatible fields".into());
                        }
                    }
                }
            }
        }
        StagePayload::VisualVocabulary { items } => {
            if items.is_empty() || items.len() > MAX_STAGE_ITEMS {
                return Err("visual vocabulary requires 1..100 items".into());
            }
            let mut ids = BTreeSet::new();
            for item in items {
                validate_slug(&item.item_id, "itemId")?;
                if !ids.insert(&item.item_id) {
                    return Err("duplicate vocabulary itemId".into());
                }
                bounded(&item.term, 1, 100, "term")?;
                bounded(&item.meaning, 1, 300, "meaning")?;
                bounded(&item.example, 1, 500, "example")?;
            }
        }
        StagePayload::Listening { segments, .. } => {
            if segments.is_empty() || segments.len() > 12 {
                return Err("listening requires 1..12 segments".into());
            }
            let mut ids = BTreeSet::new();
            for segment in segments {
                validate_slug(&segment.segment_id, "segmentId")?;
                if !ids.insert(&segment.segment_id) {
                    return Err("duplicate listening segmentId".into());
                }
                bounded(&segment.text, 1, 240, "listening text")?;
                if segment.text.split_whitespace().count() > 50 {
                    return Err("listening text exceeds 50 words".into());
                }
            }
        }
        StagePayload::Repeat { targets } => validate_pronunciation_targets(
            targets
                .iter()
                .map(|target| (&target.target_id, &target.text, target.hint.as_deref())),
            "repeat",
        )?,
        StagePayload::SpeakingCheck { targets } => {
            if targets.is_empty() || targets.len() > 12 {
                return Err("speaking_check requires 1..12 targets".into());
            }
            let mut ids = BTreeSet::new();
            for target in targets {
                validate_slug(&target.target_id, "targetId")?;
                if !ids.insert(&target.target_id) {
                    return Err("duplicate speaking_check targetId".into());
                }
                bounded(&target.instruction, 1, 300, "speaking instruction")?;
                crate::pronunciation::validate_target(&target.target_text)?;
                if let Some(hint) = &target.hint {
                    bounded(hint, 1, 300, "speaking hint")?;
                }
            }
        }
        StagePayload::Exercise { items } => crate::interactive_exercise::validate_items(items)?,
        StagePayload::GuidedConversation {
            scenario,
            student_role,
            teacher_role,
            goal,
            target_vocabulary,
            target_expressions,
            minimum_student_turns,
            recommended_student_turns,
            maximum_student_turns,
        } => {
            bounded(scenario, 1, 600, "guided scenario")?;
            bounded(student_role, 1, 100, "guided student role")?;
            bounded(teacher_role, 1, 100, "guided teacher role")?;
            bounded(goal, 1, 400, "guided goal")?;
            validate_guided_targets(target_vocabulary, 80, "target vocabulary")?;
            validate_guided_targets(target_expressions, 180, "target expression")?;
            if !(1..=12).contains(minimum_student_turns)
                || minimum_student_turns > recommended_student_turns
                || recommended_student_turns > maximum_student_turns
                || *maximum_student_turns > 20
            {
                return Err(
                    "guided turn range must satisfy 1 <= minimum <= recommended <= maximum <= 20"
                        .into(),
                );
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_guided_targets(values: &[String], max_chars: usize, label: &str) -> Result<(), String> {
    if values.len() > 12 {
        return Err(format!("{label} allows at most 12 items"));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        bounded(value, 1, max_chars, label)?;
        let canonical = value.trim().to_lowercase();
        if !seen.insert(canonical) {
            return Err(format!("duplicate {label}"));
        }
    }
    Ok(())
}

fn validate_pronunciation_targets<'a>(
    targets: impl Iterator<Item = (&'a String, &'a String, Option<&'a str>)>,
    label: &str,
) -> Result<(), String> {
    let values = targets.collect::<Vec<_>>();
    if values.is_empty() || values.len() > 12 {
        return Err(format!("{label} requires 1..12 targets"));
    }
    let mut ids = BTreeSet::new();
    for (id, text, hint) in values {
        validate_slug(id, "targetId")?;
        if !ids.insert(id) {
            return Err(format!("duplicate {label} targetId"));
        }
        crate::pronunciation::validate_target(text)?;
        if let Some(hint) = hint {
            bounded(hint, 1, 300, "target hint")?;
        }
    }
    Ok(())
}

fn validate_asset(root: &Path, asset: &LessonAsset, published: bool) -> Result<(), String> {
    if asset.path.contains('\\') || asset.path.starts_with('/') || asset.path.contains(':') {
        return Err("asset path is not relative and portable".into());
    }
    let path = Path::new(&asset.path);
    let parts: Vec<_> = path.components().collect();
    if parts.len() < 2
        || parts.first() != Some(&Component::Normal("assets".as_ref()))
        || parts
            .iter()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("asset path must stay under assets/".into());
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let allowed = match asset.r#type {
        AssetType::Image => matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "webp"),
        AssetType::Audio => extension == "wav",
    };
    if !allowed {
        return Err("asset extension is not allowed".into());
    }
    let full = root.join(path);
    if !full.is_file() {
        return Err("declared asset is missing".into());
    }
    if fs::symlink_metadata(&full)
        .map_err(|_| "asset metadata unavailable")?
        .file_type()
        .is_symlink()
    {
        return Err("asset symlinks are prohibited".into());
    }
    let canonical_root = root
        .canonicalize()
        .map_err(|_| "package path unavailable")?;
    let canonical = full.canonicalize().map_err(|_| "asset path unavailable")?;
    if !canonical.starts_with(canonical_root) {
        return Err("asset escapes its package".into());
    }
    if published
        && (asset.sha256.len() != 64
            || !asset.sha256.chars().all(|value| value.is_ascii_hexdigit()))
    {
        return Err("published asset requires a SHA-256 hash".into());
    }
    if !asset.sha256.eq_ignore_ascii_case(&sha256::file(&full)?) {
        return Err("asset SHA-256 mismatch".into());
    }
    Ok(())
}

pub(crate) fn validate_slug(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 100
        || value.starts_with('-')
        || value.ends_with('-')
        || !value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        Err(format!("{label} must be a lowercase slug"))
    } else {
        Ok(())
    }
}
fn bounded(value: &str, min: usize, max: usize, label: &str) -> Result<(), String> {
    let size = value.chars().count();
    if size < min || size > max {
        Err(format!("{label} length is outside {min}..{max}"))
    } else {
        Ok(())
    }
}
fn sanitize_error(value: &str) -> String {
    value.replace(['\r', '\n'], " ").chars().take(300).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> Value {
        serde_json::from_slice(
            &fs::read(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("test-fixtures/interactive-lessons/foundation-v1/lesson.json"),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn exercise_fixture() -> Value {
        serde_json::from_slice(
            &fs::read(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("test-fixtures/interactive-lessons-phase-t/deterministic-v1/lesson.json"),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn guided_fixture() -> Value {
        serde_json::from_slice(
            &fs::read(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("test-fixtures/interactive-lessons-phase-u/cafe-v1/lesson.json"),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn root() -> PathBuf {
        let value = std::env::temp_dir().join(format!("guided-content-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&value).unwrap();
        value
    }
    fn write(root: &Path, name: &str, value: &Value) {
        let directory = root.join(name);
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join("lesson.json"),
            serde_json::to_vec_pretty(value).unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn missing_registry_is_an_empty_library() {
        let registry = InteractiveLessonContentRegistry::load(
            std::env::temp_dir().join(uuid::Uuid::new_v4().to_string()),
        );
        assert_eq!(registry.published_count(), 0);
    }
    #[test]
    fn slug_rules_are_stable() {
        assert!(validate_slug("greetings-a1", "id").is_ok());
        assert!(validate_slug("Bad_Id", "id").is_err());
    }
    #[test]
    fn phase_v_all_canonical_stages_are_available() {
        assert!(InteractiveStageType::Listening.runtime_available(1));
        assert!(InteractiveStageType::Repeat.runtime_available(1));
        assert!(InteractiveStageType::SpeakingCheck.runtime_available(1));
        assert!(InteractiveStageType::Exercise.runtime_available(1));
        assert!(InteractiveStageType::GuidedConversation.runtime_available(1));
        assert!(InteractiveStageType::Analysis.runtime_available(1));
        assert!(InteractiveStageType::Theory.runtime_available(1));
    }
    #[test]
    fn isolated_foundation_fixture_is_valid_and_startable() {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-fixtures/interactive-lessons");
        let registry = InteractiveLessonContentRegistry::load(root);
        assert_eq!(registry.published_count(), 1);
        assert_eq!(registry.invalid_count(), 0);
        let lesson = registry.get("everyday-greetings-a1").unwrap();
        assert!(summary(&lesson).startable);
        assert_eq!(lesson.package_hash.len(), 64);
    }
    #[test]
    fn phase_v_full_and_deterministic_fixtures_are_valid_and_startable() {
        for (directory, lesson_id, stage_count) in [
            ("interactive-lessons-phase-v", "phase-v-full-cafe-a1", 8),
            (
                "interactive-lessons-phase-v-deterministic",
                "phase-v-deterministic-a1",
                3,
            ),
        ] {
            let registry = InteractiveLessonContentRegistry::load(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("test-fixtures")
                    .join(directory),
            );
            assert_eq!(registry.invalid_count(), 0);
            let lesson = registry.get(lesson_id).unwrap();
            assert_eq!(lesson.package.stages.len(), stage_count);
            assert!(summary(&lesson).startable);
            assert_eq!(
                lesson.package.stages.last().unwrap().stage_type,
                InteractiveStageType::Analysis
            );
        }
    }
    #[test]
    fn isolated_phase_s_fixture_validates_all_five_available_stage_types() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-fixtures/interactive-lessons-phase-s");
        let registry = InteractiveLessonContentRegistry::load(root);
        assert_eq!(registry.published_count(), 1);
        assert_eq!(registry.invalid_count(), 0);
        let lesson = registry.get("audio-foundation-a1").unwrap();
        assert!(summary(&lesson).startable);
        assert_eq!(lesson.package.stages.len(), 5);
        assert!(matches!(
            &lesson.package.stages[2].payload,
            StagePayload::Listening {
                reveal_text_after_first_play: true,
                ..
            }
        ));
        assert!(
            matches!(&lesson.package.stages[4].payload,StagePayload::SpeakingCheck{targets} if targets.len()==1)
        );
    }
    #[test]
    fn phase_t_fixtures_validate_six_types_and_exercise_is_available() {
        for directory in [
            "test-fixtures/interactive-lessons-phase-t",
            "test-fixtures/interactive-lessons-phase-t-full",
        ] {
            let registry = InteractiveLessonContentRegistry::load(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(directory),
            );
            assert_eq!(registry.invalid_count(), 0);
            assert_eq!(registry.published_count(), 1);
            assert!(summary(&registry.list()[0]).startable);
        }
        assert!(InteractiveStageType::Exercise.runtime_available(1));
        assert!(InteractiveStageType::GuidedConversation.runtime_available(1));
        assert!(InteractiveStageType::Analysis.runtime_available(1));
    }
    #[test]
    fn exercise_packages_reject_unknown_type_duplicate_ids_limits_and_bad_answer_keys() {
        let root = root();
        let cases: [(&str, fn(&mut Value)); 3] = [
            ("unknown", |v: &mut Value| {
                v["stages"][2]["payload"]["items"][0]["exerciseType"] = json!("essay")
            }),
            ("duplicate", |v: &mut Value| {
                v["stages"][2]["payload"]["items"][1]["exerciseId"] = json!("polite-choice")
            }),
            ("bad-key", |v: &mut Value| {
                v["stages"][2]["payload"]["items"][0]["payload"]["correctOptionId"] =
                    json!("missing")
            }),
        ];
        for (name, mutate) in cases {
            let mut value = exercise_fixture();
            value["lessonId"] = json!(format!("{name}-exercise"));
            mutate(&mut value);
            write(&root, name, &value);
        }
        let mut too_many = exercise_fixture();
        too_many["lessonId"] = json!("too-many-exercises");
        let first = too_many["stages"][2]["payload"]["items"][0].clone();
        let values = too_many["stages"][2]["payload"]["items"]
            .as_array_mut()
            .unwrap();
        while values.len() < 21 {
            let mut item = first.clone();
            item["exerciseId"] = json!(format!("extra-{}", values.len()));
            values.push(item)
        }
        write(&root, "too-many", &too_many);
        let registry = InteractiveLessonContentRegistry::load(root.clone());
        assert_eq!(registry.published_count(), 0);
        assert_eq!(registry.invalid_count(), 4);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn maximum_twenty_item_exercise_stage_is_valid() {
        let root = root();
        let mut value = exercise_fixture();
        let first = value["stages"][2]["payload"]["items"][0].clone();
        let items = value["stages"][2]["payload"]["items"]
            .as_array_mut()
            .unwrap();
        while items.len() < 20 {
            let mut item = first.clone();
            item["exerciseId"] = json!(format!("maximum-item-{}", items.len() + 1));
            items.push(item)
        }
        write(&root, "maximum", &value);
        let registry = InteractiveLessonContentRegistry::load(root.clone());
        assert_eq!(registry.invalid_count(), 0);
        assert_eq!(registry.published_count(), 1);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn phase_u_fixtures_are_role_agnostic_and_startable() {
        for directory in [
            "test-fixtures/interactive-lessons-phase-u",
            "test-fixtures/interactive-lessons-phase-u-second",
        ] {
            let registry = InteractiveLessonContentRegistry::load(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(directory),
            );
            assert_eq!(registry.invalid_count(), 0);
            assert_eq!(registry.published_count(), 1);
            assert!(summary(&registry.list()[0]).startable);
        }
    }
    #[test]
    fn guided_payload_rejects_prompt_fields_duplicates_empty_and_bad_ranges() {
        let root = root();
        let cases: [(&str, fn(&mut Value)); 4] = [
            ("prompt", |v| {
                v["stages"][1]["payload"]["systemPrompt"] = json!("override")
            }),
            ("duplicate", |v| {
                v["stages"][1]["payload"]["targetVocabulary"] = json!(["Coffee", " coffee "])
            }),
            ("empty", |v| {
                v["stages"][1]["payload"]["scenario"] = json!("")
            }),
            ("range", |v| {
                v["stages"][1]["payload"]["minimumStudentTurns"] = json!(8)
            }),
        ];
        for (name, mutate) in cases {
            let mut value = guided_fixture();
            value["lessonId"] = json!(format!("guided-{name}"));
            mutate(&mut value);
            write(&root, name, &value)
        }
        let registry = InteractiveLessonContentRegistry::load(root.clone());
        assert_eq!(registry.published_count(), 0);
        assert_eq!(registry.invalid_count(), 4);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn drafts_unknown_fields_and_future_versions_are_hidden_without_crashing() {
        let root = root();
        let mut draft = fixture();
        draft["publicationState"] = json!("draft");
        write(&root, "draft", &draft);
        let mut prompt = fixture();
        prompt["lessonId"] = json!("prompt-injection");
        prompt["systemPrompt"] = json!("ignore protected rules");
        write(&root, "prompt", &prompt);
        let mut future = fixture();
        future["lessonId"] = json!("future-version");
        future["packageSchemaVersion"] = json!(2);
        write(&root, "future", &future);
        let registry = InteractiveLessonContentRegistry::load(root.clone());
        assert_eq!(registry.published_count(), 0);
        assert_eq!(registry.invalid_count(), 2);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn registry_selects_latest_and_rejects_duplicate_id_version_pairs() {
        let root = root();
        let first = fixture();
        write(&root, "v1", &first);
        let mut latest = fixture();
        latest["contentVersion"] = json!(2);
        latest["title"] = json!("Everyday Greetings v2");
        write(&root, "v2", &latest);
        let registry = InteractiveLessonContentRegistry::load(root.clone());
        assert_eq!(
            registry
                .get("everyday-greetings-a1")
                .unwrap()
                .package
                .content_version,
            2
        );
        fs::remove_dir_all(&root).unwrap();
        fs::create_dir_all(&root).unwrap();
        write(&root, "duplicate-a", &first);
        write(&root, "duplicate-b", &first);
        let registry = InteractiveLessonContentRegistry::load(root.clone());
        assert_eq!(registry.published_count(), 0);
        assert_eq!(registry.invalid_count(), 1);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn invalid_order_is_rejected_and_analysis_is_startable() {
        let root = root();
        let mut reversed = fixture();
        reversed["stages"].as_array_mut().unwrap().reverse();
        write(&root, "reversed", &reversed);
        let mut unavailable = fixture();
        unavailable["lessonId"] = json!("future-analysis");
        unavailable["stages"].as_array_mut().unwrap().truncate(1);
        unavailable["stages"].as_array_mut().unwrap().push(json!({"stageId":"analysis","stageType":"analysis","stageSchemaVersion":1,"title":"Analysis","instructions":"Reserved stage.","required":false,"payload":{}}));
        write(&root, "unavailable", &unavailable);
        let registry = InteractiveLessonContentRegistry::load(root.clone());
        assert_eq!(registry.published_count(), 1);
        assert_eq!(registry.invalid_count(), 1);
        assert!(summary(&registry.get("future-analysis").unwrap()).startable);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn analysis_payload_is_strictly_empty() {
        let root = root();
        let mut value = fixture();
        value["lessonId"] = json!("bad-analysis-payload");
        value["stages"].as_array_mut().unwrap().push(json!({
            "stageId":"analysis",
            "stageType":"analysis",
            "stageSchemaVersion":1,
            "title":"Lesson Review",
            "instructions":"Review your practice results.",
            "required":true,
            "payload":{"passingScore":70}
        }));
        write(&root, "bad-analysis", &value);
        let registry = InteractiveLessonContentRegistry::load(root.clone());
        assert_eq!(registry.published_count(), 0);
        assert_eq!(registry.invalid_count(), 1);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn asset_hash_and_containment_are_enforced() {
        let root = root();
        let mut value = fixture();
        value["lessonId"] = json!("asset-check");
        let package = root.join("asset");
        fs::create_dir_all(package.join("assets")).unwrap();
        let bytes = b"not-a-real-png-but-hashable";
        fs::write(package.join("assets/card.png"), bytes).unwrap();
        value["assets"] = json!([{"assetId":"card","type":"image","path":"assets/card.png","sha256":sha256::bytes(bytes)}]);
        fs::write(
            package.join("lesson.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        assert_eq!(
            InteractiveLessonContentRegistry::load(root.clone()).published_count(),
            1
        );
        fs::write(package.join("assets/card.png"), b"tampered").unwrap();
        assert_eq!(
            InteractiveLessonContentRegistry::load(root.clone()).published_count(),
            0
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn package_hash_is_deterministic_and_changes_with_typed_content() {
        let root = root();
        let first = fixture();
        write(&root, "first", &first);
        let hash_a = InteractiveLessonContentRegistry::load(root.clone())
            .get("everyday-greetings-a1")
            .unwrap()
            .package_hash;
        let hash_b = InteractiveLessonContentRegistry::load(root.clone())
            .get("everyday-greetings-a1")
            .unwrap()
            .package_hash;
        assert_eq!(hash_a, hash_b);
        fs::remove_dir_all(&root).unwrap();
        fs::create_dir_all(&root).unwrap();
        let mut changed = first;
        changed["title"] = json!("Changed title");
        write(&root, "changed", &changed);
        let hash_c = InteractiveLessonContentRegistry::load(root.clone())
            .get("everyday-greetings-a1")
            .unwrap()
            .package_hash;
        assert_ne!(hash_a, hash_c);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn remote_absolute_and_traversal_assets_are_rejected() {
        let root = root();
        for (name, path) in [
            ("remote", "https://example.com/card.png"),
            ("absolute", "C:/card.png"),
            ("traversal", "assets/../card.png"),
        ] {
            let mut value = fixture();
            value["lessonId"] = json!(format!("{name}-asset"));
            value["assets"] =
                json!([{"assetId":"card","type":"image","path":path,"sha256":"00".repeat(32)}]);
            write(&root, name, &value);
        }
        let registry = InteractiveLessonContentRegistry::load(root.clone());
        assert_eq!(registry.published_count(), 0);
        assert_eq!(registry.invalid_count(), 3);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn flow_unknown_stage_duplicate_stage_and_invalid_cefr_are_rejected() {
        let root = root();
        let mut flow = fixture();
        flow["lessonId"] = json!("future-flow");
        flow["lessonFlowVersion"] = json!(2);
        write(&root, "flow", &flow);
        let mut unknown = fixture();
        unknown["lessonId"] = json!("unknown-stage");
        unknown["stages"][0]["stageType"] = json!("telepathy");
        write(&root, "unknown", &unknown);
        let mut duplicate = fixture();
        duplicate["lessonId"] = json!("duplicate-stage");
        duplicate["stages"][1]["stageType"] = json!("theory");
        duplicate["stages"][1]["payload"] = duplicate["stages"][0]["payload"].clone();
        write(&root, "duplicate", &duplicate);
        let mut cefr = fixture();
        cefr["lessonId"] = json!("invalid-cefr");
        cefr["cefrBand"] = json!("A0");
        write(&root, "cefr", &cefr);
        let registry = InteractiveLessonContentRegistry::load(root.clone());
        assert_eq!(registry.published_count(), 0);
        assert_eq!(registry.invalid_count(), 4);
        fs::remove_dir_all(root).unwrap();
    }
}
