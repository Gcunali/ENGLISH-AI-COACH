# PHASE AB+ — PRE-IMPLEMENTATION AUDIT

Audit date: 2026-08-30

Official project: `C:\ENGLISH AI COACH`

Baseline commit: `da889b2` (`Fase AB`)

## Safety baseline

- The application was stopped before the backup.
- Pre-existing working-tree changes were preserved and treated as user-owned work.
- Source backup: `C:\ENGLISH AI COACH\.backup-phase-abplus\20260830-215456`
- Consistent human database backup: `human-database\english-ai-coach.sqlite3`
- Database backup SHA-256: `5F8FE7D15E5906DDD7D2CFF87995C16D1C3AF1DBBD16F036E5B5DFF603DB67E7`
- Database schema version: 19
- SQLite integrity check: `ok`
- Foreign-key violations: 0
- Lesson package baseline: 288 files, recorded in the backup manifest.

## 1. Current AA/AB state

Phases AA and AB are present. AA provides guided lesson provenance and idempotency through migration 019. AB contains the current white/blue product UI and its verification artifacts, but its report still describes a human-gate/finalization state. The repository also contained pre-existing uncommitted UI, icon, interactive-lesson, and AB-artifact changes before AB+ began; AB+ must not discard or overwrite them blindly.

## 2. Current database and useful learning data

The human database is healthy and contains real local learning history: 14 standard lessons, 2 interactive lesson sessions, 12 vocabulary items, 9 recurring-mistake records, 18 pronunciation attempts, and 73 pronunciation word results. All 12 current vocabulary items have status `new`. The recurring-mistake policy is already honest: a mistake is confirmed only after appearing in at least two lessons. At audit time, all nine rows have `lesson_count = 1`, so Mistake Repair must correctly show an empty state until a mistake becomes confirmed.

## 3. Review and mistake provenance

The existing Review feature already builds a deterministic queue from vocabulary and confirmed recurring mistakes. It keeps original phrase, corrected phrase, explanation, and occurrence provenance. Legacy mistake occurrences and guided-lesson occurrences are stored in different paths, so AB+ must merge them without inventing provenance and without marking a mistake as resolved merely because it was practiced.

## 4. Sources available for the new practice modes

The following local, already-existing sources can be reused without changing lesson packages:

- vocabulary items and their review state;
- confirmed recurring mistakes and their real occurrences;
- completed/recent interactive lesson snapshots;
- structured vocabulary, listening, repeat, speaking, and guided-conversation content already present in lesson packages;
- existing pronunciation attempts and word results.

Daily Practice can therefore be deterministic, mixed, deduplicated, and history-aware without an LLM call.

## 5. Current gamification model

Standard lessons and guided lessons use separate immutable XP event tables, and the repository unifies them when calculating progress. Their constraints and foreign keys do not allow a practice session to be represented honestly. AB+ therefore justifies a new, additive practice-session/activity ledger in migration 020, followed by inclusion of its immutable completion events in the existing aggregate. XP must be tied to a unique persisted completion event, not to accuracy or repeated button clicks.

## 6. Current pronunciation architecture

Pronunciation score version 1 already uses a persistent Python JSONL process, local Wav2Vec2 logits, phonemization, CTC alignment, phone-to-word mapping, and persisted word results. The UI already renders expandable per-word/per-phone data. AB+ must preserve the overall score and add a separate word-feedback presentation version, explicit alignment confidence/coverage gates, focus-word selection, and pedagogical labels such as `Strong`, `Good`, and `Needs attention`. It must never claim that a low-confidence alignment proves a word was wrong.

## 7. Current Whisper lifecycle

The voice bridge is persistent during a conversation, but each transcription currently launches `whisper-cli.exe`, loads `ggml-small.en-q5_1.bin`, transcribes one file, and exits. The model is therefore reloaded for every student turn.

Controlled five-run legacy baseline on the existing `local-ai\mic-test.wav`, using 12 threads:

| Run | End-to-end latency |
| --- | ---: |
| 1 | 3135 ms |
| 2 | 2968 ms |
| 3 | 2947 ms |
| 4 | 2971 ms |
| 5 | 6249 ms |

- Median: 2971 ms
- Mean: 3654 ms
- Maximum observed: 6249 ms
- Peak working set observed: approximately 480.4 MB
- Transcript consistency: 5/5 produced `Hi, my name is Guilherme. I'm 22.`

Existing compiled Whisper libraries, headers, model, CLI, and server binaries are available locally. A small project-owned JSONL worker can therefore load the same model once without adding a dependency or downloading anything. It must have bounded requests, generation/request identifiers, one controlled restart, legacy fallback, explicit shutdown, and orphan-process protection.

## 8. Current Piper lifecycle and cache

Dynamic conversation speech already keeps a `PiperVoice` object alive within the voice session when the Python runtime succeeds. Guided lesson reference audio, however, is cached only in memory and temporary files; it is deleted on session cleanup and therefore is re-synthesized after restart. AB+ justifies a persistent static-content cache under app-local data, keyed by exact normalized text, voice/model identity, synthesis parameters, and cache-format version. Dynamic Qwen or personal text must not enter that cache.

## 9. Offline and dependency constraints

All required engines and models are already present locally. AB+ does not need a network service, cloud API, dependency installation, model download, or lesson-package rewrite. The implementation must continue to operate on the CPU-only 16 GB target.

## 10. Navigation and UI integration

The current sidebar already contains Home, New Lesson, Review, and Pronunciation. To avoid overcrowding it, AB+ should add one `Practice` destination containing Daily Practice, Dictation, Shadowing, Speaking Recall, and Mistake Repair cards, plus a shared practice-session surface. Existing visual tokens and responsive behavior should be reused.

## 11. Migration decision

Migration 020 is justified for persistent practice sessions, immutable item results/provenance, active-practice timing, and idempotent gamification events. No pronunciation schema change is required because word-level acoustic results are already persisted. No lesson package will be modified.

## 12. Release boundary

AB+ ends with learning/performance enhancements, tests, benchmarks, reports, integrity checks, and a debug standalone build. It does not include Phase AC, an installer, new lesson content, cloud features, or a new acoustic score version.
