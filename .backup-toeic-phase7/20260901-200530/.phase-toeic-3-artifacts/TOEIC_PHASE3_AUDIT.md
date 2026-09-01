# TOEIC Phase 3 — Pre-implementation Audit

Date: 2026-08-31

- Phase 1 is physically implemented: three validated six-question forms, one-play presentation, immutable first answers, persistence, results, review, and history.
- Phase 2 is physically implemented: one validated 25-question Form A, A/B/C flow, one-play/replay rules, immutable answers, persistence, deterministic analytics, review, and shared history. Its focused Rust tests and complete application regressions pass.
- Human SQLite database was backed up from `%LOCALAPPDATA%/com.englishaicoach.desktop/database/english-ai-coach.sqlite3`; pre-change SHA-256: `BFD6DD212BE1938F87AAEF479453D10FE9A32E3E439B0C878B969B12B20F122B`.
- Current schema is migration 21. Existing generic `toeic_session`, `toeic_answer`, and `toeic_presentation_attempt` tables support Part 3, set-level presentation IDs, 39 answers, immutable first attempts, and arbitrary versioned form snapshots. No migration is necessary.
- Backup: `.backup-toeic-phase3/20260831-221413`, including source/resources/configuration, human DB, and SHA-256 manifest.
- Item Bank remains bundled, local, read-only, versioned, and schema-validated.
- Piper assets physically available: `en_US-amy-medium` and `en_US-lessac-medium`; both are en-US. Static TTS cache contains 12 files. No voice/model download is needed.
- Two-speaker production is feasible using Amy and Lessac. Runtime data structures will support three speakers, but three-speaker production remains pending because only two suitable English voices are installed.
- Accent diversity is not claimed: installed production voices are en-US only.
- Optional graphics will be original, local SVG informational panels validated for safe paths. Target: two or three sets in Form A.
- Protected regressions: Parts 1 and 2, English Core v3, and all 288 Guided Lesson packages.
