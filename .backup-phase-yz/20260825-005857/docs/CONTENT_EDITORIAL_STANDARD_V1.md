# Content Editorial Standard v1

This standard governs every production A1/A2 Guided Lesson in `english-core`. Its central question is: **What should the learner be able to do after this lesson?** Every stage must help answer that question through observable communication.

## Voice, originality, and variant

- Write original en-US content. Do not imitate or copy commercial courses.
- Prefer useful everyday language, respectful neutral situations, inclusive names, and culturally portable examples.
- A1 uses concrete vocabulary, short clauses, one main pattern at a time, and direct instructions.
- A2 uses longer connected turns, a wider lexical set, controlled tense combinations, reasons/details, and realistic problem solving.
- A mission (Lesson 6) reviews and integrates its Unit. It introduces no important new grammar.
- Recycle prior language naturally. Do not make stages or Lessons isolated islands.

## Required eight-stage learning arc

Every package is required, version 1, en-US, and follows exactly:

1. **Theory** builds meaning and form with explanation, pattern, examples, a common-error note, and recap. Target 150–320 words at A1 and 200–420 at A2.
2. **Visual Vocabulary** presents 6–10 central A1 items or 7–12 A2 items. Each has a concise meaning and contextual example. Images remain optional; no downloaded assets.
3. **Listening** uses 3–5 A1 or 4–6 A2 natural segments and only previously prepared language. Text may be voiced by the existing runtime Piper.
4. **Repeat** uses 4–6 A1 or 5–7 A2 communicative chunks, not disconnected word drills.
5. **Speaking Check** uses 3–5 A1 or 4–6 A2 targets already heard/read. It introduces no grammar.
6. **Exercise** uses 8–10 A1 or 9–12 A2 valid deterministic items and normally at least four existing exercise types. Exact short answer is reserved for objectively finite answers.
7. **Guided Conversation** places the learner in the Unit situation. A1 turn range is 4/6/8; A2 is 5/7/10. Targets are grounded in earlier stages.
8. **Analysis** is the existing empty Analysis v1 payload; no Lesson-specific scoring.

## Coherence checklist

- Choose a communicative goal, a small grammar focus, 6–12 central lexical items, and 3–6 target expressions before writing.
- Reuse the vocabulary in examples, listening, pronunciation targets, exercises, and conversation.
- At least two listening chunks reappear in Repeat/Speaking; conversation targets reappear earlier.
- Distractors must be plausible but unambiguously wrong in context.
- Feedback explains the relevant pattern without exposing private answers in public DTOs.
- Avoid excessive verbatim reuse across Lessons. Recycle language inside new contexts and purposes.
- Keep Theory practical rather than textbook-like. No metalanguage beyond what a beginner needs.

## Progression

- A1 moves from identity and `be` through concrete time/family/routine/place/food/ability/shopping functions.
- A2 moves into narrated past, experiences, future, comparison, travel problems, obligations/advice, health/conditionals, opinions/relative clauses, and controlled tense integration.
- A2 must be observably more complex: longer listening/conversation, more linked clauses, 9–12 exercises, a larger lexical set, and choices requiring context.
- Do not teach B1+ as a central objective.

## Identity and publication

- Stable ID format: `a{level}-uNN-lNN-topic-slug`, for example `a1-u01-l01-hello-goodbye`.
- Every initial package uses `contentVersion: 1`.
- Author as `draft`. Publish only after parser, editorial, reference, startability, privacy, duplication, progression, and regression gates pass.
- Curriculum refs contain only exact `lessonId` and `contentVersion`.

## Editorial acceptance

A Lesson passes only if quantities are in band, all eight required stages appear in exact order, exercises are deterministic, conversation is grounded, text is original/en-US, package parsing succeeds, the exact Curriculum ref resolves, startability succeeds, and no answer key appears in a public DTO. Human audio, Bluetooth, pronunciation, guided-conversation, and pedagogical review remain explicitly pending until performed by a person.
