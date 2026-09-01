use crate::{
    database,
    lesson_modes::LessonDifficulty,
    placement::{CefrBand, PlacementConfidence},
    placement_repository::PlacementRepository,
};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const STUDENT_LEARNING_PROFILE_SCHEMA_VERSION: u32 = 1;
pub const STUDENT_PROFILE_CONTEXT_VERSION: u32 = 1;
pub const STUDENT_PROFILE_CONTEXT_MAX_CHARS: usize = 1_500;
const PROFILE_KEY: &str = "default";
const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";

pub const LEARNING_GOALS: &[(&str, &str)] = &[
    ("general_fluency", "General Fluency"),
    ("everyday_conversation", "Everyday Conversation"),
    ("travel_english", "Travel English"),
    ("professional_english", "Professional English"),
    ("job_interview", "Job Interview"),
    ("academic_english", "Academic English"),
    ("grammar_accuracy", "Grammar Accuracy"),
    ("vocabulary_growth", "Vocabulary Growth"),
    ("speaking_confidence", "Speaking Confidence"),
    ("exam_preparation", "Exam Preparation"),
];

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentPlacementDto {
    pub attempt_id: String,
    pub estimated_level: CefrBand,
    pub confidence: PlacementConfidence,
    pub assessed_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentLearningProfileDto {
    pub schema_version: u32,
    pub current_placement: Option<CurrentPlacementDto>,
    pub target_level: Option<CefrBand>,
    pub learning_goals: Vec<String>,
    pub default_lesson_difficulty: LessonDifficulty,
    pub use_profile_in_lessons: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateStudentProfileRequest {
    pub target_level: Option<CefrBand>,
    pub learning_goals: Vec<String>,
    pub default_lesson_difficulty: LessonDifficulty,
    pub use_profile_in_lessons: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentProfileContextStatusDto {
    pub enabled: bool,
    pub placement_available: bool,
    pub context_available: bool,
    pub context_version: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonStudentProfileSnapshotDto {
    pub lesson_id: String,
    pub profile_schema_version: u32,
    pub profile_context_version: u32,
    pub context_enabled: bool,
    pub placement_attempt_id: Option<String>,
    pub estimated_cefr_level: Option<CefrBand>,
    pub placement_confidence: Option<PlacementConfidence>,
    pub target_cefr_level: Option<CefrBand>,
    pub learning_goals: Vec<String>,
    pub lesson_difficulty: LessonDifficulty,
    pub created_at: String,
}

#[derive(Clone, Debug)]
pub struct PreparedStudentProfile {
    pub context: Option<String>,
    pub snapshot: LessonStudentProfileSnapshotDto,
}

#[derive(Clone)]
pub struct StudentProfileRepository {
    database: PathBuf,
    placement: PlacementRepository,
}

impl StudentProfileRepository {
    pub fn new(database: PathBuf, placement: PlacementRepository) -> Self {
        Self {
            database,
            placement,
        }
    }

    pub fn get(&self) -> Result<StudentLearningProfileDto, String> {
        let connection = database::open(&self.database)?;
        let stored = connection
            .query_row(
                "SELECT schema_version,target_cefr_level,learning_goals_json,default_lesson_difficulty,use_profile_in_lessons FROM student_learning_profile WHERE profile_key=?1",
                [PROFILE_KEY],
                |row| Ok((row.get::<_, u32>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, bool>(4)?)),
            )
            .optional()
            .map_err(db_error)?;
        let (schema_version, target, goals_json, difficulty, enabled) = stored.unwrap_or((
            STUDENT_LEARNING_PROFILE_SCHEMA_VERSION,
            None,
            "[]".into(),
            "standard".into(),
            true,
        ));
        let goals: Vec<String> = serde_json::from_str(&goals_json)
            .map_err(|error| format!("Student profile goals are invalid: {error}"))?;
        validate_goals(&goals)?;
        let current_placement =
            self.placement
                .current_result()?
                .map(|result| CurrentPlacementDto {
                    attempt_id: result.attempt.id,
                    estimated_level: result.estimated_cefr_level,
                    confidence: result.confidence,
                    assessed_at: result
                        .attempt
                        .completed_at
                        .unwrap_or(result.attempt.started_at),
                });
        Ok(StudentLearningProfileDto {
            schema_version,
            current_placement,
            target_level: target.map(|value| CefrBand::parse(&value)).transpose()?,
            learning_goals: goals,
            default_lesson_difficulty: parse_difficulty(&difficulty)?,
            use_profile_in_lessons: enabled,
        })
    }

    pub fn update(
        &self,
        request: UpdateStudentProfileRequest,
    ) -> Result<StudentLearningProfileDto, String> {
        validate_goals(&request.learning_goals)?;
        let goals_json = serde_json::to_string(&request.learning_goals)
            .map_err(|error| format!("Could not encode student profile goals: {error}"))?;
        let connection = database::open(&self.database)?;
        connection.execute(
            &format!("INSERT INTO student_learning_profile(profile_key,schema_version,target_cefr_level,learning_goals_json,default_lesson_difficulty,use_profile_in_lessons,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,{NOW},{NOW}) ON CONFLICT(profile_key) DO UPDATE SET schema_version=excluded.schema_version,target_cefr_level=excluded.target_cefr_level,learning_goals_json=excluded.learning_goals_json,default_lesson_difficulty=excluded.default_lesson_difficulty,use_profile_in_lessons=excluded.use_profile_in_lessons,updated_at={NOW}"),
            params![PROFILE_KEY, STUDENT_LEARNING_PROFILE_SCHEMA_VERSION, request.target_level.map(CefrBand::as_str), goals_json, request.default_lesson_difficulty.as_str(), request.use_profile_in_lessons],
        ).map_err(db_error)?;
        self.get()
    }

    pub fn context_status(&self) -> Result<StudentProfileContextStatusDto, String> {
        let profile = self.get()?;
        Ok(StudentProfileContextStatusDto {
            enabled: profile.use_profile_in_lessons,
            placement_available: profile.current_placement.is_some(),
            context_available: profile.use_profile_in_lessons,
            context_version: STUDENT_PROFILE_CONTEXT_VERSION,
        })
    }

    pub fn prepare_for_lesson(
        &self,
        lesson_id: &str,
        difficulty: LessonDifficulty,
    ) -> Result<PreparedStudentProfile, String> {
        let profile = self.get()?;
        let context = if profile.use_profile_in_lessons {
            Some(build_student_profile_context(&profile, difficulty)?)
        } else {
            None
        };
        let placement = if profile.use_profile_in_lessons {
            profile.current_placement.as_ref()
        } else {
            None
        };
        Ok(PreparedStudentProfile {
            context,
            snapshot: LessonStudentProfileSnapshotDto {
                lesson_id: lesson_id.to_owned(),
                profile_schema_version: STUDENT_LEARNING_PROFILE_SCHEMA_VERSION,
                profile_context_version: STUDENT_PROFILE_CONTEXT_VERSION,
                context_enabled: profile.use_profile_in_lessons,
                placement_attempt_id: placement.map(|value| value.attempt_id.clone()),
                estimated_cefr_level: placement.map(|value| value.estimated_level),
                placement_confidence: placement.map(|value| value.confidence),
                target_cefr_level: if profile.use_profile_in_lessons {
                    profile.target_level
                } else {
                    None
                },
                learning_goals: if profile.use_profile_in_lessons {
                    profile.learning_goals
                } else {
                    Vec::new()
                },
                lesson_difficulty: difficulty,
                created_at: String::new(),
            },
        })
    }

    pub fn record_snapshot(
        &self,
        snapshot: &LessonStudentProfileSnapshotDto,
    ) -> Result<(), String> {
        let goals =
            serde_json::to_string(&snapshot.learning_goals).map_err(|error| error.to_string())?;
        let connection = database::open(&self.database)?;
        connection.execute(
            &format!("INSERT INTO lesson_student_profile_snapshot(lesson_id,profile_schema_version,profile_context_version,context_enabled,placement_attempt_id,estimated_cefr_level,placement_confidence,target_cefr_level,learning_goals_json,default_lesson_difficulty,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,{NOW})"),
            params![snapshot.lesson_id, snapshot.profile_schema_version, snapshot.profile_context_version, snapshot.context_enabled, snapshot.placement_attempt_id, snapshot.estimated_cefr_level.map(CefrBand::as_str), snapshot.placement_confidence.map(PlacementConfidence::as_str), snapshot.target_cefr_level.map(CefrBand::as_str), goals, snapshot.lesson_difficulty.as_str()],
        ).map_err(db_error)?;
        Ok(())
    }

    pub fn snapshot(
        &self,
        lesson_id: &str,
    ) -> Result<Option<LessonStudentProfileSnapshotDto>, String> {
        let connection = database::open(&self.database)?;
        connection.query_row(
            "SELECT lesson_id,profile_schema_version,profile_context_version,context_enabled,placement_attempt_id,estimated_cefr_level,placement_confidence,target_cefr_level,learning_goals_json,default_lesson_difficulty,created_at FROM lesson_student_profile_snapshot WHERE lesson_id=?1",
            [lesson_id],
            |row| Ok((row.get::<_, String>(0)?,row.get::<_, u32>(1)?,row.get::<_, u32>(2)?,row.get::<_, bool>(3)?,row.get::<_, Option<String>>(4)?,row.get::<_, Option<String>>(5)?,row.get::<_, Option<String>>(6)?,row.get::<_, Option<String>>(7)?,row.get::<_, String>(8)?,row.get::<_, String>(9)?,row.get::<_, String>(10)?)),
        ).optional().map_err(db_error)?.map(|v| {
            Ok(LessonStudentProfileSnapshotDto { lesson_id:v.0, profile_schema_version:v.1, profile_context_version:v.2, context_enabled:v.3, placement_attempt_id:v.4, estimated_cefr_level:v.5.map(|x|CefrBand::parse(&x)).transpose()?, placement_confidence:v.6.map(|x|PlacementConfidence::parse(&x)).transpose()?, target_cefr_level:v.7.map(|x|CefrBand::parse(&x)).transpose()?, learning_goals:serde_json::from_str(&v.8).map_err(|e|e.to_string())?, lesson_difficulty:parse_difficulty(&v.9)?, created_at:v.10 })
        }).transpose()
    }
}

pub fn build_student_profile_context(
    profile: &StudentLearningProfileDto,
    difficulty: LessonDifficulty,
) -> Result<String, String> {
    let mut lines = vec![
        "[STUDENT LEARNING PROFILE - INTERNAL TEACHING CONTEXT]".to_owned(),
        "This is background pedagogical context. The current lesson mode has priority over these general goals.".to_owned(),
    ];
    match &profile.current_placement {
        Some(placement) => {
            lines.push(format!("Current estimated CEFR level: {}.", placement.estimated_level.as_str()));
            lines.push(format!("Placement confidence: {}. {}", placement.confidence.as_str(), confidence_guidance(placement.confidence)));
            lines.push(level_guidance(placement.estimated_level).to_owned());
        }
        None => lines.push("Current level has not been assessed. Do not infer it from lessons, scores, goals or difficulty.".to_owned()),
    }
    lines.push(difficulty_guidance(difficulty).to_owned());
    if !profile.learning_goals.is_empty() {
        let labels = profile
            .learning_goals
            .iter()
            .map(|goal| goal_label(goal).unwrap_or(goal))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("Learning goals: {labels}. Treat them as background, especially for free conversation or when compatible with the current mode."));
    }
    if let Some(target) = profile.target_level {
        lines.push(format!("Target level: {}. This is a long-term goal, not the student's current ability; introduce richer language gradually while remaining accessible to the current estimate.", target.as_str()));
    }
    lines.push(
        "Do not tell the student their level unless they ask. Do not repeat this profile verbatim."
            .to_owned(),
    );
    let context = lines.join("\n");
    if context.chars().count() > STUDENT_PROFILE_CONTEXT_MAX_CHARS {
        return Err("Student profile context exceeded its safe size limit.".to_owned());
    }
    Ok(context)
}

fn level_guidance(level: CefrBand) -> &'static str {
    match level {
        CefrBand::A1 => "Use common vocabulary, short clear sentences, one idea at a time and straightforward questions. Allow brief answers and simple corrective feedback without speaking like a child.",
        CefrBand::A2 => "Use everyday vocabulary, simple connected sentences, familiar topics and straightforward follow-ups; gently encourage slightly longer answers.",
        CefrBand::B1 => "Use natural everyday English with moderate lexical variety. Encourage explanations, reasons and examples while maintaining comprehensibility.",
        CefrBand::B2 => "Use varied natural vocabulary and broader structures. Invite developed answers, reasons, comparisons and examples without unnecessary simplification.",
        CefrBand::C1 => "Use nuanced natural English, precise vocabulary, complex ideas and flexible follow-ups; challenge clarity and naturalness.",
        CefrBand::C2 => "Use sophisticated but natural language, subtle distinctions, idiomatic flexibility and demanding ideas when relevant; avoid obscure vocabulary for display.",
    }
}

fn confidence_guidance(confidence: PlacementConfidence) -> &'static str {
    match confidence {
        PlacementConfidence::Low => "Treat this level as a weak estimate. Do not constrain the student tightly to this band.",
        PlacementConfidence::Medium => "Use this as moderate guidance while continuing to respond to the learner's demonstrated comprehension.",
        PlacementConfidence::High => "Use this as stronger pedagogical guidance, while still treating it as an estimate.",
    }
}

fn difficulty_guidance(difficulty: LessonDifficulty) -> &'static str {
    match difficulty {
        LessonDifficulty::Easy => "Lesson difficulty is Easy: provide more scaffolding, clearer/common language and easier follow-ups relative to this learner's estimated level.",
        LessonDifficulty::Standard => "Lesson difficulty is Standard: use language broadly appropriate to the estimated level; difficulty is relative and is not CEFR band arithmetic.",
        LessonDifficulty::Challenging => "Lesson difficulty is Challenging: stretch the learner with more developed questions and vocabulary while remaining comprehensible; do not convert this into a higher CEFR band.",
    }
}

