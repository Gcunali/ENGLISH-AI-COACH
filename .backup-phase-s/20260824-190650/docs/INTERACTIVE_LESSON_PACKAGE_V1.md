# Interactive Lesson Package v1

Guided Lessons are local, deterministic content packages. The product label is **Guided Lessons**; `interactive_lesson` is reserved for internal code and storage. Package schema, lesson flow, engine, session snapshot, and stage-result versions all start at `1` and evolve independently.

## Package layout

Each package is one directory containing `lesson.json` and an optional `assets/` directory. Production discovery never scans the test-fixture root. `lesson.json` is limited to 512 KB. Published packages are discovered at startup; invalid packages and drafts are omitted from the library. For duplicate `(lessonId, contentVersion)` pairs neither duplicate is eligible. The highest valid published content version for a lesson ID wins.

IDs are stable lowercase slugs containing only `a-z`, `0-9`, and hyphens (not at either end). `contentVersion` is at least 1. Limits enforced by the Rust validator include: title 100 characters; description 500; up to 5 objectives of 180 characters; up to 8 tags of 50 characters; 1–20 stages; stage title 100; stage instructions 500; and up to 100 theory blocks or vocabulary/listening/repeat items. Text inside blocks and items is also bounded. Packages accept no unknown fields.

Required top-level fields are `packageSchemaVersion`, `lessonFlowVersion`, `lessonId`, `contentVersion`, `publicationState`, `title`, `description`, `language`, `referenceLocale`, `cefrBand`, `estimatedMinutes`, `objectives`, `tags`, `stages`, and `assets`. Version 1 accepts `language: "en"`, `referenceLocale: "en-US"`, and CEFR content labels A1–C2. CEFR is metadata, not a student gate.

## Stages and capability policy

The canonical order is `theory`, `visual_vocabulary`, `listening`, `repeat`, `speaking_check`, `exercise`, `guided_conversation`, `analysis`. A type may occur at most once and every `stageId` is unique. Every stage declares `stageSchemaVersion`, `title`, `instructions`, `required`, and a typed `payload`.

In engine v1, only Theory v1 and Visual Vocabulary v1 are runtime-available. A package containing any other stage—required or optional—is visible with an unavailable reason but is not startable. An unsupported stage is never fabricated, auto-completed, or silently skipped.

Theory payloads contain 1–100 typed blocks: `paragraph` (`text`), `bullet_list` (`items`), `example` (`english`, optional `explanation`), and `callout` (`text`, optional `title`). HTML and Markdown are not interpreted. Visual Vocabulary contains typed items with `itemId`, `term`, `meaning`, `example`, and optional `imageAssetId`. Completing these stages records backend-built results only: `{schemaVersion:1, kind:"acknowledged"}` and, for Visual Vocabulary, `itemCount`. It never writes global Vocabulary.

The reserved v1 contracts for future executors are `listening.payload.segments[]` (`segmentId`, `transcript`, `audioAssetId`) and `repeat.payload.targets[]` (`targetId`, `text`, optional `audioAssetId`). The other reserved stage payloads are empty objects. Their presence does not imply runtime support.

Prompt-like package fields—including `systemPrompt`, `overridePrompt`, `developerPrompt`, and `rawPrompt`—are unknown and therefore rejected. Packages cannot change protected prompts or inject raw instructions.

## Assets and hashes

Assets declare `assetId`, `type` (`image` or `audio`), a portable relative `path` below `assets/`, and `sha256`. Allowed files are PNG, JPG/JPEG, WebP, and WAV. URLs, absolute paths, drive-qualified paths, backslashes, traversal components, executable/script/HTML extensions, symlinks, reparse escapes, missing files, and hash mismatches are rejected. Published assets require a valid 64-digit SHA-256. The registry canonicalizes package and asset paths and verifies that the target remains inside its package.

The deterministic package hash is SHA-256 over the validator's typed canonical package snapshot, including declared asset hashes. A started session stores that typed JSON and hash, never arbitrary source JSON, binary assets, answer keys, raw paths, or prompts.

## Session lifecycle

Only one Guided Lesson session may be `in_progress`. Starting another is rejected unless explicit Start Over atomically abandons the old session and creates a new row. The first stage starts active and subsequent stages pending. A backend transaction completes or skips only the current stage, activates the next stage, and advances the session together. Required stages cannot be skipped. Repeating the same completion action is idempotent. The last stage completes the session. Completed and abandoned sessions are immutable.

The session snapshot makes a lesson resumable after its source package changes or is deleted. The minimal student context snapshot stores profile schema version, placement attempt ID, estimated CEFR, placement confidence, target CEFR, and learning-goal identifiers. It stores no transcript, memory context, scores, personal facts, or model-generated text and never gates access.

Guided Lessons use their own tables, routes, and history. They do not create a normal `lesson`, start Ollama/Whisper/Piper, call the Lesson Analyzer, or write XP, streaks, achievements, weekly goals, placement, Student Profile, Learning Memory, global Vocabulary, Review, Pronunciation, or voice-performance data.
