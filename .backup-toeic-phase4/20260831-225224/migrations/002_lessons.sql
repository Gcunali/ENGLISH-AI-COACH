CREATE TABLE IF NOT EXISTS lesson (
  id TEXT PRIMARY KEY,
  started_at TEXT NOT NULL,
  ended_at TEXT,
  status TEXT NOT NULL CHECK (status IN ('starting', 'active', 'completed', 'interrupted', 'failed')),
  topic TEXT,
  mode TEXT NOT NULL,
  duration_seconds INTEGER,
  student_turn_count INTEGER NOT NULL DEFAULT 0 CHECK (student_turn_count >= 0),
  teacher_turn_count INTEGER NOT NULL DEFAULT 0 CHECK (teacher_turn_count >= 0),
  correction_count INTEGER NOT NULL DEFAULT 0 CHECK (correction_count >= 0),
  whisper_model TEXT NOT NULL,
  whisper_threads INTEGER NOT NULL,
  ollama_model TEXT NOT NULL,
  piper_voice TEXT NOT NULL,
  voice_engine_version TEXT NOT NULL,
  error_message TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS transcript_message (
  id TEXT PRIMARY KEY,
  lesson_id TEXT NOT NULL,
  sequence_index INTEGER NOT NULL CHECK (sequence_index > 0),
  turn_index INTEGER NOT NULL CHECK (turn_index > 0),
  role TEXT NOT NULL CHECK (role IN ('student', 'teacher')),
  text TEXT NOT NULL CHECK (length(trim(text)) > 0),
  source TEXT NOT NULL,
  engine_event_type TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE (lesson_id, sequence_index),
  FOREIGN KEY (lesson_id) REFERENCES lesson(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS transcript_message_lesson_order
  ON transcript_message(lesson_id, sequence_index);

CREATE TABLE IF NOT EXISTS correction_candidate (
  id TEXT PRIMARY KEY,
  lesson_id TEXT NOT NULL,
  student_message_id TEXT NOT NULL,
  teacher_message_id TEXT NOT NULL UNIQUE,
  student_text TEXT NOT NULL,
  teacher_response_text TEXT NOT NULL,
  detection_method TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (lesson_id) REFERENCES lesson(id) ON DELETE CASCADE,
  FOREIGN KEY (student_message_id) REFERENCES transcript_message(id) ON DELETE CASCADE,
  FOREIGN KEY (teacher_message_id) REFERENCES transcript_message(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS correction_candidate_lesson
  ON correction_candidate(lesson_id, created_at);

INSERT OR IGNORE INTO schema_migration(version) VALUES (2);
