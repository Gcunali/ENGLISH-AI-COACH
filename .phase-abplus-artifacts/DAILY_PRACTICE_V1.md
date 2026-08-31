# DAILY PRACTICE V1

## Contract

Daily Practice is a short, local, deterministic practice session. It is not an exam, does not produce a CEFR result, does not block Course progress, and has no global score or pass/fail state.

## Sources and selection

Only existing learner data is used:

1. confirmed recurring mistakes (`lesson_count >= 2`, status `active` or `improving`);
2. vocabulary with status `learning`;
3. listening, repeat, speaking-check, and guided-conversation material from snapshots of completed Guided Lessons;
4. vocabulary with status `new`.

Within each priority, `SHA-256(local date + stable item id)` produces a stable daily rotation. Items practiced in the preceding three days are demoted while alternatives exist. The mixed queue interleaves mistake repair, vocabulary, dictation, shadowing, and speaking recall and removes duplicate stable item IDs. If a source is absent, the remaining real sources fill the session. No Qwen call is made.

The UI requests up to seven items. A session snapshot preserves the exact selected content and provenance so a future content change cannot alter an in-progress session.

## Persistence and gamification

Migration 020 adds an additive practice ledger. Item completion and time events are idempotent. A fully completed session creates one immutable XP event worth 20 XP under rule version 1. Accuracy never changes XP. The existing gamification aggregate includes this event and qualifying active time for XP, streak, and weekly-goal calculations. Repeated clicks cannot award the event twice.

## Privacy and boundaries

Typed and self-assessment results stay in the local SQLite database. Microphone audio is temporary and is not stored in the database. Daily Practice does not change Placement, CEFR, Review scheduling, recurring-mistake status, or lesson completion.

## Empty state

With no eligible local content, no session is fabricated. The learner is directed to complete a lesson or add vocabulary.
