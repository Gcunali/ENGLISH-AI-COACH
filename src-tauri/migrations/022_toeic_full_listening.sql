CREATE TABLE IF NOT EXISTS toeic_full_listening_session (
  id TEXT PRIMARY KEY,
  family TEXT NOT NULL,
  mode TEXT NOT NULL CHECK(mode IN ('simulation','learning')),
  status TEXT NOT NULL CHECK(status IN ('in_progress','completed','abandoned')),
  current_part INTEGER NOT NULL CHECK(current_part BETWEEN 1 AND 4),
  composition_json TEXT NOT NULL CHECK(json_valid(composition_json)),
  score_profile_id TEXT,
  score_profile_version INTEGER,
  raw_correct INTEGER CHECK(raw_correct BETWEEN 0 AND 100),
  estimated_score INTEGER CHECK(estimated_score BETWEEN 5 AND 495 AND estimated_score % 5 = 0),
  range_low INTEGER CHECK(range_low BETWEEN 5 AND 495 AND range_low % 5 = 0),
  range_high INTEGER CHECK(range_high BETWEEN 5 AND 495 AND range_high % 5 = 0),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  completed_at TEXT,
  abandoned_at TEXT
);
CREATE INDEX IF NOT EXISTS toeic_full_listening_history ON toeic_full_listening_session(created_at DESC,id);
CREATE TABLE IF NOT EXISTS toeic_full_listening_part (
  full_session_id TEXT NOT NULL,
  part_number INTEGER NOT NULL CHECK(part_number BETWEEN 1 AND 4),
  toeic_session_id TEXT NOT NULL UNIQUE,
  form_id TEXT NOT NULL,
  form_version INTEGER NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('pending','in_progress','completed')),
  PRIMARY KEY(full_session_id,part_number),
  FOREIGN KEY(full_session_id) REFERENCES toeic_full_listening_session(id) ON DELETE CASCADE,
  FOREIGN KEY(toeic_session_id) REFERENCES toeic_session(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS toeic_listening_score_profile (
  profile_id TEXT NOT NULL,
  version INTEGER NOT NULL,
  methodology TEXT NOT NULL,
  mapping_json TEXT NOT NULL CHECK(json_valid(mapping_json)),
  created_at TEXT NOT NULL,
  PRIMARY KEY(profile_id,version)
);
INSERT OR IGNORE INTO toeic_listening_score_profile(profile_id,version,methodology,mapping_json,created_at)
VALUES(
  'toeic-listening-unofficial-banded',
  1,
  'Versioned piecewise interpolation over documented raw-score anchors; rounded to five points with an uncertainty band.',
  '{"anchors":[[0,5],[1,10],[10,45],[20,90],[30,145],[40,205],[50,265],[60,325],[70,380],[80,425],[90,465],[95,480],[100,495]],"uncertainty":{"rawBelow20":30,"raw20To79":25,"raw80Plus":20},"rounding":5}',
  strftime('%Y-%m-%dT%H:%M:%fZ','now')
);
INSERT OR IGNORE INTO schema_migration(version) VALUES (22);
