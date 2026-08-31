# Phase AB — Human Validation Protocol

Status: ready for execution by a real person. Record evidence in the companion result files. Do not mark a row approved without performing it.

## Preparation

1. Use the real Tauri app, not a mocked browser page.
2. Record Windows version, app build, audio devices, Bluetooth device/model and room conditions.
3. Keep normal app data; create a user-controlled backup first if desired.
4. For each issue record exact screen, action, expected result, actual result, severity and whether it reproduces.

## A. Voice streaming

- Start a new free conversation with microphone and speakers.
- Confirm the first spoken response starts cleanly.
- Hold a 20–30 minute conversation including short and long answers.
- During Teacher Thinking and Teacher Speaking, use Stop Response.
- Navigate away during voice and confirm no ghost audio or duplicate response.
- Cancel and restart once; note perceived STT, first-text and first-audio latency.

Pass: one response per turn, explicit states, reliable stop/cancel, no ghost audio.

## B. Bluetooth

- Repeat the first-response and Stop Response checks with a Bluetooth output device.
- Confirm the first chunk is not cut and the 0.5 s start silence is acceptable.
- Disconnect/reconnect once and switch output device when practical.

Pass: intelligible first chunk, controllable playback and stable recovery. Do not request timing changes from one isolated attempt.

## C. Pronunciation calibration

For every phrase below, make one normal attempt. Repeat at least four phrases, and intentionally make two attempts better and two worse.

1. `Please close the blue door.`
2. `Three thin threads.`
3. `I really enjoy reading.`
4. `Would you like a glass of water?`
5. `The weather was warm and windy.`
6. `She sells fresh vegetables at the market.`
7. `Could you explain the problem again?`
8. `I have been working here for three years.`
9. `We should have taken an earlier train.`
10. `Environmental policy requires careful international cooperation.`
11. `The research findings were unexpectedly controversial.`
12. `Despite the uncertainty, they reached a reasonable compromise.`

Also test one deliberate content mismatch, poor-quality audio, cancellation and a new attempt after cancellation.

Pass: repeated normal attempts are reasonably stable, clearly better/worse speech orders qualitatively, mismatch is rejected, and the worker recovers. Scores are pronunciation match for the known target only.

## D–I. One complete Guided Lesson per CEFR level

Complete one representative Lesson at A1, A2, B1, B2, C1 and C2. Across the six Lessons verify:

- Theory fits the level and instructions are understandable.
- Visual Vocabulary is natural and relevant.
- Listening audio/text and first playback are usable.
- Repeat is useful and reference playback precedes recording where required.
- Speaking Check makes the target and recording state clear.
- Exercises are coherent and deterministic.
- Guided Conversation stays in role/scenario, asks natural questions, encourages target vocabulary, gives useful grounded corrections and neither invents nor overproduces corrections.
- Interactive Analysis reports what was practiced, plausible grounded strengths/focus areas and separate Exercise, acoustic Pronunciation and Conversation results.
- The learner always knows the next action.

Record Lesson ID, level, start/end time and result for each.

## H. Resume and recovery (during one of the six Lessons)

1. Pause at a non-conversation stage, close normally, reopen and resume.
2. Repeat around Guided Conversation with a pending teacher response if practical.
3. Verify stage, selected Exercise attempt and selected Pronunciation attempt are preserved.
4. Confirm no duplicate opening message; retry a pending teacher response.

Pass: saved state resumes once without duplicate work or lost selections.

## J. Course and visual UX

At 1366×768, 1920×1080, a typical laptop size and the smallest supported desktop window:

- Confirm A1, A2, B1, B2, C1 and C2 are clear.
- Confirm Course progress, completed Lessons, current Unit, Continue Learning, suggested starting point and objectives are understandable.
- Confirm Placement level, Course progress, target level and practice level are not confused.
- Visit every major navigation destination and check horizontal overflow, clipped buttons, modal clipping, empty/error/loading states and readable cards.
- Approve or reject the white-and-blue visual direction.

## K. Additional content spot-check

In addition to the six full Lessons, inspect an extra sample across the 288 packages, prioritizing Unit Missions, level-final Missions and B1–C2. Record artificial language, excessive repetition, mismatched difficulty or unnatural scenarios. Do not edit packages during review.

## L. Keyboard and Windows Narrator

- Keyboard: use Tab/Shift+Tab/Enter/Space/Escape without a mouse on Welcome, Dashboard, Course, one Guided stage, Placement, Pronunciation, Settings and Backup/Restore.
- Confirm focus is always visible and order is logical.
- Narrator: confirm useful page headings, link/button names, switch state, progress values, dialog title/description, recording/playback status and live errors.

Pass: no blocker prevents understanding or completing the primary flow.

## Reporting

Fill these files:

- `HUMAN_VALIDATION_RESULTS.md`
- `PRONUNCIATION_CALIBRATION_REPORT.md`
- `VOICE_BLUETOOTH_VALIDATION.md`
- `GUIDED_CONVERSATION_VALIDATION.md`
- `CONTENT_HUMAN_REVIEW.md`
- `ACCESSIBILITY_REPORT.md`

Attach screenshots or short recordings only when the tester intentionally chooses to and they contain no unwanted personal information.
