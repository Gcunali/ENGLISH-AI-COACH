# HUMAN SMOKE TEST

Date: 2026-08-30

## Automated physical checks completed

- PASS — debug app started against the real human database and applied migration 020.
- PASS — database integrity is `ok`, with zero foreign-key violations and unchanged protected record counts.
- PASS — real Whisper model/binary completed repeated legacy and persistent transcriptions with equivalent text.
- PASS — persistent Whisper shutdown left no orphan `whisper-server` process.
- PASS — real Piper synthesis and cached replay were measured.
- PASS — the standalone Tauri debug executable was rebuilt.

## Human-operated checks still required

The coding environment cannot speak into the user's microphone, listen to the speakers, pair a Bluetooth device, or make a trustworthy subjective audio judgment. The following are therefore `PENDING USER`, not falsely marked as passed:

- Daily Practice: complete a real mixed session and confirm the completion screen/20 XP occurs once.
- Dictation: play, type, submit, inspect diff, and replay.
- Shadowing: confirm Record is locked before Listen; record, inspect acoustic/word feedback, and retry.
- Word Pronunciation: record a known phrase and confirm reliable focus words or the low-confidence unavailable state.
- Mistake Repair: current data has zero confirmed recurring mistakes, so verify the honest empty state; perform the full flow only after a real mistake reaches two lessons.
- Speaking Recall: record before reveal, inspect the transcript, reveal/hear the model, and retry.
- Voice regression: run a normal multi-turn conversation and end/cancel it.
- Bluetooth regression: verify the protected 500 ms wake silence prevents clipping on the actual device.
- Combined memory: observe a representative Voice session and a Pronunciation/Shadowing session on the 16 GB machine.

Until these checks are signed off, Phase AB+ is technically implemented but not human-approved, `FEATURE FREEZE FOR 1.0` is not declared, and Phase AC must not start.
