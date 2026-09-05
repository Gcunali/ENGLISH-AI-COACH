# TOEIC Phase 9 Test Report

Date: 2026-09-02

## Final results

- Rust full suite: **280 passed, 0 failed, 27 explicitly ignored manual tests**.
- Phase 9 Rust tests: **5 passed** (bounds, sample gate, target invariance, frozen daily/smart snapshots, scored-only exposure).
- Frontend: **170 passed in 43 files, 0 failed**.
- TypeScript: PASS.
- Lint: PASS; warnings are pre-existing hook warnings, including immutable backup copies.
- Production frontend build: PASS.
- Rust format check, offline check and native debug build: PASS.
- Voice regression in Piper environment: **18 passed**.
- Pronunciation regression: **13 passed**.
- Course regression: all 288 published Guided Lessons validated/startable in the Rust suite.
- Physical DB: schema 24, integrity `ok`, foreign-key violations 0.

The first generic Python invocation lacked NumPy and was an environment-selection error, not a product failure. Re-running with the app's owned Piper environment produced 18/18 PASS.
