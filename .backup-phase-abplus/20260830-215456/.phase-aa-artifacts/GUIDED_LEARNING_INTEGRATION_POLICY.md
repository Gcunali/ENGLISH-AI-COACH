# Guided Learning Integration Policy

Version: 1

## Completion gate

Only an `interactive_lesson_session` whose persisted status is `completed` and whose `completed_at` is present can produce global learning effects. `in_progress`, `abandoned`, and `failed` sessions are inert.

## Vocabulary

- Source: official `visual_vocabulary` items from the immutable session package snapshot.
- Free transcript and Qwen extraction are forbidden.
- Normalization reuses the global cosmetic lowercase/whitespace key.
- A new lexical item starts as `new`.
- Existing `new`, `learning`, or `known` status is preserved.
- Provenance is session + lesson + vocabulary item.
- The same session is idempotent; a repeat session adds a real occurrence without duplicating the lexical item.

## Recurring mistakes

- Source: `interactive_lesson_guided_correction`, created only from a committed student turn plus a deterministic explicit teacher correction cue.
- Raw transcript, Exercise errors, Pronunciation scores, and free-form Analysis are excluded.
- Guided occurrences retain correction/session/lesson provenance.
- Confirmation uses at least two distinct Guided Lesson IDs; repeated sessions of the same Lesson do not fabricate two Lessons.
- Global Standard and Guided evidence coexist under the existing deterministic signature.

## Memory and Review

No Guided-only memory or review system exists. The existing Learning Summary reads the global Vocabulary and confirmed Recurring Mistakes. The existing Review Engine consumes those same sources and preserves Mixed, Vocabulary, and Recurring Mistakes modes.

Full transcripts, audio, answer keys, theory, prompt text, and long conversation history never enter Learning Memory.

## Recovery

Integration is transactional and recorded in `guided_learning_integration`. Startup retries any real completed session that has no integration marker. No incomplete session can be backfilled.

