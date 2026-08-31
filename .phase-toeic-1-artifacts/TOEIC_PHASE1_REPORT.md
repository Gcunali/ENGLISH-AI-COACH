# TOEIC Phase 1 Final Report

Status: technically complete. Production content is validator-approved; visible human listening/UX acceptance remains the only gate.

## Delivery

- Separate module/navigation group: Exam preparation -> TOEIC
- Routes: `/toeic`, `/toeic/session/:sessionId`, `/toeic/results/:sessionId`, `/toeic/history`
- Landing honestly shows Parts 1-7; only Part 1 is available.
- Part 1 has 3 production forms and 18 original items/assets.
- Each form has exactly six fixed, balanced questions.
- UI is focused, white/blue, responsive, keyboard accessible, untimed, and has no countdown.

## Safety and persistence

- Pre-change backup: `.backup-toeic-phase1/20260831-164301`
- Database schema: 20 -> 21 via `021_toeic_exam_center.sql`
- Dedicated tables: session, immutable answer, presentation attempt, optional active-time event
- Exact form/item-version order is snapshotted; no reshuffle on resume.
- Answers and feedback survive navigation, hours-later resume, and process restart.
- Interrupted audio restarts from the beginning without penalty.
- Real database after migration: integrity OK, zero FK violations, existing data preserved.

## Security, grading, and feedback

- Pre-answer DTO does not contain transcripts, correct answer, correctness, explanations, or distractor metadata.
- Initial A-D audio is one-play; transcripts/replay unlock after backend answer commit.
- First answer is score-bearing and immutable in both backend logic and SQLite uniqueness.
- Grading, feedback, and analytics are deterministic and use zero Qwen calls.
- Every item has authored correct and distractor-specific explanations.
- Results show X/6 and accuracy only. No fake official or estimated 5-495 score exists.
- A future typed `ToeicScoreProfile` exists without a conversion table.

## Audio and offline operation

Piper `en_US-lessac-medium` uses the existing persistent static TTS cache and Bluetooth wake padding. The first Form A script synthesized successfully in the physical local environment. No web API, external TTS, external image hosting, microphone, or telemetry was added.

## Analytics and history

Results include skill/difficulty breakdown and common selected distractor types. Review Mistakes and Review All use complete authored content. TOEIC Performance history is independent from Lesson History and does not touch CEFR, Placement, Course, Guided Lessons, Learning Memory, Vocabulary, Review, XP, streak, or achievements.

## Verification

- Rust: 230 passed, 0 failed, 27 ignored manual tests
- Frontend: 320 passed, 0 failed across 81 files
- Typecheck, lint, formatting, offline check, Vite production build: passed
- Tauri debug no-bundle build and startup smoke: passed
- 288 Course packages: unchanged by SHA-256
- No installer was built.

## Remaining gate/debt

One short visible human acceptance pass remains for perceived audio clarity, real output-device behavior, keyboard feel, and a complete six-question visual walkthrough. Phase 2 can extend the shared item/session/result foundation with Listening Part 2 after that acceptance; no Part 2-7, Speaking/Writing, Phase AC, or installer work was started here.
