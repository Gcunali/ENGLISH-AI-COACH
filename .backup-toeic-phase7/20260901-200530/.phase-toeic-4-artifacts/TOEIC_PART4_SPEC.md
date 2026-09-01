# TOEIC Listening Part 4 — Runtime Specification

- Bank: `toeic-listening-part4-v1`
- Form A: 10 talks, exactly 3 questions per talk, 30 questions total.
- Each talk has one en-US speaker and one uninterrupted script.
- Questions and choices are visible before playback; scripts and answer keys are absent from public session DTOs.
- Initial playback is one-time. Interrupted playback may restart; completed playback may not.
- First answers are immutable. Feedback is released only after all three answers in learning mode.
- Three sets contain structured visual information.
- Practice is untimed, deterministic, offline, and isolated from CEFR, course, XP, streak, and learning memory.
