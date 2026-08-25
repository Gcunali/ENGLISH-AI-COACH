use crate::database;
use rusqlite::{params, OptionalExtension, Row, Transaction};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const NOW_SQL: &str = "strftime('%Y-%m-%dT%H:%M:%fZ', 'now')";
const CORRECTION_METHOD: &str = "teacher_cue_v1";

#[derive(Clone)]
pub struct LessonRepository {
    database: PathBuf,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonStatus {
    Starting,
    Active,
    Completed,
    Interrupted,
    Failed,
}

impl LessonStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Interrupted => "interrupted",
            Self::Failed => "failed",
        }
    }

    fn parse(value: &str) -> rusqlite::Result<Self> {
        match value {
            "starting" => Ok(Self::Starting),
            "active" => Ok(Self::Active),
            "completed" => Ok(Self::Completed),
            "interrupted" => Ok(Self::Interrupted),
            "failed" => Ok(Self::Failed),
            _ => Err(rusqlite::Error::InvalidColumnType(
                3,
                "status".to_owned(),
                rusqlite::types::Type::Text,
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Lesson {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: LessonStatus,
    pub topic: Option<String>,
    pub mode: String,
    pub duration_seconds: Option<i64>,
    pub student_turn_count: u32,
    pub teacher_turn_count: u32,
    pub correction_count: u32,
    pub whisper_model: String,
    pub whisper_threads: u16,
    pub ollama_model: String,
    pub piper_voice: String,
    pub voice_engine_version: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug)]
pub struct NewLesson {
    pub topic: Option<String>,
    pub mode: String,
    pub whisper_model: String,
    pub whisper_threads: u16,
    pub ollama_model: String,
    pub piper_voice: String,
    pub voice_engine_version: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptMessage {
    pub id: String,
    pub lesson_id: String,
    pub sequence_index: u32,
    pub turn_index: u32,
    pub role: String,
    pub text: String,
    pub source: String,
    pub engine_event_type: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectionCandidate {
    pub id: String,
    pub lesson_id: String,
    pub student_message_id: String,
    pub teacher_message_id: String,
    pub student_text: String,
    pub teacher_response_text: String,
    pub detection_method: String,
    pub created_at: String,
}

#[derive(Clone, Debug)]
pub struct TeacherPersistence {
    pub message: TranscriptMessage,
    pub correction_candidate: Option<CorrectionCandidate>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonSummary {
    pub lesson_id: String,
    pub status: LessonStatus,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_seconds: Option<i64>,
    pub student_turns: u32,
    pub teacher_turns: u32,
    pub correction_candidates: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonAnalysisInput {
    pub lesson: Lesson,
    pub transcript: Vec<TranscriptMessage>,
    pub correction_candidates: Vec<CorrectionCandidate>,
}

impl LessonRepository {
    pub fn new(database: PathBuf) -> Self {
        Self { database }
    }

    pub fn database_path(&self) -> &Path {
        &self.database
    }

    pub fn create_lesson(&self, input: &NewLesson) -> Result<Lesson, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let connection = database::open(&self.database)?;
        connection
            .execute(
                &format!(
                    "INSERT INTO lesson (
                      id, started_at, status, topic, mode, whisper_model, whisper_threads,
                      ollama_model, piper_voice, voice_engine_version, created_at, updated_at
                    ) VALUES (?1, {NOW_SQL}, 'starting', ?2, ?3, ?4, ?5, ?6, ?7, ?8, {NOW_SQL}, {NOW_SQL})"
                ),
                params![
                    id,
                    input.topic,
                    input.mode,
                    input.whisper_model,
                    input.whisper_threads,
                    input.ollama_model,
                    input.piper_voice,
                    input.voice_engine_version,
                ],
            )
            .map_err(|error| format!("Could not create lesson: {error}"))?;
        self.get_lesson(&id)?
            .ok_or_else(|| "Created lesson could not be read back.".to_owned())
    }

    pub fn mark_lesson_active(&self, lesson_id: &str) -> Result<Lesson, String> {
        let connection = database::open(&self.database)?;
        let changed = connection
            .execute(
                &format!(
                    "UPDATE lesson SET status = 'active', updated_at = {NOW_SQL}
                     WHERE id = ?1 AND status = 'starting'"
                ),
                [lesson_id],
            )
            .map_err(|error| format!("Could not activate lesson: {error}"))?;
        if changed == 0 {
            return Err("Lesson is not in starting state.".to_owned());
        }
        self.get_lesson(lesson_id)?
            .ok_or_else(|| "Activated lesson was not found.".to_owned())
    }

    pub fn insert_student_message(
        &self,
        lesson_id: &str,
        text: &str,
    ) -> Result<Option<TranscriptMessage>, String> {
        if is_technical_transcript(text) {
            return Ok(None);
        }
        let mut connection = database::open(&self.database)?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("Could not start student message transaction: {error}"))?;
        let student_count: u32 = transaction
            .query_row(
                "SELECT student_turn_count FROM lesson WHERE id = ?1 AND status IN ('starting', 'active')",
                [lesson_id],
                |row| row.get(0),
            )
            .map_err(|error| format!("Active lesson was not found: {error}"))?;
        let message = insert_message(
            &transaction,
            lesson_id,
            student_count + 1,
            "student",
            text,
            "whisper",
            "transcript",
        )?;
        transaction
            .execute(
                &format!(
                    "UPDATE lesson SET student_turn_count = student_turn_count + 1,
                     updated_at = {NOW_SQL} WHERE id = ?1"
                ),
                [lesson_id],
            )
            .map_err(|error| format!("Could not update student turn count: {error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("Could not commit student message: {error}"))?;
        Ok(Some(message))
    }

    pub fn insert_teacher_response(
        &self,
        lesson_id: &str,
        student_message_id: &str,
        text: &str,
    ) -> Result<TeacherPersistence, String> {
        let mut connection = database::open(&self.database)?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("Could not start teacher response transaction: {error}"))?;
        let (student_text, turn_index): (String, u32) = transaction
            .query_row(
                "SELECT text, turn_index FROM transcript_message
                 WHERE id = ?1 AND lesson_id = ?2 AND role = 'student'",
                params![student_message_id, lesson_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|error| {
                format!("Student message for teacher response was not found: {error}")
            })?;
        let teacher = insert_message(
            &transaction,
            lesson_id,
            turn_index,
            "teacher",
            text,
            "ollama",
            "teacher_response",
        )?;
        transaction
            .execute(
                &format!(
                    "UPDATE lesson SET teacher_turn_count = teacher_turn_count + 1,
                     updated_at = {NOW_SQL} WHERE id = ?1"
                ),
                [lesson_id],
            )
            .map_err(|error| format!("Could not update teacher turn count: {error}"))?;

        let correction_candidate = if detect_correction_candidate(text) {
            let candidate = CorrectionCandidate {
                id: uuid::Uuid::new_v4().to_string(),
                lesson_id: lesson_id.to_owned(),
                student_message_id: student_message_id.to_owned(),
                teacher_message_id: teacher.id.clone(),
                student_text,
                teacher_response_text: text.to_owned(),
                detection_method: CORRECTION_METHOD.to_owned(),
                created_at: current_timestamp(&transaction)?,
            };
            transaction
                .execute(
                    "INSERT INTO correction_candidate (
                       id, lesson_id, student_message_id, teacher_message_id, student_text,
                       teacher_response_text, detection_method, created_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        candidate.id,
                        candidate.lesson_id,
                        candidate.student_message_id,
                        candidate.teacher_message_id,
                        candidate.student_text,
                        candidate.teacher_response_text,
                        candidate.detection_method,
                        candidate.created_at,
                    ],
                )
                .map_err(|error| format!("Could not insert correction candidate: {error}"))?;
            transaction
                .execute(
                    &format!(
                        "UPDATE lesson SET correction_count = (
                           SELECT COUNT(*) FROM correction_candidate WHERE lesson_id = ?1
                         ),
                         updated_at = {NOW_SQL} WHERE id = ?1"
                    ),
                    [lesson_id],
                )
                .map_err(|error| format!("Could not update correction count: {error}"))?;
            Some(candidate)
        } else {
            None
        };
        transaction
            .commit()
            .map_err(|error| format!("Could not commit teacher response: {error}"))?;
        Ok(TeacherPersistence {
            message: teacher,
            correction_candidate,
        })
    }

    pub fn complete_lesson(&self, lesson_id: &str) -> Result<LessonSummary, String> {
        self.finish_lesson(lesson_id, LessonStatus::Completed, None)
    }

    pub fn interrupt_lesson(&self, lesson_id: &str) -> Result<LessonSummary, String> {
        self.finish_lesson(lesson_id, LessonStatus::Interrupted, None)
    }

    pub fn fail_lesson(&self, lesson_id: &str, error: &str) -> Result<LessonSummary, String> {
        self.finish_lesson(lesson_id, LessonStatus::Failed, Some(error))
    }

    fn finish_lesson(
        &self,
        lesson_id: &str,
        status: LessonStatus,
        error: Option<&str>,
    ) -> Result<LessonSummary, String> {
        let connection = database::open(&self.database)?;
        connection
            .execute(
                &format!(
                    "UPDATE lesson SET status = ?2, ended_at = {NOW_SQL},
                     duration_seconds = MAX(0, CAST((julianday({NOW_SQL}) - julianday(started_at)) * 86400 AS INTEGER)),
                     correction_count = (
                       SELECT COUNT(*) FROM correction_candidate WHERE lesson_id = ?1
                     ), error_message = ?3, updated_at = {NOW_SQL}
                     WHERE id = ?1 AND status IN ('starting', 'active')"
                ),
                params![lesson_id, status.as_str(), error],
            )
            .map_err(|db_error| format!("Could not finish lesson: {db_error}"))?;
        self.get_lesson_summary(lesson_id)?
            .ok_or_else(|| "Finished lesson was not found.".to_owned())
    }

    pub fn recover_stale_lessons(&self) -> Result<usize, String> {
        let connection = database::open(&self.database)?;
        connection
            .execute(
                &format!(
                    "UPDATE lesson SET status = 'interrupted', ended_at = {NOW_SQL},
                     duration_seconds = MAX(0, CAST((julianday({NOW_SQL}) - julianday(started_at)) * 86400 AS INTEGER)),
                     updated_at = {NOW_SQL}
                     WHERE status IN ('starting', 'active')"
                ),
                [],
            )
            .map_err(|error| format!("Could not recover interrupted lessons: {error}"))
    }

    pub fn get_lesson(&self, lesson_id: &str) -> Result<Option<Lesson>, String> {
        let connection = database::open(&self.database)?;
        connection
            .query_row(
                "SELECT id, started_at, ended_at, status, topic, mode, duration_seconds,
                 student_turn_count, teacher_turn_count, correction_count, whisper_model,
                 whisper_threads, ollama_model, piper_voice, voice_engine_version,
                 error_message, created_at, updated_at FROM lesson WHERE id = ?1",
                [lesson_id],
                row_to_lesson,
            )
            .optional()
            .map_err(|error| format!("Could not read lesson: {error}"))
    }

    pub fn get_lesson_messages(&self, lesson_id: &str) -> Result<Vec<TranscriptMessage>, String> {
        let connection = database::open(&self.database)?;
        let mut statement = connection
            .prepare(
                "SELECT id, lesson_id, sequence_index, turn_index, role, text, source,
                 engine_event_type, created_at FROM transcript_message
                 WHERE lesson_id = ?1 ORDER BY sequence_index",
            )
            .map_err(|error| format!("Could not prepare transcript query: {error}"))?;
        let messages = statement
            .query_map([lesson_id], row_to_message)
            .map_err(|error| format!("Could not query transcript: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Could not read transcript: {error}"))?;
        Ok(messages)
    }

    pub fn get_correction_candidates(
        &self,
        lesson_id: &str,
    ) -> Result<Vec<CorrectionCandidate>, String> {
        let connection = database::open(&self.database)?;
        let mut statement = connection
            .prepare(
                "SELECT id, lesson_id, student_message_id, teacher_message_id, student_text,
                 teacher_response_text, detection_method, created_at FROM correction_candidate
                 WHERE lesson_id = ?1 ORDER BY created_at, id",
            )
            .map_err(|error| format!("Could not prepare correction query: {error}"))?;
        let candidates = statement
            .query_map([lesson_id], row_to_correction)
            .map_err(|error| format!("Could not query corrections: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Could not read corrections: {error}"))?;
        Ok(candidates)
    }

    pub fn get_lesson_summary(&self, lesson_id: &str) -> Result<Option<LessonSummary>, String> {
        Ok(self.get_lesson(lesson_id)?.map(|lesson| LessonSummary {
            lesson_id: lesson.id,
            status: lesson.status,
            started_at: lesson.started_at,
            ended_at: lesson.ended_at,
            duration_seconds: lesson.duration_seconds,
            student_turns: lesson.student_turn_count,
            teacher_turns: lesson.teacher_turn_count,
            correction_candidates: lesson.correction_count,
        }))
    }

    pub fn get_analysis_input(&self, lesson_id: &str) -> Result<LessonAnalysisInput, String> {
        let lesson = self
            .get_lesson(lesson_id)?
            .ok_or_else(|| "Lesson was not found.".to_owned())?;
        Ok(LessonAnalysisInput {
            transcript: self.get_lesson_messages(lesson_id)?,
            correction_candidates: self.get_correction_candidates(lesson_id)?,
            lesson,
        })
    }
}

fn insert_message(
    transaction: &Transaction<'_>,
    lesson_id: &str,
    turn_index: u32,
    role: &str,
    text: &str,
    source: &str,
    event_type: &str,
) -> Result<TranscriptMessage, String> {
    let sequence_index: u32 = transaction
        .query_row(
            "SELECT COALESCE(MAX(sequence_index), 0) + 1 FROM transcript_message WHERE lesson_id = ?1",
            [lesson_id],
            |row| row.get(0),
        )
        .map_err(|error| format!("Could not calculate transcript sequence: {error}"))?;
    let message = TranscriptMessage {
        id: uuid::Uuid::new_v4().to_string(),
        lesson_id: lesson_id.to_owned(),
        sequence_index,
        turn_index,
        role: role.to_owned(),
        text: text.to_owned(),
        source: source.to_owned(),
        engine_event_type: event_type.to_owned(),
        created_at: current_timestamp(transaction)?,
    };
    transaction
        .execute(
            "INSERT INTO transcript_message (
               id, lesson_id, sequence_index, turn_index, role, text, source,
               engine_event_type, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                message.id,
                message.lesson_id,
                message.sequence_index,
                message.turn_index,
                message.role,
                message.text,
                message.source,
                message.engine_event_type,
                message.created_at,
            ],
        )
        .map_err(|error| format!("Could not insert transcript message: {error}"))?;
    Ok(message)
}

fn current_timestamp(transaction: &Transaction<'_>) -> Result<String, String> {
    transaction
        .query_row(&format!("SELECT {NOW_SQL}"), [], |row| row.get(0))
        .map_err(|error| format!("Could not create local timestamp: {error}"))
}

pub fn detect_correction_candidate(response: &str) -> bool {
    let normalized = normalize_for_correction_detection(response);
    [
        "small correction:",
        "a more natural way to say",
        "a better way to say",
        "you can say",
        "i think you meant",
        "i think you mean",
        "if you mean",
        "the natural way to say",
    ]
    .iter()
    .any(|cue| normalized.contains(cue))
}

fn normalize_for_correction_detection(response: &str) -> String {
    response
        .chars()
        .filter(|character| !matches!(character, '*' | '_' | '`'))
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

pub fn is_technical_transcript(text: &str) -> bool {
    matches!(
        text.trim().to_ascii_uppercase().as_str(),
        "[INAUDIBLE]" | "[SILENCE]" | "[BLANK_AUDIO]" | "[BLANK AUDIO]"
    )
}

fn row_to_lesson(row: &Row<'_>) -> rusqlite::Result<Lesson> {
    Ok(Lesson {
        id: row.get(0)?,
        started_at: row.get(1)?,
        ended_at: row.get(2)?,
        status: LessonStatus::parse(&row.get::<_, String>(3)?)?,
        topic: row.get(4)?,
        mode: row.get(5)?,
        duration_seconds: row.get(6)?,
        student_turn_count: row.get(7)?,
        teacher_turn_count: row.get(8)?,
        correction_count: row.get(9)?,
        whisper_model: row.get(10)?,
        whisper_threads: row.get(11)?,
        ollama_model: row.get(12)?,
        piper_voice: row.get(13)?,
        voice_engine_version: row.get(14)?,
        error_message: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

fn row_to_message(row: &Row<'_>) -> rusqlite::Result<TranscriptMessage> {
    Ok(TranscriptMessage {
        id: row.get(0)?,
        lesson_id: row.get(1)?,
        sequence_index: row.get(2)?,
        turn_index: row.get(3)?,
        role: row.get(4)?,
        text: row.get(5)?,
        source: row.get(6)?,
        engine_event_type: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn row_to_correction(row: &Row<'_>) -> rusqlite::Result<CorrectionCandidate> {
    Ok(CorrectionCandidate {
        id: row.get(0)?,
        lesson_id: row.get(1)?,
        student_message_id: row.get(2)?,
        teacher_message_id: row.get(3)?,
        student_text: row.get(4)?,
        teacher_response_text: row.get(5)?,
        detection_method: row.get(6)?,
        created_at: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository() -> (PathBuf, LessonRepository) {
        let directory =
            std::env::temp_dir().join(format!("english-ai-coach-lessons-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let database = directory.join("lessons.sqlite3");
        database::migrate(&database).unwrap();
        (directory, LessonRepository::new(database))
    }

    fn metadata() -> NewLesson {
        NewLesson {
            topic: None,
            mode: "free_conversation".to_owned(),
            whisper_model: "ggml-small.en-q5_1.bin".to_owned(),
            whisper_threads: 12,
            ollama_model: "qwen3.5:4b".to_owned(),
            piper_voice: "en_US-lessac-medium".to_owned(),
            voice_engine_version: "voice_v2_bridge_v1".to_owned(),
        }
    }

    #[test]
    fn persists_ordered_transcript_correction_and_summary() {
        let (directory, repository) = repository();
        let lesson = repository.create_lesson(&metadata()).unwrap();
        assert_eq!(lesson.status, LessonStatus::Starting);
        assert_eq!(
            repository.mark_lesson_active(&lesson.id).unwrap().status,
            LessonStatus::Active
        );
        let student_one = repository
            .insert_student_message(&lesson.id, "I'm terrible cooking.")
            .unwrap()
            .unwrap();
        let teacher_one = repository
            .insert_teacher_response(
                &lesson.id,
                &student_one.id,
                "A more natural way to say that is 'I'm terrible at cooking.' What do you cook?",
            )
            .unwrap();
        assert!(teacher_one.correction_candidate.is_some());
        let student_two = repository
            .insert_student_message(&lesson.id, "I usually make pasta.")
            .unwrap()
            .unwrap();
        repository
            .insert_teacher_response(
                &lesson.id,
                &student_two.id,
                "That sounds great! What sauce do you use?",
            )
            .unwrap();

        let transcript = repository.get_lesson_messages(&lesson.id).unwrap();
        assert_eq!(
            transcript
                .iter()
                .map(|message| (message.sequence_index, message.role.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (1, "student"),
                (2, "teacher"),
                (3, "student"),
                (4, "teacher")
            ]
        );
        let summary = repository.complete_lesson(&lesson.id).unwrap();
        assert_eq!(summary.status, LessonStatus::Completed);
        assert_eq!(summary.student_turns, 2);
        assert_eq!(summary.teacher_turns, 2);
        assert_eq!(summary.correction_candidates, 1);
        let analysis = repository.get_analysis_input(&lesson.id).unwrap();
        assert_eq!(analysis.transcript.len(), 4);
        assert_eq!(analysis.correction_candidates.len(), 1);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn filters_only_clear_technical_transcripts() {
        let (directory, repository) = repository();
        let lesson = repository.create_lesson(&metadata()).unwrap();
        repository.mark_lesson_active(&lesson.id).unwrap();
        for value in [
            "[INAUDIBLE]",
            "[silence]",
            " [BLANK_AUDIO] ",
            "[blank audio]",
        ] {
            assert!(repository
                .insert_student_message(&lesson.id, value)
                .unwrap()
                .is_none());
        }
        assert!(repository
            .insert_student_message(&lesson.id, "Thank you for watching")
            .unwrap()
            .is_some());
        assert_eq!(repository.get_lesson_messages(&lesson.id).unwrap().len(), 1);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn interrupts_and_fails_lessons() {
        let (directory, repository) = repository();
        let interrupted = repository.create_lesson(&metadata()).unwrap();
        repository.mark_lesson_active(&interrupted.id).unwrap();
        assert_eq!(
            repository.interrupt_lesson(&interrupted.id).unwrap().status,
            LessonStatus::Interrupted
        );
        let failed = repository.create_lesson(&metadata()).unwrap();
        assert_eq!(
            repository
                .fail_lesson(&failed.id, "engine exited")
                .unwrap()
                .status,
            LessonStatus::Failed
        );
        assert_eq!(
            repository
                .get_lesson(&failed.id)
                .unwrap()
                .unwrap()
                .error_message
                .as_deref(),
            Some("engine exited")
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn recovers_stale_starting_and_active_lessons() {
        let (directory, repository) = repository();
        let starting = repository.create_lesson(&metadata()).unwrap();
        let active = repository.create_lesson(&metadata()).unwrap();
        repository.mark_lesson_active(&active.id).unwrap();
        assert_eq!(repository.recover_stale_lessons().unwrap(), 2);
        assert_eq!(
            repository.get_lesson(&starting.id).unwrap().unwrap().status,
            LessonStatus::Interrupted
        );
        assert_eq!(
            repository.get_lesson(&active.id).unwrap().unwrap().status,
            LessonStatus::Interrupted
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn correction_detector_prefers_precision() {
        for positive in [
            "A more natural way to say that is...",
            "Small correction: use the past tense.",
            "You can say 'I go fishing.'",
            "A better way to say this is...",
            "I think you meant 'My friends help me.'",
            "I think you mean 'with biology tests.'",
            "That is a great goal! You can say 'easier than before.'",
        ] {
            assert!(detect_correction_candidate(positive), "{positive}");
        }
        for negative in [
            "That sounds great!",
            "Math is a fantastic subject to focus on!",
            "That sounds very challenging!",
            "What subjects do your friends usually help you with?",
            "I think biology is interesting.",
            "You said you enjoy tennis.",
        ] {
            assert!(!detect_correction_candidate(negative), "{negative}");
        }
    }

    #[test]
    fn detects_all_four_real_corrections_and_keeps_counts_consistent() {
        let (directory, repository) = repository();
        let lesson = repository.create_lesson(&metadata()).unwrap();
        repository.mark_lesson_active(&lesson.id).unwrap();
        let turns = [
            (
                "Yeah, I'm feeling so confident working alone.",
                "You can say \"I'm feeling very confident\" to sound more natural. Who do you like to study with?",
            ),
            (
                "You help me in biology tests.",
                "I think you meant \"My friends help me with biology tests.\" Did you find the biology test difficult?",
            ),
            (
                "Yeah, I have a lot of difficult in biology",
                "You can say \"I have a lot of **difficulty** in biology\" or \"Biology is very difficult for me.\" Do you study with your friends?",
            ),
            (
                "I study with my friends to make the hard parts easier than now.",
                "That is a great goal! You can say \"make the hard parts **easier than before**\" or just \"**easier**.\" Do you have any specific biology topics?",
            ),
        ];

        for (student_text, teacher_text) in turns {
            let student = repository
                .insert_student_message(&lesson.id, student_text)
                .unwrap()
                .unwrap();
            let persisted = repository
                .insert_teacher_response(&lesson.id, &student.id, teacher_text)
                .unwrap();
            assert!(persisted.correction_candidate.is_some());
        }

        let actual_count = repository
            .get_correction_candidates(&lesson.id)
            .unwrap()
            .len() as u32;
        let summary = repository.complete_lesson(&lesson.id).unwrap();
        let stored_count = repository
            .get_lesson(&lesson.id)
            .unwrap()
            .unwrap()
            .correction_count;
        assert_eq!(actual_count, 4);
        assert_eq!(stored_count, actual_count);
        assert_eq!(summary.correction_candidates, actual_count);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
