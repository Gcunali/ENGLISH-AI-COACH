CREATE TABLE IF NOT EXISTS pronunciation_attempt_v15 (
  id TEXT PRIMARY KEY,
  status TEXT NOT NULL CHECK(status IN ('completed','content_mismatch','insufficient_audio','alignment_failed','engine_unavailable','cancelled','failed')),
  source_type TEXT NOT NULL CHECK(source_type IN ('custom','vocabulary','diagnostic','interactive_lesson')),
  source_id TEXT,
  target_text TEXT NOT NULL CHECK(length(target_text) BETWEEN 1 AND 160),
  normalized_target TEXT NOT NULL,
  locale TEXT NOT NULL CHECK(locale='en-US'),
  engine_version INTEGER NOT NULL CHECK(engine_version=1),
  score_version INTEGER NOT NULL CHECK(score_version=1),
  result_schema_version INTEGER NOT NULL CHECK(result_schema_version=1),
  model_id TEXT NOT NULL,
  model_revision TEXT NOT NULL,
  model_manifest_hash TEXT NOT NULL CHECK(length(model_manifest_hash)=64),
  overall_score REAL CHECK(overall_score IS NULL OR overall_score BETWEEN 0 AND 100),
  confidence TEXT CHECK(confidence IS NULL OR confidence IN ('low','medium','high')),
  content_match_score REAL CHECK(content_match_score IS NULL OR content_match_score BETWEEN 0 AND 1),
  alignment_coverage REAL CHECK(alignment_coverage IS NULL OR alignment_coverage BETWEEN 0 AND 1),
  audio_duration_ms INTEGER CHECK(audio_duration_ms IS NULL OR audio_duration_ms BETWEEN 0 AND 15000),
  word_count INTEGER NOT NULL CHECK(word_count BETWEEN 1 AND 12),
  error_code TEXT,
  created_at TEXT NOT NULL,
  completed_at TEXT
);

INSERT OR IGNORE INTO pronunciation_attempt_v15
SELECT * FROM pronunciation_attempt;

CREATE TABLE IF NOT EXISTS pronunciation_word_result_v15 (
  attempt_id TEXT NOT NULL,
  word_index INTEGER NOT NULL CHECK(word_index >= 0),
  target_word TEXT NOT NULL,
  score REAL NOT NULL CHECK(score BETWEEN 0 AND 100),
  start_ms INTEGER NOT NULL CHECK(start_ms >= 0),
  end_ms INTEGER NOT NULL CHECK(end_ms >= start_ms),
  expected_phones_json TEXT NOT NULL CHECK(json_valid(expected_phones_json)),
  phone_results_json TEXT NOT NULL CHECK(json_valid(phone_results_json)),
  PRIMARY KEY(attempt_id, word_index),
  FOREIGN KEY(attempt_id) REFERENCES pronunciation_attempt_v15(id) ON DELETE CASCADE
);

INSERT OR IGNORE INTO pronunciation_word_result_v15
SELECT * FROM pronunciation_word_result;

DROP TABLE pronunciation_word_result;
DROP TABLE pronunciation_attempt;
ALTER TABLE pronunciation_attempt_v15 RENAME TO pronunciation_attempt;
ALTER TABLE pronunciation_word_result_v15 RENAME TO pronunciation_word_result;

CREATE INDEX IF NOT EXISTS pronunciation_attempt_recent
  ON pronunciation_attempt(created_at DESC);
CREATE INDEX IF NOT EXISTS pronunciation_attempt_source
  ON pronunciation_attempt(source_type, source_id, created_at DESC);

CREATE TABLE IF NOT EXISTS interactive_lesson_stage_runtime_state (
  session_id TEXT NOT NULL,
  stage_id TEXT NOT NULL,
  runtime_state_schema_version INTEGER NOT NULL CHECK(runtime_state_schema_version = 1),
  state_json TEXT NOT NULL CHECK(json_valid(state_json)),
  updated_at TEXT NOT NULL,
  PRIMARY KEY(session_id, stage_id),
  FOREIGN KEY(session_id, stage_id)
    REFERENCES interactive_lesson_stage_state(session_id, stage_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS interactive_lesson_pronunciation_attempt (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  stage_id TEXT NOT NULL,
  item_id TEXT NOT NULL,
  stage_type TEXT NOT NULL CHECK(stage_type IN ('repeat','speaking_check')),
  attempt_index INTEGER NOT NULL CHECK(attempt_index >= 1),
  status TEXT NOT NULL CHECK(status IN ('analyzing','completed','content_mismatch','insufficient_audio','alignment_failed','engine_unavailable','cancelled','failed')),
  pronunciation_attempt_id TEXT,
  result_schema_version INTEGER NOT NULL CHECK(result_schema_version = 1),
  result_json TEXT CHECK(result_json IS NULL OR json_valid(result_json)),
  error_code TEXT,
  created_at TEXT NOT NULL,
  completed_at TEXT,
  updated_at TEXT NOT NULL,
  CHECK(status != 'completed' OR pronunciation_attempt_id IS NOT NULL),
  UNIQUE(session_id, stage_id, item_id, attempt_index),
  FOREIGN KEY(session_id, stage_id)
    REFERENCES interactive_lesson_stage_state(session_id, stage_id) ON DELETE CASCADE,
  FOREIGN KEY(pronunciation_attempt_id)
    REFERENCES pronunciation_attempt(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS ix_interactive_lesson_pronunciation_item
ON interactive_lesson_pronunciation_attempt(session_id, stage_id, item_id, attempt_index);

INSERT OR IGNORE INTO schema_migration(version) VALUES (15);
