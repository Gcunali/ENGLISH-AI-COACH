CREATE TABLE IF NOT EXISTS interactive_lesson_guided_conversation_turn (
  id TEXT PRIMARY KEY NOT NULL,
  event_id TEXT NOT NULL,
  session_id TEXT NOT NULL,
  stage_id TEXT NOT NULL,
  sequence_index INTEGER NOT NULL CHECK(sequence_index >= 0),
  role TEXT NOT NULL CHECK(role IN ('student','assistant')),
  text TEXT NOT NULL CHECK(length(text) BETWEEN 1 AND 8000),
  text_schema_version INTEGER NOT NULL CHECK(text_schema_version = 1),
  word_count INTEGER NOT NULL CHECK(word_count >= 1),
  partial INTEGER NOT NULL DEFAULT 0 CHECK(partial IN (0,1)),
  created_at TEXT NOT NULL,
  committed_at TEXT NOT NULL,
  UNIQUE(session_id, stage_id, sequence_index),
  UNIQUE(session_id, stage_id, event_id),
  FOREIGN KEY(session_id, stage_id)
    REFERENCES interactive_lesson_stage_state(session_id, stage_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_guided_conversation_turn_history
ON interactive_lesson_guided_conversation_turn(session_id, stage_id, sequence_index);

INSERT OR IGNORE INTO schema_migration(version) VALUES (17);
