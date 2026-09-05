# TOEIC Forms B/C Production Report

Status date: 2026-09-02

## Part 2 — Question–Response

| Form | Items | Difficulty E/M/H | Answers A/B/C | Prompt types | Broken refs | Duplicate prompts | Startable |
|---|---:|---:|---:|---:|---:|---:|---|
| A | 25 | 7/12/6 | 9/8/8 | 16 | 0 | 0 | PASS |
| B | 25 | 7/12/6 | 9/8/8 | 16 | 0 | 0 | PASS |
| C | 25 | 7/12/6 | 8/9/8 | 16 | 0 | 0 | PASS |

Physical Part 2 total: **75 questions**.

Validation performed:

- Rust bank/schema validator: PASS.
- A/B/C form size and reference validation: PASS.
- Form B/C start and frozen first-item snapshot: PASS.
- Exact and token-similarity prompt duplicate audit (threshold 0.70): 0 findings.
- Required authored rationales and distractor feedback: PASS.
- Difficulty and answer-position limits enforced by the production validator: PASS.
- Textual TTS suitability review: PASS (plain English, no unusual proper-name or symbol dependency).
- Runtime audio lifecycle remains covered by the existing Part 2 regression suite; exhaustive human listening remains part of Phase 8.

Form A content and item versions were not changed.

## Remaining production checkpoints

## Part 3 — Conversations

| Form | Sets | Questions | Difficulty E/M/H | Answers A/B/C/D | Question types | Scenarios | Graphics | Startable |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| A | 13 | 39 | 2/7/4 | 11/9/12/7 | 14 | 13 | 3 | PASS |
| B | 13 | 39 | 3/6/4 | 10/10/10/9 | 13 | 13 | 3 | PASS |
| C | 13 | 39 | 3/6/4 | 10/10/9/10 | 13 | 13 | 2 | PASS |

Physical Part 3 total: **39 sets / 117 questions**.

Part 3 validation performed:

- Rust schema/content validator and complete-form checks: PASS.
- Form B/C start and frozen first-set snapshot: PASS.
- Set/question IDs: 156/156 unique.
- Broken form references: 0.
- New-content question similarity audit (threshold 0.70): 0 findings after two template rewrites.
- Two actual local voices (`amy`, `lessac`) with `en-US` metadata are assigned to every dialogue; no unsupported accent diversity is claimed.
- Embedded table graphics are structurally valid and require no external asset path.
- Runtime audio lifecycle, interruption, grouped feedback and resume regressions: PASS.
- Exhaustive human listening remains part of the gated Phase 8 protocol.

## Part 4 — Talks

| Form | Sets | Questions | Difficulty E/M/H | Answers A/B/C/D | Question types | Scenarios | Graphics | Startable |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| A | 10 | 30 | 2/4/4 | 8/8/7/7 | 13 | 10 | 3 | PASS |
| B | 10 | 30 | 3/4/3 | 8/7/8/7 | 12 | 10 | 2 | PASS |
| C | 10 | 30 | 3/4/3 | 7/8/7/8 | 14 | 10 | 1 | PASS |

Physical Part 4 total: **30 sets / 90 questions**.

Part 4 validation performed:

- Rust schema/content validator: PASS.
- Forms B/C startability and frozen first-talk snapshot: PASS.
- Part 4 regression suite: 5/5 PASS.
- Set/question IDs: 120/120 unique; broken references: 0.
- Ten distinct talk scenarios per Form with both supported local voices represented across each bank.
- New-content similarity findings were rewritten before publication; final threshold audit is recorded by the continuation tests.
- Embedded table graphics have valid columns and rows and require no external assets.
- Human listening/naturalness validation remains gated to Phase 8.

- Part 4 Forms B/C: complete.
## Part 5 — Incomplete Sentences

