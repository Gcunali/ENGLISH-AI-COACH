# Part 1 Test Report

Date: 2026-08-31

## TOEIC-specific automation

Passed: Item Bank parse, production count, runtime availability, duplicate statement rejection, unsafe path rejection, incomplete form rejection, public DTO answer-key security, first-answer immutability, wrong-answer persistence, audio interruption recovery, hours-later resume, app-restart/repository reopen, 0/1/3/6 result math, absence of scaled score, complete form, mistakes/all review, and separate history.

Frontend TOEIC tests passed: available/unavailable parts, untimed/legal copy, statement transcript hidden before answer, deterministic submission, authored incorrect feedback, and locked-answer copy.

## Full regressions

- `cargo test --offline`: 230 passed, 0 failed, 27 ignored manual tests.
- `npm test`: 320 passed across 81 files.
- `npm run typecheck`: passed.
- `npm run lint`: passed after hook dependency cleanup.
- `cargo fmt --all -- --check`: passed.
- `cargo check --offline`: passed (pre-existing dead-code warnings only).
- `npm run build`: passed; Vite reported only the existing large-chunk advisory.
- `tauri build --debug --no-bundle`: passed; debug executable produced, no installer.
- Voice/pronunciation regressions are included in the full Rust/frontend suites and passed.

## Physical database and Course safety

The real desktop app applied migration 021. After startup: schema 21, `integrity_check=ok`, zero foreign-key violations; existing counts stayed at 14 lessons and 2 interactive sessions; TOEIC tables began empty. All 288 Course `lesson.json` hashes exactly match the pre-change manifest (0 differences).
