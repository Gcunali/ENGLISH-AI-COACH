# Phase X Content Quality Report

Final automated review: 2026-08-25. Scope: published `english-core` v1 only.

## Inventory and identity

- Levels: A1 and A2 only; no B1–C2 package or placeholder.
- Units: 16 total, exactly 8 per Level and 6 Lessons per Unit.
- Lessons: 96 total, exactly 48 A1 and 48 A2.
- Stable IDs: 96 unique; titles: 96 unique; contentVersion: 1 throughout.
- Publication: 96 packages and one complete Curriculum published only after draft gates.
- Typed Registry result: 96 valid, 0 invalid; Curriculum Registry: 1 valid, 0 invalid/broken refs.
- Package manifest: `.phase-x-artifacts/PRODUCTION_CONTENT_MANIFEST.md` lists all 96 official typed package hashes.

## Eight-stage completeness

Each stage occurs exactly 96 times: Theory, Visual Vocabulary, Listening, Repeat, Speaking Check, Exercise, Guided Conversation, and Analysis. Every stage is required and in canonical order. Analysis uses the unchanged empty v1 payload.

## Editorial bands

| Measure | A1 | A2 | Result |
|---|---:|---:|---|
| Theory words | 226–294; avg 271.5 | 303–340; avg 318.8 | Pass; A2 is longer |
| Vocabulary items/Lesson | 8–9 | 9 | Pass |
| Listening segments/Lesson | 3 | 4 | Pass; A2 is longer |
| Repeat targets/Lesson | 5 | 5 | Pass |
| Speaking targets/Lesson | 4 | 4 | Pass |
| Exercises/Lesson | 8 | 9 | Pass; A2 has extra contextual choice |
| Conversation turns | 4/6/8 | 5/7/10 | Pass; A2 is longer |

The language progression follows the approved ledgers. Each Lesson's models recur across listening, Repeat, Speaking Check, exercise, and Guided Conversation. Lesson 6 of every Unit is an integrated mission/review without major new grammar.

## Exercise distribution

- `single_choice`: 240
- `multiple_select`: 96
- `fill_blank`: 192
- `word_order`: 96
- `matching`: 96
- `short_answer_exact`: 96

All six existing types occur. Every Lesson uses at least five types; exact short answer is limited to a displayed finite model. Official parser validates all private answer structures. Public summary/session DTO regression found no `correctOptionId`, `correctOptionIds`, `acceptedAnswers`, `correctOrder`, or `correctPairs` leak.

## Duplication and recycling

An initial QA pass found two generic closing chunks and three repeated Theory explanation forms above the allowed course-wide threshold. The authoring templates were corrected before final hashes/publication. Final audit:

- exact Repeat expressions occurring in more than six Lessons: 0;
- exact long Theory blocks occurring in more than six Lessons: 0;
- duplicated IDs: 0;
- duplicated titles: 0.

Repetition up to a Unit boundary is intentional recycling; central grammar, goals, examples, vocabulary sets, and scenarios remain Lesson-specific.

## Runtime and references

- Official package parser: 96/96.
- Exact Curriculum refs: 96/96.
- Official Guided engine startability in isolated migrated SQLite: 96/96.
- Published Course aggregation: A1 8/48, A2 8/48, total 16/96, initial progress 0% with no sessions.
- Placement B1 test: retains B1, marks neither A1 nor A2 as equivalent, and shows an honest installed A1–A2 notice.

## Size and assets

- Lesson JSON: 2,074,641 bytes total.
- Curriculum JSON: 37,682 bytes.
- Combined production content: 2,112,323 bytes (about 2.01 MiB).
- Packaged assets: 0. No image, audio, model, dependency, or tool was downloaded. Listening uses the existing runtime audio path.

## Originality and remaining human review

Content was authored for this project in en-US and does not reproduce a commercial course. Automated quality gates passed. The following are honestly **PENDING**: complete human A1 lesson, complete human A2 lesson, pedagogical review by a person, Bluetooth, human Pronunciation, and human Guided Conversation. These are not represented as completed tests.
