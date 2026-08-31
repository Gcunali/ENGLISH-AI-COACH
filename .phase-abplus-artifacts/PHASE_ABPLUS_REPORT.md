# PHASE AB+ REPORT

Date: 2026-08-30

Official project: `C:\ENGLISH AI COACH`

Status: **technical implementation and automated regression complete; human microphone/Bluetooth gate pending**.

## Safety, backup, and migration

- Backup: `C:\ENGLISH AI COACH\.backup-phase-abplus\20260830-215456`
- Consistent database backup SHA-256: `5F8FE7D15E5906DDD7D2CFF87995C16D1C3AF1DBBD16F036E5B5DFF603DB67E7`
- Schema before/after: 19 → 20 (`schema_migration`)
- Migration created: `020_learning_practice.sql`
- Justification: practice session snapshots, idempotent item results, active-time provenance, and a unique fixed-XP completion event could not be represented honestly by the lesson/guided event foreign keys.
- Audio is not stored in the database.
- Package/crate manifests and lockfiles are byte-identical to the backup; no dependency or model was added/downloaded.

## Learning features

Daily Practice, Dictation, Shadowing, Mistake Repair, and Speaking Recall share one persisted executor and one Practice Lab UI. Daily selection prioritizes confirmed mistakes, learning vocabulary, recent completed Guided Lesson targets, then new vocabulary; uses a date/stable-ID SHA-256 rotation; demotes items used within three days; interleaves skills; and deduplicates. It uses no Qwen, produces no CEFR result or aggregate score, and awards a fixed 20 XP once only after full completion.

Dictation uses hidden local Piper audio, deterministic normalized word-position comparison, exact/almost/review labels, reveal, diff, and replay. Shadowing requires completed listening before recording and reuses the protected acoustic engine. Mistake Repair uses only confirmed real original/corrected occurrences, never auto-resolves a mistake, and currently shows the correct empty state because all nine real rows have only one lesson. Speaking Recall requires production and local Whisper transcription before the model expression is revealed; an exact-expression check is informational and never pass/fail.

Word Pronunciation Feedback v1 is an additive presentation over existing Wav2Vec2/phonemization/CTC word alignment. Coverage must be at least 0.80 and confidence must not be low. Labels are Strong, Good, and Needs attention; at most three low words are focused. **Overall Pronunciation Score Version 1 and historical semantics are unchanged.**

Qwen is not used by any new practice selection, grading, pronunciation, or completion decision. Existing Conversation Teacher behavior remains unchanged.

## Performance features

Before AB+, each conversation transcription launched the legacy CLI and reloaded the model. After AB+, a lazy managed `whisper-server` keeps `ggml-small.en-q5_1.bin` warm for the voice session at 12 threads behind a loopback-only random port. Request/generation IDs, timeout, one restart, stale-result rejection, legacy fallback, explicit shutdown, and a Windows kill-on-close Job Object are present.

Legacy median was 2971 ms. Persistent all-request median was 2443 ms and warm median 2429 ms, an 18.24% improvement; transcripts were equivalent 5/5. A separate memory probe measured 362.0 MB working set and 854.6 MB private commit. No orphan server remained.

Static Piper content now uses an app-local SHA-256 cache keyed by normalized exact text, voice, model/config identities, engine/wake/format versions. It validates WAVs, regenerates corrupt entries, writes atomically, prunes at 250 MiB by modified-time recency, and can be inspected/cleared in Settings. Dynamic Qwen/personal/microphone content is excluded. Uncached synthesis measured 6222.53 ms; first cached read 19.1672 ms; warm cached median 0.0855 ms.

## Privacy and protected systems

Everything remains local/offline with no telemetry, CDN, API, cloud, external TTS, or web request. Microphone audio remains temporary. Static TTS persists only bundled/static lesson text. Placement, CEFR, Course completion, Review scheduling, Learning Memory, Conversation Teacher, Qwen `qwen3.5:4b`/`think=false`, Whisper model/threads, VAD 3.5 s, Piper voice, 500 ms Bluetooth wake, and Pronunciation Score v1 remain protected.

All 288 lesson package SHA-256 values match the pre-implementation manifest: zero missing and zero mismatches. Curriculum content was not rewritten.

## Verification

- Frontend: 40 files, 159 tests passed.
- Rust: 220 passed, 0 failed, 27 explicitly ignored physical/manual tests.
- Voice: bridge self-test passed; five legacy and five persistent real transcriptions; shutdown/no-orphan check passed.
- Pronunciation: repository/engine/content-check regressions plus Word Feedback v1 unit tests passed; physical microphone judgment pending.
- Curriculum/Guided Lesson/Exercise/Conversation/Analysis/AA/backup/diagnostics tests are included in the full Rust suite; the test proving all 288 production lessons start passed.
- `npm run typecheck`: passed.
- `npm run lint`: passed.
- `npm run build`: passed (non-blocking 501.25 kB Vite chunk warning).
- `cargo fmt --check`: passed.
- `cargo check --offline`: passed with existing/non-blocking dead-code warnings.
- `cargo test --offline`: passed.
- Tauri debug no-bundle build: passed at `src-tauri\target\debug\english-ai-coach.exe`.
- Human database: migration 20, integrity `ok`, zero foreign-key violations; protected counts unchanged (14 lessons, 2 guided sessions, 12 vocabulary, 9 recurring mistakes, 18 pronunciation attempts, 73 pronunciation word rows). New practice tables contain zero fabricated history.

## Regressions and remaining debt

No automated regression failed. Remaining technical debt is limited to existing compiler dead-code/unused warnings, the Vite chunk-size warning, and temporary benchmark directories that Windows policy did not permit this run to remove. Persistent Whisper uses an existing loopback-only local server because the bundled executable already provides reusable context; it is not externally exposed. The controlled latency sample is small (n=5), and combined heavy-engine memory requires the pending physical smoke test.

## Approval and Phase AC readiness

The implementation is ready for the user's human smoke test, but Phase AB+ cannot be declared fully approved while microphone, speaker, Bluetooth, and subjective audio checks remain pending. Consequently:

- `FEATURE FREEZE FOR 1.0`: **not declared yet**;
- readiness for Phase AC: **NO — wait for human approval**;
- installer/release candidate work: **not started**.

See `HUMAN_SMOKE_TEST.md` for the exact short checklist. After those checks pass, the allowed next action is to declare the feature freeze; Phase AC must still be started only by explicit user direction.
