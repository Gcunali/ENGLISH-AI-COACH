# TOEIC Listening Part 2 — Implemented Contract

- One complete untimed form with exactly 25 Question–Response items.
- Every item has one spoken prompt, three spoken responses (A/B/C), one immutable first answer, and deterministic grading.
- Prompt and response transcripts, answer key, and explanations are absent from the pre-answer public DTO.
- Initial audio must complete before answering and can play only once; an interrupted presentation restarts without penalty. Replay is available after answering.
- Sessions, presentation state, answers, completion, results, and history persist in the existing generic TOEIC SQLite schema.
- Results are raw X/25 only. No official or estimated scaled TOEIC score is produced.
- Course, CEFR, XP, streak, achievements, review queue, vocabulary, and teacher memory are not mutated.

The existing Phase 1 schema was sufficiently generic. No migration was created.
