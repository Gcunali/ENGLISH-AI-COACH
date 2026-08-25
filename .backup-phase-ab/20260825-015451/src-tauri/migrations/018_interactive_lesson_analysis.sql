CREATE TABLE IF NOT EXISTS interactive_lesson_analysis (
  id TEXT PRIMARY KEY NOT NULL,
  session_id TEXT NOT NULL UNIQUE REFERENCES interactive_lesson_session(id) ON DELETE CASCADE,
  stage_id TEXT NOT NULL,
  analysis_schema_version INTEGER NOT NULL CHECK(analysis_schema_version = 1),
  analysis_engine_version INTEGER NOT NULL CHECK(analysis_engine_version = 1),
  evidence_schema_version INTEGER NOT NULL CHECK(evidence_schema_version = 1),
  conversation_evaluator_version INTEGER NOT NULL CHECK(conversation_evaluator_version = 1),
  conversation_prompt_version INTEGER NOT NULL CHECK(conversation_prompt_version = 1),
  model_id TEXT,
  evidence_hash TEXT NOT NULL CHECK(length(evidence_hash) = 64),
  evidence_json TEXT NOT NULL CHECK(json_valid(evidence_json)),
  conversation_status TEXT NOT NULL CHECK(conversation_status IN ('pending','completed','insufficient_evidence','unavailable','not_practiced')),
  conversation_result_json TEXT CHECK(conversation_result_json IS NULL OR json_valid(conversation_result_json)),
  final_result_json TEXT NOT NULL CHECK(json_valid(final_result_json)),
  status TEXT NOT NULL CHECK(status IN ('pending','running','completed','partial','failed')),
  error_code TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  finalized_at TEXT,
  CHECK(finalized_at IS NULL OR status IN ('completed','partial')),
  FOREIGN KEY(session_id, stage_id)
    REFERENCES interactive_lesson_stage_state(session_id, stage_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_interactive_lesson_analysis_status
ON interactive_lesson_analysis(status, updated_at);

INSERT OR IGNORE INTO schema_migration(version) VALUES (18);
