# Phase W Initial Audit

Audited on 2026-08-24 before Phase W implementation changes.

## Baseline and decision

- Human database logical schema: 18 (`MAX(schema_migration.version)`); `PRAGMA user_version` is unused and remains 0.
- Integrity: `ok`; foreign-key violations: 0.
- Human Guided sessions, stage/runtime states, attempts, committed Guided turns and Interactive Analysis rows: all 0.
- Human standard data before W: 13 lessons, 9 analyses, 84 transcript messages, 4 Placement attempts and 1 Student Profile.
- There is no production `src-tauri/resources/interactive-lessons` root, so the human Guided library is honestly empty.
- No Course/Curriculum manifest, registry, database state, progress or route exists.
- **Decision: no Migration 019.** Definitions are bundled content; progress and recommendation are fully derived from existing Guided Session, Placement and Profile records.

## Required answers

### A. Guided Lesson discovery

`InteractiveLessonContentRegistry::load(root)` scans each immediate non-symlink child directory for `lesson.json`, strictly validates it, hashes the typed package and exposes published content.

### B. Development resources

Debug resolves `env!("CARGO_MANIFEST_DIR")/resources/interactive-lessons`. Curriculum uses sibling `resources/curriculum`; tests inject explicit roots.

### C. Packaged resources

Release resolves `app.path().resource_dir()/interactive-lessons`. W must explicitly bundle both `resources/interactive-lessons` and `resources/curriculum`, resolving the latter at `resource_dir()/curriculum`.

### D. Exact Lesson version

Unavailable before W: candidates are deduplicated by `(lessonId, contentVersion)`, then storage collapses to latest published per `lessonId`. W must retain an exact-version map while preserving existing latest `get/list` behavior.

### E. Historical registry versions

Not retained before W. Session snapshots preserve started content, but exact installed versions must be retained for pinned Curriculum refs.

### F. Completed Guided Sessions

`interactive_lesson_session` stores stable `lesson_id`, exact version, status and timestamps. W can aggregate relevant rows once, without N+1 queries.

### G. Active Guided Session

The repository selects the sole `status='in_progress'` row. Start refuses an existing active session unless explicit start-over; Course must reuse this behavior and never silently abandon.

### H. Current Placement

The official repository selects the latest completed attempt ordered by `completed_at DESC`. Student Profile reads that same result.

### I. Existing Course

None.

### J. Existing curriculum state

None.

### K. Fully derived progress

Yes: completed is any completed session keyed by stable `lesson_id`; in-progress is a matching active session without prior completion; abandoned/failed do not complete; max completed content version supports Updated.

### L. Migration 019

Not necessary. Human schema remains 18.

### M. Sidebar

`AppLayout.tsx` owns static grouped navigation and filters Guided Lessons by backend overview. Course can use an independent published-curriculum visibility flag without replacing Guided Lessons.

### N. Empty Guided Lessons

The existing page honestly says “No guided lessons are installed yet.” Direct `/course` must safely say “No course content is installed yet.” while the normal sidebar hides Course when no published Curriculum exists.

### O. Tauri bundle resources

Add the two resource directories to `bundle.resources`. Dev uses the manifest directory; release uses `resource_dir()`; no runtime absolute project path and no installer.

## Safe reuse boundary

- Reuse the Guided Lesson parser/registry metadata and hashes, official start/resume/snapshot engine, Placement/Profile reads, Phase Q UI and Phase P backup.
- Never trigger Qwen, Conversation Teacher, Whisper, Piper, Pronunciation, exercise grading, standard Lesson Analyzer or global learning/gamification writes during Course browsing.
- Curriculum is local organization metadata, not lesson content and not a second lesson engine.
