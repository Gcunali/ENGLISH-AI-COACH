# Database

SQLite is stored in application-local data. Migration `001_initial.sql` enables foreign keys and WAL mode, records the schema version, and creates the first transcript exchange and settings tables.

Only transcripts are persisted in the vertical slice. Raw audio is not. Later migrations add profiles, lessons, corrections, vocabulary, recurring mistakes, metrics, assessments, achievements, and compact learning summaries. Migrations remain forward-only and idempotent.
## Schema 17 — Guided Conversation

`interactive_lesson_guided_conversation_turn` stores only confirmed local text turns for an immutable Guided Lesson session/stage. Roles are `student` or `assistant`; system context and audio paths are never rows. `(session_id, stage_id, sequence_index)` is unique and `event_id` provides idempotency. The table is isolated from standard Lesson transcript/correction/history tables and cascades with its owning interactive stage.
