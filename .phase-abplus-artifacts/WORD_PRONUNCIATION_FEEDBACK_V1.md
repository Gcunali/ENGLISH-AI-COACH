# WORD PRONUNCIATION FEEDBACK V1

Version constant: `WORD_PRONUNCIATION_FEEDBACK_VERSION = 1`.

## Method

The presentation layer consumes the already persisted word results produced by the real pronunciation pipeline: Wav2Vec2 logits, phonemization, CTC phone alignment, and acoustic phone-to-word mapping. Whisper transcripts and Qwen are never used to calculate word feedback.

Specific feedback is available only for a completed attempt with non-low confidence, alignment coverage of at least 0.80, and at least one aligned word. Otherwise the UI says that word-level feedback is unavailable for that attempt.

Reliable words are labeled:

- `Strong`: score at least 85;
- `Good`: score from 70 through 84;
- `Needs attention`: below 70.

At most the three lowest words below 70 become focus words. Words are never called correct, wrong, failed, or mastered.

## Protected score

Pronunciation Score Version 1 is unchanged. No database migration, historical recomputation, new overall score, pass/fail threshold, or CEFR mutation was introduced.
