# TOEIC Listening Part 3 — Implemented Contract

- One complete untimed Form A: 13 conversation sets, three visible questions per set, 39 A/B/C/D questions total.
- Questions, choices, and optional structured graphic are visible before playback. Transcript, answer key, evidence, and explanations are absent from the attempt DTO.
- Conversation playback is one-time before answers. Interrupted playback can restart; completed playback cannot replay until all three answers are locked.
- Each first answer is immutable. Q1/Q2 return only `Answer recorded`; full correctness and pedagogy appear only after Q3.
- Completed-set feedback unlocks replay, transcript, evidence, explanations, Continue, and Pause & Exit.
- Raw Part 3 result only; no 5–495 estimate. Runtime uses zero Qwen calls.
