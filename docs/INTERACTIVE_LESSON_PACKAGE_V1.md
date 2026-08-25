# Interactive Lesson Package v1

Guided Lessons are local, deterministic content packages. The product label is **Guided Lessons**; `interactive_lesson` is reserved for internal code and storage. Package schema, lesson flow, engine, session snapshot, and stage-result versions all start at `1` and evolve independently.

## Package layout

Each package is one directory containing `lesson.json` and an optional `assets/` directory. Production discovery never scans the test-fixture root. `lesson.json` is limited to 512 KB. Published packages are discovered at startup; invalid packages and drafts are omitted from the library. For duplicate `(lessonId, contentVersion)` pairs neither duplicate is eligible. The highest valid published content version for a lesson ID wins.

IDs are stable lowercase slugs containing only `a-z`, `0-9`, and hyphens (not at either end). `contentVersion` is at least 1. Limits enforced by the Rust validator include: title 100 characters; description 500; up to 5 objectives of 180 characters; up to 8 tags of 50 characters; 1–20 stages; stage title 100; stage instructions 500; up to 100 theory blocks or vocabulary items; and 1–12 Listening, Repeat, or Speaking Check items. Packages accept no unknown fields.

Required top-level fields are `packageSchemaVersion`, `lessonFlowVersion`, `lessonId`, `contentVersion`, `publicationState`, `title`, `description`, `language`, `referenceLocale`, `cefrBand`, `estimatedMinutes`, `objectives`, `tags`, `stages`, and `assets`. Version 1 accepts `language: "en"`, `referenceLocale: "en-US"`, and CEFR content labels A1–C2. CEFR is metadata, not a student gate.

## Stages and capability policy

The canonical order is `theory`, `visual_vocabulary`, `listening`, `repeat`, `speaking_check`, `exercise`, `guided_conversation`, `analysis`. A type may occur at most once and every `stageId` is unique. Every stage declares `stageSchemaVersion`, `title`, `instructions`, `required`, and a typed `payload`.

In engine v1, Theory v1, Visual Vocabulary v1, Listening v1, Repeat v1, Speaking Check v1, and Exercise v1 are runtime-available. Guided Conversation and Analysis remain unavailable. A package containing an unavailable stage—required or optional—is visible with a reason but is not startable. An unsupported stage is never fabricated, auto-completed, or silently skipped.

Theory payloads contain 1–100 typed blocks: `paragraph` (`text`), `bullet_list` (`items`), `example` (`english`, optional `explanation`), and `callout` (`text`, optional `title`). HTML and Markdown are not interpreted. Visual Vocabulary contains typed items with `itemId`, `term`, `meaning`, `example`, and optional `imageAssetId`. Completing these stages records backend-built results only: `{schemaVersion:1, kind:"acknowledged"}` and, for Visual Vocabulary, `itemCount`. It never writes global Vocabulary.

Listening payloads contain `revealTextAfterFirstPlay` and 1–12 `segments` with `segmentId`, plain `text` (1–240 characters, at most 50 words), and nullable `audioAssetId`. Repeat contains 1–12 `targets` with `targetId`, pronunciation-bounded `text` (1–160 characters and 1–12 words), nullable `referenceAudioAssetId`, and nullable `hint`. Speaking Check contains 1–12 `targets` with `targetId`, `instruction`, pronunciation-bounded `targetText`, and nullable `hint`. A declared audio asset must exist, hash correctly, and be WAV; it never falls back to Piper when invalid or missing.

Listening completion requires every segment playback to finish. Repeat requires a complete reference playback before recording and one explicitly selected completed pronunciation attempt per target. Speaking Check has no reference playback and also requires one explicitly selected completed attempt per target. There is no score threshold: a completed result with any score or confidence may be selected. All other pronunciation statuses require retry.

Prompt-like package fields—including `systemPrompt`, `overridePrompt`, `developerPrompt`, and `rawPrompt`—are unknown and therefore rejected. Packages cannot change protected prompts or inject raw instructions.

## Assets and hashes

