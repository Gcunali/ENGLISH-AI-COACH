# Phase X — Pre-authoring audit

Audited on 2026-08-25 before production content authoring.

## Baseline

- Official project: `C:\ENGLISH AI COACH`.
- Phase W Curriculum Foundation v1 is complete and unchanged.
- Production `resources/curriculum` and `resources/interactive-lessons` contain only their `.keep` notices: zero human Curriculum, zero human Guided Lesson packages.
- Guided Package v1 already supports the required ordered stages: Theory, Visual Vocabulary, Listening, Repeat, Speaking Check, Exercise, Guided Conversation, Analysis.
- Images and packaged audio are optional. Production can use original text-only vocabulary cards and runtime Piper; no downloads or WAV bulk generation are needed.
- Existing exercise types cover all required authoring: single choice, multiple select, fill blank, word order, matching, and exact short answer.
- Curriculum v1 supports `english-core` with canonical A1 then A2, 8 Units per Level, 6 exact Lesson refs per Unit, all pinned to contentVersion 1.
- No engine, schema, migration, dependency, model, visual redesign, or database state is required.

## Human database before Phase X

- Logical schema: 18.
- Integrity: `ok`; foreign-key violations: 0.
- Guided sessions/stage state/runtime state/exercise attempts/guided turns/guided pronunciation/Interactive Analysis: all 0.
- Standard data: 13 Lessons, 9 analyses, 84 transcript messages.
- Placement: 4 attempts, 27 answers, 3 speaking responses.
- Student Profile/Summary: 1/1; Gamification profile/events/achievements: 1/5/3.
- Vocabulary/recurring mistakes/reviews/pronunciation/voice performance remain existing human data and must not change.

## Backup

- Pre-Phase X backup: `.backup-phase-x/20260825-002452`.
- Physical SQLite SHA-256: `F49AB7E6DD7DB23CD984798A7DF055D36A0FC5A9A2B45E1B0A2D80BCA4ABC3A8`.
- Backup SQLite: schema 18, integrity `ok`, foreign-key violations 0.
- `manifest-sha256.txt` covers all copied files.

## Authoring gates

1. Write and adopt `docs/CONTENT_EDITORIAL_STANDARD_V1.md`.
2. Produce only A1 Unit 1 / six draft packages plus draft Curriculum structure.
3. Validate the pilot through the official Rust parser and editorial QA; record `A1_UNIT1_PILOT_REVIEW.md`.
4. Only after the pilot passes, author the remaining A1 packages and validate 8/48.
5. Only after A1 passes, author A2 and validate 8/48.
6. Publish packages and `english-core` only after complete 96-package validation.

## Protected boundary

Voice, Piper/Whisper runtime, pronunciation, analyzers, prompts, Placement, Profile, Memory, Gamification, Review, database migrations, package manifests/locks, and Phase Q styling are outside the content-authoring scope. Hash and regression checks will verify preservation.
