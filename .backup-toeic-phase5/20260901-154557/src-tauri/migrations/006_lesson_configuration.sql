CREATE TABLE IF NOT EXISTS lesson_configuration_snapshot (
  lesson_id TEXT PRIMARY KEY,
  mode_id TEXT NOT NULL,
  mode_version INTEGER NOT NULL CHECK (mode_version > 0),
  lesson_mode_context_version INTEGER NOT NULL CHECK (lesson_mode_context_version > 0),
  difficulty TEXT NOT NULL CHECK (difficulty IN ('easy', 'standard', 'challenging')),
  topic TEXT,
  objective TEXT,
  scenario TEXT,
  focus_areas_json TEXT NOT NULL,
  custom_title TEXT,
  configuration_schema_version INTEGER NOT NULL CHECK (configuration_schema_version > 0),
  created_at TEXT NOT NULL,
  FOREIGN KEY (lesson_id) REFERENCES lesson(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS lesson_configuration_mode
  ON lesson_configuration_snapshot(mode_id, created_at);

INSERT OR IGNORE INTO schema_migration(version) VALUES (6);
