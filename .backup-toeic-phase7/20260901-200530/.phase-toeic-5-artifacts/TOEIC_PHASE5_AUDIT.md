# TOEIC Phase 5 Audit

Date: 2026-09-01

## Current architecture

- TOEIC sessions and first-attempt answers are stored generically in `toeic_session` and `toeic_answer`.
- Migration 021 already reserves `part5_incomplete_sentence` and the `reading` section.
- Bundled content is loaded from `src-tauri/resources/toeic/item-bank-v1`; production questions are not copied into SQLite.
- Parts 1–4 and the 100-question Full Listening parent/child architecture are present.
- The unofficial Listening profile is isolated in its own versioned module and tables.

## Decision

No migration is necessary. Part 5 will use a dedicated runtime module over the existing generic TOEIC tables and a bundled `part5.json` bank. A frozen form snapshot will preserve item IDs, versions, order, authored explanations, and taxonomy for every attempt.

## Content strategy

Phase 5 publishes Form A first: 30 original questions, followed by automated structural, balance, duplication, explanation, and ambiguity-oriented checks. Forms B/C are deferred rather than filled with lower-confidence items. Runtime grading and feedback make zero Qwen calls and load no audio models.
