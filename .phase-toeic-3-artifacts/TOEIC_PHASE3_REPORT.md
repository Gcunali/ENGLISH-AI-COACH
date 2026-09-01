# TOEIC Phase 3 Final Report

## Outcome

Listening Part 3 — Conversations is integrated and ready as one complete production Form A. Work stopped before Part 4 and Reading. No Listening 5–495 score was created.

## Safety and persistence

- Backup: `.backup-toeic-phase3/20260831-221413` with source/resources/configuration, human DB, and manifest.
- Human DB SHA-256 before/after: `BFD6DD212BE1938F87AAEF479453D10FE9A32E3E439B0C878B969B12B20F122B`; implementation did not mutate it.
- Schema before/after: 21. No migration: the generic TOEIC tables already support grouped sets, 39 first answers, presentation state, snapshots, history, and results.
- Form snapshot freezes form/version and ordered set IDs. Content remains bundled and read-only; scripts are not copied into SQLite.

## Architecture and content

- One Form A; 13 sets; 39 questions; exactly three A/B/C/D questions per set.
- Group-level one-play conversation presentation with turn-level Piper synthesis.
- Amy/lessac alternation gives two distinguishable voices. Data validation supports 2–3 speakers; three-speaker production is pending because only two suitable English voices exist.
- Both installed voices are en-US. Accent diversity is not claimed or artificially simulated.
- Three original structured graphics are supported and published.
- Questions/choices/graphics are public before audio; scripts, keys, explanations, and evidence remain backend-only until all three answers are locked.
- Q1/Q2 submission exposes only recorded selection. Q3 unlocks correctness, explanations, evidence, listening notes, transcript, and replay.
- First answers are immutable and deterministic (1/0); runtime grading/analytics/feedback make zero Qwen calls.

## Resume and interruption

- Untimed; no countdown, expiration, or time-based scoring.
- Before audio: free resume. During interrupted audio: presentation is interrupted and restarts from the beginning without penalty.
- After completed audio: no replay before Q3. Answers already submitted remain locked after restart.
- After each completed set: Continue and Pause & Exit are explicit safe points.
- Automated four-set pause/resume restored Conversation 5 with 12 answers and unchanged ordering.

## Results, review, and history

- Result: X/39 and accuracy, with deterministic question-type, difficulty, scenario, and skill breakdowns.
- Review Mistakes returns complete erroneous conversation sets with transcript, three questions, answers, explanations, evidence, graphics, and notes.
- Shared TOEIC history recognizes Part 3 and uses a 39-question denominator. Parts 1 and 2 remain routed to their original sessions/results.
- Structural Listening coverage is 70/100; it is never converted to a scaled score.

## Validation

- Part 3 focused Rust: 5/5 passed, including no-leak, feedback-after-Q3, immutable answer, interruption, Conversation-5 resume, and full 39-question completion/review.
- Full Rust: 240 passed, 0 failed, 27 existing manual gates ignored. `cargo fmt --check` and `cargo check --offline` passed.
- Frontend typecheck and lint passed; Vite production build passed.
- Full frontend rerun: 163 files and 648 tests passed, 0 failed. An earlier unrelated Review timing fluctuation also passed immediately in isolation.
- Tauri debug no-bundle build passed. Executable: `src-tauri/target/debug/english-ai-coach.exe`. No installer generated.
- Piper real synthesis passed for Amy and Lessac.
- Course regression: all 288 production packages remained present and the Rust startability/validation gates passed.
- Part 1 and Part 2 regression tests passed in the full Rust suite.
- Invalid sets: 0; invalid questions: 0; broken forms: 0.

## Fidelity limitations

- Human full-form listening/naturality/ambiguity smoke test: PENDING, explicitly not fabricated.
- Production has one quality-controlled form, not three padded forms.
- Only en-US voices are installed; three-speaker and accent-diverse production remain pending.

The platform is structurally ready for Phase 4 — Listening Part 4: Talks, but Phase 4 was not started.
