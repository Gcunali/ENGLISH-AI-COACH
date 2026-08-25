use rusqlite::{params, Connection};
use std::path::Path;

pub fn migrate(path: &Path) -> Result<(), String> {
    let connection =
        Connection::open(path).map_err(|error| format!("Database unavailable: {error}"))?;
    connection
        .execute_batch(include_str!("../migrations/001_initial.sql"))
        .map_err(|error| format!("Database migration failed: {error}"))?;
    Ok(())
}

pub fn save_exchange(path: &Path, student: &str, teacher: &str) -> Result<(), String> {
    let connection =
        Connection::open(path).map_err(|error| format!("Database unavailable: {error}"))?;
    connection.execute("INSERT INTO conversation_exchange (id, student_text, teacher_text) VALUES (?1, ?2, ?3)", params![uuid::Uuid::new_v4().to_string(), student, teacher])
        .map_err(|error| format!("Could not save transcript: {error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{migrate, save_exchange};
    use rusqlite::Connection;

    #[test]
    fn migrates_and_saves_an_exchange() {
        let directory =
            std::env::temp_dir().join(format!("english-ai-coach-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).expect("create test directory");
        let database = directory.join("test.sqlite3");
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
}
