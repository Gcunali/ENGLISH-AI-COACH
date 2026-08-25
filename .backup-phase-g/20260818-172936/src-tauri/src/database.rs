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
    fn migrates_a_new_database_through_version_two() {
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
        assert_eq!(versions, vec![1, 2, 3]);
        for table in [
            "lesson",
            "transcript_message",
            "correction_candidate",
            "lesson_analysis",
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
        assert_eq!(migration_count, 3);
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
        assert_eq!(versions, vec![1, 2, 3]);
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_key_errors, 0);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
