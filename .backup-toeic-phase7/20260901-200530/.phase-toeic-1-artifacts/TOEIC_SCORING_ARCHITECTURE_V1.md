# TOEIC Scoring Architecture v1

Phase 1 scoring is deliberately limited to deterministic raw Part 1 performance.

- Each question contributes exactly 0 or 1 point.
- The first committed A-D answer is immutable through a database uniqueness constraint and backend rejection.
- Reading feedback, replaying audio, reviewing transcripts, or reopening a session never changes the stored answer.
- Results show `X / 6`, accuracy, skill evidence, difficulty evidence, and selected distractor types.
- Part 1 never produces a Listening 5-495 score or Total 10-990 score.

The typed but unused `ToeicScoreProfile` contract reserves: profile ID, version, section, form family, calibration method, calibrated conversion table, and confidence metadata. Phase 1 ships no profile and no invented mapping. A future estimate requires all 100 Listening or all 100 Reading questions and must be labeled unofficial/estimated.

TOEIC results do not update Placement, CEFR, Course progress, Guided Lessons, Learning Memory, Vocabulary, Review, XP, streak, or achievements.
