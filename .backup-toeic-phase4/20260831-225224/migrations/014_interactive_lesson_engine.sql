CREATE TABLE IF NOT EXISTS interactive_lesson_session (
    id TEXT PRIMARY KEY NOT NULL,
    lesson_id TEXT NOT NULL,
    lesson_content_version INTEGER NOT NULL CHECK (lesson_content_version >= 1),
    package_schema_version INTEGER NOT NULL CHECK (package_schema_version >= 1),
    lesson_flow_version INTEGER NOT NULL CHECK (lesson_flow_version >= 1),
    package_hash TEXT NOT NULL,
    engine_version INTEGER NOT NULL CHECK (engine_version >= 1),
    snapshot_version INTEGER NOT NULL CHECK (snapshot_version >= 1),
    status TEXT NOT NULL CHECK (status IN ('in_progress', 'completed', 'abandoned', 'failed')),
    stage_count INTEGER NOT NULL CHECK (stage_count > 0),
    current_stage_index INTEGER NOT NULL CHECK (current_stage_index >= 0),
    package_snapshot_json TEXT NOT NULL CHECK (json_valid(package_snapshot_json)),
    student_context_snapshot_json TEXT NOT NULL CHECK (json_valid(student_context_snapshot_json)),
    started_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT,
    abandoned_at TEXT,
    failure_code TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_interactive_lesson_one_active
ON interactive_lesson_session(status) WHERE status = 'in_progress';
CREATE INDEX IF NOT EXISTS ix_interactive_lesson_recent
ON interactive_lesson_session(started_at DESC);

CREATE TABLE IF NOT EXISTS interactive_lesson_stage_state (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES interactive_lesson_session(id) ON DELETE CASCADE,
    stage_id TEXT NOT NULL,
    sequence_index INTEGER NOT NULL CHECK (sequence_index >= 0),
    stage_type TEXT NOT NULL CHECK (stage_type IN ('theory','visual_vocabulary','listening','repeat','speaking_check','exercise','guided_conversation','analysis')),
    stage_schema_version INTEGER NOT NULL CHECK (stage_schema_version >= 1),
    required INTEGER NOT NULL CHECK (required IN (0, 1)),
    status TEXT NOT NULL CHECK (status IN ('pending', 'active', 'completed', 'skipped')),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count IN (0, 1)),
    completion_result_version INTEGER,
    completion_json TEXT CHECK (completion_json IS NULL OR json_valid(completion_json)),
    started_at TEXT,
    completed_at TEXT,
    skipped_at TEXT,
    updated_at TEXT NOT NULL,
    UNIQUE(session_id, stage_id),
    UNIQUE(session_id, sequence_index)
);
CREATE INDEX IF NOT EXISTS ix_interactive_lesson_stage_session
ON interactive_lesson_stage_state(session_id, sequence_index);

INSERT OR IGNORE INTO schema_migration(version) VALUES (14);
