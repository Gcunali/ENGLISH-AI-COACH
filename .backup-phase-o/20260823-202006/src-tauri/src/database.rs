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
    fn migrates_a_new_database_through_version_eleven() {
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
        assert_eq!(versions, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
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
        assert_eq!(migration_count, 11);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
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
        assert_eq!(versions, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
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
        assert_eq!(versions, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
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
        assert_eq!(versions, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
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
        assert_eq!(versions, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
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
        assert_eq!(versions, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
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
        assert_eq!(versions, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
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
        assert_eq!(versions, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
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
        assert_eq!(versions, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        assert_eq!((lessons, reviews, metrics), (1, 1, 0));
        assert_eq!(setting, "true");
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_keys, 0);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
