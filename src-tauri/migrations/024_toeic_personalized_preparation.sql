CREATE TABLE IF NOT EXISTS toeic_personalized_practice_session (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL CHECK(kind IN ('smart','recent_mistakes','daily')),
  requested_count INTEGER NOT NULL CHECK(requested_count IN (10,12,15,20)),
  status TEXT NOT NULL CHECK(status IN ('in_progress','completed','abandoned')),
  focus_json TEXT NOT NULL CHECK(json_valid(focus_json)),
  correct INTEGER CHECK(correct >= 0 AND correct <= requested_count),
  answered INTEGER NOT NULL DEFAULT 0 CHECK(answered >= 0 AND answered <= requested_count),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  completed_at TEXT,
  abandoned_at TEXT
);

CREATE INDEX IF NOT EXISTS toeic_personalized_practice_history
  ON toeic_personalized_practice_session(created_at DESC, id);

CREATE UNIQUE INDEX IF NOT EXISTS toeic_one_active_personalized_practice
  ON toeic_personalized_practice_session(kind)
  WHERE status = 'in_progress';

CREATE TABLE IF NOT EXISTS toeic_personalized_practice_step (
  practice_session_id TEXT NOT NULL,
  step_number INTEGER NOT NULL CHECK(step_number >= 1 AND step_number <= 4),
  part_number INTEGER NOT NULL CHECK(part_number BETWEEN 1 AND 7),
  form_id TEXT NOT NULL,
  form_version INTEGER NOT NULL CHECK(form_version >= 1),
  toeic_session_id TEXT NOT NULL UNIQUE,
  quota INTEGER NOT NULL CHECK(quota BETWEEN 1 AND 20),
  baseline_answered INTEGER NOT NULL CHECK(baseline_answered >= 0),
  frozen_item_ids_json TEXT NOT NULL CHECK(json_valid(frozen_item_ids_json)),
  status TEXT NOT NULL CHECK(status IN ('pending','in_progress','completed')),
  PRIMARY KEY(practice_session_id, step_number),
  FOREIGN KEY(practice_session_id) REFERENCES toeic_personalized_practice_session(id) ON DELETE CASCADE,
  FOREIGN KEY(toeic_session_id) REFERENCES toeic_session(id) ON DELETE CASCADE
);

INSERT OR IGNORE INTO settings(key, value_json)
VALUES('toeic_target_score', '750');

INSERT OR IGNORE INTO schema_migration(version) VALUES(24);
