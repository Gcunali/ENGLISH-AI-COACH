# Guided Gamification Integration V1

- Rule version: `GUIDED_XP_RULE_VERSION = 1`.
- Award: exactly 60 XP per officially completed Guided session.
- Idempotency key: `(session_id, rule_version)`.
- A new completed repeat session earns another 60 XP.
- Scores, Pronunciation, Exercise accuracy, Analysis, Placement, and CEFR never change this award.
- A completed Guided session contributes its local completion day to the existing streak.
- Weekly Goal uses only foreground heartbeat events of 1–30 seconds. The UI records 15-second heartbeats only while the lesson is in progress, the document is visible, and the window has focus.
- No `completed_at - started_at` duration and no retroactive duration are used.
- Standard and Guided practice events are combined for XP level, minutes, streak, and time achievements while preserving Standard-only conversation-lesson achievements.

Guided/Course achievements are deterministic: first/10/50 Guided sessions, first completed six-Lesson unit, each 48-Lesson Course level, and all 288 unique Course Lesson IDs. These are Course milestones, not proficiency or CEFR certification.

