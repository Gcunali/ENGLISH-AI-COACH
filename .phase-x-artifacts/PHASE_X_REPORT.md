# Phase X — Real A1 + A2 Production Content

Completed on 2026-08-25 in `C:\ENGLISH AI COACH`. Work stops at Phase X; Phase Y was not started.

## Outcome

- `english-core` v1 is published with A1 then A2 only.
- A1: exactly 8 Units / 48 Lessons.
- A2: exactly 8 Units / 48 Lessons.
- Total: exactly 16 Units / 96 original en-US Guided Lessons.
- Every Lesson has contentVersion 1 and the exact eight required stages in canonical order.
- Typed Package Registry: 96 published, 0 invalid.
- Curriculum Registry: 1 published, 0 invalid, 96/96 exact refs resolved.
- Startability: 96/96 through the official Guided engine in migrated isolated SQLite.
- Human database: unchanged, schema 18, no artificial progress.

## Backup and audit

- Backup: `.backup-phase-x/20260825-002452`.
- Backup hash manifest: `.backup-phase-x/20260825-002452/manifest-sha256.txt`.
- Pre-Phase X SQLite SHA-256: `F49AB7E6DD7DB23CD984798A7DF055D36A0FC5A9A2B45E1B0A2D80BCA4ABC3A8`.
- Audit: `.phase-x-artifacts/PHASE_X_AUDIT.md`.
- Editorial standard: `docs/CONTENT_EDITORIAL_STANDARD_V1.md`.

## Files created

- Production Curriculum: `src-tauri/resources/curriculum/english-core/curriculum.json`.
- 96 package files under `src-tauri/resources/interactive-lessons/<lessonId>-v1/lesson.json`.
- Six curriculum/grammar/vocabulary documents under `docs/content` for A1 and A2.
- Phase artifacts: audit, pilot review, A1 report, A2 report, content quality report, production manifest, final hashes, this report, and the two reproducible authoring scripts.

## Files modified

- `src-tauri/src/interactive_lesson_content.rs`: content-wide parser/stage/publication tests only.
- `src-tauri/src/interactive_lesson_engine.rs`: 48/96 startability and DTO privacy regressions only.
- `src-tauri/src/curriculum.rs`: production 16/96/0%-progress/Placement tests and controlled manifest writer only.
- `src/pages/CoursePage.tsx`: minimal honest notice when Placement is above the installed A1–A2 range.
- `src/pages/CoursePage.test.tsx`: regression for Placement B1 with installed A1–A2.

No runtime engine, stage, exercise type, analyzer, voice/pronunciation component, migration, dependency, model, database schema, or visual architecture changed.

## Editorial sequence and gates

1. Content Editorial Standard v1 was written before Lessons.
2. Only A1 Unit 1 / Meeting People (6 draft Lessons) was authored.
3. Pilot passed the official parser, exact draft refs, stage quantities, startability 6/6, and answer-key privacy. Review: `.phase-x-artifacts/A1_UNIT1_PILOT_REVIEW.md`.
4. A1 expanded to 8/48 draft and passed parser, editorial bands, refs, IDs, missions, and startability 48/48. Report: `A1_PRODUCTION_REPORT.md`.
5. A2 was authored only after A1 approval and passed the complete 8/48 gate. Report: `A2_PRODUCTION_REPORT.md`.
6. All 96 draft packages passed parser and startability before the one final transition to published.

## Content architecture and quality

- Theory: A1 226–294 words (avg 271.5); A2 303–340 (avg 318.8).
- Vocabulary: A1 8–9 items; A2 9.
- Listening: A1 3 segments; A2 4.
- Repeat: 5 targets at both levels.
- Speaking Check: 4 previously introduced targets.
- Exercises: A1 8; A2 9; all six existing types used.
- Guided Conversation: A1 4/6/8 turns; A2 5/7/10.
- Analysis: unchanged empty Analysis v1 payload.
- Exercise distribution: 240 single choice, 96 multiple select, 192 fill blank, 96 word order, 96 matching, 96 exact short answer.
- IDs duplicated: 0; titles duplicated: 0; invalid packages: 0; broken refs: 0.
- Exact repeated Repeat chunks above six Lessons: 0; exact repeated long Theory blocks above six Lessons: 0.
- A2 is measurably more complex through longer Theory/listening/conversation, larger vocabulary, an additional exercise, linked clauses, tense relationships, reasons, contrast, and problem solving.
- Complete quality evidence: `.phase-x-artifacts/CONTENT_QUALITY_REPORT.md`.

## Curriculum and UI

