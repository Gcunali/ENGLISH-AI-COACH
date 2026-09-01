# TOEIC Listening Part 1 - Runtime Specification

## Session lifecycle

`start -> in_progress -> completed | abandoned`

A form snapshot fixes `formId`, `formVersion`, six item IDs/versions, and order. SQLite is updated after every answer and explicit advance. An answered question remains current until Continue, so feedback state is recoverable. There is no session timeout or countdown.

## Question lifecycle

1. Backend returns photograph plus A-D labels only.
2. User starts one initial Piper presentation of all four statements.
3. A completed presentation enables answer buttons. No pre-answer replay or transcript is available.
4. Backend validates current session/item, grades against the private bank, and atomically inserts the first answer.
5. Correct answers show concise feedback; incorrect answers immediately show correct answer, both statements, authored rationales, language focus, and vocabulary.
6. Transcript and unlimited pedagogical replay are available only after the answer exists.
7. Continue advances or completes the sixth question.

If startup finds a `started` presentation, it marks it `interrupted`. Resume permits a fresh full presentation with no penalty and creates no answer.

## Results and review

Completed forms expose Part 1 raw performance and deterministic breakdowns. Review Mistakes filters incorrect items; Review All includes all six. Both include photo, full transcript, selected/correct choices, explanation, language focus, and tags. TOEIC history is queried from dedicated TOEIC tables only.

## Accessibility and privacy

A-D are native buttons with keyboard operation, visible focus, and screen-reader labels. Correct/incorrect states include icons and text rather than color alone. No microphone is used. Audio and images remain local; there are zero Qwen or network calls.
