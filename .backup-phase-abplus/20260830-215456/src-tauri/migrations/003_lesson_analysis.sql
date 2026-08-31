CREATE TABLE IF NOT EXISTS lesson_analysis (
  id TEXT PRIMARY KEY,
  lesson_id TEXT NOT NULL UNIQUE,
  status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'completed', 'failed', 'insufficient_data')),
  schema_version INTEGER NOT NULL,
  prompt_version INTEGER NOT NULL,
  analyzer_model TEXT NOT NULL,
  started_at TEXT,
  completed_at TEXT,
  overall_score INTEGER CHECK (overall_score BETWEEN 0 AND 100),
  fluency_score INTEGER CHECK (fluency_score BETWEEN 0 AND 100),
  grammar_score INTEGER CHECK (grammar_score BETWEEN 0 AND 100),
  vocabulary_score INTEGER CHECK (vocabulary_score BETWEEN 0 AND 100),
  comprehension_score INTEGER CHECK (comprehension_score BETWEEN 0 AND 100),
  interaction_score INTEGER CHECK (interaction_score BETWEEN 0 AND 100),
  pronunciation_score INTEGER CHECK (pronunciation_score IS NULL),
  summary TEXT,
  raw_json TEXT,
  error_message TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (lesson_id) REFERENCES lesson(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS lesson_analysis_status
  ON lesson_analysis(status, updated_at);

INSERT OR IGNORE INTO schema_migration(version) VALUES (3);