- Path: `src-tauri/resources/curriculum/english-core/curriculum.json`.
- Curriculum ID/version/state: `english-core` / 1 / published.
- Ordering: canonical A1 then A2; no B1–C2 content or placeholder.
- Course with no completed Guided sessions derives exactly 0/96 and 0% progress.
- Human current Guided session count is 0, so the real Course starts at 0%.
- If Placement is B1/B2/C1/C2, the exact Placement remains visible and unchanged; neither A1 nor A2 is marked equivalent. UI states that the installed Course currently includes A1–A2.
- No Level lock, prerequisite, or fake recommendation was added.

## Content size and assets

- Lesson JSON: 2,074,641 bytes.
- Curriculum JSON: 37,682 bytes.
- Combined: 2,112,323 bytes (about 2.01 MiB).
- Assets: 0 packaged image/audio files. No download occurred; existing runtime audio semantics remain available.
- Official typed hashes for all Lessons: `.phase-x-artifacts/PRODUCTION_CONTENT_MANIFEST.md`.

## Automated validation

- Rust focused production tests: 3/3 passed (parser/publication, Curriculum 16/96/0%, startability 96/96).
- Rust full: 207 passed, 26 explicit manual/physical audits ignored, 0 failed.
- Frontend full: 37 files / 154 tests passed, 0 failed.
- Typecheck: passed.
- Lint: passed.
- Rust fmt: passed.
- Rust check offline: passed; only established dead-code warnings.
- Voice Python: 18/18 passed.
- Pronunciation Python: 12/12 passed.
- Vite: passed, 1,864 modules.
- Tauri debug `--no-bundle`: passed; executable `src-tauri/target/debug/english-ai-coach.exe`.
- No installer was built.

## Human database preservation

Before and after are identical:

- schema 18; integrity `ok`; foreign-key violations 0;
- Guided session/stage/runtime/exercise/conversation/pronunciation/Analysis counts all 0;
- standard Lessons 13, analyses 9, transcripts 84, corrections 7;
- Placement attempts/answers/speaking 4/27/3;
- Profile/Summary 1/1; teacher-memory snapshots 8;
- Gamification profile/events/achievements 1/5/3;
- global/lesson vocabulary 3/3; recurring mistakes/occurrences 6/6;
- Reviews 0/0; Pronunciation attempt/words 1/3; Voice Performance 2.

No Migration 019 or later migration was created. No completion, Analysis, Guided Session, XP, streak, achievement, memory, vocabulary, Review, or pronunciation row was fabricated.

## Protected hashes

- Voice v2/STABLE: `F56E16A7...23D2E2D` each.
- Voice Streaming Runtime v1: `8A8BB8FB...9E2EAFE`.
- Pronunciation engine/core: `FA28B35D...BE0968` / `0ED7D587...7A8E71F`.
- Exercise Engine: `B9364253...B41F97A`.
- Guided Conversation: `88077435...A0005`.
- Interactive Analysis: `13DFB371...AE7944`.
- Conversation Teacher / standard analyzer prompts: `8B5E0791...056F19D` / `6D4CB204...7C8C41`.
- Placement scoring/evaluator: `5075B4FD...E8BAB5C` / `828B5B24...D7FF7A`.
- `package.json`, `package-lock.json`, `Cargo.toml`, and `Cargo.lock` are SHA-identical to the pre-X backup.

## Problems found and corrected

- Pilot vocabulary fill-blank could omit a visible blank due to case-sensitive example replacement; authoring template corrected before A1 scale.
- Initial QA found two overly repeated closing chunks and repeated Theory explanations; templates made Unit/Lesson-specific, final duplication audit passed, then hashes were regenerated.
- Placement above A2 needed explicit installed-range copy; minimal UI notice plus regression added.
- The initial backup inventory did not anticipate those two later UI files. Their exact pre-fix Phase W state was reconstructed and explicitly labeled under `reconstructed-pre-ui-fix`; it is not misrepresented as a timestamped capture.
- One test command used `--exact` with a short name and ran zero tests; it was rerun correctly and 48/48 A1 startability passed.
- Pronunciation was first invoked one directory too high; correct-directory rerun passed 12/12.
- Tauri executable was open; human DB had zero active Guided sessions, the window accepted normal close, and the retry passed. No force kill.

## Human pending work

Explicitly **PENDING**, with no fabricated result:

- complete human A1 Lesson;
- complete human A2 Lesson;
- human pedagogical review;
- Bluetooth validation;
- human Pronunciation validation;
- human Guided Conversation validation.

These remain for the later human-validation phase, principally Phase AB.

## Final boundaries

No B1–C2, Learning Memory integration, Vocabulary integration, Review integration, Guided XP, streak, achievements, AI recommendation, visual redesign, installer, auto-update, PDF, cloud, telemetry, dependency/model/image/audio/tool download, or Phase Y work was performed.
