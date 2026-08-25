use crate::database;
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use std::path::Path;

const WELCOME_SEEN_KEY: &str = "phase_q_welcome_seen";

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WelcomeStateDto {
    pub should_show: bool,
    pub has_seen: bool,
    pub existing_user: bool,
}

pub fn welcome_state(path: &Path) -> Result<WelcomeStateDto, String> {
    let connection = database::open(path)?;
    let stored: Option<String> = connection
        .query_row(
            "SELECT value_json FROM settings WHERE key=?1",
            [WELCOME_SEEN_KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Could not read welcome preference: {error}"))?;
    let existing_user = has_existing_activity(&connection)?;
    let has_seen = match stored.as_deref() {
        Some("true") => true,
        Some("false") => false,
        Some(_) => return Err("Welcome preference is invalid.".to_owned()),
        None if existing_user => {
            persist_seen(&connection, true)?;
            true
        }
        None => false,
    };
    Ok(WelcomeStateDto {
        should_show: !has_seen && !existing_user,
        has_seen,
        existing_user,
    })
}

pub fn set_welcome_seen(path: &Path, seen: bool) -> Result<WelcomeStateDto, String> {
    let connection = database::open(path)?;
    persist_seen(&connection, seen)?;
    let existing_user = has_existing_activity(&connection)?;
    Ok(WelcomeStateDto {
        should_show: !seen && !existing_user,
        has_seen: seen,
        existing_user,
    })
}

fn persist_seen(connection: &rusqlite::Connection, seen: bool) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO settings(key,value_json,updated_at) VALUES(?1,?2,strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT(key) DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at",
            params![WELCOME_SEEN_KEY, if seen { "true" } else { "false" }],
        )
        .map_err(|error| format!("Could not persist welcome preference: {error}"))?;
    Ok(())
}

fn has_existing_activity(connection: &rusqlite::Connection) -> Result<bool, String> {
    let count: i64 = connection
        .query_row(
            "SELECT
              (SELECT COUNT(*) FROM lesson) +
              (SELECT COUNT(*) FROM placement_attempt) +
              (SELECT COUNT(*) FROM pronunciation_attempt) +
              (SELECT COUNT(*) FROM review_session)",
            [],
            |row| row.get(0),
        )
        .map_err(|error| format!("Could not detect existing learning activity: {error}"))?;
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::{set_welcome_seen, welcome_state, WELCOME_SEEN_KEY};
    use crate::database;
    use rusqlite::Connection;

    fn test_database() -> (std::path::PathBuf, std::path::PathBuf) {
        let directory =
            std::env::temp_dir().join(format!("english-ai-coach-ux-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).expect("create test directory");
        let path = directory.join("test.sqlite3");
        database::migrate(&path).expect("migrate database");
        (directory, path)
    }

    #[test]
    fn new_user_sees_welcome_until_dismissed() {
        let (directory, path) = test_database();
        assert!(welcome_state(&path).unwrap().should_show);
        let dismissed = set_welcome_seen(&path, true).unwrap();
        assert!(dismissed.has_seen);
        assert!(!dismissed.should_show);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn existing_user_is_bypassed_and_marked_seen() {
        let (directory, path) = test_database();
        let connection = Connection::open(&path).unwrap();
        connection.execute(
            "INSERT INTO lesson(id,started_at,ended_at,status,mode,whisper_model,whisper_threads,ollama_model,piper_voice,voice_engine_version,created_at,updated_at)
             VALUES('existing','2026-01-01T00:00:00Z','2026-01-01T00:10:00Z','completed','conversation','model',12,'qwen','voice','test','2026-01-01T00:00:00Z','2026-01-01T00:10:00Z')",
            [],
        ).unwrap();
        drop(connection);
        let state = welcome_state(&path).unwrap();
        assert!(state.existing_user);
        assert!(state.has_seen);
        assert!(!state.should_show);
        let stored: String = Connection::open(&path)
            .unwrap()
            .query_row(
                "SELECT value_json FROM settings WHERE key=?1",
                [WELCOME_SEEN_KEY],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored, "true");
        std::fs::remove_dir_all(directory).unwrap();
    }
}
