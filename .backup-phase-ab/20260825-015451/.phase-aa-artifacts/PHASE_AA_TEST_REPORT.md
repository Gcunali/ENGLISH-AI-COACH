# Phase AA Test Report

Date: 2026-08-25

## Automated results

- Rust library: 212 passed, 0 failed, 26 explicitly ignored/manual.
- New Guided integration TEMP tests: 3 passed.
- Guided XP/repeat-session TEMP test: passed.
- Placement-at-zero-progress recommendation TEMP test: passed.
- Frontend Vitest: 154 passed, 0 failed across 37 files.
- TypeScript typecheck: passed.
- Oxlint: passed.
- Frontend production build: passed (1,864 modules).
- Voice/streaming Python: 18 passed.
- Pronunciation Python: 12 passed.
- Tauri debug application and MSI/NSIS bundles: built successfully.

The Tauri bundler bootstrap fetched its signed WiX/NSIS helper archives because they were absent from the local tool cache. This was an environment-validation deviation from the requested no-download workflow; no application dependency, model, lesson content, or runtime feature was downloaded. Later build validation must use the already present cache or `--no-bundle`.

## Covered invariants

- Completed-only global effects.
- Vocabulary idempotency and preservation of manual `known`.
- Structured corrections only; raw transcript vocabulary excluded.
- One distinct Guided Lesson does not confirm a recurring mistake; two do.
- Review sees Guided vocabulary through the existing queue.
- Active-session, next-in-sequence, Placement, and no-Placement recommendations.
- Course progress remains lesson-ID derived and score-free.
- Guided XP exactly once per session; repeat session earns legitimate XP.
- Active heartbeat duration, combined minutes, and streak.
- Guided/Course achievements use real persisted completion facts.
- No Placement/CEFR mutation and no score-based XP.

## Human database

- Migrated 18 → 19 twice to prove idempotency.
- Sessions: 0 before/after.
- Completed Guided sessions: 0 before/after.
- Existing vocabulary items: 3 before/after.
- Fabricated integrations/active events/Guided XP: 0/0/0.
- `integrity_check`: `ok`.
- foreign-key violations: 0.

