# A1 Unit 1 Pilot Review — Meeting People

Reviewed on 2026-08-25 against Content Editorial Standard v1.

## Scope

Six original en-US draft packages were reviewed:

1. `a1-u01-l01-hello-goodbye`
2. `a1-u01-l02-whats-your-name`
3. `a1-u01-l03-countries-nationalities`
4. `a1-u01-l04-personal-information`
5. `a1-u01-l05-i-am-you-are-he-is`
6. `a1-u01-l06-introductions-mission`

The draft `english-core` manifest contains A1 Unit 1 only and pins all six to contentVersion 1.

## Automated gate

- Official Guided package parser: 6 valid, 0 invalid.
- Publication during authoring: all 6 `draft`; draft Curriculum hidden from human UI.
- Required stage order: exact eight stages in all 6; all `required=true`.
- Theory: 226–245 words, within the A1 target 150–320.
- Visual Vocabulary: 8–9 items per Lesson, within 6–10.
- Listening: 3 segments per Lesson, within 3–5.
- Repeat: 5 targets per Lesson, within 4–6.
- Speaking Check: 4 targets per Lesson, within 3–5.
- Exercises: 8 per Lesson, using six existing deterministic types.
- Guided Conversation: 4/6/8 student turns in every Lesson.
- Analysis: exact empty Analysis v1 payload.
- Exact Curriculum references: 6/6 resolved, 0 broken.
- Startability: all 6 started at Theory through the official engine in isolated migrated TEMP SQLite databases.
- Privacy: official public summary/session serialization contains none of `correctOptionId`, `correctOptionIds`, `acceptedAnswers`, `correctOrder`, or `correctPairs`.

Rust gate result: 3 focused manual pilot tests passed, zero failed.

## Editorial review

- Each Lesson answers a distinct communicative goal and reuses central vocabulary/expressions through listening, pronunciation, deterministic practice, and conversation.
- Grammar progresses from fixed greetings to name questions, country/nationality, personal WH questions, explicit `be` agreement, then integrated mission review.
- Lesson 6 adds no important grammar and functions as review plus first-meeting mission.
- No external image/audio asset exists; vocabulary remains usable without forced imagery and listening uses existing runtime audio semantics.
- No copied commercial-course content, B1+ focus, AI recommendation, score customization, or downstream learning integration was introduced.

## Correction made before approval

The first draft of the reusable vocabulary fill-blank item attempted a case-sensitive replacement inside an example. Some examples could display no visible blank. The authoring template was changed to ask for the exact vocabulary item from its meaning, then all six packages were regenerated and the three official gates rerun successfully. No engine change was needed.

## Pilot decision

**APPROVED.** The standard is fit for continued A1 production. The remaining A1 Units may now be authored, while all packages and the Curriculum remain draft until the complete 96-Lesson gate.

## Human validations

PENDING: full human A1 lesson, pedagogical review, Bluetooth, human Pronunciation, and human Guided Conversation. No result was fabricated.
