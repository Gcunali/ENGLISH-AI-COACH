# Phase AB — UX Audit Before

Date: 2026-08-25
Scope: source-level audit before the Phase AB visual changes. This is not a human usability result.

## Main findings

- The primary identity was dark navy (`#080b11`) with neon green (`#a8ff60`), contrary to the required white-and-blue direction.
- Shared primitives already existed for page headers, cards, notices, buttons, dialogs, toggles, loading, empty and error states, but many feature-local utility classes still assumed dark surfaces.
- Keyboard focus, a skip link, semantic headings, labeled controls, progress ARIA values, reduced-motion handling and responsive navigation already existed.
- Course already separated Placement estimate, Course progress and target level, and exposed deterministic Continue Learning.
- Guided Lesson Runner already exposed lesson/stage title, stage sequence, progress, Continue/Skip, playback and recording actions, saved/resume state and explicit error recovery.
- Voice workspace already exposed Listening, Recording/Speaking, Thinking, Teacher Speaking, Stopping, Paused, Error and completion states, with Stop Response and End Lesson separated.
- Pronunciation copy correctly described a known-target pronunciation practice score and did not label it as CEFR, accent quality or an English score.
- Interactive Analysis kept Exercise, acoustic Pronunciation and Conversation dimensions separate.
- First-use onboarding was optional and non-blocking, but emphasized free conversation rather than the structured Course path and did not explain the complete local-data/backup model.
- The installed browser's headless screenshot mode did not emit an image in this environment. No screenshot or visual approval is claimed.

## Screen inventory

| Area | Source audit before AB | Main issue or existing protection |
|---|---|---|
| Welcome / Onboarding | Needs refinement | Non-blocking, but Course path and privacy explanation were incomplete. |
| Dashboard | Functional | Dark visual identity and dark microphone artwork. |
| Course / Level / Unit | Functional | Clear A1–C2 hierarchy, progress and recommendation semantics already present. |
| Guided Lesson Runner | Functional | Stage and action model already explicit; dark feature-local surfaces remained. |
| Theory / Visual Vocabulary | Functional | Shared runner hierarchy; dark cards. |
| Listening / Repeat / Speaking Check | Functional | Playback/recording states and disabled-state guards present. Physical usability pending. |
| Exercise | Functional | Labeled inputs and deterministic feedback; dark controls. |
| Guided Conversation | Functional | Turn roles, pending response and controls present. Human naturality pending. |
| Analysis | Functional | Metrics remain separate and scoped. Human plausibility pending. |
| Free Conversation | Functional | Explicit voice states and cancellation. Physical voice validation pending. |
| Placement | Functional | Optional and separately labeled as estimate. |
| Profile | Functional | Placement, target and practice preferences are distinct. |
| Vocabulary / Review | Functional | Empty/error states and local provenance exist. |
| Pronunciation | Functional | Known-target copy is correct. Human calibration pending. |
| Progress / Achievements / History | Functional | Derived data and accessible progress elements exist. |
| Settings / Diagnostics | Functional | Consolidated sections, local diagnostics and privacy-safe report. |
| Backup / Restore | Functional | Confirmation, safety backup and active-runtime protection exist. Human flow pending. |

## Required actions

1. Replace the dark/neon design tokens with a white-and-blue system.
2. Normalize legacy dark utility classes through shared compatibility rules without changing feature logic.
3. Refine onboarding around optional Placement, Course, Guided Lesson and local privacy.
4. Preserve semantic colors only for status/feedback and keep text/icon labels.
5. Increase primary control targets to at least 44 px and retain reduced-motion/high-contrast behavior.
6. Run automated regression, then stop for physical voice, Bluetooth, visual, content and accessibility validation.
