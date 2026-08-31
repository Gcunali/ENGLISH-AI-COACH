use crate::database;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const VOICE_STREAMING_RUNTIME_VERSION: u32 = 1;
const STREAMING_SETTING_KEY: &str = "use_streaming_voice_response";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceTurnPerformanceDto {
    pub turn_id: String,
    pub runtime_version: u32,
    pub streaming_enabled: bool,
    pub stt_ms: Option<u32>,
    pub llm_ttft_ms: Option<u32>,
    pub llm_first_sentence_ms: Option<u32>,
    pub llm_total_ms: Option<u32>,
    pub first_tts_ms: Option<u32>,
    pub speech_end_to_first_audio_ms: Option<u32>,
    pub last_voice_to_first_audio_ms: Option<u32>,
    pub capture_end_to_first_audio_ms: Option<u32>,
    pub tts_total_ms: Option<u32>,
    pub teacher_playback_ms: Option<u32>,
    pub teacher_turn_total_ms: Option<u32>,
    pub tts_chunk_count: u32,
    pub cancelled: bool,
    pub fallback_used: bool,
    pub created_at: String,
}

#[derive(Clone)]
pub struct VoicePerformanceRepository {
    database_path: PathBuf,
}

impl VoicePerformanceRepository {
    pub fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }

    pub fn streaming_enabled(&self) -> Result<bool, String> {
        let connection = database::open(&self.database_path)?;
        let value: Option<String> = connection
            .query_row(
                "SELECT value_json FROM settings WHERE key=?1",
                [STREAMING_SETTING_KEY],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| format!("Could not read voice performance setting: {error}"))?;
        match value.as_deref() {
            None | Some("true") => Ok(true),
            Some("false") => Ok(false),
            Some(_) => Err("Voice performance setting is invalid.".to_owned()),
        }
    }

    pub fn set_streaming_enabled(&self, enabled: bool) -> Result<bool, String> {
        let connection = database::open(&self.database_path)?;
        connection
            .execute(
                "INSERT INTO settings(key,value_json,updated_at) VALUES(?1,?2,strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                 ON CONFLICT(key) DO UPDATE SET value_json=excluded.value_json, updated_at=excluded.updated_at",
                params![STREAMING_SETTING_KEY, if enabled { "true" } else { "false" }],
            )
            .map_err(|error| format!("Could not persist voice performance setting: {error}"))?;
        Ok(enabled)
    }

    pub fn record(
        &self,
        lesson_id: Option<&str>,
        metric: &VoiceTurnPerformanceDto,
    ) -> Result<bool, String> {
        validate_metric(metric)?;
        let connection = database::open(&self.database_path)?;
        let changed = connection
            .execute(
                "INSERT OR IGNORE INTO voice_turn_performance(
                   id,lesson_id,turn_id,runtime_version,streaming_enabled,stt_ms,llm_ttft_ms,
                   llm_first_sentence_ms,llm_total_ms,first_tts_ms,speech_end_to_first_audio_ms,
                   last_voice_to_first_audio_ms,capture_end_to_first_audio_ms,tts_total_ms,
                   teacher_playback_ms,teacher_turn_total_ms,tts_chunk_count,cancelled,
                   fallback_used,created_at
                 ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",
                params![
                    uuid::Uuid::new_v4().to_string(),
                    lesson_id,
                    metric.turn_id,
                    metric.runtime_version,
                    metric.streaming_enabled,
                    metric.stt_ms,
                    metric.llm_ttft_ms,
                    metric.llm_first_sentence_ms,
                    metric.llm_total_ms,
                    metric.first_tts_ms,
                    metric.speech_end_to_first_audio_ms,
                    metric.last_voice_to_first_audio_ms,
                    metric.capture_end_to_first_audio_ms,
                    metric.tts_total_ms,
                    metric.teacher_playback_ms,
                    metric.teacher_turn_total_ms,
                    metric.tts_chunk_count,
                    metric.cancelled,
                    metric.fallback_used,
                    metric.created_at,
                ],
            )
            .map_err(|error| {
                format!("Could not persist local voice performance metric: {error}")
            })?;
        Ok(changed == 1)
    }
}

