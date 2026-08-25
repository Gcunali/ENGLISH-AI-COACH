use crate::{
    database,
    lesson_analysis::{
        overall_score, LessonAnalysis, LessonAnalysisPayload, LessonAnalysisStatus,
        ANALYSIS_SCHEMA_VERSION, ANALYZER_PROMPT_VERSION,
    },
};
use rusqlite::{params, OptionalExtension, Row};
use std::path::PathBuf;

const NOW_SQL: &str = "strftime('%Y-%m-%dT%H:%M:%fZ', 'now')";

#[derive(Clone)]
pub struct LessonAnalysisRepository {
    database: PathBuf,
}

impl LessonAnalysisRepository {
    pub fn new(database: PathBuf) -> Self {
        Self { database }
    }

    pub fn create_pending(
        &self,
        lesson_id: &str,
        analyzer_model: &str,
    ) -> Result<LessonAnalysis, String> {
        if let Some(existing) = self.get_by_lesson(lesson_id)? {
            return Ok(existing);
        }
        let connection = database::open(&self.database)?;
        connection
            .execute(
                &format!(
                    "INSERT INTO lesson_analysis (
                       id, lesson_id, status, schema_version, prompt_version, analyzer_model,
                       created_at, updated_at
                     ) VALUES (?1, ?2, 'pending', ?3, ?4, ?5, {NOW_SQL}, {NOW_SQL})"
                ),
                params![
                    uuid::Uuid::new_v4().to_string(),
                    lesson_id,
                    ANALYSIS_SCHEMA_VERSION,
                    ANALYZER_PROMPT_VERSION,
                    analyzer_model,
                ],
            )
            .map_err(|error| format!("Could not create lesson analysis: {error}"))?;
        self.get_by_lesson(lesson_id)?
            .ok_or_else(|| "Created lesson analysis could not be read back.".to_owned())
    }

    pub fn mark_running(&self, lesson_id: &str) -> Result<LessonAnalysis, String> {
        let connection = database::open(&self.database)?;
        let changed = connection
            .execute(
                &format!(
                    "UPDATE lesson_analysis SET status = 'running', started_at = {NOW_SQL},
                     completed_at = NULL, error_message = NULL, updated_at = {NOW_SQL}
                     WHERE lesson_id = ?1 AND status = 'pending'"
                ),
                [lesson_id],
            )
            .map_err(|error| format!("Could not start lesson analysis: {error}"))?;
        if changed == 0 {
            return Err("Lesson analysis is not pending.".to_owned());
        }
        self.get_by_lesson(lesson_id)?
            .ok_or_else(|| "Running lesson analysis was not found.".to_owned())
    }

    pub fn save_completed(
        &self,
        lesson_id: &str,
        payload: &LessonAnalysisPayload,
        canonical_json: &str,
    ) -> Result<LessonAnalysis, String> {
        let overall = overall_score(&payload.scores);
        let connection = database::open(&self.database)?;
        let changed = connection
            .execute(
                &format!(
                    "UPDATE lesson_analysis SET status = 'completed', completed_at = {NOW_SQL},
                     overall_score = ?2, fluency_score = ?3, grammar_score = ?4,
                     vocabulary_score = ?5, comprehension_score = ?6, interaction_score = ?7,
                     pronunciation_score = NULL, summary = ?8, raw_json = ?9,
                     error_message = NULL, updated_at = {NOW_SQL}
                     WHERE lesson_id = ?1 AND status = 'running'"
                ),
                params![
                    lesson_id,
                    overall,
                    payload.scores.fluency,
                    payload.scores.grammar,
                    payload.scores.vocabulary,
                    payload.scores.comprehension,
                    payload.scores.interaction,
                    payload.summary,
                    canonical_json,
                ],
            )
            .map_err(|error| format!("Could not persist completed analysis: {error}"))?;
        if changed == 0 {
            return Err("Lesson analysis is not running.".to_owned());
        }
        self.get_by_lesson(lesson_id)?
            .ok_or_else(|| "Completed lesson analysis was not found.".to_owned())
    }

    pub fn mark_failed(&self, lesson_id: &str, error: &str) -> Result<LessonAnalysis, String> {
        let connection = database::open(&self.database)?;
        connection
            .execute(
                &format!(
                    "UPDATE lesson_analysis SET status = 'failed', completed_at = {NOW_SQL},
                     error_message = ?2, updated_at = {NOW_SQL}
                     WHERE lesson_id = ?1 AND status IN ('pending', 'running')"
                ),
                params![lesson_id, compact_error(error)],
            )
            .map_err(|db_error| format!("Could not mark analysis as failed: {db_error}"))?;
        self.get_by_lesson(lesson_id)?
            .ok_or_else(|| "Failed lesson analysis was not found.".to_owned())
    }

    pub fn mark_insufficient_data(
        &self,
        lesson_id: &str,
        student_turns: u32,
    ) -> Result<LessonAnalysis, String> {
        let connection = database::open(&self.database)?;
        connection
            .execute(
                &format!(
                    "UPDATE lesson_analysis SET status = 'insufficient_data', completed_at = {NOW_SQL},
                     error_message = ?2, updated_at = {NOW_SQL}
                     WHERE lesson_id = ?1 AND status = 'pending'"
                ),
                params![
                    lesson_id,
                    format!(
                        "At least 3 valid student turns are required; this lesson has {student_turns}."
                    )
                ],
            )
            .map_err(|error| format!("Could not mark insufficient analysis data: {error}"))?;
        self.get_by_lesson(lesson_id)?
            .ok_or_else(|| "Insufficient-data analysis was not found.".to_owned())
    }

    pub fn reset_failed_for_retry(&self, lesson_id: &str) -> Result<LessonAnalysis, String> {
        let connection = database::open(&self.database)?;
        let changed = connection
            .execute(
                &format!(
                    "UPDATE lesson_analysis SET status = 'pending', started_at = NULL,
                     completed_at = NULL, overall_score = NULL, fluency_score = NULL,
                     grammar_score = NULL, vocabulary_score = NULL, comprehension_score = NULL,
                     interaction_score = NULL, pronunciation_score = NULL, summary = NULL,
                     raw_json = NULL, error_message = NULL, updated_at = {NOW_SQL}
                     WHERE lesson_id = ?1 AND status = 'failed'"
                ),
                [lesson_id],
            )
            .map_err(|error| format!("Could not reset failed analysis: {error}"))?;
        if changed == 0 {
            return Err("Only a failed analysis can be retried.".to_owned());
        }
        self.get_by_lesson(lesson_id)?
            .ok_or_else(|| "Retryable lesson analysis was not found.".to_owned())
    }

    pub fn recover_interrupted(&self) -> Result<usize, String> {
        let connection = database::open(&self.database)?;
        connection
            .execute(
                &format!(
                    "UPDATE lesson_analysis SET status = 'failed', completed_at = {NOW_SQL},
                     error_message = 'analysis interrupted', updated_at = {NOW_SQL}
                     WHERE status IN ('pending', 'running')"
                ),
                [],
            )
            .map_err(|error| format!("Could not recover interrupted analyses: {error}"))
    }

    pub fn get_by_lesson(&self, lesson_id: &str) -> Result<Option<LessonAnalysis>, String> {
        let connection = database::open(&self.database)?;
        connection
            .query_row(
                "SELECT id, lesson_id, status, schema_version, prompt_version, analyzer_model,
                 started_at, completed_at, overall_score, raw_json, error_message,
                 created_at, updated_at FROM lesson_analysis WHERE lesson_id = ?1",
                [lesson_id],
                row_to_analysis,
            )
            .optional()
            .map_err(|error| format!("Could not read lesson analysis: {error}"))
    }

    #[cfg(test)]
    fn raw_json(&self, lesson_id: &str) -> Option<String> {
        database::open(&self.database)
            .unwrap()
            .query_row(
                "SELECT raw_json FROM lesson_analysis WHERE lesson_id = ?1",
                [lesson_id],
                |row| row.get(0),
            )
            .unwrap()
    }
}

