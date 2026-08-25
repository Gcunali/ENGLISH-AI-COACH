CREATE TABLE IF NOT EXISTS app_system_event (
  id TEXT PRIMARY KEY,
  event_schema_version INTEGER NOT NULL CHECK(event_schema_version = 1),
  severity TEXT NOT NULL CHECK(severity IN ('warning','error','recovery')),
  component TEXT NOT NULL,
  event_code TEXT NOT NULL,
  details_json TEXT CHECK(details_json IS NULL OR json_valid(details_json)),
  occurred_at TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS app_system_event_recent
  ON app_system_event(occurred_at DESC, id DESC);

INSERT OR IGNORE INTO schema_migration(version) VALUES (13);
