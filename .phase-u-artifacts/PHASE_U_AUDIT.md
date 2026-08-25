# PHASE U — INITIAL AUDIT

Date: 2026-08-24
Project: `C:\ENGLISH AI COACH`

## Capability before

- Theory, Visual Vocabulary, Listening, Repeat, Speaking Check and Exercise: READY (schema v1).
- Guided Conversation and Analysis: NOT READY.
- Package schema, lesson flow, engine and snapshot versions: 1.
- Human database schema: 16.

## Required findings

A. Voice receives the unchanged base prompt from `voice_coach_v2.py`; Rust passes Lesson Mode, Student Profile and Learning Memory blocks through environment variables. The bridge builds one system message.
B. Context, streaming-enabled and temp-audio-root settings still use environment variables.
C. Python stdout is JSONL for events; stdin is JSONL for cancel/shutdown controls.
D. The bridge keeps in-memory ordered messages and appends confirmed student/final teacher turns once.
E. Initial persisted history is not supported before Phase U.
F. Standard student turns are persisted by `LessonSessionManager::enrich_event` into `transcript_message`.
G. Standard authoritative assistant finals use the same path; deltas are not persisted.
H. Cancelled streaming follows Phase N delivered-partial semantics; generation IDs reject stale output.
I. Correction detection is deterministic and runs only on authoritative teacher text. Standard correction tables feed global Lesson history, so Guided data needs isolation.
J. Student Profile is rendered in Rust and passed as a separate context block; Guided sessions already hold an immutable minimal profile snapshot.
K. Learning Memory is rendered in Rust and passed when enabled; Guided may read but must not write it.
L. `VoiceEngineManager` owns one child process and rejects concurrent starts; Windows Job Object cleans its process tree.
M. The single voice process plus Guided audio locks prevent concurrent app-owned voice pipelines; a persisted session alone holds no audio lock.
N. Generalize only runtime owner/persistence sink and structured Guided configuration/history. Reuse bridge, streaming runtime, Whisper, Ollama parser, sentence chunker, Piper and cancellation.
O. Migration 017 is necessary for restart-safe Guided text turns without contaminating standard Lesson history. Structured Guided corrections are deferred unless clean isolated reuse is proven.

## Protected baseline

- `voice_coach_v2.py`: `F56E16A71130C0BC4974DF13038D5937DA611AF3C90B2CE4C7891F28523D2E2D`
- `voice_coach_v2_STABLE.py`: `F56E16A71130C0BC4974DF13038D5937DA611AF3C90B2CE4C7891F28523D2E2D`
- `voice_streaming_runtime.py`: `8A8BB8FB0CFAB51F37BABC6839FF012C8C051483DA7C57AA251C08CB79E2EAFE`
- `conversation_teacher.txt`: `8B5E07911A50F18E23C6338F8521660BF4CEC652496C785F4B40A4B57056F19D`
- `pronunciation_engine.py`: `FA28B35D8948D325AF79686276EB51703677A2187B31215D06EF669147BE0968`

Backup: `C:\ENGLISH AI COACH\.backup-phase-u\20260824-213346`
Pre-017 database SHA-256: `7296CA0867A4E76888596E22E1768C0E640E4D6E08C7773B6627466BCBD3213C`
