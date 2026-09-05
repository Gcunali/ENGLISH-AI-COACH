CREATE TABLE IF NOT EXISTS review_session (
  id TEXT PRIMARY KEY,
  status TEXT NOT NULL CHECK (status IN ('in_progress','completed','abandoned','failed')),
  mode TEXT NOT NULL CHECK (mode IN ('mixed','vocabulary','mistakes')),
  requested_item_count INTEGER NOT NULL CHECK (requested_item_count IN (5,10,15)),
  actual_item_count INTEGER NOT NULL CHECK (actual_item_count > 0 AND actual_item_count <= requested_item_count),
  reviewed_item_count INTEGER NOT NULL DEFAULT 0 CHECK (reviewed_item_count >= 0 AND reviewed_item_count <= actual_item_count),
  queue_version INTEGER NOT NULL CHECK (queue_version > 0),
  item_snapshot_version INTEGER NOT NULL CHECK (item_snapshot_version > 0),
  started_at TEXT NOT NULL,
  completed_at TEXT,
  abandoned_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS review_session_one_active
  ON review_session(status) WHERE status = 'in_progress';
CREATE INDEX IF NOT EXISTS review_session_recent
  ON review_session(started_at DESC, id DESC);

CREATE TABLE IF NOT EXISTS review_session_item (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  sequence_index INTEGER NOT NULL CHECK (sequence_index >= 0),
  item_type TEXT NOT NULL CHECK (item_type IN ('vocabulary','recurring_mistake')),
  vocabulary_item_id TEXT,
  recurring_mistake_id TEXT,
  content_json TEXT NOT NULL CHECK (length(trim(content_json)) > 0),
  review_outcome TEXT CHECK (review_outcome IN ('keep_practicing','mark_learning','mark_known','review_again','reviewed')),
  reviewed_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  CHECK (
    (item_type = 'vocabulary' AND vocabulary_item_id IS NOT NULL AND recurring_mistake_id IS NULL) OR
    (item_type = 'recurring_mistake' AND vocabulary_item_id IS NULL AND recurring_mistake_id IS NOT NULL)
  ),
  CHECK ((review_outcome IS NULL AND reviewed_at IS NULL) OR (review_outcome IS NOT NULL AND reviewed_at IS NOT NULL)),
  UNIQUE(session_id, sequence_index),
  FOREIGN KEY (session_id) REFERENCES review_session(id) ON DELETE CASCADE,
  FOREIGN KEY (vocabulary_item_id) REFERENCES vocabulary_item(id) ON DELETE RESTRICT,
  FOREIGN KEY (recurring_mistake_id) REFERENCES recurring_mistake(id) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX IF NOT EXISTS review_item_unique_vocabulary
  ON review_session_item(session_id, vocabulary_item_id) WHERE vocabulary_item_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS review_item_unique_mistake
  ON review_session_item(session_id, recurring_mistake_id) WHERE recurring_mistake_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS review_item_vocabulary_history
  ON review_session_item(vocabulary_item_id, reviewed_at) WHERE reviewed_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS review_item_mistake_history
  ON review_session_item(recurring_mistake_id, reviewed_at) WHERE reviewed_at IS NOT NULL;

INSERT OR IGNORE INTO schema_migration(version) VALUES (10);
