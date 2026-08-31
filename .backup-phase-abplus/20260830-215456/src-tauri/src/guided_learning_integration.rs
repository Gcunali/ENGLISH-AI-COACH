use crate::{
    database,
    interactive_lesson::{InteractiveLessonPackage, StagePayload},
    learning_memory_repository::normalize_vocabulary_key,
};
use rusqlite::{params, OptionalExtension, Transaction};
use serde::Serialize;
use std::{collections::BTreeMap, path::PathBuf};

pub const GUIDED_LEARNING_INTEGRATION_VERSION: u32 = 1;
const NOW_SQL: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";

#[derive(Clone)]
pub struct GuidedLearningIntegrationRepository {
    database: PathBuf,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuidedLearningSyncResult {
    pub inspected_sessions: u32,
    pub integrated_sessions: u32,
    pub vocabulary_occurrences_created: u32,
    pub mistake_occurrences_created: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuidedPracticeTimeResult {
    pub session_id: String,
    pub active_seconds: u64,
    pub event_recorded: bool,
}

#[derive(Clone)]
struct VocabularyEntry {
    display: String,
    meaning: String,
    example: String,
    occurrences: u32,
}

impl GuidedLearningIntegrationRepository {
    pub fn new(database: PathBuf) -> Self {
        Self { database }
    }

    pub fn sync_all_completed(&self) -> Result<GuidedLearningSyncResult, String> {
        let connection = database::open(&self.database)?;
        let session_ids = connection
            .prepare(
                "SELECT s.id FROM interactive_lesson_session s
                 LEFT JOIN guided_learning_integration i ON i.session_id=s.id
                 WHERE s.status='completed' AND i.session_id IS NULL
                 ORDER BY s.completed_at,s.id",
            )
            .map_err(db)?
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(db)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db)?;
        drop(connection);
        let mut total = GuidedLearningSyncResult {
            inspected_sessions: session_ids.len() as u32,
            ..Default::default()
        };
        for session_id in session_ids {
            let result = self.sync_completed(&session_id)?;
            total.integrated_sessions += result.integrated_sessions;
            total.vocabulary_occurrences_created += result.vocabulary_occurrences_created;
            total.mistake_occurrences_created += result.mistake_occurrences_created;
        }
        Ok(total)
    }

    pub fn sync_completed(&self, session_id: &str) -> Result<GuidedLearningSyncResult, String> {
        let mut connection = database::open(&self.database)?;
        let transaction = connection.transaction().map_err(db)?;
        let source: Option<(String, String, String)> = transaction
            .query_row(
                "SELECT lesson_id,package_snapshot_json,completed_at
                 FROM interactive_lesson_session WHERE id=?1 AND status='completed' AND completed_at IS NOT NULL",
                [session_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(db)?;
        let Some((lesson_id, package_json, completed_at)) = source else {
            return Ok(GuidedLearningSyncResult {
                inspected_sessions: 1,
                ..Default::default()
            });
        };
        let already_integrated: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM guided_learning_integration WHERE session_id=?1)",
                [session_id],
                |row| row.get(0),
            )
            .map_err(db)?;
        if already_integrated {
            return Ok(GuidedLearningSyncResult {
                inspected_sessions: 1,
                ..Default::default()
            });
        }
        let package: InteractiveLessonPackage =
            serde_json::from_str(&package_json).map_err(|error| {
                format!("Completed Guided Lesson package snapshot is invalid: {error}")
            })?;
        if package.lesson_id != lesson_id {
            return Err("Completed Guided Lesson identity does not match its package snapshot.".into());
        }

        let mut result = GuidedLearningSyncResult {
            inspected_sessions: 1,
            integrated_sessions: 1,
            ..Default::default()
        };
        let vocabulary = official_vocabulary(&package);
        for (canonical, entry) in vocabulary {
            transaction
                .execute(
                    &format!(
                        "INSERT INTO vocabulary_item(
                           id,canonical_text,display_text,meaning,first_seen_at,last_seen_at,
                           lesson_count,occurrence_count,status,created_at,updated_at
                         ) VALUES(?1,?2,?3,?4,?5,?5,0,0,'new',{NOW_SQL},{NOW_SQL})
                         ON CONFLICT(canonical_text) DO NOTHING"
                    ),
                    params![
                        uuid::Uuid::new_v4().to_string(),
                        canonical,
                        entry.display,
                        entry.meaning,
                        completed_at,
                    ],
                )
                .map_err(db)?;
            let vocabulary_id: String = transaction
                .query_row(
                    "SELECT id FROM vocabulary_item WHERE canonical_text=?1",
                    [&canonical],
                    |row| row.get(0),
                )
                .map_err(db)?;
            result.vocabulary_occurrences_created += transaction
                .execute(
                    &format!(
                        "INSERT OR IGNORE INTO guided_session_vocabulary(
                           id,session_id,lesson_id,vocabulary_item_id,example,occurrence_count,created_at
                         ) VALUES(?1,?2,?3,?4,?5,?6,{NOW_SQL})"
                    ),
                    params![
                        uuid::Uuid::new_v4().to_string(),
                        session_id,
                        lesson_id,
                        vocabulary_id,
                        entry.example,
                        entry.occurrences,
                    ],
                )
                .map_err(db)? as u32;
            recalculate_vocabulary(&transaction, &vocabulary_id)?;
        }

