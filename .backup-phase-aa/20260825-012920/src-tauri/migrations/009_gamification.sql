CREATE TABLE IF NOT EXISTS gamification_xp_event (
  id TEXT PRIMARY KEY,
  event_type TEXT NOT NULL CHECK (event_type = 'qualifying_lesson_completed'),
  source_type TEXT NOT NULL CHECK (source_type = 'lesson'),
  source_id TEXT NOT NULL,
  rule_version INTEGER NOT NULL CHECK (rule_version > 0),
  xp_amount INTEGER NOT NULL CHECK (xp_amount >= 0),
  activity_day TEXT NOT NULL CHECK (length(activity_day) = 10),
  created_at TEXT NOT NULL,
  UNIQUE(event_type, source_type, source_id, rule_version),
  FOREIGN KEY (source_id) REFERENCES lesson(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS gamification_xp_activity_day
  ON gamification_xp_event(activity_day);

CREATE TABLE IF NOT EXISTS gamification_profile (
  profile_key TEXT PRIMARY KEY CHECK (profile_key = 'default'),
  schema_version INTEGER NOT NULL CHECK (schema_version > 0),
  weekly_goal_minutes INTEGER NOT NULL
    CHECK (weekly_goal_minutes BETWEEN 30 AND 600 AND weekly_goal_minutes % 15 = 0),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS achievement_unlock (
  achievement_id TEXT NOT NULL,
  achievement_version INTEGER NOT NULL CHECK (achievement_version > 0),
  unlocked_at TEXT NOT NULL,
  trigger_value INTEGER,
  created_at TEXT NOT NULL,
  PRIMARY KEY (achievement_id, achievement_version)
);

INSERT OR IGNORE INTO gamification_profile (
  profile_key, schema_version, weekly_goal_minutes, created_at, updated_at
) VALUES (
  'default', 1, 90,
  strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')
);

INSERT OR IGNORE INTO schema_migration(version) VALUES (9);
