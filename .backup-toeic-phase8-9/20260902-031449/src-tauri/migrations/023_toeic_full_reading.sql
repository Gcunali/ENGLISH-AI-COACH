CREATE TABLE IF NOT EXISTS toeic_full_reading_session (
 id TEXT PRIMARY KEY, family TEXT NOT NULL, mode TEXT NOT NULL CHECK(mode IN ('simulation','learning')), status TEXT NOT NULL CHECK(status IN ('in_progress','completed','abandoned')),
 current_part INTEGER NOT NULL CHECK(current_part BETWEEN 5 AND 7), composition_json TEXT NOT NULL CHECK(json_valid(composition_json)), score_profile_id TEXT, score_profile_version INTEGER,
 raw_correct INTEGER CHECK(raw_correct BETWEEN 0 AND 100), estimated_score INTEGER CHECK(estimated_score BETWEEN 5 AND 495 AND estimated_score % 5=0), range_low INTEGER, range_high INTEGER,
 created_at TEXT NOT NULL, updated_at TEXT NOT NULL, completed_at TEXT, abandoned_at TEXT
);
CREATE INDEX IF NOT EXISTS toeic_full_reading_history ON toeic_full_reading_session(created_at DESC,id);
CREATE TABLE IF NOT EXISTS toeic_full_reading_part (
 full_session_id TEXT NOT NULL, part_number INTEGER NOT NULL CHECK(part_number BETWEEN 5 AND 7), toeic_session_id TEXT NOT NULL UNIQUE, form_id TEXT NOT NULL, form_version INTEGER NOT NULL,
 status TEXT NOT NULL CHECK(status IN ('pending','in_progress','completed')), PRIMARY KEY(full_session_id,part_number), FOREIGN KEY(full_session_id) REFERENCES toeic_full_reading_session(id) ON DELETE CASCADE,
 FOREIGN KEY(toeic_session_id) REFERENCES toeic_session(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS toeic_reading_score_profile (profile_id TEXT NOT NULL,version INTEGER NOT NULL,methodology TEXT NOT NULL,mapping_json TEXT NOT NULL CHECK(json_valid(mapping_json)),created_at TEXT NOT NULL,PRIMARY KEY(profile_id,version));
INSERT OR IGNORE INTO toeic_reading_score_profile(profile_id,version,methodology,mapping_json,created_at) VALUES('toeic-reading-unofficial-banded',1,'Conservative versioned piecewise practice calibration; five-point rounding and uncertainty band.','{"anchors":[[0,5],[1,10],[10,40],[20,85],[30,135],[40,190],[50,250],[60,310],[70,365],[80,415],[90,460],[95,480],[100,495]],"uncertainty":{"rawBelow20":35,"raw20To79":30,"raw80Plus":25},"rounding":5}',strftime('%Y-%m-%dT%H:%M:%fZ','now'));
CREATE TABLE IF NOT EXISTS toeic_full_lr_session (
 id TEXT PRIMARY KEY,family TEXT NOT NULL,status TEXT NOT NULL CHECK(status IN ('in_progress','completed','abandoned')),listening_session_id TEXT NOT NULL UNIQUE,reading_session_id TEXT NOT NULL UNIQUE,
 listening_raw INTEGER,reading_raw INTEGER,total_raw INTEGER,listening_estimate INTEGER,reading_estimate INTEGER,total_estimate INTEGER,range_low INTEGER,range_high INTEGER,
 listening_profile_version INTEGER,reading_profile_version INTEGER,created_at TEXT NOT NULL,updated_at TEXT NOT NULL,completed_at TEXT,
 FOREIGN KEY(listening_session_id) REFERENCES toeic_full_listening_session(id),FOREIGN KEY(reading_session_id) REFERENCES toeic_full_reading_session(id)
);
INSERT OR IGNORE INTO schema_migration(version) VALUES(23);
