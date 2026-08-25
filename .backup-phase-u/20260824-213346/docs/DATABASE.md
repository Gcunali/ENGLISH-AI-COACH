# Database

SQLite is stored in application-local data. Migration `001_initial.sql` enables foreign keys and WAL mode, records the schema version, and creates the first transcript exchange and settings tables.

Only transcripts are persisted in the vertical slice. Raw audio is not. Later migrations add profiles, lessons, corrections, vocabulary, recurring mistakes, metrics, assessments, achievements, and compact learning summaries. Migrations remain forward-only and idempotent.