fn validate_goals(goals: &[String]) -> Result<(), String> {
    if goals.len() > 3 {
        return Err("Select no more than 3 learning goals.".to_owned());
    }
    let mut unique = std::collections::HashSet::new();
    for goal in goals {
        if goal_label(goal).is_none() {
            return Err(format!("Unknown learning goal: {goal}"));
        }
        if !unique.insert(goal) {
            return Err(format!("Duplicate learning goal: {goal}"));
        }
    }
    Ok(())
}

fn goal_label(goal: &str) -> Option<&'static str> {
    LEARNING_GOALS
        .iter()
        .find(|(id, _)| *id == goal)
        .map(|(_, label)| *label)
}
fn parse_difficulty(value: &str) -> Result<LessonDifficulty, String> {
    match value {
        "easy" => Ok(LessonDifficulty::Easy),
        "standard" => Ok(LessonDifficulty::Standard),
        "challenging" => Ok(LessonDifficulty::Challenging),
        _ => Err(format!("Invalid lesson difficulty: {value}")),
    }
}
fn db_error(error: rusqlite::Error) -> String {
    format!("Student profile database operation failed: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    fn repository() -> (PathBuf, StudentProfileRepository) {
        let directory =
            std::env::temp_dir().join(format!("student-profile-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("test.sqlite3");
        database::migrate(&path).unwrap();
        let placement = PlacementRepository::new(path.clone()).unwrap();
        (directory, StudentProfileRepository::new(path, placement))
    }

    #[test]
    fn defaults_and_validation_are_controlled() {
        let (directory, repository) = repository();
        let profile = repository.get().unwrap();
        assert_eq!(profile.target_level, None);
        assert!(profile.learning_goals.is_empty());
        assert_eq!(
            profile.default_lesson_difficulty,
            LessonDifficulty::Standard
        );
        assert!(profile.use_profile_in_lessons);
        let invalid = UpdateStudentProfileRequest {
            target_level: None,
            learning_goals: vec!["general_fluency".into(), "general_fluency".into()],
            default_lesson_difficulty: LessonDifficulty::Easy,
            use_profile_in_lessons: true,
        };
        assert!(repository
            .update(invalid)
            .unwrap_err()
            .contains("Duplicate"));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn context_separates_target_and_relative_difficulty() {
        let profile = StudentLearningProfileDto {
            schema_version: 1,
            current_placement: None,
            target_level: Some(CefrBand::B2),
            learning_goals: vec!["vocabulary_growth".into()],
            default_lesson_difficulty: LessonDifficulty::Standard,
            use_profile_in_lessons: true,
        };
        let context =
            build_student_profile_context(&profile, LessonDifficulty::Challenging).unwrap();
        assert!(context.contains("Current level has not been assessed"));
        assert!(context.contains("Target level: B2"));
        assert!(!context.contains("Current estimated CEFR level: B2"));
        assert!(context.contains("current lesson mode has priority"));
        assert!(context.chars().count() <= 1_500);
        assert_eq!(
            context,
            build_student_profile_context(&profile, LessonDifficulty::Challenging).unwrap()
        );
    }

    #[test]
    fn update_persists_and_rejects_unknown_or_excess_goals() {
        let (directory, repository) = repository();
        let updated = repository
            .update(UpdateStudentProfileRequest {
                target_level: Some(CefrBand::C1),
                learning_goals: vec!["general_fluency".into(), "speaking_confidence".into()],
                default_lesson_difficulty: LessonDifficulty::Challenging,
                use_profile_in_lessons: false,
            })
            .unwrap();
        assert_eq!(updated.target_level, Some(CefrBand::C1));
        let reopened = StudentProfileRepository::new(
            repository.database.clone(),
            PlacementRepository::new(repository.database.clone()).unwrap(),
        )
        .get()
        .unwrap();
        assert_eq!(reopened, updated);
        for goals in [
            vec!["unknown".into()],
            vec![
                "general_fluency".into(),
                "travel_english".into(),
                "grammar_accuracy".into(),
                "vocabulary_growth".into(),
            ],
        ] {
            assert!(repository
                .update(UpdateStudentProfileRequest {
                    target_level: None,
                    learning_goals: goals,
                    default_lesson_difficulty: LessonDifficulty::Standard,
                    use_profile_in_lessons: true
                })
                .is_err());
        }
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn current_level_tracks_only_latest_completed_placement_and_snapshot_stays_frozen() {
        let (directory, repository) = repository();
        let connection = database::open(&repository.database).unwrap();
        let insert = |id: &str, status: &str, level: Option<&str>, completed: Option<&str>| {
            connection.execute("INSERT INTO placement_attempt(id,status,test_version,question_bank_version,scoring_version,speaking_prompt_version,started_at,completed_at,overall_estimated_level,confidence,speaking_status,created_at,updated_at) VALUES(?1,?2,1,1,1,1,?3,?4,?5,?6,'skipped',?3,?3)", params![id,status,id,completed,level,level.map(|_|"medium")]).unwrap();
        };
        insert(
            "completed-a",
            "completed",
            Some("B1"),
            Some("2026-01-01T00:00:00Z"),
        );
        insert("retake", "in_progress", None, None);
        assert_eq!(
            repository
                .get()
                .unwrap()
                .current_placement
                .unwrap()
                .estimated_level,
            CefrBand::B1
        );
        connection.execute("INSERT INTO lesson(id,started_at,status,mode,whisper_model,whisper_threads,ollama_model,piper_voice,voice_engine_version,created_at,updated_at) VALUES('lesson-profile','now','starting','free_conversation','w',1,'q','p','v','now','now')", []).unwrap();
        drop(connection);
        let prepared = repository
            .prepare_for_lesson("lesson-profile", LessonDifficulty::Easy)
            .unwrap();
        repository.record_snapshot(&prepared.snapshot).unwrap();
        let connection = database::open(&repository.database).unwrap();
        connection
            .execute(
                "UPDATE placement_attempt SET status='abandoned' WHERE id='retake'",
                [],
            )
            .unwrap();
        connection.execute("INSERT INTO placement_attempt(id,status,test_version,question_bank_version,scoring_version,speaking_prompt_version,started_at,completed_at,overall_estimated_level,confidence,speaking_status,created_at,updated_at) VALUES('completed-b','completed',1,1,1,1,'2026-02-01','2026-02-01','B2','high','skipped','2026-02-01','2026-02-01')", []).unwrap();
        drop(connection);
        assert_eq!(
            repository
                .get()
                .unwrap()
                .current_placement
                .unwrap()
                .estimated_level,
            CefrBand::B2
        );
        let frozen = repository.snapshot("lesson-profile").unwrap().unwrap();
        assert_eq!(frozen.estimated_cefr_level, Some(CefrBand::B1));
        assert_eq!(frozen.lesson_difficulty, LessonDifficulty::Easy);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn low_confidence_context_is_bounded_and_contains_no_scores_or_personal_facts() {
        let profile = StudentLearningProfileDto {
            schema_version: 1,
            current_placement: Some(CurrentPlacementDto {
                attempt_id: "a".into(),
                estimated_level: CefrBand::B1,
                confidence: PlacementConfidence::Low,
                assessed_at: "now".into(),
            }),
            target_level: Some(CefrBand::C1),
            learning_goals: vec!["travel_english".into()],
            default_lesson_difficulty: LessonDifficulty::Standard,
            use_profile_in_lessons: true,
        };
        let context =
            build_student_profile_context(&profile, LessonDifficulty::Challenging).unwrap();
        assert!(context.contains("weak estimate"));
        assert!(context.contains("Target level: C1"));
        assert!(!context.contains("B1 +"));
        for forbidden in [
            "Overall 95",
            "Grammar 70",
            "John",
            "Brazil",
            "fishing",
            "transcript",
        ] {
            assert!(!context.contains(forbidden));
        }
        assert!(context.chars().count() <= STUDENT_PROFILE_CONTEXT_MAX_CHARS);
    }

    #[test]
    fn disabled_profile_snapshot_records_no_profile_metadata() {
        let (directory, repository) = repository();
        repository
            .update(UpdateStudentProfileRequest {
                target_level: Some(CefrBand::C2),
                learning_goals: vec!["academic_english".into()],
                default_lesson_difficulty: LessonDifficulty::Challenging,
                use_profile_in_lessons: false,
            })
            .unwrap();
        let prepared = repository
            .prepare_for_lesson("future-lesson", LessonDifficulty::Easy)
            .unwrap();
        assert!(prepared.context.is_none());
        assert!(!prepared.snapshot.context_enabled);
        assert_eq!(prepared.snapshot.target_cefr_level, None);
        assert!(prepared.snapshot.learning_goals.is_empty());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
