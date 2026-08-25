# Phase V Initial Audit

Audited on 2026-08-24 before implementation changes.

## Baseline

- Human database schema: 17.
- `PRAGMA integrity_check`: `ok`.
- Foreign-key violations: 0.
- Guided sessions, stage states, runtime states, pronunciation attempts, exercise attempts and committed conversation turns: all 0 in the human database.
- Standard lessons: 12; standard lesson analyses: 8; transcript messages: 84.
- Package Schema and Flow Version are both 1.
- Capabilities before Phase V: Theory, Visual Vocabulary, Listening, Repeat, Speaking Check, Exercise and Guided Conversation READY; Analysis NOT READY.

## Required audit answers

### A. Stage completion persistence

`interactive_lesson_stage_state` persists status, attempt count, result version and typed JSON in `completion_json`. The repository completes a stage transactionally, activates the next stage, and completes the session only after the final stage. Guided Conversation uses the same transition semantics with its own completion payload.

### B. Selected pronunciation attempts

Repeat and Speaking Check selection lives in the versioned `interactive_lesson_stage_runtime_state.state_json`. Stage completion freezes the ordered selected attempt IDs in `completion_json.selectedAttemptIds`. Analysis must use those frozen IDs and join them to `interactive_lesson_pronunciation_attempt` and persisted acoustic results; it must never choose the highest score.

### C. Selected exercise attempts

`interactive_lesson_exercise_attempt.selected = 1` is protected by a partial unique index per session/stage/exercise. The Exercise completion result also freezes exact deterministic totals. Analysis can verify both and reads only selected rows; it does not regrade responses or inspect answer keys.

### D. Guided Conversation transcript

Committed turns are stored append-only in `interactive_lesson_guided_conversation_turn`, uniquely ordered by `sequence_index`, with only `student` and `assistant` roles. Draft streaming deltas are not persisted. Analysis must query only rows for the same session and stage in sequence order.

### E. Structured Guided corrections

No structured Guided Conversation correction table or payload exists in Phase U. Phase V will not invent a correction parser or extra LLM call; evaluator focus areas are sufficient.

### F. Standard Lesson Analyzer and Ollama

The standard analyzer calls local `http://127.0.0.1:11434/api/chat` through a no-proxy client, uses the lesson's fixed local model, `stream=false`, `think=false`, JSON format, temperature 0.1 and an 8192-token context. It is coupled to standard lesson rows and synchronizes Learning Memory after success, so the new analyzer must be a separate domain.

### G. Strict JSON validation

The standard analyzer deserializes a typed payload and applies semantic validation. Interactive analysis will use its own `deny_unknown_fields` DTOs plus explicit score, enum, text-length, Markdown and student-turn-reference checks.

### H. Repair attempt

The standard analyzer permits exactly one structure-only repair after a non-empty invalid response. Interactive analysis may reuse that policy pattern but not the prompt, schema, tables or scoring.

### I. Safe reuse boundary

Safe reusable patterns are the local no-proxy HTTP client, one-call-plus-one-repair control flow, typed JSON serialization, transaction style, SHA-256 helper, reliability backup infrastructure and shared Phase Q UI primitives. Standard analyzer prompts, DTOs, repository, Learning Memory sync and score semantics are not reusable.

### J. Current completed Guided Lesson UI

Recent sessions are listed separately from standard History and route back to the existing Guided session page. A completed Guided session currently shows only a generic completion card and explicitly states that analysis is unavailable. Phase V must render the persisted analysis there without rerunning Qwen.

### K. Migration 018 rationale

Create one `interactive_lesson_analysis` table with a unique `session_id`, version metadata, immutable evidence hash/JSON, deterministic final JSON, optional conversation result/model metadata, lifecycle status, sanitized error code and timestamps. No existing table needs semantic alteration and migrations 001-017 remain untouched. There is no backfill because the human database has no Guided sessions.

## Evidence architecture decision

The immutable package snapshot and committed prior-stage records are the source of truth. The evidence builder produces a typed canonical snapshot without audio, system prompts, exercise responses/keys or raw acoustic frames. Conversation evidence persists committed turn IDs, sequence, role, word counts and a content hash; the evaluator request is reconstructed from immutable committed text. The SHA-256 is calculated over canonical serialized evidence. Retry reuses the same stored evidence and hash.

## Protected pre-phase hashes

- `voice_coach_v2.py`: `F56E16A71130C0BC4974DF13038D5937DA611AF3C90B2CE4C7891F28523D2E2D`
- `voice_coach_v2_STABLE.py`: `F56E16A71130C0BC4974DF13038D5937DA611AF3C90B2CE4C7891F28523D2E2D`
- `voice_streaming_runtime.py`: `8A8BB8FB0CFAB51F37BABC6839FF012C8C051483DA7C57AA251C08CB79E2EAFE`
- `pronunciation_engine.py`: `FA28B35D8948D325AF79686276EB51703677A2187B31215D06EF669147BE0968`
- `pronunciation_core.py`: `0ED7D58735C64844D0B45EDB6455929DA437E3FD009557DE45C64D0057A8E71F`
- `conversation_teacher.txt`: `8B5E07911A50F18E23C6338F8521660BF4CEC652496C785F4B40A4B57056F19D`
- `lesson_analyzer_v1.txt`: `6D4CB204B7D74C337546D466BCAB309A87C0859C6B38E1E067C3FA7A5D7C8C41`
- `guided_conversation_policy_v1.txt`: `C52A3480744B8A228BF1F85B29DD6019C637CD411BE9B732DC50E176EB9904A4`
- `package.json`: `572A27D52E3A998A6A22F4BF642A39B6D855708925E7BF9DB2EAB7E8AFDA5E9D`
- `package-lock.json`: `5CE35927AFEB8FEA1C35A4DC1E87439553BC48545BEDAB715A79A97628990317`

## Scope constraints

No CEFR, Student Profile, Learning Memory, Vocabulary, Recurring Mistake, Review, XP, streak, weekly-goal or achievement mutation is authorized. No final/overall English score, pass/fail, mastery claim, cloud service, dependency, crate, model download, installer or curriculum work is part of Phase V.
