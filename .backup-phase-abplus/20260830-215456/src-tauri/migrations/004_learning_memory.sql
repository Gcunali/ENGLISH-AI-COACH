CREATE TABLE IF NOT EXISTS vocabulary_item (
  id TEXT PRIMARY KEY,
  canonical_text TEXT NOT NULL UNIQUE CHECK (length(trim(canonical_text)) > 0),
  display_text TEXT NOT NULL CHECK (length(trim(display_text)) > 0),
  meaning TEXT NOT NULL CHECK (length(trim(meaning)) > 0),
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  lesson_count INTEGER NOT NULL DEFAULT 0 CHECK (lesson_count >= 0),
  occurrence_count INTEGER NOT NULL DEFAULT 0 CHECK (occurrence_count >= 0),
  status TEXT NOT NULL DEFAULT 'new' CHECK (status IN ('new', 'learning', 'known')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS vocabulary_item_status_last_seen
  ON vocabulary_item(status, last_seen_at DESC);

CREATE INDEX IF NOT EXISTS vocabulary_item_last_seen
  ON vocabulary_item(last_seen_at DESC);

CREATE TABLE IF NOT EXISTS lesson_vocabulary (
  id TEXT PRIMARY KEY,
  lesson_id TEXT NOT NULL,
  vocabulary_item_id TEXT NOT NULL,
  source_analysis_id TEXT NOT NULL,
  example TEXT NOT NULL,
  occurrence_count INTEGER NOT NULL DEFAULT 1 CHECK (occurrence_count > 0),
  created_at TEXT NOT NULL,
  UNIQUE (lesson_id, vocabulary_item_id),
  FOREIGN KEY (lesson_id) REFERENCES lesson(id) ON DELETE CASCADE,
  FOREIGN KEY (vocabulary_item_id) REFERENCES vocabulary_item(id) ON DELETE CASCADE,
  FOREIGN KEY (source_analysis_id) REFERENCES lesson_analysis(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS lesson_vocabulary_item_lesson
  ON lesson_vocabulary(vocabulary_item_id, lesson_id);

CREATE INDEX IF NOT EXISTS lesson_vocabulary_analysis
  ON lesson_vocabulary(source_analysis_id);

CREATE TABLE IF NOT EXISTS recurring_mistake (
  id TEXT PRIMARY KEY,
  signature TEXT NOT NULL UNIQUE CHECK (length(trim(signature)) > 0),
  category TEXT NOT NULL CHECK (category IN (
    'grammar', 'vocabulary', 'word_choice', 'verb_tense', 'preposition',
    'article', 'word_order', 'naturalness', 'other'
  )),
  title TEXT NOT NULL CHECK (length(trim(title)) > 0),
  explanation TEXT NOT NULL CHECK (length(trim(explanation)) > 0),
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  lesson_count INTEGER NOT NULL DEFAULT 0 CHECK (lesson_count >= 0),
  occurrence_count INTEGER NOT NULL DEFAULT 0 CHECK (occurrence_count >= 0),
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'improving', 'resolved')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS recurring_mistake_confirmed
  ON recurring_mistake(lesson_count DESC, last_seen_at DESC);

CREATE TABLE IF NOT EXISTS recurring_mistake_occurrence (
  id TEXT PRIMARY KEY,
  recurring_mistake_id TEXT NOT NULL,
  lesson_id TEXT NOT NULL,
  analysis_id TEXT NOT NULL,
  source_index INTEGER NOT NULL CHECK (source_index >= 0),
  original TEXT NOT NULL CHECK (length(trim(original)) > 0),
  corrected TEXT NOT NULL CHECK (length(trim(corrected)) > 0),
  explanation TEXT NOT NULL CHECK (length(trim(explanation)) > 0),
  created_at TEXT NOT NULL,
  UNIQUE (analysis_id, source_index),
  FOREIGN KEY (recurring_mistake_id) REFERENCES recurring_mistake(id) ON DELETE CASCADE,
  FOREIGN KEY (lesson_id) REFERENCES lesson(id) ON DELETE CASCADE,
  FOREIGN KEY (analysis_id) REFERENCES lesson_analysis(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS recurring_mistake_occurrence_mistake_lesson
  ON recurring_mistake_occurrence(recurring_mistake_id, lesson_id);

CREATE INDEX IF NOT EXISTS recurring_mistake_occurrence_analysis
  ON recurring_mistake_occurrence(analysis_id);

INSERT OR IGNORE INTO schema_migration(version) VALUES (4);
