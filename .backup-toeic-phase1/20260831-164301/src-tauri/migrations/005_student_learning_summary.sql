CREATE TABLE IF NOT EXISTS student_learning_summary (
  profile_key TEXT PRIMARY KEY CHECK (profile_key = 'default'),
  schema_version INTEGER NOT NULL CHECK (schema_version > 0),
  generated_at TEXT NOT NULL,
  analyzed_lesson_count INTEGER NOT NULL CHECK (analyzed_lesson_count >= 0),
  completed_lesson_count INTEGER NOT NULL CHECK (completed_lesson_count >= 0),
  content_json TEXT NOT NULL CHECK (length(trim(content_json)) > 0),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS lesson_teacher_memory (
  lesson_id TEXT PRIMARY KEY,
  memory_enabled INTEGER NOT NULL CHECK (memory_enabled IN (0, 1)),
  context_loaded INTEGER NOT NULL CHECK (context_loaded IN (0, 1)),
  context_version INTEGER,
  summary_schema_version INTEGER NOT NULL,
  analyzed_lesson_count_used INTEGER NOT NULL CHECK (analyzed_lesson_count_used >= 0),
  created_at TEXT NOT NULL,
  FOREIGN KEY (lesson_id) REFERENCES lesson(id) ON DELETE CASCADE
);

INSERT OR IGNORE INTO settings(key, value_json)
VALUES ('use_learning_memory_in_lessons', 'true');

INSERT OR IGNORE INTO schema_migration(version) VALUES (5);
