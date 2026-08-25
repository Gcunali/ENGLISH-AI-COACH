CREATE TABLE IF NOT EXISTS voice_turn_performance (
  id TEXT PRIMARY KEY,
  lesson_id TEXT,
  turn_id TEXT NOT NULL UNIQUE CHECK (length(trim(turn_id)) > 0),
  runtime_version INTEGER NOT NULL CHECK (runtime_version > 0),
  streaming_enabled INTEGER NOT NULL CHECK (streaming_enabled IN (0,1)),
  stt_ms INTEGER CHECK (stt_ms IS NULL OR stt_ms >= 0),
  llm_ttft_ms INTEGER CHECK (llm_ttft_ms IS NULL OR llm_ttft_ms >= 0),
  llm_first_sentence_ms INTEGER CHECK (llm_first_sentence_ms IS NULL OR llm_first_sentence_ms >= 0),
  llm_total_ms INTEGER CHECK (llm_total_ms IS NULL OR llm_total_ms >= 0),
  first_tts_ms INTEGER CHECK (first_tts_ms IS NULL OR first_tts_ms >= 0),
  speech_end_to_first_audio_ms INTEGER CHECK (speech_end_to_first_audio_ms IS NULL OR speech_end_to_first_audio_ms >= 0),
  last_voice_to_first_audio_ms INTEGER CHECK (last_voice_to_first_audio_ms IS NULL OR last_voice_to_first_audio_ms >= 0),
  capture_end_to_first_audio_ms INTEGER CHECK (capture_end_to_first_audio_ms IS NULL OR capture_end_to_first_audio_ms >= 0),
  tts_total_ms INTEGER CHECK (tts_total_ms IS NULL OR tts_total_ms >= 0),
  teacher_playback_ms INTEGER CHECK (teacher_playback_ms IS NULL OR teacher_playback_ms >= 0),
  teacher_turn_total_ms INTEGER CHECK (teacher_turn_total_ms IS NULL OR teacher_turn_total_ms >= 0),
  tts_chunk_count INTEGER NOT NULL DEFAULT 0 CHECK (tts_chunk_count >= 0),
  cancelled INTEGER NOT NULL CHECK (cancelled IN (0,1)),
  fallback_used INTEGER NOT NULL CHECK (fallback_used IN (0,1)),
  created_at TEXT NOT NULL,
  FOREIGN KEY (lesson_id) REFERENCES lesson(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS voice_turn_performance_lesson
  ON voice_turn_performance(lesson_id, created_at);
CREATE INDEX IF NOT EXISTS voice_turn_performance_comparison
  ON voice_turn_performance(streaming_enabled, created_at DESC);

INSERT OR IGNORE INTO settings(key, value_json)
VALUES ('use_streaming_voice_response', 'true');

INSERT OR IGNORE INTO schema_migration(version) VALUES (11);