Assets declare `assetId`, `type` (`image` or `audio`), a portable relative `path` below `assets/`, and `sha256`. Allowed files are PNG, JPG/JPEG, WebP, and WAV. URLs, absolute paths, drive-qualified paths, backslashes, traversal components, executable/script/HTML extensions, symlinks, reparse escapes, missing files, and hash mismatches are rejected. Published assets require a valid 64-digit SHA-256. The registry canonicalizes package and asset paths and verifies that the target remains inside its package.

The deterministic package hash is SHA-256 over the validator's typed canonical package snapshot, including declared asset hashes. A started session stores that typed JSON and hash, never arbitrary source JSON, binary assets, answer keys, raw paths, or prompts.

## Session lifecycle

Only one Guided Lesson session may be `in_progress`. Starting another is rejected unless explicit Start Over atomically abandons the old session and creates a new row. The first stage starts active and subsequent stages pending. A backend transaction completes or skips only the current stage, activates the next stage, and advances the session together. Required stages cannot be skipped. Repeating the same completion action is idempotent. The last stage completes the session. Completed and abandoned sessions are immutable.

The session snapshot makes a lesson resumable after its source package changes or is deleted. The minimal student context snapshot stores profile schema version, placement attempt ID, estimated CEFR, placement confidence, target CEFR, and learning-goal identifiers. It stores no transcript, memory context, scores, personal facts, or model-generated text and never gates access.

Guided Lessons use their own tables, routes, and history. They do not create a normal `lesson`, start Ollama, call the Lesson Analyzer, or write XP, streaks, achievements, weekly goals, placement, Student Profile, Learning Memory, global Vocabulary, Review, or voice-performance data. Listening/Repeat reference audio uses the isolated Guided audio runtime and Piper only when no asset is declared. Repeat/Speaking Check reuse the existing Whisper content check and Wav2Vec2 acoustic engine. Their acoustic results use `source_type = interactive_lesson` and are excluded from standalone Pronunciation history. Audio, transcript, and temporary paths are never persisted.

## Exercise stage v1

Exercise v1 is local and deterministic. Its engine, stage schema, attempt result, response, and `english_basic_v1` normalization versions are all `1`. A stage payload contains `items` with 1–20 entries. Every entry has a unique slug-like `exerciseId`, an `exerciseType`, plain-text `prompt` (maximum 500 characters), nullable plain-text `instructions` and `hint` (maximum 300 each), a typed `payload`, and package-authored `feedback`. Correct/incorrect feedback and the optional explanation are limited to 600 characters.

The six supported types are:

- `single_choice`: 2–8 `{optionId,text}` options and one existing `correctOptionId`; grading is exact ID equality.
- `multiple_select`: 2–10 options and at least one unique `correctOptionIds`; grading is exact set equality, independent of order and without partial credit.
- `fill_blank`: one `prefix`, one `suffix`, 1–12 `acceptedAnswers`, and `normalizationProfile: "english_basic_v1"`.
- `word_order`: 2–20 stable `{tokenId,text}` tokens and a `correctOrder` containing every token ID exactly once. IDs make duplicate word text unambiguous.
- `matching`: 2–10 unique left items, the same number of unique right items, and complete one-to-one `correctPairs`.
- `short_answer_exact`: 1–12 objectively enumerable `acceptedAnswers` with `normalizationProfile: "english_basic_v1"`. It must not be used for open or semantic questions.

Text answers are normalized by Unicode NFKC, trimming, collapsing internal whitespace, converting typographic apostrophes/quotes, deterministic lowercase, and removing at most one terminal `.`, `?`, or `!`. Internal punctuation, apostrophes, and hyphens remain significant. The engine performs no spelling correction, stemming, lemmatization, translation, synonym inference, or semantic similarity.

Answer keys are private package data stored in the immutable backend session snapshot. The pre-submit public DTO contains public options, sentence context, tokens, matching items, prompt, instructions, and hint, but never `correctOptionId`, `correctOptionIds`, `acceptedAnswers`, `correctOrder`, `correctPairs`, or post-submit feedback. After a real submission, the persisted attempt DTO may expose one typed canonical expected answer with the package-authored feedback.

