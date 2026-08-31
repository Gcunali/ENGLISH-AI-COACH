use rusqlite::{params, Connection};
use std::{path::Path, time::Duration};

pub(crate) fn open(path: &Path) -> Result<Connection, String> {
    let connection =
        Connection::open(path).map_err(|error| format!("Database unavailable: {error}"))?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|error| format!("Could not configure database timeout: {error}"))?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|error| format!("Could not enable SQLite foreign keys: {error}"))?;
    Ok(connection)
}

pub fn migrate(path: &Path) -> Result<(), String> {
    let mut connection = open(path)?;
    connection
        .execute_batch(include_str!("../migrations/001_initial.sql"))
        .map_err(|error| format!("Database migration failed: {error}"))?;
    let has_fifteen: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migration WHERE version=15)",
            [],
            |row| row.get(0),
        )
        .map_err(|error| format!("Could not inspect database migration level: {error}"))?;
    let transaction = connection
        .transaction()
        .map_err(|error| format!("Could not start database migration: {error}"))?;
    transaction
        .execute_batch(include_str!("../migrations/002_lessons.sql"))
        .map_err(|error| format!("Lesson migration failed: {error}"))?;
    transaction
        .execute_batch(include_str!("../migrations/003_lesson_analysis.sql"))
        .map_err(|error| format!("Lesson analysis migration failed: {error}"))?;
    transaction
        .execute_batch(include_str!("../migrations/004_learning_memory.sql"))
        .map_err(|error| format!("Learning memory migration failed: {error}"))?;
    transaction
        .execute_batch(include_str!(
            "../migrations/005_student_learning_summary.sql"
        ))
        .map_err(|error| format!("Student learning summary migration failed: {error}"))?;
    transaction
        .execute_batch(include_str!("../migrations/006_lesson_configuration.sql"))
        .map_err(|error| format!("Lesson configuration migration failed: {error}"))?;
    transaction
        .execute_batch(include_str!("../migrations/007_placement_test.sql"))
        .map_err(|error| format!("Placement test migration failed: {error}"))?;
    transaction
        .execute_batch(include_str!(
            "../migrations/008_student_learning_profile.sql"
        ))
        .map_err(|error| format!("Student learning profile migration failed: {error}"))?;
    transaction
        .execute_batch(include_str!("../migrations/009_gamification.sql"))
        .map_err(|error| format!("Gamification migration failed: {error}"))?;
    transaction
        .execute_batch(include_str!("../migrations/010_review_system.sql"))
        .map_err(|error| format!("Review system migration failed: {error}"))?;
    transaction
        .execute_batch(include_str!("../migrations/011_voice_performance.sql"))
        .map_err(|error| format!("Voice performance migration failed: {error}"))?;
    transaction
        .execute_batch(include_str!("../migrations/012_pronunciation_engine.sql"))
        .map_err(|error| format!("Pronunciation engine migration failed: {error}"))?;
    transaction
        .execute_batch(include_str!("../migrations/013_platform_reliability.sql"))
        .map_err(|error| format!("Platform reliability migration failed: {error}"))?;
    transaction
        .execute_batch(include_str!(
            "../migrations/014_interactive_lesson_engine.sql"
        ))
        .map_err(|error| format!("Interactive lesson engine migration failed: {error}"))?;
    if !has_fifteen {
        transaction
            .execute_batch(include_str!(
                "../migrations/015_interactive_lesson_audio_practice.sql"
            ))
            .map_err(|error| {
                format!("Interactive lesson audio practice migration failed: {error}")
            })?;
    }
    transaction
        .execute_batch(include_str!(
            "../migrations/016_interactive_lesson_exercise_engine.sql"
        ))
        .map_err(|error| format!("Interactive lesson exercise migration failed: {error}"))?;
    transaction
        .execute_batch(include_str!(
            "../migrations/017_interactive_lesson_guided_conversation.sql"
        ))
        .map_err(|error| {
            format!("Interactive lesson guided conversation migration failed: {error}")
        })?;
    transaction
        .execute_batch(include_str!(
            "../migrations/018_interactive_lesson_analysis.sql"
        ))
        .map_err(|error| format!("Interactive lesson analysis migration failed: {error}"))?;
    transaction
        .execute_batch(include_str!(
            "../migrations/019_guided_learning_integration.sql"
        ))
        .map_err(|error| format!("Guided learning integration migration failed: {error}"))?;
    transaction
        .commit()
        .map_err(|error| format!("Could not commit database migration: {error}"))?;
    Ok(())
}

