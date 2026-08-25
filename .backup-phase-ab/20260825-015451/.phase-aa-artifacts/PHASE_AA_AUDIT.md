# Phase AA — Audit

Date: 2026-08-25

## Outcome

Migration 019 is required. Schema 18 cannot safely express Guided provenance for global vocabulary or recurring mistakes, cannot store reliable active Guided practice time, and the XP ledger accepts only Standard Lesson sources.

The migration will be limited to these integration needs. Curriculum progress, completion, hierarchy, and all 288 Lesson Packages remain unchanged.

## Answers to the mandatory audit

1. Guided data usable without a new table: official completion state, stable `lesson_id`, package snapshot, official Visual Vocabulary, course progress, stage participation, pronunciation attempts, deterministic exercise results, Guided Conversation turns, and Interactive Analysis. Only completed sessions may generate global learning effects.
2. Provenance is insufficient for safe global integration. Existing `lesson_vocabulary` and `recurring_mistake_occurrence` require Standard Lesson/Analysis foreign keys. A stable Guided session relation is needed for idempotency and distinct-session counting.
3. Reliable Guided active duration does not exist. `completed_at - started_at` can include pauses. A foreground heartbeat ledger must begin at Phase AA with no retroactive fabrication.
4. The current XP ledger does not support Guided sources. Its checks and foreign key accept only `lesson` and `qualifying_lesson_completed`.
5. New achievements do not need achievement-specific schema: `achievement_unlock` is generic. Repository criteria must be extended deterministically.
6. Learning Summary already reads global Vocabulary and confirmed Recurring Mistakes automatically. Its Standard Lesson score/strength sections remain separate.
7. Review already consumes global `vocabulary_item` and confirmed `recurring_mistake` automatically. No new review engine is needed.

## Structured evidence policy

- Vocabulary: only official `visual_vocabulary` items from the immutable session package snapshot.
- Mistakes: only explicit structured Guided correction records. Raw transcript, free-form Analysis, Exercise errors, and Pronunciation scores are excluded.
- Memory: only through the existing global Vocabulary and Recurring Mistakes repositories; never full transcripts, audio, answer keys, theory, or long conversation history.
- Pronunciation remains an independent acoustic signal and never changes XP, progress, CEFR, or recurring grammar mistakes.

## Migration 019 scope

- Guided integration provenance/idempotency.
- Guided vocabulary occurrence provenance.
- Structured Guided correction evidence and recurring-mistake provenance.
- Active Guided practice heartbeat seconds.
- Guided XP source support while preserving the existing ledger.

No curriculum, course progress, lesson package, or duplicated completion tables will be added.

## Baseline and human data

- Human database schema before AA: 18.
- Human database SHA-256 before AA: `F49AB7E6DD7DB23CD984798A7DF055D36A0FC5A9A2B45E1B0A2D80BCA4ABC3A8`.
- Existing Guided completions: 0; no completion will be fabricated for testing.
- Installed curriculum: 6 levels, 48 units, 288 published/startable lessons.
- Required backup: `.backup-phase-aa/20260825-012920`.