Each submission creates an immutable row with typed response/result JSON and a monotonic per-item attempt index. Retry creates another row. Continue explicitly selects exactly the displayed attempt and advances; the engine never substitutes the best attempt. Completion requires one selected submitted attempt per item, regardless of correctness. The summary reports selected correct/incorrect counts, total attempts, and nearest-integer Exercise Accuracy. It has no pass/fail status, creates zero XP, and does not affect CEFR or any global learning data.

Example item:

```json
{
  "exerciseId": "polite-request",
  "exerciseType": "single_choice",
  "prompt": "Choose the polite request.",
  "instructions": null,
  "hint": "Look for please.",
  "payload": {
    "options": [
      {"optionId": "a", "text": "Give me coffee."},
      {"optionId": "b", "text": "I'd like a coffee, please."}
    ],
    "correctOptionId": "b"
  },
  "feedback": {
    "correct": "That is a polite request.",
    "incorrect": "Review the request form.",
    "explanation": "I'd like ... please is polite."
  }
}
```
# Guided Conversation v1 (Phase U)

`guided_conversation` is a content-guided, AI-driven stage. Its package payload is data, never a system prompt. The strict payload requires `scenario` (1–600 chars), `studentRole` and `teacherRole` (1–100), `goal` (1–400), zero to 12 unique `targetVocabulary` items (1–80), zero to 12 unique `targetExpressions` (1–180), and integer turn limits satisfying `1 <= minimumStudentTurns <= recommendedStudentTurns <= maximumStudentTurns <= 20` with minimum at most 12.

Fields resembling prompts—such as `systemPrompt`, `developerPrompt`, `overridePrompt`, `rawPrompt`, or `teacherSystemMessage`—are rejected as unknown fields. Scenario, roles, goal and target language are delimited reference data. The engine composes the unchanged base teacher prompt, static Guided policy v1, bounded immutable snapshot context, optional Student Profile, optional Learning Memory, then a static final guardrail. Exercise answer keys, accepted answers, correct ordering/pairs, private feedback and attempt responses are excluded.

Entering a Guided Conversation does not start inference, microphone capture or playback. `Start Conversation` is an explicit user gesture. A new conversation may generate one teacher opening; resume rebuilds ordered history from committed SQLite turns and never regenerates or autoplays the opening.

Only confirmed non-empty student transcripts count. Teacher turns never count. Completion is deterministic: committed student turns must reach `minimumStudentTurns`, no owned voice operation may be active, and the learner must explicitly choose `Finish Conversation`. Recommended turns are guidance only. At maximum, new student capture is disabled, but completion is never automatic. Grammar quality and target-language usage are not completion gates; there is no score or pass/fail.

Guided text turns are stored separately from standard Lessons. Student audio and temporary Piper output are not retained. Guided corrections are intentionally not promoted to global Corrections, Recurring Mistakes, Learning Memory, Vocabulary, Review, CEFR, pronunciation, XP, streaks or achievements in v1.

## Analysis v1

`analysis` is engine-owned and, when present, must be the final stage. Its strict package payload is exactly `{}`. Authors cannot supply weights, pass scores, CEFR rules, prompts, rubrics or custom evaluators. A required Analysis stage is startable when every stage in the package is supported; an optional Analysis stage follows the normal optional-stage skip rule.

The engine builds one immutable evidence snapshot from the session's package snapshot, completed/skipped stage results, selected Exercise attempts, selected acoustic Pronunciation attempts and committed Guided Conversation turns. It never reads the mutable source package after session start. Exercise Accuracy remains an Exercise Engine result; Acoustic Match remains a Pronunciation Engine result; stage status remains participation evidence; and only conversation language is interpreted by the local conversation evaluator.

The persisted result is deliberately multidimensional: participation, Conversation Grammar/Vocabulary/Conversational Fluency/Interaction, Exercise performance, Pronunciation practice, strengths, focus areas and objectives practiced. Package objectives are never labeled mastered. No overall/final English score, metric average, pass/fail or CEFR result is created. Analysis does not write Learning Memory, global Vocabulary, Recurring Mistakes, Review, XP, streaks, weekly goals or achievements.