fn row_to_analysis(row: &Row<'_>) -> rusqlite::Result<LessonAnalysis> {
    let raw_json: Option<String> = row.get(9)?;
    let payload = raw_json
        .as_deref()
        .map(serde_json::from_str::<LessonAnalysisPayload>)
        .transpose()
        .map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                9,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
    Ok(LessonAnalysis {
        id: row.get(0)?,
        lesson_id: row.get(1)?,
        status: LessonAnalysisStatus::parse(&row.get::<_, String>(2)?)?,
        schema_version: row.get(3)?,
        prompt_version: row.get(4)?,
        analyzer_model: row.get(5)?,
        started_at: row.get(6)?,
        completed_at: row.get(7)?,
        overall_score: row.get(8)?,
        scores: payload.as_ref().map(|value| value.scores.clone()),
        strengths: payload
            .as_ref()
            .map(|value| value.strengths.clone())
            .unwrap_or_default(),
        priority_improvements: payload
            .as_ref()
            .map(|value| value.priority_improvements.clone())
            .unwrap_or_default(),
        corrections: payload
            .as_ref()
            .map(|value| value.corrections.clone())
            .unwrap_or_default(),
        natural_alternatives: payload
            .as_ref()
            .map(|value| value.natural_alternatives.clone())
            .unwrap_or_default(),
        vocabulary: payload
            .as_ref()
            .map(|value| value.vocabulary.clone())
            .unwrap_or_default(),
        recurring_patterns: payload
            .as_ref()
            .map(|value| value.recurring_patterns.clone())
            .unwrap_or_default(),
        next_lesson_recommendations: payload
            .as_ref()
            .map(|value| value.next_lesson_recommendations.clone())
            .unwrap_or_default(),
        summary: payload.as_ref().map(|value| value.summary.clone()),
        pronunciation_available: payload
            .as_ref()
            .map(|value| value.pronunciation_available)
            .unwrap_or(false),
        error_message: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn compact_error(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(1_000)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        lesson_analysis::{
            LessonAnalysisCorrection, LessonAnalysisCorrectionCategory, LessonAnalysisScores,
        },
        lesson_repository::{LessonRepository, NewLesson},
    };

    fn repositories() -> (PathBuf, LessonRepository, LessonAnalysisRepository, String) {
        let directory = std::env::temp_dir().join(format!(
            "english-ai-coach-analysis-repository-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("analysis.sqlite3");
        database::migrate(&path).unwrap();
        let lessons = LessonRepository::new(path.clone());
        let lesson = lessons
            .create_lesson(&NewLesson {
                topic: None,
                mode: "free_conversation".to_owned(),
                whisper_model: "whisper".to_owned(),
                whisper_threads: 12,
                ollama_model: "qwen3.5:4b".to_owned(),
                piper_voice: "lessac".to_owned(),
                voice_engine_version: "voice-v2".to_owned(),
            })
            .unwrap();
        lessons.mark_lesson_active(&lesson.id).unwrap();
        lessons.complete_lesson(&lesson.id).unwrap();
        (
            directory,
            lessons,
            LessonAnalysisRepository::new(path),
            lesson.id,
        )
    }

    fn payload() -> LessonAnalysisPayload {
        LessonAnalysisPayload {
            schema_version: 1,
            scores: LessonAnalysisScores {
                fluency: 71,
                grammar: 62,
                vocabulary: 68,
                comprehension: 83,
                interaction: 84,
                pronunciation: None,
            },
            strengths: vec![],
            priority_improvements: vec![],
            corrections: vec![LessonAnalysisCorrection {
                original: "I play yesterday".to_owned(),
                corrected: "I played yesterday".to_owned(),
                explanation: "Use o passado.".to_owned(),
                category: LessonAnalysisCorrectionCategory::VerbTense,
            }],
            natural_alternatives: vec![],
            vocabulary: vec![],
            recurring_patterns: vec![],
            next_lesson_recommendations: vec![],
            summary: "Resumo pedagógico.".to_owned(),
            pronunciation_available: false,
        }
    }

    #[test]
    fn persists_completed_analysis_raw_json_and_deterministic_overall() {
        let (directory, _lessons, repository, lesson_id) = repositories();
        let pending = repository.create_pending(&lesson_id, "qwen3.5:4b").unwrap();
        assert_eq!(pending.status, LessonAnalysisStatus::Pending);
        assert_eq!(
            repository.mark_running(&lesson_id).unwrap().status,
            LessonAnalysisStatus::Running
        );
        let payload = payload();
        let raw_json = serde_json::to_string(&payload).unwrap();
        let completed = repository
            .save_completed(&lesson_id, &payload, &raw_json)
            .unwrap();
        assert_eq!(completed.status, LessonAnalysisStatus::Completed);
        assert_eq!(completed.overall_score, Some(74));
        assert_eq!(completed.scores.unwrap().pronunciation, None);
        assert_eq!(
            repository.raw_json(&lesson_id).as_deref(),
            Some(raw_json.as_str())
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn prevents_duplicate_analysis_for_one_lesson() {
        let (directory, _lessons, repository, lesson_id) = repositories();
        let first = repository.create_pending(&lesson_id, "qwen3.5:4b").unwrap();
        let second = repository.create_pending(&lesson_id, "qwen3.5:4b").unwrap();
        assert_eq!(first.id, second.id);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn failed_analysis_can_be_explicitly_retried() {
        let (directory, _lessons, repository, lesson_id) = repositories();
        repository.create_pending(&lesson_id, "qwen3.5:4b").unwrap();
        repository.mark_running(&lesson_id).unwrap();
        assert_eq!(
            repository
                .mark_failed(&lesson_id, "bad json")
                .unwrap()
                .status,
            LessonAnalysisStatus::Failed
        );
        assert_eq!(
            repository
                .reset_failed_for_retry(&lesson_id)
                .unwrap()
                .status,
            LessonAnalysisStatus::Pending
        );
        assert_eq!(
            repository.mark_running(&lesson_id).unwrap().status,
            LessonAnalysisStatus::Running
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn recovers_pending_and_running_analysis_as_failed() {
        let (directory, lessons, repository, first_lesson_id) = repositories();
        repository
            .create_pending(&first_lesson_id, "qwen3.5:4b")
            .unwrap();
        repository.mark_running(&first_lesson_id).unwrap();

        let second = lessons
            .create_lesson(&NewLesson {
                topic: None,
                mode: "free_conversation".to_owned(),
                whisper_model: "whisper".to_owned(),
                whisper_threads: 12,
                ollama_model: "qwen3.5:4b".to_owned(),
                piper_voice: "lessac".to_owned(),
                voice_engine_version: "voice-v2".to_owned(),
            })
            .unwrap();
        lessons.mark_lesson_active(&second.id).unwrap();
        lessons.complete_lesson(&second.id).unwrap();
        repository.create_pending(&second.id, "qwen3.5:4b").unwrap();

        assert_eq!(repository.recover_interrupted().unwrap(), 2);
        for lesson_id in [first_lesson_id, second.id] {
            let analysis = repository.get_by_lesson(&lesson_id).unwrap().unwrap();
            assert_eq!(analysis.status, LessonAnalysisStatus::Failed);
            assert_eq!(
                analysis.error_message.as_deref(),
                Some("analysis interrupted")
            );
        }
        std::fs::remove_dir_all(directory).unwrap();
    }
}
