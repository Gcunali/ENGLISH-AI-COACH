CREATE TABLE IF NOT EXISTS guided_learning_integration (
  session_id TEXT PRIMARY KEY,
  lesson_id TEXT NOT NULL,
  integration_version INTEGER NOT NULL CHECK (integration_version > 0),
  integrated_at TEXT NOT NULL,
  FOREIGN KEY (session_id) REFERENCES interactive_lesson_session(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS guided_session_vocabulary (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  lesson_id TEXT NOT NULL,
  vocabulary_item_id TEXT NOT NULL,
  example TEXT NOT NULL,
  occurrence_count INTEGER NOT NULL DEFAULT 1 CHECK (occurrence_count > 0),
  created_at TEXT NOT NULL,
  UNIQUE (session_id, vocabulary_item_id),
  FOREIGN KEY (session_id) REFERENCES interactive_lesson_session(id) ON DELETE CASCADE,
  FOREIGN KEY (vocabulary_item_id) REFERENCES vocabulary_item(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS guided_session_vocabulary_item
  ON guided_session_vocabulary(vocabulary_item_id, lesson_id, session_id);

CREATE TABLE IF NOT EXISTS interactive_lesson_guided_correction (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  stage_id TEXT NOT NULL,
  student_turn_id TEXT NOT NULL,
  teacher_turn_id TEXT NOT NULL UNIQUE,
  source_index INTEGER NOT NULL CHECK (source_index >= 0),
  category TEXT NOT NULL CHECK (category IN (
    'grammar', 'vocabulary', 'word_choice', 'verb_tense', 'preposition',
    'article', 'word_order', 'naturalness', 'other'
  )),
  original TEXT NOT NULL CHECK (length(trim(original)) > 0),
  corrected TEXT NOT NULL CHECK (length(trim(corrected)) > 0),
  explanation TEXT NOT NULL CHECK (length(trim(explanation)) > 0),
  detection_method TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE (session_id, source_index),
  FOREIGN KEY (session_id) REFERENCES interactive_lesson_session(id) ON DELETE CASCADE,
  FOREIGN KEY (student_turn_id) REFERENCES interactive_lesson_guided_conversation_turn(id) ON DELETE CASCADE,
  FOREIGN KEY (teacher_turn_id) REFERENCES interactive_lesson_guided_conversation_turn(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS guided_correction_session_stage
  ON interactive_lesson_guided_correction(session_id, stage_id, source_index);

CREATE TABLE IF NOT EXISTS guided_recurring_mistake_occurrence (
  id TEXT PRIMARY KEY,
  recurring_mistake_id TEXT NOT NULL,
  session_id TEXT NOT NULL,
  lesson_id TEXT NOT NULL,
  correction_id TEXT NOT NULL UNIQUE,
  source_index INTEGER NOT NULL CHECK (source_index >= 0),
  original TEXT NOT NULL CHECK (length(trim(original)) > 0),
  corrected TEXT NOT NULL CHECK (length(trim(corrected)) > 0),
  explanation TEXT NOT NULL CHECK (length(trim(explanation)) > 0),
  created_at TEXT NOT NULL,
  FOREIGN KEY (recurring_mistake_id) REFERENCES recurring_mistake(id) ON DELETE CASCADE,
  FOREIGN KEY (session_id) REFERENCES interactive_lesson_session(id) ON DELETE CASCADE,
  FOREIGN KEY (correction_id) REFERENCES interactive_lesson_guided_correction(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS guided_mistake_occurrence_mistake
  ON guided_recurring_mistake_occurrence(recurring_mistake_id, lesson_id, session_id);

CREATE TABLE IF NOT EXISTS interactive_lesson_active_practice_event (
  event_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  duration_seconds INTEGER NOT NULL CHECK (duration_seconds BETWEEN 1 AND 30),
  recorded_at TEXT NOT NULL,
  FOREIGN KEY (session_id) REFERENCES interactive_lesson_session(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS guided_active_practice_session_time
  ON interactive_lesson_active_practice_event(session_id, recorded_at);

CREATE TABLE IF NOT EXISTS guided_gamification_xp_event (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  rule_version INTEGER NOT NULL CHECK (rule_version > 0),
  xp_amount INTEGER NOT NULL CHECK (xp_amount >= 0),
  activity_day TEXT NOT NULL CHECK (length(activity_day) = 10),
  created_at TEXT NOT NULL,
  UNIQUE (session_id, rule_version),
  FOREIGN KEY (session_id) REFERENCES interactive_lesson_session(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS guided_gamification_xp_activity_day
  ON guided_gamification_xp_event(activity_day);

INSERT OR IGNORE INTO schema_migration(version) VALUES (19);