        let corrections = transaction
            .prepare(
                "SELECT id,source_index,category,original,corrected,explanation
                 FROM interactive_lesson_guided_correction
                 WHERE session_id=?1 ORDER BY source_index,id",
            )
            .map_err(db)?
            .query_map([session_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, u32>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(db)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db)?;
        for (correction_id, source_index, category, original, corrected, explanation) in
            corrections
        {
            let signature = mistake_signature(&category, &original, &corrected);
            let title = mistake_title(&category, &corrected);
            transaction
                .execute(
                    &format!(
                        "INSERT INTO recurring_mistake(
                           id,signature,category,title,explanation,first_seen_at,last_seen_at,
                           lesson_count,occurrence_count,status,created_at,updated_at
                         ) VALUES(?1,?2,?3,?4,?5,?6,?6,0,0,'active',{NOW_SQL},{NOW_SQL})
                         ON CONFLICT(signature) DO NOTHING"
                    ),
                    params![
                        uuid::Uuid::new_v4().to_string(),
                        signature,
                        category,
                        title,
                        explanation,
                        completed_at,
                    ],
                )
                .map_err(db)?;
            let mistake_id: String = transaction
                .query_row(
                    "SELECT id FROM recurring_mistake WHERE signature=?1",
                    [&signature],
                    |row| row.get(0),
                )
                .map_err(db)?;
            result.mistake_occurrences_created += transaction
                .execute(
                    &format!(
                        "INSERT OR IGNORE INTO guided_recurring_mistake_occurrence(
                           id,recurring_mistake_id,session_id,lesson_id,correction_id,source_index,
                           original,corrected,explanation,created_at
                         ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,{NOW_SQL})"
                    ),
                    params![
                        uuid::Uuid::new_v4().to_string(),
                        mistake_id,
                        session_id,
                        lesson_id,
                        correction_id,
                        source_index,
                        original,
                        corrected,
                        explanation,
                    ],
                )
                .map_err(db)? as u32;
            recalculate_mistake(&transaction, &mistake_id)?;
        }
        transaction
            .execute(
                &format!(
                    "INSERT INTO guided_learning_integration(session_id,lesson_id,integration_version,integrated_at)
                     VALUES(?1,?2,?3,{NOW_SQL})"
                ),
                params![session_id, lesson_id, GUIDED_LEARNING_INTEGRATION_VERSION],
            )
            .map_err(db)?;
        transaction.commit().map_err(db)?;
        Ok(result)
    }

    pub fn record_active_practice(
        &self,
        session_id: &str,
        event_id: &str,
        duration_seconds: u32,
    ) -> Result<GuidedPracticeTimeResult, String> {
        if event_id.trim().is_empty() || event_id.chars().count() > 100 {
            return Err("Guided practice event id is invalid.".into());
        }
        if !(1..=30).contains(&duration_seconds) {
            return Err("Guided active practice heartbeat must represent 1-30 seconds.".into());
        }
        let connection = database::open(&self.database)?;
        let status: Option<String> = connection
            .query_row(
                "SELECT status FROM interactive_lesson_session WHERE id=?1",
                [session_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(db)?;
        if status.as_deref() != Some("in_progress") {
            return Err("Active practice time is accepted only for an in-progress Guided Lesson.".into());
        }
        let inserted = connection
            .execute(
                &format!(
                    "INSERT OR IGNORE INTO interactive_lesson_active_practice_event(
                       event_id,session_id,duration_seconds,recorded_at
                     ) VALUES(?1,?2,?3,{NOW_SQL})"
                ),
                params![event_id, session_id, duration_seconds],
            )
            .map_err(db)?
            > 0;
        let active_seconds = connection
            .query_row(
                "SELECT COALESCE(SUM(duration_seconds),0)
                 FROM interactive_lesson_active_practice_event WHERE session_id=?1",
                [session_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(db)?
            .max(0) as u64;
        Ok(GuidedPracticeTimeResult {
            session_id: session_id.to_owned(),
            active_seconds,
            event_recorded: inserted,
        })
    }
}

fn official_vocabulary(package: &InteractiveLessonPackage) -> BTreeMap<String, VocabularyEntry> {
    let mut result = BTreeMap::new();
    for stage in &package.stages {
        let StagePayload::VisualVocabulary { items } = &stage.payload else {
            continue;
        };
        for item in items {
            let display = collapse(&item.term);
            let canonical = normalize_vocabulary_key(&display);
            result
                .entry(canonical)
                .and_modify(|entry: &mut VocabularyEntry| entry.occurrences += 1)
                .or_insert_with(|| VocabularyEntry {
                    display,
                    meaning: collapse(&item.meaning),
                    example: collapse(&item.example),
                    occurrences: 1,
                });
        }
    }
    result
}

fn recalculate_vocabulary(tx: &Transaction<'_>, vocabulary_id: &str) -> Result<(), String> {
    tx.execute(
        &format!(
            "UPDATE vocabulary_item SET
               lesson_count=(
                 SELECT COUNT(*) FROM (
                   SELECT 'standard:'||lesson_id AS source_lesson FROM lesson_vocabulary WHERE vocabulary_item_id=?1
                   UNION
                   SELECT 'guided:'||lesson_id FROM guided_session_vocabulary WHERE vocabulary_item_id=?1
                 )
               ),
               occurrence_count=(
                 SELECT COALESCE(SUM(value),0) FROM (
                   SELECT occurrence_count AS value FROM lesson_vocabulary WHERE vocabulary_item_id=?1
                   UNION ALL
                   SELECT occurrence_count FROM guided_session_vocabulary WHERE vocabulary_item_id=?1
                 )
               ),
               first_seen_at=(
                 SELECT MIN(seen_at) FROM (
                   SELECT l.started_at AS seen_at FROM lesson_vocabulary v JOIN lesson l ON l.id=v.lesson_id WHERE v.vocabulary_item_id=?1
                   UNION ALL
                   SELECT s.completed_at FROM guided_session_vocabulary v JOIN interactive_lesson_session s ON s.id=v.session_id WHERE v.vocabulary_item_id=?1
                 )
               ),
               last_seen_at=(
                 SELECT MAX(seen_at) FROM (
                   SELECT l.started_at AS seen_at FROM lesson_vocabulary v JOIN lesson l ON l.id=v.lesson_id WHERE v.vocabulary_item_id=?1
                   UNION ALL
                   SELECT s.completed_at FROM guided_session_vocabulary v JOIN interactive_lesson_session s ON s.id=v.session_id WHERE v.vocabulary_item_id=?1
                 )
               ),updated_at={NOW_SQL} WHERE id=?1"
        ),
        [vocabulary_id],
    )
    .map_err(db)?;
    Ok(())
}

fn recalculate_mistake(tx: &Transaction<'_>, mistake_id: &str) -> Result<(), String> {
    tx.execute(
        &format!(
            "UPDATE recurring_mistake SET
               lesson_count=(
                 SELECT COUNT(*) FROM (
                   SELECT 'standard:'||lesson_id AS source_lesson FROM recurring_mistake_occurrence WHERE recurring_mistake_id=?1
                   UNION
                   SELECT 'guided:'||lesson_id FROM guided_recurring_mistake_occurrence WHERE recurring_mistake_id=?1
                 )
               ),
               occurrence_count=(
                 (SELECT COUNT(*) FROM recurring_mistake_occurrence WHERE recurring_mistake_id=?1)+
                 (SELECT COUNT(*) FROM guided_recurring_mistake_occurrence WHERE recurring_mistake_id=?1)
               ),
               first_seen_at=(
                 SELECT MIN(seen_at) FROM (
                   SELECT l.started_at AS seen_at FROM recurring_mistake_occurrence o JOIN lesson l ON l.id=o.lesson_id WHERE o.recurring_mistake_id=?1
                   UNION ALL
                   SELECT s.completed_at FROM guided_recurring_mistake_occurrence o JOIN interactive_lesson_session s ON s.id=o.session_id WHERE o.recurring_mistake_id=?1
                 )
               ),
               last_seen_at=(
                 SELECT MAX(seen_at) FROM (
                   SELECT l.started_at AS seen_at FROM recurring_mistake_occurrence o JOIN lesson l ON l.id=o.lesson_id WHERE o.recurring_mistake_id=?1
                   UNION ALL
                   SELECT s.completed_at FROM guided_recurring_mistake_occurrence o JOIN interactive_lesson_session s ON s.id=o.session_id WHERE o.recurring_mistake_id=?1
                 )
               ),updated_at={NOW_SQL} WHERE id=?1"
        ),
        [mistake_id],
    )
    .map_err(db)?;
    Ok(())
}

fn mistake_signature(category: &str, original: &str, corrected: &str) -> String {
    format!(
        "{}|{}|{}",
        category,
        collapse(original).to_lowercase(),
        collapse(corrected).to_lowercase()
    )
}

fn mistake_title(category: &str, corrected: &str) -> String {
    let label = category.replace('_', " ");
    let mut chars = label.chars();
    let label = chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default();
    let concise = collapse(corrected).chars().take(100).collect::<String>();
    format!("{label}: \"{concise}\"")
}

fn collapse(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn db(error: rusqlite::Error) -> String {
    format!("Guided learning integration database error: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;
    use rusqlite::Connection;

    fn harness() -> (PathBuf, PathBuf, GuidedLearningIntegrationRepository, String) {
        let directory = std::env::temp_dir()
            .join(format!("guided-learning-integration-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("test.sqlite3");
        database::migrate(&path).unwrap();
        let registry = crate::interactive_lesson_content::InteractiveLessonContentRegistry::load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/interactive-lessons"),
        );
        let lesson = registry.get("a1-u01-l01-hello-goodbye").unwrap().package;
        let second = registry.get("a1-u01-l02-whats-your-name").unwrap().package;
        let package = serde_json::to_string(&lesson).unwrap();
        let second_package = serde_json::to_string(&second).unwrap();
        let connection = Connection::open(&path).unwrap();
        for (id, lesson_id, snapshot, completed) in [
            ("guided-1", lesson.lesson_id.as_str(), package.as_str(), "2026-08-24T10:00:00Z"),
            ("guided-2", second.lesson_id.as_str(), second_package.as_str(), "2026-08-25T10:00:00Z"),
        ] {
            connection.execute("INSERT INTO interactive_lesson_session(id,lesson_id,lesson_content_version,package_schema_version,lesson_flow_version,package_hash,engine_version,snapshot_version,status,stage_count,current_stage_index,package_snapshot_json,student_context_snapshot_json,started_at,updated_at,completed_at) VALUES(?1,?2,1,1,1,?3,1,1,'completed',1,0,?4,'{}',?5,?5,?5)",params![id,lesson_id,"a".repeat(64),snapshot,completed]).unwrap();
        }
        drop(connection);
        (
            directory,
            path.clone(),
            GuidedLearningIntegrationRepository::new(path),
            lesson.lesson_id,
        )
    }

    #[test]
    fn completed_vocabulary_is_idempotent_and_preserves_manual_status() {
        let (directory, path, repo, _) = harness();
        let first = repo.sync_completed("guided-1").unwrap();
        assert!(first.vocabulary_occurrences_created > 0);
        let review = crate::review_repository::ReviewRepository::new(path.clone())
            .overview()
            .unwrap();
        assert!(review.vocabulary.total_eligible_count > 0);
        let connection = Connection::open(&path).unwrap();
        let id: String = connection
            .query_row("SELECT id FROM vocabulary_item LIMIT 1", [], |row| row.get(0))
            .unwrap();
        connection
            .execute("UPDATE vocabulary_item SET status='known' WHERE id=?1", [&id])
            .unwrap();
        drop(connection);
        assert_eq!(repo.sync_completed("guided-1").unwrap().integrated_sessions, 0);
        let connection = Connection::open(&path).unwrap();
        let state: (String, i64) = connection
            .query_row(
                "SELECT status,occurrence_count FROM vocabulary_item WHERE id=?1",
                [&id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(state.0, "known");
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM guided_learning_integration", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn structured_mistake_requires_two_distinct_guided_lessons_to_confirm() {
        let (directory, path, repo, _) = harness();
        let connection = Connection::open(&path).unwrap();
        for (index, session) in ["guided-1", "guided-2"].into_iter().enumerate() {
            connection.execute("INSERT INTO interactive_lesson_stage_state(id,session_id,stage_id,sequence_index,stage_type,stage_schema_version,required,status,attempt_count,updated_at) VALUES(?1,?2,'conversation',0,'guided_conversation',1,1,'completed',1,'now')",params![format!("stage-{index}"),session]).unwrap();
            let student = format!("student-{index}");
            let teacher = format!("teacher-{index}");
            connection.execute("INSERT INTO interactive_lesson_guided_conversation_turn(id,event_id,session_id,stage_id,sequence_index,role,text,text_schema_version,word_count,partial,created_at,committed_at) VALUES(?1,?1,?2,'conversation',0,'student','I go yesterday',1,3,0,'now','now')",params![student,session]).unwrap();
            connection.execute("INSERT INTO interactive_lesson_guided_conversation_turn(id,event_id,session_id,stage_id,sequence_index,role,text,text_schema_version,word_count,partial,created_at,committed_at) VALUES(?1,?1,?2,'conversation',1,'assistant','Small correction: I went yesterday.',1,4,0,'now','now')",params![teacher,session]).unwrap();
            connection.execute("INSERT INTO interactive_lesson_guided_correction(id,session_id,stage_id,student_turn_id,teacher_turn_id,source_index,category,original,corrected,explanation,detection_method,created_at) VALUES(?1,?2,'conversation',?3,?4,0,'naturalness','I go yesterday','I went yesterday','Small correction: I went yesterday.','guided_teacher_cue_v1','now')",params![format!("correction-{index}"),session,student,teacher]).unwrap();
        }
        drop(connection);
        repo.sync_completed("guided-1").unwrap();
        let connection = Connection::open(&path).unwrap();
        assert_eq!(connection.query_row("SELECT lesson_count FROM recurring_mistake",[],|r|r.get::<_,i64>(0)).unwrap(),1);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM vocabulary_item WHERE canonical_text='i go yesterday'",[],|r|r.get::<_,i64>(0)).unwrap(),0);
        drop(connection);
        repo.sync_completed("guided-2").unwrap();
        let connection = Connection::open(&path).unwrap();
        assert_eq!(connection.query_row("SELECT lesson_count FROM recurring_mistake",[],|r|r.get::<_,i64>(0)).unwrap(),2);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn active_time_accepts_only_idempotent_foreground_sized_events() {
        let (directory, path, repo, _) = harness();
        let connection = Connection::open(&path).unwrap();
        connection.execute("UPDATE interactive_lesson_session SET status='in_progress',completed_at=NULL WHERE id='guided-1'",[]).unwrap();
        drop(connection);
        assert_eq!(repo.record_active_practice("guided-1","tick-1",15).unwrap().active_seconds,15);
        let repeated=repo.record_active_practice("guided-1","tick-1",15).unwrap();
        assert!(!repeated.event_recorded);
        assert_eq!(repeated.active_seconds,15);
        assert!(repo.record_active_practice("guided-1","bad",31).is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    #[ignore = "manual Phase AA migration audit against the user's physical SQLite database"]
    fn physical_phase_aa_migrates_without_fabricating_guided_learning() {
        let path = PathBuf::from(std::env::var_os("LOCALAPPDATA").expect("LOCALAPPDATA"))
            .join("com.englishaicoach.desktop")
            .join("database")
            .join("english-ai-coach.sqlite3");
        let before = database::open(&path).unwrap();
        let sessions_before: i64 = before.query_row("SELECT COUNT(*) FROM interactive_lesson_session",[],|row|row.get(0)).unwrap();
        let completed_before: i64 = before.query_row("SELECT COUNT(*) FROM interactive_lesson_session WHERE status='completed'",[],|row|row.get(0)).unwrap();
        let vocabulary_before: i64 = before.query_row("SELECT COUNT(*) FROM vocabulary_item",[],|row|row.get(0)).unwrap();
        drop(before);
        database::migrate(&path).unwrap();
        database::migrate(&path).unwrap();
        let connection = database::open(&path).unwrap();
        let version: i64 = connection.query_row("SELECT MAX(version) FROM schema_migration",[],|row|row.get(0)).unwrap();
        let integrity: String = connection.query_row("PRAGMA integrity_check",[],|row|row.get(0)).unwrap();
        let foreign_keys: i64 = connection.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check",[],|row|row.get(0)).unwrap();
        let after: (i64,i64,i64,i64,i64,i64) = connection.query_row("SELECT
          (SELECT COUNT(*) FROM interactive_lesson_session),
          (SELECT COUNT(*) FROM interactive_lesson_session WHERE status='completed'),
          (SELECT COUNT(*) FROM vocabulary_item),
          (SELECT COUNT(*) FROM guided_learning_integration),
          (SELECT COUNT(*) FROM interactive_lesson_active_practice_event),
          (SELECT COUNT(*) FROM guided_gamification_xp_event)",[],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?))).unwrap();
        assert_eq!(version,19);
        assert_eq!(integrity,"ok");
        assert_eq!(foreign_keys,0);
        assert_eq!((after.0,after.1,after.2),(sessions_before,completed_before,vocabulary_before));
        assert_eq!((after.3,after.4,after.5),(0,0,0));
        println!("PHASE_AA_PHYSICAL schema={version} sessions={} completed={} vocabulary={} integrations={} active_events={} guided_xp={} integrity={integrity} fk={foreign_keys}",after.0,after.1,after.2,after.3,after.4,after.5);
    }
}