fn validate_metric(metric: &VoiceTurnPerformanceDto) -> Result<(), String> {
    if metric.turn_id.trim().is_empty() || metric.turn_id.len() > 100 {
        return Err("Voice performance turn id is invalid.".to_owned());
    }
    if metric.runtime_version != VOICE_STREAMING_RUNTIME_VERSION {
        return Err(format!(
            "Unsupported voice streaming runtime version {}.",
            metric.runtime_version
        ));
    }
    if metric.created_at.trim().is_empty() || metric.created_at.len() > 64 {
        return Err("Voice performance timestamp is invalid.".to_owned());
    }
    const MAX_DURATION_MS: u32 = 60 * 60 * 1000;
    for value in [
        metric.stt_ms,
        metric.llm_ttft_ms,
        metric.llm_first_sentence_ms,
        metric.llm_total_ms,
        metric.first_tts_ms,
        metric.speech_end_to_first_audio_ms,
        metric.last_voice_to_first_audio_ms,
        metric.capture_end_to_first_audio_ms,
        metric.tts_total_ms,
        metric.teacher_playback_ms,
        metric.teacher_turn_total_ms,
    ] {
        if value.is_some_and(|duration| duration > MAX_DURATION_MS) {
            return Err("Voice performance duration is outside the supported range.".to_owned());
        }
    }
    if metric.tts_chunk_count > 20 {
        return Err("Voice performance chunk count is outside the supported range.".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    fn repository() -> (PathBuf, VoicePerformanceRepository) {
        let root = std::env::temp_dir().join(format!("voice-performance-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let db = root.join("test.sqlite3");
        database::migrate(&db).unwrap();
        (root, VoicePerformanceRepository::new(db))
    }

    fn metric(turn_id: &str) -> VoiceTurnPerformanceDto {
        VoiceTurnPerformanceDto {
            turn_id: turn_id.to_owned(),
            runtime_version: 1,
            streaming_enabled: true,
            stt_ms: Some(100),
            llm_ttft_ms: Some(50),
            llm_first_sentence_ms: Some(200),
            llm_total_ms: Some(600),
            first_tts_ms: Some(120),
            speech_end_to_first_audio_ms: Some(4100),
            last_voice_to_first_audio_ms: Some(4100),
            capture_end_to_first_audio_ms: Some(600),
            tts_total_ms: Some(240),
            teacher_playback_ms: Some(900),
            teacher_turn_total_ms: Some(1600),
            tts_chunk_count: 2,
            cancelled: false,
            fallback_used: false,
            created_at: "2026-08-22T00:00:00Z".to_owned(),
        }
    }

    #[test]
    fn setting_defaults_on_and_persists() {
        let (root, repository) = repository();
        assert!(repository.streaming_enabled().unwrap());
        assert!(!repository.set_streaming_enabled(false).unwrap());
        assert!(
            !VoicePerformanceRepository::new(repository.database_path.clone())
                .streaming_enabled()
                .unwrap()
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metrics_are_validated_and_idempotent() {
        let (root, repository) = repository();
        assert!(repository.record(None, &metric("turn-1")).unwrap());
        assert!(!repository.record(None, &metric("turn-1")).unwrap());
        let mut invalid = metric("turn-2");
        invalid.tts_chunk_count = 21;
        assert!(repository.record(None, &invalid).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "manual migration and preservation audit against the user's physical SQLite database"]
    fn physical_phase_n_migrates_without_fabricating_metrics() {
        let path = PathBuf::from(std::env::var_os("LOCALAPPDATA").expect("LOCALAPPDATA"))
            .join("com.englishaicoach.desktop")
            .join("database")
            .join("english-ai-coach.sqlite3");
        let tables = [
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
        ];
        let before = database::open(&path).unwrap();
        let before_counts = tables
            .iter()
            .map(|table| {
                before
                    .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                        row.get::<_, i64>(0)
                    })
                    .unwrap()
            })
            .collect::<Vec<_>>();
        drop(before);
        database::migrate(&path).unwrap();
        let repository = VoicePerformanceRepository::new(path.clone());
        let setting = repository.streaming_enabled().unwrap();
        let connection = database::open(&path).unwrap();
        let after_counts = tables
            .iter()
            .map(|table| {
                connection
                    .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                        row.get::<_, i64>(0)
                    })
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let metrics: i64 = connection
            .query_row("SELECT COUNT(*) FROM voice_turn_performance", [], |row| {
                row.get(0)
            })
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
        let foreign_keys: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(before_counts, after_counts);
        assert_eq!(metrics, 0);
        assert_eq!(versions, (1..=11).collect::<Vec<_>>());
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_keys, 0);
        drop(connection);
        database::migrate(&path).unwrap();
        assert_eq!(
            database::open(&path)
                .unwrap()
                .query_row("SELECT COUNT(*) FROM voice_turn_performance", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        println!("PHASE_N migration=11 streaming_default={} metrics={} lessons={} transcript_messages={} analyses={} vocabulary={} mistakes={} placements={} xp_events={} achievements={} review_sessions={} integrity={} foreign_keys={}", setting, metrics, after_counts[0], after_counts[1], after_counts[3], after_counts[4], after_counts[6], after_counts[11], after_counts[16], after_counts[18], after_counts[19], integrity, foreign_keys);
    }
}
