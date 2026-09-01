CREATE TABLE IF NOT EXISTS learning_practice_session (
  id TEXT PRIMARY KEY,
  mode TEXT NOT NULL CHECK(mode IN ('daily','dictation','shadowing','mistake_repair','speaking_recall')),
  status TEXT NOT NULL CHECK(status IN ('in_progress','completed','abandoned')),
  schema_version INTEGER NOT NULL CHECK(schema_version=1),
  selection_version INTEGER NOT NULL CHECK(selection_version=1),
  requested_item_count INTEGER NOT NULL CHECK(requested_item_count BETWEEN 1 AND 20),
  actual_item_count INTEGER NOT NULL CHECK(actual_item_count BETWEEN 0 AND requested_item_count),
  items_json TEXT NOT NULL CHECK(json_valid(items_json)),
  started_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  completed_at TEXT,
  abandoned_at TEXT
);

CREATE INDEX IF NOT EXISTS learning_practice_session_recent
  ON learning_practice_session(started_at DESC, id);

CREATE TABLE IF NOT EXISTS learning_practice_item_result (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  item_id TEXT NOT NULL,
  result_schema_version INTEGER NOT NULL CHECK(result_schema_version=1),
  result_json TEXT NOT NULL CHECK(json_valid(result_json)),
  completed_at TEXT NOT NULL,
  UNIQUE(session_id, item_id),
  FOREIGN KEY(session_id) REFERENCES learning_practice_session(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS learning_practice_result_item_recent
  ON learning_practice_item_result(item_id, completed_at DESC);

CREATE TABLE IF NOT EXISTS learning_practice_active_time_event (
  event_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  duration_seconds INTEGER NOT NULL CHECK(duration_seconds BETWEEN 1 AND 30),
  recorded_at TEXT NOT NULL,
  FOREIGN KEY(session_id) REFERENCES learning_practice_session(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS learning_practice_active_time_session
  ON learning_practice_active_time_event(session_id, recorded_at);

CREATE TABLE IF NOT EXISTS learning_practice_xp_event (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  rule_version INTEGER NOT NULL CHECK(rule_version=1),
  xp_amount INTEGER NOT NULL CHECK(xp_amount=20),
  activity_day TEXT NOT NULL CHECK(length(activity_day)=10),
  created_at TEXT NOT NULL,
  UNIQUE(session_id, rule_version),
  FOREIGN KEY(session_id) REFERENCES learning_practice_session(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS learning_practice_xp_activity_day
  ON learning_practice_xp_event(activity_day);

INSERT OR IGNORE INTO schema_migration(version) VALUES (20);
