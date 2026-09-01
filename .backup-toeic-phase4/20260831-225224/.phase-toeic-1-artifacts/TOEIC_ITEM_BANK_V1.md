# TOEIC Item Bank v1

## Location and ownership

- Bundled root: `src-tauri/resources/toeic/item-bank-v1`
- Manifest: `bank.json`
- Runtime ownership: read-only `ToeicItemBank` Rust registry
- User database stores sessions and answers only; production questions are not copied into SQLite.

## Versioning and publication

The bank has `bankSchemaVersion` and `bankId`. Every item has a stable `itemId`, positive `itemVersion`, `publicationState`, typed section/part/difficulty, skill tags, a hashed local image, four statements, a deterministic answer key, and authored feedback. Every form has a stable `formId`, `formVersion`, publication state, and fixed item/version order.

Published Phase 1 content: 18 unique items in three fixed six-item forms. Each form has two easy, two medium, and two hard items.

## Validation contract

Startup validation rejects unsupported bank schemas, unknown fields, duplicate IDs/versions, non-Part-1 production items, invalid tags/distractor types, empty or duplicate statements, anything other than exactly A-D, invalid answer keys, missing rationales, incomplete distractor feedback, unsafe/symlink asset paths, absent or implausibly sized PNGs, SHA-256 mismatches, incomplete/duplicate forms, draft references, and unbalanced published forms.

Supported future typed parts exist for Parts 1-7, but `runtimeAvailable` is true only for `part1_photograph`.

## Pre-answer security

The public question DTO contains only item identity/version, position, base64 photograph, MIME type, A-D labels, and presentation status. It has no statement text, correct answer, correctness flag, explanation, distractor metadata, or skill metadata. The regression test serializes this DTO and checks those fields are absent.

## Authorship

All 18 scenarios, statements, choices, explanations, and generated images were created for English AI Coach. No ETS sample question, photograph, audio, preparation-book content, web bank, or third-party image was used.