pub fn save_exchange(path: &Path, student: &str, teacher: &str) -> Result<(), String> {
    let connection = open(path)?;
    connection.execute("INSERT INTO conversation_exchange (id, student_text, teacher_text) VALUES (?1, ?2, ?3)", params![uuid::Uuid::new_v4().to_string(), student, teacher])
        .map_err(|error| format!("Could not save transcript: {error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{migrate, save_exchange};
    use rusqlite::Connection;

    fn test_database() -> (std::path::PathBuf, std::path::PathBuf) {
        let directory =
            std::env::temp_dir().join(format!("english-ai-coach-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).expect("create test directory");
        let database = directory.join("test.sqlite3");
        (directory, database)
    }

    #[test]
    fn migrates_and_saves_an_exchange() {
        let (directory, database) = test_database();
        migrate(&database).expect("migrate database");
        save_exchange(&database, "Hello", "How are you?").expect("save exchange");
        let connection = Connection::open(&database).expect("open database");
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM conversation_exchange", [], |row| {
                row.get(0)
            })
            .expect("count rows");
        assert_eq!(count, 1);
        drop(connection);
        std::fs::remove_dir_all(&directory).expect("remove isolated test directory");
    }

    #[test]
    fn migrates_a_new_database_through_version_fifteen() {
        let (directory, database) = test_database();
        migrate(&database).expect("migrate database");
        let connection = Connection::open(&database).expect("open database");
        let versions: Vec<i64> = connection
            .prepare("SELECT version FROM schema_migration ORDER BY version")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(
            versions,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]
        );
        for table in [
            "lesson",
            "transcript_message",
            "correction_candidate",
            "lesson_analysis",
            "vocabulary_item",
            "lesson_vocabulary",
            "recurring_mistake",
            "recurring_mistake_occurrence",
            "student_learning_summary",
            "lesson_teacher_memory",
            "lesson_configuration_snapshot",
            "placement_attempt",
            "placement_answer",
            "placement_speaking_response",
            "student_learning_profile",
            "lesson_student_profile_snapshot",
            "gamification_xp_event",
            "gamification_profile",
            "achievement_unlock",
            "review_session",
            "review_session_item",
            "voice_turn_performance",
            "pronunciation_attempt",
            "pronunciation_word_result",
            "app_system_event",
            "interactive_lesson_session",
            "interactive_lesson_stage_state",
            "interactive_lesson_stage_runtime_state",
            "interactive_lesson_pronunciation_attempt",
            "interactive_lesson_exercise_attempt",
            "interactive_lesson_guided_conversation_turn",
            "guided_learning_integration",
            "guided_session_vocabulary",
            "interactive_lesson_guided_correction",
            "guided_recurring_mistake_occurrence",
            "interactive_lesson_active_practice_event",
            "guided_gamification_xp_event",
        ] {
            let exists: bool = connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert!(exists, "missing table {table}");
        }
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn upgrades_version_one_and_is_idempotent_without_losing_legacy_data() {
        let (directory, database) = test_database();
        let connection = Connection::open(&database).unwrap();
        connection
            .execute_batch(include_str!("../migrations/001_initial.sql"))
            .unwrap();
        connection
            .execute(
                "INSERT INTO conversation_exchange(id, student_text, teacher_text) VALUES ('legacy', 'Hi', 'Hello')",
                [],
            )
            .unwrap();
        drop(connection);

        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        let legacy_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM conversation_exchange", [], |row| {
                row.get(0)
            })
            .unwrap();
        let migration_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM schema_migration", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(legacy_count, 1);
        assert_eq!(migration_count, 19);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn upgrades_version_thirteen_to_fourteen_without_touching_existing_data() {
        let (directory, database) = test_database();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        connection.execute("INSERT INTO conversation_exchange(id, student_text, teacher_text) VALUES ('before-014', 'Hi', 'Hello')", []).unwrap();
        connection.execute_batch("DROP TABLE guided_gamification_xp_event; DROP TABLE interactive_lesson_active_practice_event; DROP TABLE guided_recurring_mistake_occurrence; DROP TABLE interactive_lesson_guided_correction; DROP TABLE guided_session_vocabulary; DROP TABLE guided_learning_integration; DROP TABLE interactive_lesson_analysis; DROP TABLE interactive_lesson_guided_conversation_turn; DROP TABLE interactive_lesson_exercise_attempt; DROP TABLE interactive_lesson_pronunciation_attempt; DROP TABLE interactive_lesson_stage_runtime_state; DROP TABLE interactive_lesson_stage_state; DROP TABLE interactive_lesson_session; DELETE FROM schema_migration WHERE version>=14;").unwrap();
        drop(connection);
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        let preserved: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM conversation_exchange WHERE id='before-014'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migration", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(preserved, 1);
        assert_eq!(version, 19);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn upgrades_fourteen_to_fifteen_preserving_pronunciation_and_expanding_provenance() {
        let (directory, database) = test_database();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        connection.execute("INSERT INTO pronunciation_attempt(id,status,source_type,target_text,normalized_target,locale,engine_version,score_version,result_schema_version,model_id,model_revision,model_manifest_hash,overall_score,confidence,content_match_score,alignment_coverage,audio_duration_ms,word_count,created_at,completed_at) VALUES('legacy-pron','completed','custom','hello','hello','en-US',1,1,1,'model','revision',?1,42,'low',1,1,500,1,'now','now')",["a".repeat(64)]).unwrap();
        connection.execute_batch("DROP TABLE interactive_lesson_pronunciation_attempt; DROP TABLE interactive_lesson_stage_runtime_state; DELETE FROM schema_migration WHERE version=15;").unwrap();
        drop(connection);
        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        let preserved:i64=connection.query_row("SELECT COUNT(*) FROM pronunciation_attempt WHERE id='legacy-pron' AND overall_score=42",[],|row|row.get(0)).unwrap();
        let sql: String = connection
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='pronunciation_attempt'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let version: i64 = connection
            .query_row("SELECT max(version) FROM schema_migration", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(preserved, 1);
        assert!(sql.contains("interactive_lesson"));
        assert_eq!(version, 19);
        assert_eq!(
            connection
                .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
        assert!(connection
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .query([])
            .unwrap()
            .next()
            .unwrap()
            .is_none());
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    #[ignore = "manual migration audit against the user's physical SQLite database"]
    fn physical_phase_s_migrates_without_fabricating_guided_data() {
        let database = std::path::PathBuf::from(
            std::env::var("EAC_PHASE_S_PHYSICAL_DB").expect("EAC_PHASE_S_PHYSICAL_DB"),
        );
        let tables = [
            "lesson",
            "transcript_message",
            "lesson_analysis",
            "vocabulary_item",
            "recurring_mistake",
            "placement_attempt",
            "gamification_xp_event",
            "achievement_unlock",
            "review_session",
            "pronunciation_attempt",
            "voice_turn_performance",
            "interactive_lesson_session",
            "interactive_lesson_stage_state",
        ];
        let before = Connection::open(&database).unwrap();
        let counts = tables
            .iter()
            .map(|table| {
                before
                    .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                        row.get::<_, i64>(0)
                    })
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            before
                .query_row("SELECT max(version) FROM schema_migration", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .unwrap(),
            14
        );
        drop(before);
        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let after = Connection::open(&database).unwrap();
        let after_counts = tables
            .iter()
            .map(|table| {
                after
                    .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                        row.get::<_, i64>(0)
                    })
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(counts, after_counts);
        assert_eq!(
            after
                .query_row("SELECT max(version) FROM schema_migration", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .unwrap(),
            15
        );
        assert_eq!(
            after
                .query_row(
                    "SELECT COUNT(*) FROM interactive_lesson_stage_runtime_state",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        assert_eq!(
            after
                .query_row(
                    "SELECT COUNT(*) FROM interactive_lesson_pronunciation_attempt",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        assert_eq!(
            after
                .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
        assert!(after
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .query([])
            .unwrap()
            .next()
            .unwrap()
            .is_none());
        println!("physical_phase_s|schema=15|counts={after_counts:?}|integrity=ok|foreign_keys=0");
    }

    #[test]
    fn upgrades_sixteen_to_seventeen_idempotently_without_fabricated_guided_turns() {
        let (directory, database) = test_database();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        connection.execute_batch("DROP TABLE guided_recurring_mistake_occurrence; DROP TABLE interactive_lesson_guided_correction; DROP TABLE guided_gamification_xp_event; DROP TABLE interactive_lesson_active_practice_event; DROP TABLE guided_session_vocabulary; DROP TABLE guided_learning_integration; DELETE FROM schema_migration WHERE version=19; DROP TABLE interactive_lesson_guided_conversation_turn; DELETE FROM schema_migration WHERE version=17;").unwrap();
        drop(connection);
        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        let version: i64 = connection
            .query_row("SELECT max(version) FROM schema_migration", [], |row| {
                row.get(0)
            })
            .unwrap();
        let attempts: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM interactive_lesson_guided_conversation_turn",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, 19);
        assert_eq!(attempts, 0);
        assert_eq!(
            connection
                .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
        assert!(connection
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .query([])
            .unwrap()
            .next()
            .unwrap()
            .is_none());
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    #[ignore = "manual Phase T migration audit against the user's physical SQLite database"]
    fn physical_phase_t_migrates_without_fabricating_exercise_data() {
        let database = std::path::PathBuf::from(
            std::env::var("EAC_PHASE_T_PHYSICAL_DB").expect("EAC_PHASE_T_PHYSICAL_DB"),
        );
        let tables = [
            "lesson",
            "transcript_message",
            "lesson_analysis",
            "vocabulary_item",
            "recurring_mistake",
            "placement_attempt",
            "gamification_xp_event",
            "achievement_unlock",
            "review_session",
            "pronunciation_attempt",
            "voice_turn_performance",
            "interactive_lesson_session",
            "interactive_lesson_stage_state",
            "interactive_lesson_stage_runtime_state",
            "interactive_lesson_pronunciation_attempt",
        ];
        let before = Connection::open(&database).unwrap();
        let counts = tables
            .iter()
            .map(|table| {
                before
                    .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                        row.get::<_, i64>(0)
                    })
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            before
                .query_row("SELECT max(version) FROM schema_migration", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .unwrap(),
            15
        );
        drop(before);
        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let after = Connection::open(&database).unwrap();
        let after_counts = tables
            .iter()
            .map(|table| {
                after
                    .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                        row.get::<_, i64>(0)
                    })
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(counts, after_counts);
        assert_eq!(
            after
                .query_row("SELECT max(version) FROM schema_migration", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .unwrap(),
            16
        );
        assert_eq!(
            after
                .query_row(
                    "SELECT COUNT(*) FROM interactive_lesson_exercise_attempt",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        assert_eq!(
            after
                .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
        assert!(after
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .query([])
            .unwrap()
            .next()
            .unwrap()
            .is_none());
        println!("physical_phase_t|schema=16|counts={after_counts:?}|exercise_attempts=0|integrity=ok|foreign_keys=0");
    }

    #[test]
    fn upgrades_version_two_preserves_lessons_and_database_integrity() {
        let (directory, database) = test_database();
        let connection = Connection::open(&database).unwrap();
        connection
            .execute_batch(include_str!("../migrations/001_initial.sql"))
            .unwrap();
        connection
            .execute_batch(include_str!("../migrations/002_lessons.sql"))
            .unwrap();
        connection
            .execute(
                "INSERT INTO lesson (
               id, started_at, ended_at, status, mode, student_turn_count,
               teacher_turn_count, correction_count, whisper_model, whisper_threads,
               ollama_model, piper_voice, voice_engine_version, created_at, updated_at
             ) VALUES (
               'existing-lesson', '2026-08-17T00:00:00Z', '2026-08-17T00:05:00Z',
               'completed', 'free_conversation', 3, 3, 1, 'whisper.bin', 12,
               'qwen3.5:4b', 'lessac', 'voice-v2', 'now', 'now'
             )",
                [],
            )
            .unwrap();
        drop(connection);

        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        let lesson_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM lesson WHERE id = 'existing-lesson'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let versions: Vec<i64> = connection
            .prepare("SELECT version FROM schema_migration ORDER BY version")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        let foreign_key_errors: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(lesson_count, 1);
        assert_eq!(
            versions,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]
        );
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_key_errors, 0);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn upgrades_version_three_idempotently_and_preserves_analysis_integrity() {
        let (directory, database) = test_database();
        let connection = Connection::open(&database).unwrap();
        connection
            .execute_batch(include_str!("../migrations/001_initial.sql"))
            .unwrap();
        connection
            .execute_batch(include_str!("../migrations/002_lessons.sql"))
            .unwrap();
        connection
            .execute_batch(include_str!("../migrations/003_lesson_analysis.sql"))
            .unwrap();
        connection
            .execute(
                "INSERT INTO lesson (
               id, started_at, ended_at, status, mode, student_turn_count,
               teacher_turn_count, correction_count, whisper_model, whisper_threads,
               ollama_model, piper_voice, voice_engine_version, created_at, updated_at
             ) VALUES ('v3-lesson', '2026-08-17T00:00:00Z', '2026-08-17T00:05:00Z',
               'completed', 'free_conversation', 3, 3, 1, 'whisper.bin', 12,
               'qwen3.5:4b', 'lessac', 'voice-v2', 'now', 'now')",
                [],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO lesson_analysis (
               id, lesson_id, status, schema_version, prompt_version, analyzer_model,
               overall_score, raw_json, created_at, updated_at
             ) VALUES ('v3-analysis', 'v3-lesson', 'completed', 1, 1, 'qwen3.5:4b',
               81, '{}', 'now', 'now')",
                [],
            )
            .unwrap();
        drop(connection);

        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        let versions: Vec<i64> = connection
            .prepare("SELECT version FROM schema_migration ORDER BY version")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let analysis_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM lesson_analysis WHERE id = 'v3-analysis'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        let foreign_key_errors: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(
            versions,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]
        );
        assert_eq!(analysis_count, 1);
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_key_errors, 0);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn upgrades_version_four_idempotently_and_preserves_learning_memory() {
        let (directory, database) = test_database();
        let connection = Connection::open(&database).unwrap();
        for migration in [
            include_str!("../migrations/001_initial.sql"),
            include_str!("../migrations/002_lessons.sql"),
            include_str!("../migrations/003_lesson_analysis.sql"),
            include_str!("../migrations/004_learning_memory.sql"),
        ] {
            connection.execute_batch(migration).unwrap();
        }
        connection
            .execute(
                "INSERT INTO vocabulary_item (
               id, canonical_text, display_text, meaning, first_seen_at, last_seen_at,
               lesson_count, occurrence_count, status, created_at, updated_at
             ) VALUES ('existing-vocabulary', 'terrible at', 'terrible at', 'muito ruim em',
               '2026-01-01', '2026-01-01', 1, 1, 'learning', 'now', 'now')",
                [],
            )
            .unwrap();
        drop(connection);

        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        let versions: Vec<i64> = connection
            .prepare("SELECT version FROM schema_migration ORDER BY version")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let vocabulary_status: String = connection
            .query_row(
                "SELECT status FROM vocabulary_item WHERE id = 'existing-vocabulary'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let memory_default: String = connection
            .query_row(
                "SELECT value_json FROM settings WHERE key = 'use_learning_memory_in_lessons'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        let foreign_key_errors: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(
            versions,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]
        );
        assert_eq!(vocabulary_status, "learning");
        assert_eq!(memory_default, "true");
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_key_errors, 0);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn upgrades_version_five_without_backfilling_old_lessons() {
        let (directory, database) = test_database();
        let connection = Connection::open(&database).unwrap();
        for migration in [
            include_str!("../migrations/001_initial.sql"),
            include_str!("../migrations/002_lessons.sql"),
            include_str!("../migrations/003_lesson_analysis.sql"),
            include_str!("../migrations/004_learning_memory.sql"),
            include_str!("../migrations/005_student_learning_summary.sql"),
        ] {
            connection.execute_batch(migration).unwrap();
        }
        connection.execute("INSERT INTO lesson (id,started_at,status,mode,whisper_model,whisper_threads,ollama_model,piper_voice,voice_engine_version,created_at,updated_at) VALUES ('legacy-v5','now','completed','free_conversation','w',12,'q','p','v','now','now')", []).unwrap();
        drop(connection);
        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        let versions: Vec<i64> = connection
            .prepare("SELECT version FROM schema_migration ORDER BY version")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let snapshots: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM lesson_configuration_snapshot",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        let foreign_keys: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(
            versions,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]
        );
        assert_eq!(snapshots, 0);
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_keys, 0);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn upgrades_version_six_idempotently_without_touching_lessons() {
        let (directory, database) = test_database();
        let connection = Connection::open(&database).unwrap();
        for migration in [
            include_str!("../migrations/001_initial.sql"),
            include_str!("../migrations/002_lessons.sql"),
            include_str!("../migrations/003_lesson_analysis.sql"),
            include_str!("../migrations/004_learning_memory.sql"),
            include_str!("../migrations/005_student_learning_summary.sql"),
            include_str!("../migrations/006_lesson_configuration.sql"),
        ] {
            connection.execute_batch(migration).unwrap();
        }
        connection.execute("INSERT INTO lesson (id,started_at,status,mode,whisper_model,whisper_threads,ollama_model,piper_voice,voice_engine_version,created_at,updated_at) VALUES ('before-placement','now','completed','free_conversation','w',12,'q','p','v','now','now')", []).unwrap();
        drop(connection);
        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        let versions: Vec<i64> = connection
            .prepare("SELECT version FROM schema_migration ORDER BY version")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let lessons: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM lesson WHERE id='before-placement'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        let foreign_keys: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            versions,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]
        );
        assert_eq!(lessons, 1);
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_keys, 0);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn upgrades_version_eight_to_gamification_idempotently() {
        let (directory, database) = test_database();
        let connection = Connection::open(&database).unwrap();
        for migration in [
            include_str!("../migrations/001_initial.sql"),
            include_str!("../migrations/002_lessons.sql"),
            include_str!("../migrations/003_lesson_analysis.sql"),
            include_str!("../migrations/004_learning_memory.sql"),
            include_str!("../migrations/005_student_learning_summary.sql"),
            include_str!("../migrations/006_lesson_configuration.sql"),
            include_str!("../migrations/007_placement_test.sql"),
            include_str!("../migrations/008_student_learning_profile.sql"),
        ] {
            connection.execute_batch(migration).unwrap();
        }
        drop(connection);
        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        let versions: Vec<i64> = connection
            .prepare("SELECT version FROM schema_migration ORDER BY version")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let goal: i64 = connection
            .query_row(
                "SELECT weekly_goal_minutes FROM gamification_profile WHERE profile_key='default'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            versions,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]
        );
        assert_eq!(goal, 90);
        assert_eq!(integrity, "ok");
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn upgrades_version_nine_to_review_without_backfill() {
        let (directory, database) = test_database();
        let connection = Connection::open(&database).unwrap();
        for migration in [
            include_str!("../migrations/001_initial.sql"),
            include_str!("../migrations/002_lessons.sql"),
            include_str!("../migrations/003_lesson_analysis.sql"),
            include_str!("../migrations/004_learning_memory.sql"),
            include_str!("../migrations/005_student_learning_summary.sql"),
            include_str!("../migrations/006_lesson_configuration.sql"),
            include_str!("../migrations/007_placement_test.sql"),
            include_str!("../migrations/008_student_learning_profile.sql"),
            include_str!("../migrations/009_gamification.sql"),
        ] {
            connection.execute_batch(migration).unwrap();
        }
        drop(connection);
        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        let reviews: i64 = connection
            .query_row("SELECT COUNT(*) FROM review_session", [], |r| r.get(0))
            .unwrap();
        let versions: Vec<i64> = connection
            .prepare("SELECT version FROM schema_migration ORDER BY version")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        let foreign_keys: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(reviews, 0);
        assert_eq!(
            versions,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]
        );
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_keys, 0);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn upgrades_version_ten_to_voice_performance_without_backfill() {
        let (directory, database) = test_database();
        let connection = Connection::open(&database).unwrap();
        for migration in [
            include_str!("../migrations/001_initial.sql"),
            include_str!("../migrations/002_lessons.sql"),
            include_str!("../migrations/003_lesson_analysis.sql"),
            include_str!("../migrations/004_learning_memory.sql"),
            include_str!("../migrations/005_student_learning_summary.sql"),
            include_str!("../migrations/006_lesson_configuration.sql"),
            include_str!("../migrations/007_placement_test.sql"),
            include_str!("../migrations/008_student_learning_profile.sql"),
            include_str!("../migrations/009_gamification.sql"),
            include_str!("../migrations/010_review_system.sql"),
        ] {
            connection.execute_batch(migration).unwrap();
        }
        connection.execute("INSERT INTO lesson(id,started_at,status,mode,whisper_model,whisper_threads,ollama_model,piper_voice,voice_engine_version,created_at,updated_at) VALUES('v10-lesson','now','completed','free_conversation','small',12,'qwen3.5:4b','lessac','v2','now','now')", []).unwrap();
        connection.execute("INSERT INTO review_session(id,status,mode,requested_item_count,actual_item_count,reviewed_item_count,queue_version,item_snapshot_version,started_at,created_at,updated_at) VALUES('v10-review','abandoned','mixed',5,1,0,1,1,'now','now','now')", []).unwrap();
        drop(connection);

        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        let versions: Vec<i64> = connection
            .prepare("SELECT version FROM schema_migration ORDER BY version")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let lessons: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM lesson WHERE id='v10-lesson'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let reviews: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM review_session WHERE id='v10-review'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let metrics: i64 = connection
            .query_row("SELECT COUNT(*) FROM voice_turn_performance", [], |row| {
                row.get(0)
            })
            .unwrap();
        let setting: String = connection
            .query_row(
                "SELECT value_json FROM settings WHERE key='use_streaming_voice_response'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        let foreign_keys: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(
            versions,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]
        );
        assert_eq!((lessons, reviews, metrics), (1, 1, 0));
        assert_eq!(setting, "true");
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_keys, 0);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn upgrades_version_eleven_to_pronunciation_without_fabricated_attempts() {
        let (directory, database) = test_database();
        let connection = Connection::open(&database).unwrap();
        for migration in [
            include_str!("../migrations/001_initial.sql"),
            include_str!("../migrations/002_lessons.sql"),
            include_str!("../migrations/003_lesson_analysis.sql"),
            include_str!("../migrations/004_learning_memory.sql"),
            include_str!("../migrations/005_student_learning_summary.sql"),
            include_str!("../migrations/006_lesson_configuration.sql"),
            include_str!("../migrations/007_placement_test.sql"),
            include_str!("../migrations/008_student_learning_profile.sql"),
            include_str!("../migrations/009_gamification.sql"),
            include_str!("../migrations/010_review_system.sql"),
            include_str!("../migrations/011_voice_performance.sql"),
        ] {
            connection.execute_batch(migration).unwrap();
        }
        connection.execute("INSERT INTO lesson(id,started_at,status,mode,whisper_model,whisper_threads,ollama_model,piper_voice,voice_engine_version,created_at,updated_at) VALUES('phase-o-existing','now','completed','free_conversation','small',12,'qwen3.5:4b','lessac','v2','now','now')", []).unwrap();
        drop(connection);
        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let connection = Connection::open(&database).unwrap();
        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migration", [], |row| {
                row.get(0)
            })
            .unwrap();
        let lessons: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM lesson WHERE id='phase-o-existing'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let attempts: i64 = connection
            .query_row("SELECT COUNT(*) FROM pronunciation_attempt", [], |row| {
                row.get(0)
            })
            .unwrap();
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        let foreign_keys: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!((version, lessons, attempts), (19, 1, 0));
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_keys, 0);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    #[ignore = "manual Phase U migration audit against the user's physical SQLite database"]
    fn physical_phase_u_migrates_without_fabricating_guided_conversation_data() {
        let database = std::path::PathBuf::from(
            std::env::var("EAC_PHASE_U_PHYSICAL_DB").expect("EAC_PHASE_U_PHYSICAL_DB"),
        );
        let tables = [
            "lesson",
            "transcript_message",
            "lesson_analysis",
            "vocabulary_item",
            "recurring_mistake",
            "placement_attempt",
            "gamification_xp_event",
            "achievement_unlock",
            "review_session",
            "pronunciation_attempt",
            "voice_turn_performance",
            "interactive_lesson_session",
            "interactive_lesson_stage_state",
            "interactive_lesson_stage_runtime_state",
            "interactive_lesson_pronunciation_attempt",
            "interactive_lesson_exercise_attempt",
        ];
        let before = Connection::open(&database).unwrap();
        let counts: Vec<i64> = tables
            .iter()
            .map(|table| {
                before
                    .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                    .unwrap()
            })
            .collect();
        let before_version: i64 = before
            .query_row("SELECT MAX(version) FROM schema_migration", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(before_version, 16);
        drop(before);
        migrate(&database).unwrap();
        migrate(&database).unwrap();
        let after = Connection::open(&database).unwrap();
        let after_counts: Vec<i64> = tables
            .iter()
            .map(|table| {
                after
                    .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                    .unwrap()
            })
            .collect();
        assert_eq!(counts, after_counts);
        let version: i64 = after
            .query_row("SELECT MAX(version) FROM schema_migration", [], |r| {
                r.get(0)
            })
            .unwrap();
        let turns: i64 = after
            .query_row(
                "SELECT COUNT(*) FROM interactive_lesson_guided_conversation_turn",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let integrity: String = after
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        let foreign: i64 = after
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            (version, turns, integrity.as_str(), foreign),
            (18, 0, "ok", 0)
        );
        println!("physical_phase_u|schema=18|counts={after_counts:?}|guided_turns=0|integrity=ok|foreign_keys=0");
    }
}
