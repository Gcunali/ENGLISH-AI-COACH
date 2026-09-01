# TOEIC Reading Part 5 Specification

- Runtime type: `part5_incomplete_sentence`.
- Published pilot: Form A, exactly 30 frozen/versioned questions.
- One sentence, exactly one `_____` blank, and four visible A–D choices.
- First answer is persisted once and cannot be changed.
- Learning mode returns authored feedback immediately; simulation mode locks and advances without feedback.
- Sessions are untimed and resume at the exact current question.
- Runtime performs zero Qwen calls and loads no speech/audio model.
- Results are raw Part 5 performance only; no Reading scaled score exists.