| Form | Items | Grammar/Vocabulary | Difficulty E/M/H | Answers A/B/C/D | Subcategories | Startable |
|---|---:|---:|---:|---:|---:|---|
| A | 30 | 18/12 | 9/14/7 | 8/8/7/7 | 25 | PASS |
| B | 30 | 18/12 | 7/17/6 | 8/8/7/7 | 27 | PASS |
| C | 30 | 18/12 | 7/17/6 | 7/8/8/7 | 27 | PASS |

Physical Part 5 total: **90 questions**.

Part 5 validation performed:

- Published schema, one blank, four unique choices, answer key and completed-sentence validation: PASS.
- Authored correct and distractor explanations: PASS.
- Forms B/C startability and frozen first-item snapshot: PASS.
- Part 5 regression suite: 7/7 PASS.
- Item IDs: 90/90 unique; broken references: 0.
- Grammar/vocabulary target: 60%/40% for every Form.
- New-content sentence similarity audit (threshold 0.65): 0 findings.
- Automated ambiguity checks found no duplicate alternatives or completion mismatch; human editorial validation remains Phase 8 work.

- Part 5 Forms B/C: complete.
## Part 6 — Text Completion

| Form | Sets | Questions | Answers A/B/C/D | Sentence insertions | Document types | Minimum passage length | Startable |
|---|---:|---:|---:|---:|---:|---:|---|
| A | 4 | 16 | 5/4/4/3 | 4 | 4 | 790 | PASS |
| B | 4 | 16 | 5/4/4/3 | 4 | 4 | 766 | PASS |
| C | 4 | 16 | 4/4/4/4 | 4 | 4 | 806 | PASS |

Physical Part 6 total: **12 sets / 48 questions**.

Part 6 validation performed:

- Published schema, four sets/Form, four questions/set and one insertion/set: PASS.
- Blank occurrence, four unique choices, completed context and authored explanations: PASS.
- Forms B/C startability and frozen first-set snapshot: PASS.
- Part 6 regression suite: 5/5 PASS.
- Set/item IDs: 60/60 unique; broken references: 0.
- Passage similarity audit (threshold 0.45): 0 findings.
- All passages exceed the validator's 450-character contextual minimum and use cohesion across surrounding sentences.

- Part 6 Forms B/C: complete.

## Part 7 — Reading Comprehension

| Form | Sets | Questions | Single/Multiple questions | Double/Triple sets | Answers A/B/C/D | Cross-document questions | Startable |
|---|---:|---:|---:|---:|---:|---:|---|
| A | 15 | 54 | 29/25 | 2/3 | 14/14/13/13 | 18 | PASS |
| B | 15 | 54 | 29/25 | 2/3 | 14/14/13/13 | 10 | PASS |
| C | 15 | 54 | 29/25 | 2/3 | 13/13/14/14 | 10 | PASS |

Physical Part 7 total: **45 sets / 162 questions**.

Part 7 validation performed:

- Published schema, fifteen sets/Form, 29 single-passage and 25 multiple-document questions/Form: PASS.
- Two double-document and three triple-document sets in every Form: PASS.
- Forms A/B/C startability and frozen first-set/item snapshots: PASS.
- Part 7 regression suite: 5/5 PASS.
- Set/item IDs: 207/207 unique; broken references: 0.
- Passage similarity audit across all Forms (Jaccard threshold 0.65): 0 findings.
- Forms B/C include authored evidence in the existing `completedContext` field and explicit correct/distractor explanations without changing schema version 1.
- Cross-document questions require combining dates, quantities, schedules, decisions, or consequences from at least two documents.
- Automated checks cannot certify naturalness or ambiguity with human judgment; that remains explicitly gated to Phase 8.

- Part 7 Forms B/C: complete.

All physical Forms B/C for Parts 2–7 are complete. Part 1 physically contains A/B/C. Matching Listening, Reading and Full L&R A/B/C family snapshots, pause/reopen/resume, ownership and global regressions pass; Blocker 7 is CLOSED.
