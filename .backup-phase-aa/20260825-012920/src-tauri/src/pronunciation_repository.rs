use crate::{
    database,
    pronunciation::{PronunciationAttemptDto, PronunciationResult},
};
use rusqlite::{params, OptionalExtension};
use std::path::PathBuf;

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";

#[derive(Clone)]
pub struct PronunciationRepository {
    database: PathBuf,
}

impl PronunciationRepository {
    pub fn new(database: PathBuf) -> Self {
        Self { database }
    }
    pub fn save(
        &self,
        result: &PronunciationResult,
        source_type: &str,
        source_id: Option<&str>,
    ) -> Result<PronunciationAttemptDto, String> {
        if !matches!(
            source_type,
            "custom" | "vocabulary" | "diagnostic" | "interactive_lesson"
        ) {
            return Err("Invalid pronunciation source type.".into());
        }
        let mut connection = database::open(&self.database)?;
        let transaction = connection.transaction().map_err(db)?;
        let id = uuid::Uuid::new_v4().to_string();
        let normalized = result
            .target_text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        let word_count = result
            .target_text
            .split_whitespace()
            .filter(|word| word.chars().any(char::is_alphabetic))
            .count() as u32;
        transaction.execute(&format!("INSERT INTO pronunciation_attempt(id,status,source_type,source_id,target_text,normalized_target,locale,engine_version,score_version,result_schema_version,model_id,model_revision,model_manifest_hash,overall_score,confidence,content_match_score,alignment_coverage,audio_duration_ms,word_count,created_at,completed_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,{NOW},{NOW})"),params![id,result.status,source_type,source_id,result.target_text,normalized,result.locale,result.engine_version,result.score_version,result.schema_version,result.model_id,result.model_revision,result.model_manifest_hash,result.overall_score,result.confidence,result.content_match_score,result.alignment_coverage,result.duration_ms,word_count]).map_err(db)?;
        for word in &result.words {
            let expected =
                serde_json::to_string(&word.expected_phones).map_err(|e| e.to_string())?;
            let phones = serde_json::to_string(&word.phone_results).map_err(|e| e.to_string())?;
            transaction.execute("INSERT INTO pronunciation_word_result(attempt_id,word_index,target_word,score,start_ms,end_ms,expected_phones_json,phone_results_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![id,word.index,word.word,word.score,word.start_ms,word.end_ms,expected,phones]).map_err(db)?;
        }
        transaction.commit().map_err(db)?;
        self.get(&id)?
            .ok_or_else(|| "Pronunciation attempt was not readable after save.".into())
    }
    pub fn list(&self, limit: u32) -> Result<Vec<PronunciationAttemptDto>, String> {
        let connection = database::open(&self.database)?;
        let mut statement=connection.prepare("SELECT id,status,source_type,source_id,target_text,locale,overall_score,confidence,content_match_score,alignment_coverage,audio_duration_ms,created_at,completed_at FROM pronunciation_attempt WHERE source_type IN ('custom','vocabulary') ORDER BY created_at DESC,id DESC LIMIT ?1").map_err(db)?;
        let rows = statement
            .query_map([limit.clamp(1, 20)], map_attempt)
            .map_err(db)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db)?;
        rows.into_iter()
            .map(|mut a| {
                a.words = self.words(&a.id)?;
                Ok(a)
            })
            .collect()
    }
    pub fn get(&self, id: &str) -> Result<Option<PronunciationAttemptDto>, String> {
        let connection = database::open(&self.database)?;
        let mut value=connection.query_row("SELECT id,status,source_type,source_id,target_text,locale,overall_score,confidence,content_match_score,alignment_coverage,audio_duration_ms,created_at,completed_at FROM pronunciation_attempt WHERE id=?1",[id],map_attempt).optional().map_err(db)?;
        if let Some(attempt) = value.as_mut() {
            attempt.words = self.words(id)?
        }
        Ok(value)
    }
    fn words(
        &self,
        id: &str,
    ) -> Result<Vec<crate::pronunciation::PronunciationWordResult>, String> {
        let connection = database::open(&self.database)?;
        let mut statement=connection.prepare("SELECT word_index,target_word,score,start_ms,end_ms,expected_phones_json,phone_results_json FROM pronunciation_word_result WHERE attempt_id=?1 ORDER BY word_index").map_err(db)?;
        let results = statement
            .query_map([id], |row| {
                let expected: String = row.get(5)?;
                let phones: String = row.get(6)?;
                Ok((
                    row.get::<_, u32>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, u32>(3)?,
                    row.get::<_, u32>(4)?,
                    expected,
                    phones,
                ))
            })
            .map_err(db)?
            .map(|value| {
                let (i, w, s, start, end, e, p) = value.map_err(db)?;
                Ok(crate::pronunciation::PronunciationWordResult {
                    index: i,
                    word: w,
                    score: s,
                    start_ms: start,
                    end_ms: end,
                    expected_phones: serde_json::from_str(&e).map_err(|x| x.to_string())?,
                    phone_results: serde_json::from_str(&p).map_err(|x| x.to_string())?,
                })
            })
            .collect();
        results
    }
}
fn map_attempt(row: &rusqlite::Row<'_>) -> rusqlite::Result<PronunciationAttemptDto> {
    Ok(PronunciationAttemptDto {
        id: row.get(0)?,
        status: row.get(1)?,
        source_type: row.get(2)?,
        source_id: row.get(3)?,
        target_text: row.get(4)?,
        locale: row.get(5)?,
        overall_score: row.get(6)?,
        confidence: row.get(7)?,
        content_match_score: row.get(8)?,
        alignment_coverage: row.get(9)?,
        audio_duration_ms: row.get(10)?,
        created_at: row.get(11)?,
        completed_at: row.get(12)?,
        words: vec![],
    })
}
fn db(error: rusqlite::Error) -> String {
    format!("Pronunciation database error: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{database, pronunciation::*};
    #[test]
    fn persists_results_without_audio_or_transcript() {
        let dir = std::env::temp_dir().join(format!("pron-repo-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let dbp = dir.join("a.db");
        database::migrate(&dbp).unwrap();
        let repo = PronunciationRepository::new(dbp.clone());
        let result = PronunciationResult {
            schema_version: 1,
            engine_version: 1,
            score_version: 1,
            status: "content_mismatch".into(),
            target_text: "think".into(),
            locale: "en-US".into(),
            model_id: PRONUNCIATION_MODEL_ID.into(),
            model_revision: PRONUNCIATION_MODEL_REVISION.into(),
            model_manifest_hash: "a".repeat(64),
            overall_score: None,
            confidence: None,
            content_match_score: Some(0.0),
            alignment_coverage: None,
            duration_ms: Some(500),
            words: vec![],
            issues: vec![],
            quality_warnings: vec![],
            heard_text: Some("sink".into()),
            analysis_ms: Some(2),
        };
        repo.save(&result, "custom", None).unwrap();
        let c = rusqlite::Connection::open(dbp).unwrap();
        let sql: String = c
            .query_row(
                "SELECT sql FROM sqlite_master WHERE name='pronunciation_attempt'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(!sql.contains("transcript"));
        assert!(!sql.contains("audio_path"));
        assert_eq!(repo.list(20).unwrap().len(), 1);
        drop(c);
        std::fs::remove_dir_all(dir).unwrap()
    }
    #[test]
    fn guided_low_score_keeps_provenance_and_is_hidden_from_standalone_history() {
        let dir = std::env::temp_dir().join(format!("pron-guided-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let dbp = dir.join("a.db");
        database::migrate(&dbp).unwrap();
        let repo = PronunciationRepository::new(dbp.clone());
        let result = PronunciationResult {
            schema_version: 1,
            engine_version: 1,
            score_version: 1,
            status: "completed".into(),
            target_text: "hello".into(),
            locale: "en-US".into(),
            model_id: PRONUNCIATION_MODEL_ID.into(),
            model_revision: PRONUNCIATION_MODEL_REVISION.into(),
            model_manifest_hash: "a".repeat(64),
            overall_score: Some(12.0),
            confidence: Some("low".into()),
            content_match_score: Some(1.0),
            alignment_coverage: Some(0.5),
            duration_ms: Some(600),
            words: vec![],
            issues: vec![],
            quality_warnings: vec![],
            heard_text: Some("hello".into()),
            analysis_ms: Some(3),
        };
        let saved = repo
            .save(&result, "interactive_lesson", Some("guided-attempt"))
            .unwrap();
        assert_eq!(saved.overall_score, Some(12.0));
        assert_eq!(saved.source_type, "interactive_lesson");
        assert!(repo.list(20).unwrap().is_empty());
        assert!(repo.get(&saved.id).unwrap().is_some());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    #[ignore = "audits the user's physical database only when explicitly requested"]
    fn physical_phase_o_migrates_without_fabricating_attempts() {
        let database_path = std::path::PathBuf::from(
            r"C:\Users\guicu\AppData\Local\com.englishaicoach.desktop\database\english-ai-coach.sqlite3",
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
            "voice_turn_performance",
        ];
        let before_connection = database::open(&database_path).unwrap();
        let before = tables.map(|table| {
            before_connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap()
        });
        drop(before_connection);
        database::migrate(&database_path).unwrap();
        database::migrate(&database_path).unwrap();
        let connection = database::open(&database_path).unwrap();
        let after = tables.map(|table| {
            connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap()
        });
        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migration", [], |r| {
                r.get(0)
            })
            .unwrap();
        let attempts: i64 = connection
            .query_row("SELECT COUNT(*) FROM pronunciation_attempt", [], |r| {
                r.get(0)
            })
            .unwrap();
        let words: i64 = connection
            .query_row("SELECT COUNT(*) FROM pronunciation_word_result", [], |r| {
                r.get(0)
            })
            .unwrap();
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        let foreign_keys: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(before, after);
        assert_eq!((version, attempts, words), (12, 0, 0));
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_keys, 0);
        println!("PHASE_O migration={version} attempts={attempts} words={words} lessons={} transcripts={} analyses={} vocabulary={} mistakes={} placements={} xp={} achievements={} reviews={} voice_metrics={} integrity={integrity} foreign_keys={foreign_keys}",after[0],after[1],after[2],after[3],after[4],after[5],after[6],after[7],after[8],after[9]);
    }
}
