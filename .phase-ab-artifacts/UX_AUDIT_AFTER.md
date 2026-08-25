# Phase AB — UX Audit After Automated Work

Date: 2026-08-25
Status: automated implementation complete; human visual/usability approval pending.

## Implemented

- Consolidated the product identity around white surfaces, very light blue backgrounds and blue primary actions.
- Removed navy/neon green from the primary design tokens and dashboard microphone artwork.
- Standardized cards, shadows, borders, buttons, form controls, progress, notices, dialogs, focus and toggles through shared CSS tokens.
- Added a compatibility layer for older feature-local dark utility classes so the major screens follow the same light system without modifying pedagogical behavior.
- Kept red, amber and green only as semantic feedback, with accompanying labels/icons and light-theme contrast adjustments.
- Raised shared button/icon targets to 44 px.
- Preserved visible keyboard focus, skip navigation, semantic progress values and reduced-motion behavior; added a higher-contrast media preference.
- Refined first-use onboarding to explain local operation, default audio behavior, local learning data, user-controlled backups, optional Placement, Course, Guided Lessons and free practice.
- Made Course the primary onboarding action without preventing Placement deferral or free navigation.
- Preserved the clear separation of Placement estimate, Course progress, target level, practice level and pronunciation match.

## Automated checks

- Targeted onboarding/Dashboard/Course tests: passed (11 tests).
- TypeScript typecheck: passed.
- Lint: passed.
- Frontend production build: passed.
- Full regression passed: frontend 154/154, Rust 214/214 automated tests, Voice Python 18/18 and Pronunciation Python 12/12; the native debug build also passed without invoking a bundler.

## Still requires a person

- Visual approval at the supported desktop sizes.
- Keyboard traversal in the real Tauri window.
- Windows Narrator smoke test.
- Physical microphone, speaker and Bluetooth behavior.
- Pronunciation calibration with real recordings.
- Six complete Guided Lessons (A1–C2), resume/recovery and content naturality.
- Guided Conversation and Interactive Analysis plausibility.

The Phase AB human gate is not approved by this document.
