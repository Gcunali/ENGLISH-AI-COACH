# MISTAKE REPAIR V1

## Eligibility and source of truth

Only a recurring mistake confirmed across at least two distinct lessons and still `active` or `improving` is eligible. The most recent real occurrence supplies `What I said`; the persisted corrected sentence supplies the better version; the existing explanation and occurrence source are retained. Legacy and Guided Lesson occurrence tables are merged read-only for selection.

## Flow

V1 implements Recognize, Rebuild, and Say It. It shows the original and persisted correction, asks the learner to rebuild the complete phrase, then uses the corrected sentence as the known acoustic target. It does not ask Qwen to generate an answer key or invent a fill blank. Because V1 does not generate a reliable fill/word-order transformation, those optional steps are omitted.

Completing the exercise records practice provenance only. It never deletes the mistake, marks it resolved/mastered, changes its status, or mutates Review history.

## Current human data

The current database has nine recurring-mistake rows, all with `lesson_count = 1`. The correct production behavior today is therefore an honest empty state, not fabricated content.
