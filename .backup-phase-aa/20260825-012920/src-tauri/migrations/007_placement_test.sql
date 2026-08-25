CREATE TABLE IF NOT EXISTS placement_attempt (
  id TEXT PRIMARY KEY,
  status TEXT NOT NULL CHECK (status IN ('in_progress', 'completed', 'abandoned', 'failed')),
  test_version INTEGER NOT NULL CHECK (test_version > 0),
  question_bank_version INTEGER NOT NULL CHECK (question_bank_version > 0),
  scoring_version INTEGER NOT NULL CHECK (scoring_version > 0),
  speaking_prompt_version INTEGER NOT NULL CHECK (speaking_prompt_version > 0),
  speaking_evaluator_version INTEGER,
  speaking_schema_version INTEGER,
  started_at TEXT NOT NULL,
  completed_at TEXT,
  grammar_level TEXT CHECK (grammar_level IN ('A1','A2','B1','B2','C1','C2')),
  vocabulary_level TEXT CHECK (vocabulary_level IN ('A1','A2','B1','B2','C1','C2')),
  reading_level TEXT CHECK (reading_level IN ('A1','A2','B1','B2','C1','C2')),
  spoken_production_level TEXT CHECK (spoken_production_level IN ('A1','A2','B1','B2','C1','C2')),
  overall_estimated_level TEXT CHECK (overall_estimated_level IN ('A1','A2','B1','B2','C1','C2')),
  confidence TEXT CHECK (confidence IN ('low','medium','high')),
  speaking_status TEXT NOT NULL DEFAULT 'pending' CHECK (speaking_status IN ('pending','completed','skipped','unavailable')),
  speaking_evaluator_json TEXT,
  error_message TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS placement_answer (
  id TEXT PRIMARY KEY,
  attempt_id TEXT NOT NULL,
  question_id TEXT NOT NULL,
  skill TEXT NOT NULL CHECK (skill IN ('grammar','vocabulary','reading')),
  cefr_band TEXT NOT NULL CHECK (cefr_band IN ('A1','A2','B1','B2','C1','C2')),
  selected_option_id TEXT NOT NULL,
  is_correct INTEGER NOT NULL CHECK (is_correct IN (0,1)),
  answered_at TEXT NOT NULL,
  UNIQUE(attempt_id, question_id),
  FOREIGN KEY (attempt_id) REFERENCES placement_attempt(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS placement_answer_attempt
  ON placement_answer(attempt_id, answered_at);

CREATE TABLE IF NOT EXISTS placement_speaking_response (
  id TEXT PRIMARY KEY,
  attempt_id TEXT NOT NULL,
  prompt_id TEXT NOT NULL,
  prompt_version INTEGER NOT NULL CHECK (prompt_version > 0),
  prompt_text_snapshot TEXT NOT NULL,
  sequence_index INTEGER NOT NULL CHECK (sequence_index >= 0),
  transcript TEXT NOT NULL CHECK (length(trim(transcript)) > 0),
  word_count INTEGER NOT NULL CHECK (word_count > 0),
  status TEXT NOT NULL CHECK (status = 'confirmed'),
  created_at TEXT NOT NULL,
  UNIQUE(attempt_id, prompt_id),
  UNIQUE(attempt_id, sequence_index),
  FOREIGN KEY (attempt_id) REFERENCES placement_attempt(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS placement_speaking_attempt
  ON placement_speaking_response(attempt_id, sequence_index);

INSERT OR IGNORE INTO schema_migration(version) VALUES (7);
