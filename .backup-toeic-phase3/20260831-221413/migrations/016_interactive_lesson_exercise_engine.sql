CREATE TABLE IF NOT EXISTS interactive_lesson_exercise_attempt (
  id TEXT PRIMARY KEY NOT NULL,
  submission_id TEXT NOT NULL,
  session_id TEXT NOT NULL,
  stage_id TEXT NOT NULL,
  exercise_id TEXT NOT NULL,
  exercise_type TEXT NOT NULL CHECK(exercise_type IN ('single_choice','multiple_select','fill_blank','word_order','matching','short_answer_exact')),
  attempt_index INTEGER NOT NULL CHECK(attempt_index >= 1),
  response_schema_version INTEGER NOT NULL CHECK(response_schema_version = 1),
  response_json TEXT NOT NULL CHECK(json_valid(response_json)),
  result_schema_version INTEGER NOT NULL CHECK(result_schema_version = 1),
  result_json TEXT NOT NULL CHECK(json_valid(result_json)),
  correct INTEGER NOT NULL CHECK(correct IN (0,1)),
  selected INTEGER NOT NULL DEFAULT 0 CHECK(selected IN (0,1)),
  submitted_at TEXT NOT NULL,
  selected_at TEXT,
  created_at TEXT NOT NULL,
  UNIQUE(session_id, stage_id, exercise_id, attempt_index),
  UNIQUE(session_id, stage_id, exercise_id, submission_id),
  FOREIGN KEY(session_id, stage_id)
    REFERENCES interactive_lesson_stage_state(session_id, stage_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_interactive_lesson_exercise_attempt_item
ON interactive_lesson_exercise_attempt(session_id, stage_id, exercise_id, attempt_index);

CREATE UNIQUE INDEX IF NOT EXISTS ux_interactive_lesson_exercise_selected
ON interactive_lesson_exercise_attempt(session_id, stage_id, exercise_id)
WHERE selected = 1;

INSERT OR IGNORE INTO schema_migration(version) VALUES (16);
