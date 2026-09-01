use crate::{
    database,
    interactive_lesson::{summary, PublicationState, RegisteredLesson},
    interactive_lesson_content::InteractiveLessonContentRegistry,
    placement::{CefrBand, PlacementConfidence},
    sha256,
    student_profile_repository::StudentProfileRepository,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub const CURRICULUM_SCHEMA_VERSION: u32 = 1;
pub const CURRICULUM_REGISTRY_VERSION: u32 = 1;
pub const CURRICULUM_PROGRESS_VERSION: u32 = 1;
pub const CURRICULUM_RECOMMENDATION_VERSION: u32 = 2;
pub const CURRICULUM_TAXONOMY_VERSION: u32 = 1;
const MAX_CURRICULUM_BYTES: u64 = 1024 * 1024;
const MAX_TOTAL_LESSONS: usize = 500;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillFocus {
    Grammar,
    Vocabulary,
    Listening,
    Pronunciation,
    Speaking,
    Interaction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CurriculumManifest {
    curriculum_schema_version: u32,
    curriculum_id: String,
    curriculum_version: u32,
    publication_state: PublicationState,
    title: String,
    description: String,
    target_language: String,
    reference_locale: String,
    levels: Vec<CurriculumLevel>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CurriculumLevel {
    level_id: String,
    cefr_level: CefrBand,
    title: String,
    description: String,
    objectives: Vec<String>,
    units: Vec<CurriculumUnit>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CurriculumUnit {
    unit_id: String,
    title: String,
    description: String,
    objectives: Vec<String>,
    skill_focus: Vec<SkillFocus>,
    grammar_topics: Vec<String>,
    vocabulary_topics: Vec<String>,
    communicative_functions: Vec<String>,
    lessons: Vec<CurriculumLessonRef>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CurriculumLessonRef {
    lesson_id: String,
    content_version: u32,
}

#[derive(Clone, Debug)]
struct RegisteredCurriculum {
    manifest: CurriculumManifest,
    curriculum_hash: String,
    lessons: BTreeMap<(String, u32), RegisteredLesson>,
}

#[derive(Clone, Debug, Default)]
pub struct CurriculumRegistry {
    published: BTreeMap<String, RegisteredCurriculum>,
    #[cfg_attr(not(test), allow(dead_code))]
    invalid: Vec<String>,
}

impl CurriculumRegistry {
    pub fn load(root: PathBuf, lessons: &InteractiveLessonContentRegistry) -> Self {
        if !root.is_dir() {
            return Self::default();
        }
        let mut entries = match fs::read_dir(root) {
            Ok(values) => values.filter_map(Result::ok).collect::<Vec<_>>(),
            Err(_) => return Self::default(),
        };
        entries.sort_by_key(|entry| entry.file_name());
        let mut candidates: BTreeMap<(String, u32), Vec<RegisteredCurriculum>> = BTreeMap::new();
        let mut invalid = Vec::new();
        for entry in entries {
            let path = entry.path();
            let metadata = match fs::symlink_metadata(&path) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                continue;
            }
            match load_curriculum(&path, lessons) {
                Ok(value) => candidates
                    .entry((
                        value.manifest.curriculum_id.clone(),
                        value.manifest.curriculum_version,
                    ))
                    .or_default()
                    .push(value),
                Err(error) => invalid.push(format!(
                    "{}: {}",
                    entry.file_name().to_string_lossy(),
                    sanitize(&error)
                )),
            }
        }
        let mut published = BTreeMap::new();
        for ((curriculum_id, _), versions) in candidates {
            if versions.len() != 1 {
                invalid.push(format!(
                    "{curriculum_id}: duplicate curriculum id and version"
                ));
                continue;
            }
            let value = versions.into_iter().next().expect("one curriculum");
            if !matches!(
                value.manifest.publication_state,
                PublicationState::Published
            ) {
                continue;
            }
            let replace = published
                .get(&curriculum_id)
                .map(|current: &RegisteredCurriculum| {
                    current.manifest.curriculum_version < value.manifest.curriculum_version
                })
                .unwrap_or(true);
            if replace {
                published.insert(curriculum_id, value);
            }
        }
        Self { published, invalid }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn published_count(&self) -> usize {
        self.published.len()
    }

    #[cfg(test)]
    fn invalid_count(&self) -> usize {
        self.invalid.len()
    }
}

#[derive(Clone)]
pub struct CurriculumService {
    registry: CurriculumRegistry,
    database: PathBuf,
    profiles: StudentProfileRepository,
}

impl CurriculumService {
    pub fn new(
        registry: CurriculumRegistry,
        database: PathBuf,
        profiles: StudentProfileRepository,
    ) -> Self {
        Self {
            registry,
            database,
            profiles,
        }
    }

    pub fn catalog(&self) -> Result<CurriculumCatalogDto, String> {
        let profile = self.profiles.get()?;
        let connection = database::open(&self.database)?;
        let sessions = load_session_aggregates(&connection)?;
        let active_session = sessions.iter().find_map(|(lesson_id, value)| {
            value
                .active_session_id
                .as_ref()
                .map(|session_id| CurriculumActiveSessionDto {
                    session_id: session_id.clone(),
                    lesson_id: lesson_id.clone(),
                    content_version: value.active_content_version.unwrap_or(1),
                })
        });
        let curricula = self
            .registry
            .published
            .values()
            .map(|value| build_course(value, &sessions, &profile))
            .collect::<Vec<_>>();
        let continue_learning = continue_learning(&curricula, active_session.as_ref());
        let practice_suggestion = practice_suggestion(&connection)?;
        Ok(CurriculumCatalogDto {
            registry_version: CURRICULUM_REGISTRY_VERSION,
            progress_version: CURRICULUM_PROGRESS_VERSION,
            recommendation_version: CURRICULUM_RECOMMENDATION_VERSION,
            taxonomy_version: CURRICULUM_TAXONOMY_VERSION,
            published_curriculum_count: curricula.len(),
            active_session,
            continue_learning,
            practice_suggestion,
            curricula,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurriculumCatalogDto {
    pub registry_version: u32,
    pub progress_version: u32,
    pub recommendation_version: u32,
    pub taxonomy_version: u32,
    pub published_curriculum_count: usize,
    pub active_session: Option<CurriculumActiveSessionDto>,
    pub continue_learning: CurriculumNextStepDto,
    pub practice_suggestion: Option<CurriculumPracticeSuggestionDto>,
    pub curricula: Vec<CourseDto>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CurriculumNextStepKind {
    ResumeLesson,
    StartLesson,
    ChooseLevel,
    CourseComplete,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurriculumNextStepDto {
    pub kind: CurriculumNextStepKind,
    pub title: String,
    pub description: String,
    pub action_label: String,
    pub curriculum_id: Option<String>,
    pub level_id: Option<String>,
    pub unit_id: Option<String>,
    pub lesson_id: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurriculumPracticeSuggestionDto {
    pub kind: String,
    pub title: String,
    pub description: String,
    pub item_count: u32,
    pub route: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurriculumActiveSessionDto {
    pub session_id: String,
    pub lesson_id: String,
    pub content_version: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseDto {
    pub curriculum_id: String,
    pub curriculum_version: u32,
    pub curriculum_hash: String,
    pub title: String,
    pub description: String,
    pub target_language: String,
    pub reference_locale: String,
    pub suggested_level: Option<CefrBand>,
    pub placement_confidence: Option<PlacementConfidence>,
    pub target_level: Option<CefrBand>,
    pub progress: CurriculumProgressDto,
    pub levels: Vec<CourseLevelDto>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseLevelDto {
    pub level_id: String,
    pub cefr_level: CefrBand,
    pub title: String,
    pub description: String,
    pub objectives: Vec<String>,
    pub recommended: bool,
    pub target: bool,
    pub progress: CurriculumProgressDto,
    pub units: Vec<CourseUnitDto>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseUnitDto {
    pub unit_id: String,
    pub title: String,
    pub description: String,
    pub objectives: Vec<String>,
    pub skill_focus: Vec<SkillFocus>,
    pub grammar_topics: Vec<String>,
    pub vocabulary_topics: Vec<String>,
    pub communicative_functions: Vec<String>,
    pub progress: CurriculumProgressDto,
    pub lessons: Vec<CourseLessonDto>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CurriculumLessonStatus {
    NotStarted,
    InProgress,
    Completed,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseLessonDto {
    pub lesson_id: String,
    pub content_version: u32,
    pub title: String,
    pub description: String,
    pub cefr_band: CefrBand,
    pub estimated_minutes: u32,
    pub objectives: Vec<String>,
    pub startable: bool,
    pub unavailable_reasons: Vec<String>,
    pub status: CurriculumLessonStatus,
    pub completion_count: u32,
    pub has_updated_content_available: bool,
    pub active_session_id: Option<String>,
    pub active_session_content_version: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurriculumProgressDto {
    pub total_lessons: u32,
    pub completed_lessons: u32,
    pub in_progress_lessons: u32,
    pub percent: u32,
}

#[derive(Clone, Debug, Default)]
struct SessionAggregate {
    active_session_id: Option<String>,
    active_content_version: Option<u32>,
    max_completed_content_version: Option<u32>,
    completion_count: u32,
}

fn load_session_aggregates(
    connection: &Connection,
) -> Result<BTreeMap<String, SessionAggregate>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id,lesson_id,lesson_content_version,status FROM interactive_lesson_session",
        )
        .map_err(db)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u32>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(db)?;
    let mut result = BTreeMap::<String, SessionAggregate>::new();
    for row in rows {
        let (id, lesson_id, version, status) = row.map_err(db)?;
        let entry = result.entry(lesson_id).or_default();
        match status.as_str() {
            "in_progress" => {
                entry.active_session_id = Some(id);
                entry.active_content_version = Some(version);
            }
            "completed" => {
                entry.completion_count = entry.completion_count.saturating_add(1);
                entry.max_completed_content_version = Some(
                    entry
                        .max_completed_content_version
                        .map(|current| current.max(version))
                        .unwrap_or(version),
                );
            }
            "abandoned" | "failed" => {}
            _ => return Err("Stored Guided Session status is invalid.".into()),
        }
    }
    Ok(result)
}

fn build_course(
    registered: &RegisteredCurriculum,
    sessions: &BTreeMap<String, SessionAggregate>,
    profile: &crate::student_profile_repository::StudentLearningProfileDto,
) -> CourseDto {
    let suggested = profile
        .current_placement
        .as_ref()
        .map(|value| value.estimated_level);
    let levels = registered
        .manifest
        .levels
        .iter()
        .map(|level| {
            let units = level
                .units
                .iter()
                .map(|unit| {
                    let lessons = unit
                        .lessons
                        .iter()
                        .map(|reference| {
                            let lesson = registered
                                .lessons
                                .get(&(reference.lesson_id.clone(), reference.content_version))
                                .expect("validated exact lesson");
                            let descriptor = summary(lesson);
                            let aggregate = sessions
                                .get(&reference.lesson_id)
                                .cloned()
                                .unwrap_or_default();
                            let completed = aggregate.max_completed_content_version.is_some();
                            let status = if completed {
                                CurriculumLessonStatus::Completed
                            } else if aggregate.active_session_id.is_some() {
                                CurriculumLessonStatus::InProgress
                            } else {
                                CurriculumLessonStatus::NotStarted
                            };
                            CourseLessonDto {
                                lesson_id: reference.lesson_id.clone(),
                                content_version: reference.content_version,
                                title: descriptor.title,
                                description: descriptor.description,
                                cefr_band: descriptor.cefr_band,
                                estimated_minutes: descriptor.estimated_minutes,
                                objectives: descriptor.objectives,
                                startable: descriptor.startable,
                                unavailable_reasons: descriptor.unavailable_reasons,
                                status,
                                completion_count: aggregate.completion_count,
                                has_updated_content_available: aggregate
                                    .max_completed_content_version
                                    .is_some_and(|version| reference.content_version > version),
                                active_session_id: aggregate.active_session_id,
                                active_session_content_version: aggregate.active_content_version,
                            }
                        })
                        .collect::<Vec<_>>();
                    CourseUnitDto {
                        unit_id: unit.unit_id.clone(),
                        title: unit.title.clone(),
                        description: unit.description.clone(),
                        objectives: unit.objectives.clone(),
                        skill_focus: unit.skill_focus.clone(),
                        grammar_topics: unit.grammar_topics.clone(),
                        vocabulary_topics: unit.vocabulary_topics.clone(),
                        communicative_functions: unit.communicative_functions.clone(),
                        progress: progress_for_lessons(&lessons),
                        lessons,
                    }
                })
                .collect::<Vec<_>>();
            CourseLevelDto {
                level_id: level.level_id.clone(),
                cefr_level: level.cefr_level,
                title: level.title.clone(),
                description: level.description.clone(),
                objectives: level.objectives.clone(),
                recommended: suggested == Some(level.cefr_level),
                target: profile.target_level == Some(level.cefr_level),
                progress: progress_for_units(&units),
                units,
            }
        })
        .collect::<Vec<_>>();
    CourseDto {
        curriculum_id: registered.manifest.curriculum_id.clone(),
        curriculum_version: registered.manifest.curriculum_version,
        curriculum_hash: registered.curriculum_hash.clone(),
        title: registered.manifest.title.clone(),
        description: registered.manifest.description.clone(),
        target_language: registered.manifest.target_language.clone(),
        reference_locale: registered.manifest.reference_locale.clone(),
        suggested_level: suggested,
        placement_confidence: profile
            .current_placement
            .as_ref()
            .map(|value| value.confidence),
        target_level: profile.target_level,
        progress: progress_for_levels(&levels),
        levels,
    }
}

fn continue_learning(
    courses: &[CourseDto],
    active: Option<&CurriculumActiveSessionDto>,
) -> CurriculumNextStepDto {
    if let Some(active) = active {
        if let Some((course, level, unit, lesson)) = find_lesson(courses, &active.lesson_id) {
            return next_step(
                CurriculumNextStepKind::ResumeLesson,
                format!("Resume {}", lesson.title),
                "Your active Guided Lesson is saved on this computer.".into(),
                "Resume Lesson",
                Some(course),
                Some(level),
                Some(unit),
                Some(lesson),
                Some(active.session_id.clone()),
            );
        }
    }
    let Some(course) = courses.first() else {
        return next_step(
            CurriculumNextStepKind::ChooseLevel,
            "Choose your next learning step".into(),
            "No published local Course is available.".into(),
            "Open Guided Lessons",
            None,
            None,
            None,
            None,
            None,
        );
    };
    if course.progress.total_lessons > 0
        && course.progress.completed_lessons == course.progress.total_lessons
    {
        return next_step(
            CurriculumNextStepKind::CourseComplete,
            "English Course complete".into(),
            "All 288 Lesson IDs have been completed at least once. This records Course completion, not CEFR certification.".into(),
            "Review Course",
            Some(course),
            None,
            None,
            None,
            None,
        );
    }
    if course.progress.completed_lessons > 0 {
        if let Some((level, unit, lesson)) = first_lesson(course, |lesson| {
            lesson.status != CurriculumLessonStatus::Completed
        }) {
            return next_step(
                CurriculumNextStepKind::StartLesson,
                lesson.title.clone(),
                format!(
                    "Continue in sequence with the next incomplete {} lesson.",
                    level.cefr_level.as_str()
                ),
                "Continue Learning",
                Some(course),
                Some(level),
                Some(unit),
                Some(lesson),
                None,
            );
        }
    }
    if let Some(suggested) = course.suggested_level {
        if let Some(level) = course
            .levels
            .iter()
            .find(|level| level.cefr_level == suggested)
        {
            if let Some((unit, lesson)) = level
                .units
                .iter()
                .find_map(|unit| unit.lessons.first().map(|lesson| (unit, lesson)))
            {
                return next_step(
                    CurriculumNextStepKind::StartLesson,
                    lesson.title.clone(),
                    format!(
                        "Suggested starting point from your latest Placement result: {}.",
                        suggested.as_str()
                    ),
                    "Start Suggested Lesson",
                    Some(course),
                    Some(level),
                    Some(unit),
                    Some(lesson),
                    None,
                );
            }
        }
    }
    next_step(
        CurriculumNextStepKind::ChooseLevel,
        "Choose a Course level".into(),
        "You can start at any level. The Placement Test can provide an optional starting-point suggestion.".into(),
        "Choose a Level",
        Some(course),
        None,
        None,
        None,
        None,
    )
}

fn first_lesson(
    course: &CourseDto,
    predicate: impl Fn(&CourseLessonDto) -> bool,
) -> Option<(&CourseLevelDto, &CourseUnitDto, &CourseLessonDto)> {
    for level in &course.levels {
        for unit in &level.units {
            if let Some(lesson) = unit.lessons.iter().find(|lesson| predicate(lesson)) {
                return Some((level, unit, lesson));
            }
        }
    }
    None
}

fn find_lesson<'a>(
    courses: &'a [CourseDto],
    lesson_id: &str,
) -> Option<(
    &'a CourseDto,
    &'a CourseLevelDto,
    &'a CourseUnitDto,
    &'a CourseLessonDto,
)> {
    for course in courses {
        if let Some((level, unit, lesson)) =
            first_lesson(course, |lesson| lesson.lesson_id == lesson_id)
        {
            return Some((course, level, unit, lesson));
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn next_step(
    kind: CurriculumNextStepKind,
    title: String,
    description: String,
    action_label: &str,
    course: Option<&CourseDto>,
    level: Option<&CourseLevelDto>,
    unit: Option<&CourseUnitDto>,
    lesson: Option<&CourseLessonDto>,
    session_id: Option<String>,
) -> CurriculumNextStepDto {
    CurriculumNextStepDto {
        kind,
        title,
        description,
        action_label: action_label.into(),
        curriculum_id: course.map(|value| value.curriculum_id.clone()),
        level_id: level.map(|value| value.level_id.clone()),
        unit_id: unit.map(|value| value.unit_id.clone()),
        lesson_id: lesson.map(|value| value.lesson_id.clone()),
        session_id,
    }
}

fn practice_suggestion(
    connection: &Connection,
) -> Result<Option<CurriculumPracticeSuggestionDto>, String> {
    let mistakes: u32 = connection
        .query_row(
            "SELECT COUNT(*) FROM recurring_mistake WHERE lesson_count>=2 AND status!='resolved'",
            [],
            |row| row.get(0),
        )
        .map_err(db)?;
    if mistakes > 0 {
        return Ok(Some(CurriculumPracticeSuggestionDto {
            kind: "recurring_mistakes".into(),
            title: "Review recurring mistakes".into(),
            description: "Practice confirmed patterns without blocking your next Course lesson."
                .into(),
            item_count: mistakes,
            route: "/review?mode=mistakes".into(),
        }));
    }
    let vocabulary: u32 = connection
        .query_row(
            "SELECT COUNT(*) FROM vocabulary_item WHERE status IN('new','learning')",
            [],
            |row| row.get(0),
        )
        .map_err(db)?;
    Ok((vocabulary > 0).then(|| CurriculumPracticeSuggestionDto {
        kind: "vocabulary".into(),
        title: "Review vocabulary".into(),
        description: "Revisit new and learning vocabulary from completed lessons.".into(),
        item_count: vocabulary,
        route: "/review?mode=vocabulary".into(),
    }))
}

fn progress_for_lessons(lessons: &[CourseLessonDto]) -> CurriculumProgressDto {
    progress(
        lessons.len(),
        lessons
            .iter()
            .filter(|lesson| lesson.status == CurriculumLessonStatus::Completed)
            .count(),
        lessons
            .iter()
            .filter(|lesson| lesson.status == CurriculumLessonStatus::InProgress)
            .count(),
    )
}

fn progress_for_units(units: &[CourseUnitDto]) -> CurriculumProgressDto {
    aggregate_progress(units.iter().map(|unit| unit.progress))
}

fn progress_for_levels(levels: &[CourseLevelDto]) -> CurriculumProgressDto {
    aggregate_progress(levels.iter().map(|level| level.progress))
}

fn aggregate_progress(
    values: impl Iterator<Item = CurriculumProgressDto>,
) -> CurriculumProgressDto {
    let (total, completed, active) = values.fold((0usize, 0usize, 0usize), |state, value| {
        (
            state.0 + value.total_lessons as usize,
            state.1 + value.completed_lessons as usize,
            state.2 + value.in_progress_lessons as usize,
        )
    });
    progress(total, completed, active)
}

fn progress(total: usize, completed: usize, active: usize) -> CurriculumProgressDto {
    CurriculumProgressDto {
        total_lessons: total as u32,
        completed_lessons: completed as u32,
        in_progress_lessons: active as u32,
        percent: if total == 0 {
            0
        } else {
            ((completed * 100 + total / 2) / total) as u32
        },
    }
}

fn load_curriculum(
    directory: &Path,
    lessons: &InteractiveLessonContentRegistry,
) -> Result<RegisteredCurriculum, String> {
    let path = directory.join("curriculum.json");
    let metadata = fs::metadata(&path).map_err(|_| "curriculum.json is missing".to_owned())?;
    if metadata.len() > MAX_CURRICULUM_BYTES {
        return Err("curriculum.json exceeds 1 MiB".into());
    }
    let manifest: CurriculumManifest = serde_json::from_slice(
        &fs::read(&path).map_err(|_| "curriculum.json could not be read".to_owned())?,
    )
    .map_err(|error| format!("curriculum.json is invalid: {error}"))?;
    validate_manifest(&manifest)?;
    let mut resolved = BTreeMap::new();
    let mut hashes = Vec::new();
    for level in &manifest.levels {
        for unit in &level.units {
            for reference in &unit.lessons {
                let lesson = lessons
                    .get_exact(&reference.lesson_id, reference.content_version)
                    .ok_or_else(|| {
                        format!(
                            "referenced lesson {}@{} is unavailable",
                            reference.lesson_id, reference.content_version
                        )
                    })?;
                if matches!(manifest.publication_state, PublicationState::Published)
                    && !matches!(
                        lesson.package.publication_state,
                        PublicationState::Published
                    )
                {
                    return Err("published curriculum references a draft lesson".into());
                }
                if lesson.package.cefr_band != level.cefr_level {
                    return Err("referenced lesson CEFR does not match its curriculum level".into());
                }
                hashes.push(lesson.package_hash.clone());
                resolved.insert(
                    (reference.lesson_id.clone(), reference.content_version),
                    lesson,
                );
            }
        }
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct HashInput<'a> {
        manifest: &'a CurriculumManifest,
        referenced_package_hashes: &'a [String],
    }
    let canonical = serde_json::to_vec(&HashInput {
        manifest: &manifest,
        referenced_package_hashes: &hashes,
    })
    .map_err(|error| error.to_string())?;
    Ok(RegisteredCurriculum {
        manifest,
        curriculum_hash: sha256::bytes(&canonical),
        lessons: resolved,
    })
}

fn validate_manifest(value: &CurriculumManifest) -> Result<(), String> {
    if value.curriculum_schema_version != CURRICULUM_SCHEMA_VERSION {
        return Err("unsupported curriculumSchemaVersion".into());
    }
    validate_slug(&value.curriculum_id, "curriculumId")?;
    if value.curriculum_version == 0 {
        return Err("curriculumVersion must be at least 1".into());
    }
    bounded(&value.title, 1, 100, "title")?;
    bounded(&value.description, 1, 600, "description")?;
    if value.target_language != "en" || value.reference_locale != "en-US" {
        return Err("only en / en-US curricula are supported".into());
    }
    if value.levels.is_empty() || value.levels.len() > 6 {
        return Err("levels must contain 1..6 entries".into());
    }
    let mut level_ids = BTreeSet::new();
    let mut unit_ids = BTreeSet::new();
    let mut lesson_ids = BTreeSet::new();
    let mut previous = 0;
    let mut total_lessons = 0usize;
    for level in &value.levels {
        let expected_id = level.cefr_level.as_str().to_ascii_lowercase();
        if level.level_id != expected_id {
            return Err("levelId and cefrLevel do not match".into());
        }
        if !level_ids.insert(level.level_id.clone()) {
            return Err("duplicate levelId".into());
        }
        if level.cefr_level.ordinal() <= previous {
            return Err("levels are not in canonical CEFR order".into());
        }
        previous = level.cefr_level.ordinal();
        bounded(&level.title, 1, 100, "level title")?;
        bounded(&level.description, 1, 600, "level description")?;
        validate_texts(&level.objectives, 12, 200, "level objectives")?;
        if level.units.is_empty() || level.units.len() > 30 {
            return Err("units must contain 1..30 entries".into());
        }
        for unit in &level.units {
            validate_slug(&unit.unit_id, "unitId")?;
            if !unit_ids.insert(unit.unit_id.clone()) {
                return Err("duplicate unitId".into());
            }
            bounded(&unit.title, 1, 100, "unit title")?;
            bounded(&unit.description, 1, 600, "unit description")?;
            validate_texts(&unit.objectives, 10, 200, "unit objectives")?;
            if unit.skill_focus.len() > 6
                || unit.skill_focus.iter().collect::<BTreeSet<_>>().len() != unit.skill_focus.len()
            {
                return Err("skillFocus contains duplicates or exceeds its limit".into());
            }
            validate_texts(&unit.grammar_topics, 12, 120, "grammarTopics")?;
            validate_texts(&unit.vocabulary_topics, 12, 120, "vocabularyTopics")?;
            validate_texts(
                &unit.communicative_functions,
                12,
                120,
                "communicativeFunctions",
            )?;
            if unit.lessons.is_empty() || unit.lessons.len() > 30 {
                return Err("lessons must contain 1..30 entries".into());
            }
            total_lessons += unit.lessons.len();
            for lesson in &unit.lessons {
                validate_slug(&lesson.lesson_id, "lessonId")?;
                if lesson.content_version == 0 {
                    return Err("lesson contentVersion must be at least 1".into());
                }
                if !lesson_ids.insert(lesson.lesson_id.clone()) {
                    return Err("duplicate lessonId in curriculum".into());
                }
            }
        }
    }
    if total_lessons > MAX_TOTAL_LESSONS {
        return Err("curriculum exceeds 500 lesson references".into());
    }
    Ok(())
}

fn validate_texts(values: &[String], max: usize, length: usize, name: &str) -> Result<(), String> {
    if values.len() > max {
        return Err(format!("{name} exceeds its collection limit"));
    }
    for value in values {
        bounded(value, 1, length, name)?;
    }
    Ok(())
}

fn bounded(value: &str, min: usize, max: usize, name: &str) -> Result<(), String> {
    let length = value.trim().chars().count();
    if length < min || length > max || value.chars().any(char::is_control) {
        return Err(format!("{name} is outside its plain-text limit"));
    }
    Ok(())
}

fn validate_slug(value: &str, name: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 100
        || value.starts_with('-')
        || value.ends_with('-')
        || value.contains("--")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(format!("{name} must be a stable lowercase slug"));
    }
    Ok(())
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(240)
        .collect()
}

fn db(error: rusqlite::Error) -> String {
    format!("Curriculum progress database operation failed: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        database, interactive_lesson::StartInteractiveLessonRequest,
        interactive_lesson_engine::InteractiveLessonEngine,
        interactive_lesson_repository::InteractiveLessonRepository,
        placement_repository::PlacementRepository,
    };
    use serde_json::{json, Value};
    use std::time::Instant;

    fn roots() -> (PathBuf, PathBuf) {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        (
            manifest.join("test-fixtures/curriculum-phase-w"),
            manifest.join("test-fixtures/interactive-lessons-phase-w"),
        )
    }

    fn registries() -> (CurriculumRegistry, InteractiveLessonContentRegistry) {
        let (curricula, lessons_root) = roots();
        let lessons = InteractiveLessonContentRegistry::load(lessons_root);
        (CurriculumRegistry::load(curricula, &lessons), lessons)
    }

    fn temp_service() -> (PathBuf, CurriculumService) {
        let root = std::env::temp_dir().join(format!("curriculum-w-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let database_path = root.join("test.sqlite3");
        database::migrate(&database_path).unwrap();
        let (registry, _) = registries();
        let placement = PlacementRepository::new(database_path.clone()).unwrap();
        let profiles = StudentProfileRepository::new(database_path.clone(), placement);
        (
            root,
            CurriculumService::new(registry, database_path, profiles),
        )
    }

    fn manifest_value() -> Value {
        serde_json::from_str(include_str!(
            "../test-fixtures/curriculum-phase-w/test-english-course/curriculum.json"
        ))
        .unwrap()
    }

    #[test]
    fn registry_loads_published_hides_draft_isolates_invalid_and_caches() {
        let started = Instant::now();
        let (registry, lessons) = registries();
        assert_eq!(registry.published_count(), 2);
        assert_eq!(registry.invalid_count(), 1);
        assert!(lessons.get_exact("test-introductions", 1).is_some());
        assert!(lessons.get_exact("test-introductions", 2).is_some());
        let first = registry.published.get("test-english-course").unwrap();
        let second = CurriculumRegistry::load(roots().0, &lessons)
            .published
            .remove("test-english-course")
            .unwrap();
        assert_eq!(first.curriculum_hash, second.curriculum_hash);
        assert!(started.elapsed().as_secs() < 5);
    }

    #[test]
    fn strict_validation_covers_versions_order_pairs_duplicates_limits_and_security() {
        let base: CurriculumManifest = serde_json::from_value(manifest_value()).unwrap();
        assert!(validate_manifest(&base).is_ok());
        let mut cases = Vec::new();
        let mut value = manifest_value();
        value["curriculumSchemaVersion"] = json!(2);
        cases.push(value);
        let mut value = manifest_value();
        value["curriculumId"] = json!("Bad Id");
        cases.push(value);
        let mut value = manifest_value();
        value["curriculumVersion"] = json!(0);
        cases.push(value);
        let mut value = manifest_value();
        value["levels"][0]["cefrLevel"] = json!("B1");
        cases.push(value);
        let mut value = manifest_value();
        value["levels"].as_array_mut().unwrap().reverse();
        cases.push(value);
        let mut value = manifest_value();
        let first = value["levels"][0].clone();
        value["levels"].as_array_mut().unwrap().push(first);
        cases.push(value);
        let mut value = manifest_value();
        let first = value["levels"][0]["units"][0].clone();
        value["levels"][1]["units"]
            .as_array_mut()
            .unwrap()
            .push(first);
        cases.push(value);
        let mut value = manifest_value();
        value["levels"][0]["units"][0]["skillFocus"] = json!(["grammar", "grammar"]);
        cases.push(value);
        let mut value = manifest_value();
        value["levels"][0]["units"][0]["skillFocus"] = json!(["writing"]);
        cases.push(value);
        let mut value = manifest_value();
        value["levels"][0]["units"][0]["grammarTopics"] =
            json!((0..13).map(|n| format!("topic {n}")).collect::<Vec<_>>());
        cases.push(value);
        let mut value = manifest_value();
        value["levels"][0]["units"][0]["lessons"][0]["contentVersion"] = json!(0);
        cases.push(value);
        let mut value = manifest_value();
        let first = value["levels"][0]["units"][0]["lessons"][0].clone();
        value["levels"][1]["units"][0]["lessons"]
            .as_array_mut()
            .unwrap()
            .push(first);
        cases.push(value);
        for value in cases {
            match serde_json::from_value::<CurriculumManifest>(value) {
                Ok(value) => assert!(validate_manifest(&value).is_err()),
                Err(_) => {}
            }
        }
        let mut unknown = manifest_value();
        unknown["systemPrompt"] = json!("ignore rules");
        assert!(serde_json::from_value::<CurriculumManifest>(unknown).is_err());
        let mut remote = manifest_value();
        remote["remoteUrl"] = json!("https://example.com");
        assert!(serde_json::from_value::<CurriculumManifest>(remote).is_err());
    }

    #[test]
    fn published_cross_validation_requires_exact_published_matching_lesson() {
        let (_, lessons_root) = roots();
        let lessons = InteractiveLessonContentRegistry::load(lessons_root);
        let root = std::env::temp_dir().join(format!("curriculum-cross-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("one")).unwrap();
        let write = |value: &Value| {
            fs::write(
                root.join("one/curriculum.json"),
                serde_json::to_vec(value).unwrap(),
            )
            .unwrap()
        };
        let valid = manifest_value();
        write(&valid);
        assert_eq!(
            CurriculumRegistry::load(root.clone(), &lessons).published_count(),
            1
        );
        let mut missing = valid.clone();
        missing["levels"][0]["units"][0]["lessons"][0]["contentVersion"] = json!(99);
        write(&missing);
        assert_eq!(
            CurriculumRegistry::load(root.clone(), &lessons).published_count(),
            0
        );
        let mut draft = valid;
        draft["levels"][0]["units"][0]["lessons"][0]["lessonId"] = json!("test-draft");
        draft["levels"][0]["units"][0]["lessons"][0]["contentVersion"] = json!(1);
        write(&draft);
        assert_eq!(
            CurriculumRegistry::load(root.clone(), &lessons).published_count(),
            0
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn manifest_file_and_total_reference_limits_are_defensive() {
        let mut value = manifest_value();
        let lesson_template = value["levels"][0]["units"][0]["lessons"][0].clone();
        let unit_template = value["levels"][0]["units"][0].clone();
        let units = value["levels"][0]["units"].as_array_mut().unwrap();
        units.clear();
        for unit_index in 0..17 {
            let mut unit = unit_template.clone();
            unit["unitId"] = json!(format!("a1-unit-{unit_index}"));
            let lessons = unit["lessons"].as_array_mut().unwrap();
            lessons.clear();
            for lesson_index in 0..30 {
                let mut lesson = lesson_template.clone();
                lesson["lessonId"] = json!(format!("test-{unit_index}-{lesson_index}"));
                lessons.push(lesson);
            }
            units.push(unit);
        }
        value["levels"].as_array_mut().unwrap().truncate(1);
        assert!(validate_manifest(&serde_json::from_value(value).unwrap()).is_err());

        let root = std::env::temp_dir().join(format!("curriculum-size-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("large")).unwrap();
        fs::write(
            root.join("large/curriculum.json"),
            vec![b' '; MAX_CURRICULUM_BYTES as usize + 1],
        )
        .unwrap();
        let lessons = InteractiveLessonContentRegistry::load(roots().1);
        let registry = CurriculumRegistry::load(root.clone(), &lessons);
        assert_eq!(registry.published_count(), 0);
        assert_eq!(registry.invalid_count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn canonical_hash_ignores_json_formatting_and_includes_exact_package_hash() {
        let root = std::env::temp_dir().join(format!("curriculum-hash-{}", uuid::Uuid::new_v4()));
        let lesson_root = root.join("lessons");
        let curriculum_root = root.join("curricula");
        fs::create_dir_all(lesson_root.join("intro")).unwrap();
        fs::create_dir_all(curriculum_root.join("course")).unwrap();
        let mut curriculum = manifest_value();
        curriculum["levels"].as_array_mut().unwrap().truncate(1);
        curriculum["levels"][0]["units"][0]["lessons"][0]["contentVersion"] = json!(1);
        fs::write(
            curriculum_root.join("course/curriculum.json"),
            serde_json::to_vec_pretty(&curriculum).unwrap(),
        )
        .unwrap();
        let source = roots().1.join("intro-v1/lesson.json");
        fs::copy(&source, lesson_root.join("intro/lesson.json")).unwrap();
        let lessons = InteractiveLessonContentRegistry::load(lesson_root.clone());
        let first = CurriculumRegistry::load(curriculum_root.clone(), &lessons)
            .published
            .remove("test-english-course")
            .unwrap()
            .curriculum_hash;
        fs::write(
            curriculum_root.join("course/curriculum.json"),
            serde_json::to_vec(&curriculum).unwrap(),
        )
        .unwrap();
        let same = CurriculumRegistry::load(curriculum_root.clone(), &lessons)
            .published
            .remove("test-english-course")
            .unwrap()
            .curriculum_hash;
        assert_eq!(first, same);
        let mut package: Value = serde_json::from_slice(&fs::read(&source).unwrap()).unwrap();
        package["stages"][0]["payload"]["blocks"][0]["text"] =
            json!("Changed exact package content.");
        fs::write(
            lesson_root.join("intro/lesson.json"),
            serde_json::to_vec(&package).unwrap(),
        )
        .unwrap();
        let changed_lessons = InteractiveLessonContentRegistry::load(lesson_root);
        let changed = CurriculumRegistry::load(curriculum_root, &changed_lessons)
            .published
            .remove("test-english-course")
            .unwrap()
            .curriculum_hash;
        assert_ne!(first, changed);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "manual Phase X gate for the draft A1 Unit 1 production Curriculum"]
    fn physical_phase_x_a1_unit1_draft_curriculum_resolves() {
        let manifest_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources");
        let lessons =
            InteractiveLessonContentRegistry::load(manifest_root.join("interactive-lessons"));
        let registry = CurriculumRegistry::load(manifest_root.join("curriculum"), &lessons);
        assert_eq!(registry.invalid_count(), 0);
        assert_eq!(registry.published_count(), 0);
        for (lesson_id, content_version) in [
            ("a1-u01-l01-hello-goodbye", 1),
            ("a1-u01-l02-whats-your-name", 1),
            ("a1-u01-l03-countries-nationalities", 1),
            ("a1-u01-l04-personal-information", 1),
            ("a1-u01-l05-i-am-you-are-he-is", 1),
            ("a1-u01-l06-introductions-mission", 1),
        ] {
            assert!(lessons.get_exact(lesson_id, content_version).is_some());
        }
    }

    #[test]
    fn progress_is_derived_by_stable_id_ignores_scores_and_noncompleted_sessions() {
        let (root, service) = temp_service();
        let db_path = root.join("test.sqlite3");
        let connection = database::open(&db_path).unwrap();
        let insert = |id: &str, lesson: &str, version: u32, status: &str| {
            connection.execute("INSERT INTO interactive_lesson_session(id,lesson_id,lesson_content_version,package_schema_version,lesson_flow_version,package_hash,engine_version,snapshot_version,status,stage_count,current_stage_index,package_snapshot_json,student_context_snapshot_json,started_at,updated_at,completed_at,abandoned_at) VALUES(?1,?2,?3,1,1,?4,1,1,?5,1,0,'{}','{}','now','now',CASE WHEN ?5='completed' THEN 'now' END,CASE WHEN ?5='abandoned' THEN 'now' END)",rusqlite::params![id,lesson,version,"a".repeat(64),status]).unwrap()
        };
        insert("abandoned", "test-travel-problems", 1, "abandoned");
        insert("failed", "test-travel-problems", 1, "failed");
        insert("completed-1", "test-introductions", 1, "completed");
        insert("completed-2", "test-introductions", 1, "completed");
        let catalog = service.catalog().unwrap();
        let course = catalog
            .curricula
            .iter()
            .find(|value| value.curriculum_id == "test-english-course")
            .unwrap();
        let lessons = course
            .levels
            .iter()
            .flat_map(|level| &level.units)
            .flat_map(|unit| &unit.lessons)
            .collect::<Vec<_>>();
        let intro = lessons
            .iter()
            .find(|lesson| lesson.lesson_id == "test-introductions")
            .unwrap();
        let travel = lessons
            .iter()
            .find(|lesson| lesson.lesson_id == "test-travel-problems")
            .unwrap();
        assert_eq!(intro.status, CurriculumLessonStatus::Completed);
        assert_eq!(intro.completion_count, 2);
        assert!(intro.has_updated_content_available);
        assert_eq!(travel.status, CurriculumLessonStatus::NotStarted);
        assert_eq!(course.progress.completed_lessons, 1);
        assert_eq!(course.progress.percent, 50);
        assert_eq!(
            catalog.continue_learning.kind,
            CurriculumNextStepKind::StartLesson
        );
        assert_eq!(
            catalog.continue_learning.lesson_id.as_deref(),
            Some("test-travel-problems")
        );
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn active_old_version_and_placement_are_recommendations_not_locks_or_progress() {
        let (root, service) = temp_service();
        let connection = database::open(&root.join("test.sqlite3")).unwrap();
        connection.execute("INSERT INTO interactive_lesson_session(id,lesson_id,lesson_content_version,package_schema_version,lesson_flow_version,package_hash,engine_version,snapshot_version,status,stage_count,current_stage_index,package_snapshot_json,student_context_snapshot_json,started_at,updated_at) VALUES('active-old','test-introductions',1,1,1,?1,1,1,'in_progress',1,0,'{\"contentVersion\":1}','{}','now','now')",["b".repeat(64)]).unwrap();
        connection.execute("INSERT INTO placement_attempt(id,status,test_version,question_bank_version,scoring_version,speaking_prompt_version,started_at,completed_at,overall_estimated_level,confidence,speaking_status,created_at,updated_at) VALUES('placement-b1','completed',1,1,1,1,'now','now','B1','high','skipped','now','now')",[]).unwrap();
        let catalog = service.catalog().unwrap();
        let course = catalog
            .curricula
            .iter()
            .find(|value| value.curriculum_id == "test-english-course")
            .unwrap();
        assert_eq!(course.suggested_level, Some(CefrBand::B1));
        assert_eq!(course.progress.completed_lessons, 0);
        assert!(course.levels.iter().all(|level| !level.units.is_empty()));
        let intro = &course.levels[0].units[0].lessons[0];
        assert_eq!(intro.status, CurriculumLessonStatus::InProgress);
        assert_eq!(intro.active_session_id.as_deref(), Some("active-old"));
        assert_eq!(intro.active_session_content_version, Some(1));
        assert_eq!(
            catalog.continue_learning.kind,
            CurriculumNextStepKind::ResumeLesson
        );
        assert_eq!(
            catalog.continue_learning.session_id.as_deref(),
            Some("active-old")
        );
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn no_placement_never_fabricates_a1_and_safe_dto_has_no_private_content() {
        let (root, service) = temp_service();
        let catalog = service.catalog().unwrap();
        assert_eq!(
            catalog.continue_learning.kind,
            CurriculumNextStepKind::ChooseLevel
        );
        let json = serde_json::to_string(&catalog).unwrap();
        assert!(json.contains("\"suggestedLevel\":null"));
        for forbidden in [
            "answerKey",
            "acceptedAnswers",
            "correctAnswer",
            "systemPrompt",
            "analysisPrompt",
            "teacherPrompt",
        ] {
            assert!(!json.contains(forbidden));
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn placement_with_zero_progress_suggests_first_lesson_of_that_level_without_locking_access() {
        let (root, service) = temp_service();
        let connection = database::open(&root.join("test.sqlite3")).unwrap();
        connection.execute("INSERT INTO placement_attempt(id,status,test_version,question_bank_version,scoring_version,speaking_prompt_version,started_at,completed_at,overall_estimated_level,confidence,speaking_status,created_at,updated_at) VALUES('placement-b1-only','completed',1,1,1,1,'now','now','B1','high','skipped','now','now')",[]).unwrap();
        drop(connection);
        let catalog = service.catalog().unwrap();
        assert_eq!(
            catalog.continue_learning.kind,
            CurriculumNextStepKind::StartLesson
        );
        assert_eq!(
            catalog.continue_learning.lesson_id.as_deref(),
            Some("test-travel-problems")
        );
        assert!(catalog
            .curricula
            .iter()
            .flat_map(|course| &course.levels)
            .all(|level| !level.units.is_empty()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "manual TEMP physical Curriculum navigation/progress validation"]
    fn physical_temp_curriculum_navigation_and_progress_validation() {
        let database_path = PathBuf::from(
            std::env::var_os("EAC_PHASE_W_TEMP_DB")
                .expect("EAC_PHASE_W_TEMP_DB must name the controlled TEMP database"),
        );
        if database_path.exists() {
            fs::remove_file(&database_path).unwrap();
        }
        database::migrate(&database_path).unwrap();
        let (curriculum_root, lessons_root) = roots();
        let lessons = InteractiveLessonContentRegistry::load(lessons_root);
        let registry = CurriculumRegistry::load(curriculum_root, &lessons);
        let placement = PlacementRepository::new(database_path.clone()).unwrap();
        let profiles = StudentProfileRepository::new(database_path.clone(), placement);
        let service = CurriculumService::new(registry, database_path.clone(), profiles.clone());
        let engine = InteractiveLessonEngine::new(
            lessons,
            InteractiveLessonRepository::new(database_path.clone()),
            profiles,
            database_path.parent().unwrap().join("phase-w-temp-assets"),
        );

        let initial = service.catalog().unwrap();
        let course = initial
            .curricula
            .iter()
            .find(|value| value.curriculum_id == "test-english-course")
            .unwrap();
        assert_eq!(course.suggested_level, None);
        assert_eq!(course.progress.percent, 0);
        assert_eq!(
            course
                .levels
                .iter()
                .map(|level| level.cefr_level)
                .collect::<Vec<_>>(),
            vec![CefrBand::A1, CefrBand::B1]
        );

        let connection = database::open(&database_path).unwrap();
        connection.execute("INSERT INTO placement_attempt(id,status,test_version,question_bank_version,scoring_version,speaking_prompt_version,started_at,completed_at,overall_estimated_level,confidence,speaking_status,created_at,updated_at) VALUES('phase-w-placement','completed',1,1,1,1,'now','now','B1','high','skipped','now','now')", []).unwrap();
        drop(connection);
        let recommended = service.catalog().unwrap();
        let course = recommended
            .curricula
            .iter()
            .find(|value| value.curriculum_id == "test-english-course")
            .unwrap();
        assert_eq!(course.suggested_level, Some(CefrBand::B1));
        assert_eq!(course.progress.completed_lessons, 0);

        let active = engine
            .start(StartInteractiveLessonRequest {
                lesson_id: "test-introductions".into(),
                content_version: Some(1),
                start_over: false,
            })
            .unwrap();
        assert_eq!(active.content_version, 1);
        assert!(engine
            .start(StartInteractiveLessonRequest {
                lesson_id: "test-travel-problems".into(),
                content_version: Some(1),
                start_over: false,
            })
            .is_err());
        let active_catalog = service.catalog().unwrap();
        let course = active_catalog
            .curricula
            .iter()
            .find(|value| value.curriculum_id == "test-english-course")
            .unwrap();
        let intro = &course.levels[0].units[0].lessons[0];
        assert_eq!(intro.status, CurriculumLessonStatus::InProgress);
        assert_eq!(intro.active_session_content_version, Some(1));
        assert_eq!(engine.resume(&active.id).unwrap().content_version, 1);

        let connection = database::open(&database_path).unwrap();
        connection.execute("UPDATE interactive_lesson_session SET status='completed',completed_at='now',updated_at='now' WHERE id=?1", [&active.id]).unwrap();
        drop(connection);
        let completed_catalog = service.catalog().unwrap();
        let course = completed_catalog
            .curricula
            .iter()
            .find(|value| value.curriculum_id == "test-english-course")
            .unwrap();
        let intro = &course.levels[0].units[0].lessons[0];
        assert_eq!(intro.status, CurriculumLessonStatus::Completed);
        assert!(intro.has_updated_content_available);
        assert_eq!(course.progress.completed_lessons, 1);
        assert_eq!(course.progress.percent, 50);

        let connection = database::open(&database_path).unwrap();
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        let foreign_keys: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .unwrap();
        let snapshot_version: i64 = connection.query_row("SELECT json_extract(package_snapshot_json,'$.contentVersion') FROM interactive_lesson_session WHERE id=?1", [&active.id], |row| row.get(0)).unwrap();
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_keys, 0);
        assert_eq!(snapshot_version, 1);
        println!(
            "{}",
            json!({
                "tempDatabase": database_path,
                "schema": connection.query_row("SELECT MAX(version) FROM schema_migration", [], |row| row.get::<_,u32>(0)).unwrap(),
                "publishedCurricula": completed_catalog.published_curriculum_count,
                "suggestedLevel": course.suggested_level,
                "allLevelsAccessible": course.levels.len(),
                "startedSessionContentVersion": active.content_version,
                "resumedSnapshotContentVersion": snapshot_version,
                "completedLessons": course.progress.completed_lessons,
                "courseProgressPercent": course.progress.percent,
                "updatedContentAvailable": intro.has_updated_content_available,
                "integrity": integrity,
                "foreignKeys": foreign_keys,
            })
        );
    }

    #[test]
    fn production_english_core_resolves_every_installed_level_at_zero_progress() {
        let resources = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources");
        let lessons = InteractiveLessonContentRegistry::load(resources.join("interactive-lessons"));
        assert!(matches!(lessons.published_count(), 192 | 288));
        let registry = CurriculumRegistry::load(resources.join("curriculum"), &lessons);
        if registry.invalid_count() != 0 {
            eprintln!("Production curriculum errors: {:#?}", registry.invalid);
        }
        assert_eq!(registry.invalid_count(), 0);
        assert_eq!(registry.published_count(), 1);

        let root = std::env::temp_dir().join(format!("production-course-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let database_path = root.join("course.sqlite3");
        database::migrate(&database_path).unwrap();
        let placement = PlacementRepository::new(database_path.clone()).unwrap();
        let profiles = StudentProfileRepository::new(database_path.clone(), placement);
        let service = CurriculumService::new(registry, database_path.clone(), profiles);
        let catalog = service.catalog().unwrap();
        assert_eq!(catalog.published_curriculum_count, 1);
        let course = &catalog.curricula[0];
        assert_eq!(course.curriculum_id, "english-core");
        let expected_levels = if lessons.published_count() == 192 {
            vec![CefrBand::A1, CefrBand::A2, CefrBand::B1, CefrBand::B2]
        } else {
            vec![
                CefrBand::A1,
                CefrBand::A2,
                CefrBand::B1,
                CefrBand::B2,
                CefrBand::C1,
                CefrBand::C2,
            ]
        };
        assert_eq!(course.levels.len(), expected_levels.len());
        assert_eq!(
            course
                .levels
                .iter()
                .map(|level| level.cefr_level)
                .collect::<Vec<_>>(),
            expected_levels
        );
        assert_eq!(
            course
                .levels
                .iter()
                .map(|level| level.units.len())
                .sum::<usize>(),
            course.levels.len() * 8
        );
        assert_eq!(
            course.progress.total_lessons,
            lessons.published_count() as u32
        );
        assert_eq!(course.progress.completed_lessons, 0);
        assert_eq!(course.progress.percent, 0);
        assert!(course
            .levels
            .iter()
            .flat_map(|level| &level.units)
            .flat_map(|unit| &unit.lessons)
            .all(|lesson| lesson.status == CurriculumLessonStatus::NotStarted));

        let connection = database::open(&database_path).unwrap();
        connection.execute("INSERT INTO placement_attempt(id,status,test_version,question_bank_version,scoring_version,speaking_prompt_version,started_at,completed_at,overall_estimated_level,confidence,speaking_status,created_at,updated_at) VALUES('phase-x-placement-b1','completed',1,1,1,1,'now','now','B1','high','skipped','now','now')", []).unwrap();
        drop(connection);
        let outside = service.catalog().unwrap();
        let course = &outside.curricula[0];
        assert_eq!(course.suggested_level, Some(CefrBand::B1));
        assert_eq!(
            course
                .levels
                .iter()
                .filter(|level| level.recommended)
                .count(),
            1
        );
        assert!(
            course
                .levels
                .iter()
                .find(|level| level.cefr_level == CefrBand::B1)
                .unwrap()
                .recommended
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "writes the controlled Phase X production content manifest artifact"]
    fn physical_phase_x_write_production_content_manifest() {
        use std::fmt::Write as _;

        let project = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();
        let resources = project.join("src-tauri").join("resources");
        let lessons = InteractiveLessonContentRegistry::load(resources.join("interactive-lessons"));
        let registry = CurriculumRegistry::load(resources.join("curriculum"), &lessons);
        let curriculum = registry.published.get("english-core").unwrap();
        let mut output = String::from(
            "# Production Content Manifest\n\nGenerated from the official typed Package Registry.\n\n| lessonId | level | unit | title | contentVersion | publicationState | packageHash |\n|---|---|---|---|---:|---|---|\n",
        );
        let mut count = 0;
        for level in &curriculum.manifest.levels {
            for unit in &level.units {
                for reference in &unit.lessons {
                    let lesson = curriculum
                        .lessons
                        .get(&(reference.lesson_id.clone(), reference.content_version))
                        .unwrap();
                    writeln!(
                        output,
                        "| {} | {} | {} | {} | {} | published | `{}` |",
                        lesson.package.lesson_id,
                        level.cefr_level.as_str(),
                        unit.title,
                        lesson.package.title,
                        lesson.package.content_version,
                        lesson.package_hash
                    )
                    .unwrap();
                    count += 1;
                }
            }
        }
        assert_eq!(count, 96);
        fs::create_dir_all(project.join(".phase-x-artifacts")).unwrap();
        fs::write(
            project
                .join(".phase-x-artifacts")
                .join("PRODUCTION_CONTENT_MANIFEST.md"),
            output,
        )
        .unwrap();
    }
}
