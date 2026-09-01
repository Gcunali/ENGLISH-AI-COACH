CREATE TABLE IF NOT EXISTS toeic_session (
  id TEXT PRIMARY KEY,
  form_id TEXT NOT NULL,
  form_version INTEGER NOT NULL CHECK(form_version >= 1),
  section TEXT NOT NULL CHECK(section IN ('listening','reading')),
  part TEXT NOT NULL CHECK(part IN (
    'part1_photograph','part2_question_response','part3_conversation','part4_talk',
    'part5_incomplete_sentence','part6_text_completion','part7_reading_comprehension'
  )),
  status TEXT NOT NULL CHECK(status IN ('in_progress','completed','abandoned')),
  schema_version INTEGER NOT NULL CHECK(schema_version = 1),
  form_snapshot_json TEXT NOT NULL CHECK(json_valid(form_snapshot_json)),
  current_question_index INTEGER NOT NULL CHECK(current_question_index >= 0 AND current_question_index <= 200),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  completed_at TEXT,
  abandoned_at TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS toeic_one_active_form_session
  ON toeic_session(form_id, form_version)
  WHERE status = 'in_progress';

CREATE INDEX IF NOT EXISTS toeic_session_history
  ON toeic_session(created_at DESC, id);

CREATE TABLE IF NOT EXISTS toeic_answer (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  item_id TEXT NOT NULL,
  item_version INTEGER NOT NULL CHECK(item_version >= 1),
  selected_choice TEXT NOT NULL CHECK(selected_choice IN ('A','B','C','D')),
  is_correct INTEGER NOT NULL CHECK(is_correct IN (0,1)),
  first_attempt INTEGER NOT NULL CHECK(first_attempt = 1),
  answered_at TEXT NOT NULL,
  UNIQUE(session_id, item_id, item_version),
  FOREIGN KEY(session_id) REFERENCES toeic_session(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS toeic_answer_session
  ON toeic_answer(session_id, answered_at, id);

CREATE TABLE IF NOT EXISTS toeic_presentation_attempt (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  item_id TEXT NOT NULL,
  item_version INTEGER NOT NULL CHECK(item_version >= 1),
  status TEXT NOT NULL CHECK(status IN ('started','completed','interrupted')),
  started_at TEXT NOT NULL,
  completed_at TEXT,
  interrupted_at TEXT,
  FOREIGN KEY(session_id) REFERENCES toeic_session(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS toeic_presentation_session_item
  ON toeic_presentation_attempt(session_id, item_id, item_version, started_at);

CREATE UNIQUE INDEX IF NOT EXISTS toeic_one_started_presentation
  ON toeic_presentation_attempt(session_id, item_id, item_version)
  WHERE status = 'started';

CREATE TABLE IF NOT EXISTS toeic_active_time_event (
  event_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  duration_seconds INTEGER NOT NULL CHECK(duration_seconds BETWEEN 1 AND 30),
  recorded_at TEXT NOT NULL,
  FOREIGN KEY(session_id) REFERENCES toeic_session(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS toeic_active_time_session
  ON toeic_active_time_event(session_id, recorded_at);

INSERT OR IGNORE INTO schema_migration(version) VALUES (21);
