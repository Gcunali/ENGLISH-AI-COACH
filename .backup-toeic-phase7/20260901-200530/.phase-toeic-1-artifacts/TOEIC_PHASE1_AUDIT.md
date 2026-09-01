# TOEIC PHASE 1 — PRE-IMPLEMENTATION AUDIT

Audit date: 2026-08-31

Official project: `C:\ENGLISH AI COACH`

## Baseline and safety

- Git worktree before TOEIC work: clean.
- Backup: `C:\ENGLISH AI COACH\.backup-toeic-phase1\20260831-164301`
- Consistent human database backup: `human-database\english-ai-coach.sqlite3`
- Database backup SHA-256: `634B9EBEF5D0CCA27F391534638CFD1AA3EFF9C6FCF8E2B2D91E5765911B44C6`
- Protected Lesson Package manifest: 288 entries.
- Human database: schema 20 in `schema_migration`, `PRAGMA integrity_check = ok`, zero foreign-key violations.

## Current AB+ status

AB+ is technically implemented and its automated regressions passed, but its physical microphone/Bluetooth approval remains pending. TOEIC Part 1 does not depend on Daily Practice, Whisper, Shadowing, or pronunciation. It may safely reuse only the existing static Piper cache interface. Phase AC and the installer remain out of scope.

## Current navigation

The React Router application uses one `AppLayout` and grouped sidebar navigation. Existing domains include Practice, Learning, Assessment/Profile, and System. TOEIC should be a separate `Exam Preparation` group with a single `TOEIC` entry, plus dedicated landing, session, results/review, and history routes. It must not reuse Guided Lesson routes or history.

## Current resource architecture

Bundled Course content is read from `src-tauri/resources`, validated in Rust, and copied by Tauri resource configuration. The Course has an independent typed registry and 288 immutable `lesson.json` packages. TOEIC will use its own typed registry and root:

`src-tauri/resources/toeic/item-bank-v1`

The TOEIC bank will remain bundled, schema-validated, versioned, read-only at runtime, and separate from SQLite. SQLite will store only form/session snapshots, first answers, presentation interruptions, active-time events, and terminal state.

## Piper and TTS cache

AB+ provides a persistent static Piper cache under app-local data. It uses `en_US-lessac-medium`, model/config identities, exact normalized text, engine/wake/cache versions, atomic WAV publication, corruption validation, and a 250 MiB cap. TOEIC can reuse the same runtime for author-controlled static statements without changing Piper semantics. Transcripts remain backend-side until an answer is committed.

## Image asset availability

The current bundled resource tree contains zero PNG/JPG/JPEG/WebP images suitable for Part 1. The available image-generation facility can produce original raster assets, so this phase will attempt 18 original photo-like scenes and inspect them before publication. If any scene is ambiguous or visually inadequate, the relevant item/form must remain draft and the engine may still be declared complete with content pending.

## Persistence and migration decision

The next real migration is 021. A dedicated migration is necessary because no existing table can represent immutable TOEIC first answers, stable form/item-version snapshots, interrupted presentations, or TOEIC-only active time without contaminating Course/Review/Gamification domains. Item-bank content and images will not be duplicated into SQLite.

## Scoring architecture decision

Phase 1 records raw `0/1` per first answer and Part 1 accuracy only. A typed future `ToeicScoreProfile` contract will describe calibrated mappings, but no conversion table or 5–495/10–990 estimate will be created. TOEIC results never mutate Placement or CEFR.

## Independence guarantees

TOEIC Phase 1 will not write to Course progress, Guided Lesson completion, Learning Memory, Vocabulary, Review, XP, streak, achievements, Placement, pronunciation, or voice tables. It is untimed by design, uses zero Qwen calls, performs deterministic backend grading, and never exposes answer keys in the pre-answer public DTO.
