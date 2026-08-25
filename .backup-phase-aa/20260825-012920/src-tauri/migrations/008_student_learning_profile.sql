CREATE TABLE IF NOT EXISTS student_learning_profile (
  profile_key TEXT PRIMARY KEY CHECK (profile_key = 'default'),
  schema_version INTEGER NOT NULL CHECK (schema_version > 0),
  target_cefr_level TEXT CHECK (target_cefr_level IN ('A1','A2','B1','B2','C1','C2')),
  learning_goals_json TEXT NOT NULL,
  default_lesson_difficulty TEXT NOT NULL CHECK (default_lesson_difficulty IN ('easy','standard','challenging')),
  use_profile_in_lessons INTEGER NOT NULL CHECK (use_profile_in_lessons IN (0,1)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS lesson_student_profile_snapshot (
  lesson_id TEXT PRIMARY KEY,
  profile_schema_version INTEGER NOT NULL CHECK (profile_schema_version > 0),
  profile_context_version INTEGER NOT NULL CHECK (profile_context_version > 0),
  context_enabled INTEGER NOT NULL CHECK (context_enabled IN (0,1)),
  placement_attempt_id TEXT,
  estimated_cefr_level TEXT CHECK (estimated_cefr_level IN ('A1','A2','B1','B2','C1','C2')),
  placement_confidence TEXT CHECK (placement_confidence IN ('low','medium','high')),
  target_cefr_level TEXT CHECK (target_cefr_level IN ('A1','A2','B1','B2','C1','C2')),
  learning_goals_json TEXT NOT NULL,
  default_lesson_difficulty TEXT NOT NULL CHECK (default_lesson_difficulty IN ('easy','standard','challenging')),
  created_at TEXT NOT NULL,
  FOREIGN KEY (lesson_id) REFERENCES lesson(id) ON DELETE CASCADE,
  FOREIGN KEY (placement_attempt_id) REFERENCES placement_attempt(id) ON DELETE SET NULL
);

INSERT OR IGNORE INTO student_learning_profile (
  profile_key, schema_version, target_cefr_level, learning_goals_json,
  default_lesson_difficulty, use_profile_in_lessons, created_at, updated_at
) VALUES (
  'default', 1, NULL, '[]', 'standard', 1,
  strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')
);

INSERT OR IGNORE INTO schema_migration(version) VALUES (8);
