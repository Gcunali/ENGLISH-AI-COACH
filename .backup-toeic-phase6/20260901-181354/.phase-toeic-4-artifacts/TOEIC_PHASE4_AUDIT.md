# TOEIC Phase 4 — Pre-implementation Audit

Date: 2026-08-31

- Parts 1–3 are physically implemented, bundled, startable, resumable, deterministically graded, and covered by the full Rust/frontend regressions. Their final reports were read.
- Current Listening coverage is structurally 70/100: Part 1 = 6, Part 2 = 25, Part 3 = 39.
- Human database is schema 21, integrity previously validated, and unchanged by Phase 3. Pre-change SHA-256: `BFD6DD212BE1938F87AAEF479453D10FE9A32E3E439B0C878B969B12B20F122B`.
- Backup: `.backup-toeic-phase4/20260831-225224`, including source, migrations, TOEIC banks/reports, configuration, human DB, and SHA-256 manifest.
- Part 3 grouped architecture can be specialized for Part 4's one-speaker talks without changing existing TOEIC answer or presentation tables.
- Full Listening requires durable parent/part composition, simulation mode, frozen score profile/result, and section transitions. Existing schema 21 cannot represent these without overloading a single-part session, so migration 022 is necessary.
- Full Listening will own four hidden child `toeic_session` records and expose only sanitized wrapper DTOs during Simulation Mode. Child attempts will be excluded from ordinary part overviews/history to prevent feedback bypass.
- Installed Piper voices: `en_US-amy-medium`, `en_US-lessac-medium`; both en-US. Static cache has 12 entries. No downloads are required or authorized.
- Part 4 strategy: alternate Amy/Lessac between talks; one consistent voice inside each monologue; no simulated accents.
- Score strategy: versioned, monotonic, banded Practice Calibration Profile v1 with a central 5-point estimate and uncertainty range. It will be persisted only after 100 first answers; no linear `raw × 4.95`, official claim, CEFR mutation, or Course mutation.
- Protected regressions: Parts 1–3, English Core v3, all 288 Guided Lesson packages, Voice, and Pronunciation.
