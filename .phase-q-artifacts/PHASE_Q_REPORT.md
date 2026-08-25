# Fase Q - Product Finalization, UX, Accessibility and Settings Consolidation

Date: 2026-08-23  
Official project: `C:\ENGLISH AI COACH`  
Final status: **implemented and validated; no next phase started**.

## A-G. Files, backup and database architecture

### A. Files created

Product source:

- `src/components/ConfirmationDialog.tsx`
- `src/components/ConfirmationDialog.test.tsx`
- `src/components/ProductUI.tsx`
- `src/components/ProductUI.test.tsx`
- `src/components/RouteErrorBoundary.tsx`
- `src/components/RouteErrorBoundary.test.tsx`
- `src/components/WelcomePanel.tsx`
- `src/components/WelcomePanel.test.tsx`
- `src/pages/NotFoundPage.tsx`
- `src-tauri/src/ux_preferences.rs`

Audit/evidence:

- `.phase-q-artifacts/UX_AUDIT_BEFORE.md`
- `.phase-q-artifacts/UX_AUDIT_AFTER.md`
- `.phase-q-artifacts/PHASE_Q_REPORT.md`
- DPI-aware `after-*.png` screenshots listed in FH.

### B. Files modified

- App/shell: `src/App.tsx`, `src/App.test.tsx`, `src/components/AppLayout.tsx`, `src/styles.css`.
- Shared UI: `src/components/DataBackupSection.tsx` and test, `LessonAnalysisReport.tsx`, `PageState.tsx`, `PracticeConsistency.tsx`.
- Pages: Achievements (+ test), Dashboard (+ test), Diagnostics (+ test), History, Lesson Details, New Lesson (+ test), Placement (+ test), Placement Result, Profile, Progress, Pronunciation, Review (+ test), Review Session (+ test), Settings (+ test), Vocabulary (+ test), Vocabulary Details.
- Bridge/types: `src/services/native.ts`, `src/types/index.ts`.
- Tauri registration only: `src-tauri/src/lib.rs`.

No Python file, prompt, scoring engine, queue engine, migration, package manifest, lockfile or Tauri configuration was modified.

### C. Backup pre-Fase Q

Official valid backup: `C:\ENGLISH AI COACH\.backup-phase-q\20260823-221601`.

It contains 83 protected source files plus the manifest (84 files total). An earlier directory, `20260823-221536`, contains an invalid one-file wildcard attempt; it was detected before product edits, excluded from evidence and retained only for traceability.

### D. Manifest SHA-256

- Manifest: `.backup-phase-q/20260823-221601/manifest-sha256.txt`
- Entries: 83
- Manifest SHA-256: `1E0E35580D512691AE3CA812F26AA6CFF5F3F6EC0BADC0AC9C269837A0920C9C`

### E. Database backup

No Phase Q database backup was required: there was no migration or destructive data transform. Physical validation changed only the existing key/value Settings table: it added the UX-only `phase_q_welcome_seen=true`. Existing Backup/Restore facilities and their safety backups were not invoked.

### F. Migration 014

Migration 014 was **not** created.

### G. Migration justification

The existing `settings(key, value_json, updated_at)` table is the correct persistence boundary for one UX preference. Adding a schema/table/column would create needless migration risk. New/empty and existing-user behavior is tested with isolated temporary databases.

## H-O. Audit and priorities

### H. UX Audit Before path

`.phase-q-artifacts/UX_AUDIT_BEFORE.md`

### I. UX Audit After path

`.phase-q-artifacts/UX_AUDIT_AFTER.md`

### J. Routes audited

`/`, `/dashboard`, `/lesson/new`, `/history`, `/history/:lessonId`, `/progress`, `/vocabulary`, `/vocabulary/:vocabularyId`, `/review`, `/review/session/:sessionId`, `/placement`, `/placement/results/:attemptId`, `/profile`, `/achievements`, `/pronunciation`, `/settings`, `/diagnostics` and unknown `*`.

### K. Findings

- P0: 2
- P1: 14
- P2: 8
- P3: 5

### L. P0 corrected

1. Defined the missing Pronunciation primary/secondary button and eyebrow patterns through the shared system.
2. Added a route-level React Error Boundary so one render failure does not blank the shell.

### M. P1 corrected

All 14: shared page primitives; unified buttons/states; accessible confirmation dialog; final-only streaming announcement; skip link; friendly 404; actionable first use; persisted onboarding with existing-user bypass; actionable empty states; semantic status colors; complete focus rules; grouped/scrollable navigation; long-content protection; consistent busy feedback.

### N. P2 corrected

Six materially corrected: page width hierarchy, semantic radii/padding, typography hierarchy, date formatting duplication in Pronunciation, repeated cards/headers, and visible Hear-target explanation. Two were reduced but retained: remaining compressed legacy files and no global toast queue.

### O. Deliberately deferred

Human Narrator certification, automated visual-regression dependency, full mobile product layout, localization/theme switching and low-value scaffold/dead-file removal.

## P-AI. Design system and shared states

### P. Tokens before

`--bg`, `--panel`, `--panel-solid`, `--border`, `--muted`, `--text`, `--accent`, `--accent-soft`, `--danger`, `--warning`; many local raw colors/radii remained.

### Q. Final tokens

The previous tokens plus `--success`, `--info`, `--focus`, `--surface-subtle`, `--surface-raised`, `--radius`, `--radius-sm`, `--radius-lg`, and `--page-gap`.

### R. Typography hierarchy

Shared eyebrow -> page title -> page description -> section title/description -> metric label/value/detail. Page titles use responsive `clamp`, controlled line height and safe wrapping.

### S. Spacing strategy

PageShell owns width; PageHeader owns top hierarchy; sections use repeatable 1-1.75rem gaps; compact states/actions stack under 640 px; individual pages no longer need bespoke top margins.

### T. Surface/card strategy

`glass` is retained as the branded surface. State, metric, notice, dialog and settings surfaces now use named patterns and semantic radii instead of route-specific visual inventions.

### U. Status colors

Success green, warning amber, error red, info blue and neutral muted; status is always paired with text and/or icon, never conveyed by color alone.

### V. Focus strategy

Two-pixel high-contrast `--focus` outline with three-pixel offset for buttons, links, inputs, selects, textareas, summaries, switches and explicit tab stops. The first Tab exposes the skip link.

### W. Reduced motion

Pulse/audio bars only animate under `no-preference`; under `reduce`, smooth scrolling is disabled and animations/transitions collapse to 0.01 ms/one iteration.

### X. PageShell

Three widths: narrow 800 px, standard 1120 px, wide 1440 px, always `width:100%`, centered and shrink-safe.

### Y. PageHeader

Supports eyebrow, title, description, back control and responsive actions with one heading hierarchy.

### Z. Shared components created

PageShell, PageHeader, SectionHeader, InlineNotice, StatusBadge, MetricCard, ToggleRow, ConfirmationDialog, RouteErrorBoundary and WelcomePanel.

### AA. Shared components reused

Existing PageState components, PracticeConsistency, DataBackupSection, LessonAnalysisReport, PlacementSummaryCard and ReviewOverviewCard were integrated with the new semantics rather than rewritten.

### AB. Duplication removed

Repeated page headers, notices, metrics, toggles, destructive confirmations, button variants and state action layouts were centralized.

### AC. Button system

Primary, secondary, ghost, danger and icon variants; common minimum height, radius, padding, disabled opacity/cursor, focus and hover behavior.

### AD. Form system

Native label/fieldset/radio/select/textarea behavior is preserved. Switches expose `role=switch`, `aria-checked`, visible label/description and guarded autosave.

### AE. Error state

Accessible alert card with safe message and optional Retry action. Route Error Boundary provides recovery to Home/retry without a blank app.

### AF. Empty state

Explains why content is absent and can render a primary/secondary next action.

### AG. Loading strategy

Compact `role=status`, polite live region and `aria-busy=true`; mutation controls show busy labels and stay disabled while pending.

### AH. Confirmation dialog

Named modal, safe initial Cancel focus, Tab/Shift+Tab trap, Escape, backdrop behavior, busy protection and return to the originating element. Used by Restore, Review, Review Session and Placement destructive decisions.

### AI. Toast strategy

No new toast dependency. Persistent page feedback uses InlineNotice/live regions; the existing achievement-unlocked message remains an ephemeral `role=status`. This is adequate for current mutation volume.

## AJ-BJ. Shell, navigation and route UX

### AJ. App shell

Skip link, route titles, route error boundary, friendly 404, shrink-safe main content and grouped navigation were added.

### AK. Sidebar

Independent vertical scroll, persistent privacy card, readable active item, protected content width.

### AL. Navigation grouping

Practice; Learning; Assessment & Profile; System.

### AM. Active route

`aria-current=page`, contrasting surface and a left accent indicator. Nested routes retain the correct parent context.

### AN. Route scroll

Main document scroll resets on pathname changes; sidebar scroll remains independent instead of jumping.

### AO. Horizontal overflow

Main flex children use `min-width:0`; cards and text wrap; diagnostic grids collapse; compact navigation owns its intentional horizontal scrolling and hides the bulky Windows scrollbar.

### AP. Breakpoints

1180 px collapses content sidebars, 760 px replaces the desktop sidebar with compact navigation, 640 px stacks actions and welcome layout.

### AQ. Practical minimum viewport

Configured and physically validated minimum: 720 x 640 logical pixels. At 150% Windows scaling this is approximately 1080 x 960 physical pixels.

### AR. Windows scaling

Validated on the physical machine at 150%. Full HD, 1366 physical pixels wide and minimum configured width all remained usable. The initial capture issue was caused by the screenshot process lacking DPI awareness, not app overflow.

### AS. Settings toggle fix

Confirmed. Toggle visuals, accessible state, busy lock and autosave work. Physical round trip briefly changed Faster voice responses; the second click during busy was correctly ignored, then the original `true` value was restored through the UI and verified read-only in SQLite.

### AT. Dashboard

Shared header, reduced first-use metric noise, actionable first steps, clearer local status, safe wrapping and final-only teacher announcement.

### AU. New Lesson

Shared header/back action, equalized mode cards, difficulty explanations, visible disabled reason, validation and busy copy.

### AV. Active Lesson

Existing voice state machine and controls are unchanged. Layout panels use the responsive content grid and only completed teacher text enters the screen-reader status. No human voice session was performed in Phase Q.

### AW. Voice-state copy

Primary UX speaks in terms of listening, thinking, speaking, ready/unavailable and local processing; implementation names remain confined to Diagnostics/advanced detail.

### AX. History

Start Lesson CTA, actionable empty state, full wrapping titles, clearer badges, filters/pagination preserved.

### AY. Lesson Details

Shared width, predictable back path, readable summary/setup/profile sections, technical detail remains disclosed on demand.

### AZ. Progress

CEFR, Practice Level/XP and lesson scores stay separate; missing data is never zero; no-data state leads to a lesson; progress elements are semantic.

### BA. Vocabulary

Status meanings are visible, provenance is explicit, initial empty state leads to practice and detail text wraps.

### BB. Review

Consistent Review vocabulary, deterministic/local disclaimer, self-assessment is explicitly not right/wrong, and start-over/abandon use the shared dialog.

### BC. Placement

Shared hierarchy, duration/assessment limits, independent skill semantics and safe confirmations. No scoring/bank/evaluator change.

### BD. Profile

Current estimate, target and preferences remain separate; local-storage statement and explanatory copy are clearer.

### BE. Achievements

XP disclaimer, semantic progress, distinct success/error channels and clearer goal busy state.

### BF. Pronunciation

Complete button styling, target validation, engine loading/unavailable notice, visible Hear-target limitation, low-confidence notice, standardized date and long-text behavior.

### BG. Settings consolidation

One discoverable page for preferences and system links; it does not duplicate Student Profile or Diagnostics controls.

### BH. Final Settings sections

Voice; Learning; Privacy & Data; System.

### BI. Data & Backup

Preserved in Privacy & Data. Internal navigation, button semantics, long backup IDs, busy state and focus-managed Restore confirmation improved. Backup/Restore validation/safety semantics were not changed.

### BJ. Diagnostics

Read-only purpose is explicit, component grid is responsive, healthy empty events are positive rather than ambiguous, JSON/event content wraps, and buttons use the shared system.

## BK-CE. Onboarding, errors and accessibility

### BK. First-run architecture

Two Tauri commands read/write one existing Settings key. Activity detection checks lessons, placement attempts, pronunciation attempts and review sessions.

### BL. Welcome UX

Non-blocking WelcomePanel on Home with clear Start Conversation / Placement paths and dismissal.

### BM. Existing-user detection

Any existing activity bypasses welcome and persists the seen flag.

### BN. Human existing-user confirmation

The physical database had 12 lessons and other activity. On launch, no welcome was shown and `phase_q_welcome_seen=true` was persisted.

### BO. Onboarding persistence

Verified by two isolated Rust tests: empty DB shows/dismisses/persists; existing DB bypasses and marks seen.

### BP. 404

Unknown routes render a friendly Page not found screen with safe Home action; no silent redirect.

### BQ. Error Boundary

Per-route boundary keyed by pathname preserves the shell and offers Retry/Back to Home.

### BR. Accessibility scope

Landmarks, headings, labels, controls, states, dialogs, focus, keyboard, live regions, progress, contrast, reduced motion, long content and practical scaling.

### BS. WCAG principles

Relevant Perceivable, Operable, Understandable and Robust principles were checked; no certification claim.

### BT. Headings

One route-level h1 through PageHeader; section h2 hierarchy retained.

### BU. Landmarks

Main, navigation, aside, sections and dialogs are semantic and named where necessary.

### BV. Skip link

First Tab displays `Skip to main content`; Enter targets the focusable main landmark.

### BW. Keyboard navigation

Primary actions are native buttons/links; first-Tab smoke test passed; switches, radios, fields and summaries retain native keyboard behavior.

### BX. Focus management

Visible focus on all interactive families; route changes move scroll predictably; no focus styling is removed.

### BY. Dialog focus

Initial safe focus, containment, Escape and focus return are unit-tested.

### BZ. Icons

Decorative icons use `aria-hidden`; icon-only buttons receive accessible names.

### CA. Screen-reader status

Loading and saves use restrained polite regions; errors use alerts; persistent explanatory notices are not needlessly live.

### CB. Streaming accessibility

Draft deltas are no longer within a live transcript container. Only the latest final teacher response is announced.

### CC. Reduced motion

See W; physical UI adds no required motion path.

### CD. Contrast

Real screenshots were visually checked: accent-on-dark, body/muted text, status surfaces and focus ring remain distinguishable. Formal instrument certification is deferred.

### CE. Semantic HTML

Added semantic progress, dialog/status/alert/switch behavior, active-route semantics and actionable state controls; no click-only div controls were found.

## CF-CU. Formatting, terminology, performance and dead code

### CF. Date formatting

`formatLocalDate` is the shared formatter; duplicate Pronunciation formatting was removed. Locale remains the user's Windows locale.

### CG. Duration formatting

`formatDuration` remains the single lesson/dashboard duration policy; unavailable is not treated as zero.

### CH. Number formatting

Counts remain plain deterministic integers; seconds/minutes and CEFR labels are not locale-abbreviated or fabricated.

### CI. UI glossary

Home, New Lesson, Review, Pronunciation, Vocabulary, Progress, History, Placement Test, Student Profile, Achievements, Settings and System Diagnostics. CEFR estimate, Practice Level/XP and lesson score are distinct terms.

### CJ. Copy changes

Added provenance, limitations, disabled reasons, next actions, local/privacy wording, review self-assessment wording and healthy-empty messages.

### CK. Jargon removed

Model/runtime identifiers are kept in Diagnostics advanced context; main UX uses local AI, conversation, response and practice language.

### CL. Frontend performance

Production JS: 406.68 kB / 116.85 kB gzip; CSS: 45.22 kB / 9.07 kB gzip. No dependency or route-fetch expansion.

### CM. Re-renders

Active clock renders only during active voice states and cleans up. Correction ID Set remains memoized. No new global state subscription.

### CN. Polling

No new polling. The single one-second timer is an active-session elapsed clock, not a data poll.

### CO. Large data

History and Vocabulary pagination tests/fixtures passed; responsive screenshots used existing multi-row human data. Virtualization is not warranted.

### CP. Dead-code audit

Frontend import/interaction/mojibake/polling searches completed; no click-only div controls or corrupted source strings remain.

### CQ. Rust warnings before

Eight pre-existing dead-code warning groups.

### CR. Rust warnings after

The same eight groups; no new Phase Q warning. Test linking adds only the normal Windows linker message and duplicate warning reporting.

### CS. Dead code removed

No risky Rust compatibility hook was removed. UI duplication was removed through shared components.

### CT. Dead code retained

NewLesson metadata fields, repository/manual methods, local Ollama health hook, path display helper, Placement status/question helpers and database-path helpers. Unused PlaceholderPage/scaffold assets are low-risk future cleanup.

### CU. Frontend duplication

Headers, notices, metrics, toggles, buttons, state actions and destructive confirmations are now shared; legacy domain-specific report cards remain intentionally specialized.

## CV-DO. Test coverage by area

### CV. Shared components

ProductUI, ConfirmationDialog, WelcomePanel and RouteErrorBoundary tests pass.

### CW. Dashboard

Updated Dashboard tests pass, including new first-use/streaming semantics.

### CX. New Lesson

Updated mode, validation and start behavior tests pass.

### CY. Active Lesson

Dashboard conversation-store/voice UI tests plus Rust voice/session regressions pass; no human microphone session was created.

### CZ. History

Existing pagination/filter/detail logic passes in the full suite; physical list/detail routes were inspected.

### DA. Lesson Details

Physical existing lesson rendered correctly; repository/detail tests pass in Rust/frontend suite.

### DB. Progress

Existing progress/consistency and repository tests pass; physical route inspected.

### DC. Vocabulary

Updated Vocabulary tests pass; physical list/detail routes inspected.

### DD. Review

Updated Review and Review Session tests pass; Rust deterministic queue/repository tests pass.

### DE. Placement

Updated Placement tests pass; scoring/bank/evaluator/repository Rust tests pass; physical landing/result inspected.

### DF. Profile

Profile repository tests pass; physical profile inspected.

### DG. Achievements

Updated Achievements tests and gamification Rust tests pass; physical route inspected.

### DH. Pronunciation

Frontend route tests, Rust engine/repository tests and 12/12 Python core tests pass.

### DI. Settings

Updated Settings/DataBackup tests pass; physical sections and toggle autosave verified.

### DJ. Diagnostics

Updated Diagnostics tests and Rust diagnostics/reliability tests pass; physical status is All systems ready.

### DK. Onboarding

Welcome component tests and two Rust temporary-database tests pass.

### DL. 404

App route test passes for friendly unknown-route behavior.

### DM. Error Boundary

Fallback and recovery test passes.

### DN. Navigation

App/AppLayout route-title, active-navigation, layout and skip-link behavior covered by tests plus physical keyboard smoke.

### DO. Accessibility semantics

Shared-component semantic queries, dialogs, switches, notices, states, progress and route tests pass. Narrator remains human-pending.

## DP-DZ. Build and regression results

### DP. Typecheck

PASS - `npm.cmd run typecheck`.

### DQ. Lint

PASS - `npm.cmd run lint` / oxlint, zero reported issues.

### DR. Full Vitest

PASS - 32 files, 125 tests, 0 failures.

### DS. Rust fmt

PASS - `cargo fmt --check`.

### DT. Rust check

PASS offline; eight known warning groups, unchanged.

### DU. Rust tests

PASS offline - 147 passed, 0 failed, 14 explicitly manual/ignored, 0 measured.

### DV. Python modifications

No Python file modified.

### DW. Voice Streaming regression

PASS - 15/15 with `local-ai/piper/.venv`.

### DX. Pronunciation regression

PASS - 12/12 with `local-ai/pronunciation/.venv`.

### DY. Vite build

PASS - 1855 modules; JS 406.68 kB (116.85 gzip), CSS 45.22 kB (9.07 gzip).

### DZ. Tauri debug build

PASS - `src-tauri/target/debug/english-ai-coach.exe`; `--no-bundle`; no installer generated.

## EA-EG. Physical data preservation

### EA. Counts before

`achievement_unlock=3`, `app_system_event=0`, `conversation_exchange=0`, `correction_candidate=7`, `gamification_profile=1`, `gamification_xp_event=5`, `lesson=12`, `lesson_analysis=8`, `lesson_configuration_snapshot=5`, `lesson_student_profile_snapshot=3`, `lesson_teacher_memory=7`, `lesson_vocabulary=3`, `placement_answer=27`, `placement_attempt=4`, `placement_speaking_response=3`, `pronunciation_attempt=1`, `pronunciation_word_result=3`, `recurring_mistake=6`, `recurring_mistake_occurrence=6`, `review_session=0`, `review_session_item=0`, `schema_migration=13`, `settings=2`, `student_learning_profile=1`, `student_learning_summary=1`, `transcript_message=84`, `vocabulary_item=3`, `voice_turn_performance=2`.

### EB. Counts after

Every count above is identical except `settings=3`, caused solely by `phase_q_welcome_seen=true`. Final settings: onboarding seen `true`, learning memory `true`, faster voice responses `true`. SQLite integrity `ok`; foreign-key violations `0`; logical migrations `13`.

### EC. Pedagogical data

Preserved: lessons, transcripts, analyses, corrections, memory, vocabulary, recurring mistakes, snapshots and profile counts are byte-logically unchanged by Phase Q.

### ED. XP

Preserved: profile 1, XP events 5, unlocks 3.

### EE. CEFR

Preserved: attempts 4, answers 27, speaking responses 3; current physical estimate remains A2/low confidence.

### EF. Review

Preserved: 0 sessions / 0 items. Physical Review was opened read-only; no session started.

### EG. Pronunciation

Preserved: 1 attempt / 3 word results. Physical page was opened; no recording started.

## EH-FH. Physical validation and screenshots

### EH. Dashboard

PASS in real Tauri at Full HD/150%; no welcome incorrectly shown to the existing user.

### EI. New Lesson

PASS; mode grid, hierarchy and sidebar inspected. No lesson started.

### EJ. Active Lesson

PENDING human voice-session layout validation by design. Automated active-state tests passed; no microphone/session mutation was authorized or necessary.

### EK. History

PASS; list and existing Lesson Details inspected.

### EL. Progress

PASS; CEFR and Practice Consistency hierarchy inspected.

### EM. Vocabulary

PASS; list, glossary and existing item detail inspected.

### EN. Review

PASS landing route; no review session started.

### EO. Placement

PASS landing and existing result; no attempt started.

### EP. Profile

PASS read-only inspection; no preference changed.

### EQ. Achievements

PASS read-only inspection; no goal saved.

### ER. Pronunciation

PASS loading/form layout; no recording started.

### ES. Settings

PASS Full HD and minimum width; toggle saved/locked/restored correctly.

### ET. Diagnostics

PASS; All systems ready: Database, Conversation and Pronunciation ready.

### EU. Full HD

PASS at 1920 x 1080 physical desktop / Windows 150%; real app window captured DPI-aware at 1942 x 1106 including borders.

### EV. 1366 x 768 result

Width behavior validated at 1366 physical pixels under 150% scaling; height was 1000 physical because the configured 640 logical minimum becomes 960 physical. Two-column Diagnostics layout, no clipping.

### EW. Smaller window

PASS at configured minimum 720 logical / 1080 physical width under 150%; compact nav and single-column Settings usable.

### EX. Horizontal overflow

No page-level horizontal overflow in inspected routes. Compact navigation intentionally scrolls horizontally; scrollbar is visually hidden and a partially visible item cues continuation.

### EY. Sidebar scroll

PASS; independent scrollbar reaches System links and preserves main scroll behavior.

### EZ. Settings toggle

PASS; final persisted value restored to `true` through the UI.

### FA. Keyboard-only smoke

PASS for initial Tab/skip-link focus, visible focus and main-target architecture; dialog keyboard behavior is unit-tested.

### FB. Narrator

**PENDENTE** - requires a human auditory smoke test. No result was fabricated.

### FC. New-user temporary DB

PASS - isolated Rust test shows welcome, dismissal and persistence.

### FD. Existing-user physical DB

PASS - existing activity bypasses onboarding; seen flag persisted; no pedagogical row changed.

### FE. Long content

PASS through wrapping/min-width semantic audit, long diagnostic/backup styles and tests. Long History titles no longer use inaccessible truncation.

### FF. Large data fixture

PASS existing paginated History/Vocabulary fixtures and physical multi-row data; no virtualization need found.

### FG. Before screenshot paths

No new strict pre-edit screenshot was captured, and none is fabricated. Immediate pre-Q physical baseline is preserved by Phase P evidence, notably `.phase-p-artifacts/dashboard-startup.png`, `.phase-p-artifacts/settings-final-inspected.png`, `.phase-p-artifacts/settings-responsive-final2.png`; source baseline is `.phase-q-artifacts/UX_AUDIT_BEFORE.md` plus the official backup.

### FH. After screenshot paths

- `after-dashboard-fullhd-150pct.png`
- `after-new-lesson-fullhd-150pct.png`
- `after-history-fullhd-150pct.png`
- `after-lesson-details-fullhd-150pct.png`
- `after-progress-fullhd-150pct.png`
- `after-vocabulary-fullhd-150pct.png`
- `after-vocabulary-details-fullhd-150pct.png`
- `after-review-fullhd-150pct.png`
- `after-placement-fullhd-150pct.png`
- `after-placement-result-fullhd-150pct.png`
- `after-profile-fullhd-150pct.png`
- `after-achievements-fullhd-150pct.png`
- `after-pronunciation-fullhd-150pct.png`
- `after-settings-fullhd-150pct.png`
- `after-settings-minwidth-150pct-final.png`
- `after-diagnostics-fullhd-150pct.png`
- `after-diagnostics-1366wide-150pct.png`
- `after-keyboard-skip-link-fullhd-150pct.png`

All are under `.phase-q-artifacts`.

## FI-FK. Remaining debt, problems and readiness

### FI. Remaining UX debt

Human Narrator matrix; formal contrast/AT certification; optional visual regression tooling; full mobile/localization/theme work; low-value scaffold cleanup; physical Active Lesson voice-session layout smoke.

### FJ. Problems encountered

1. First backup wildcard produced one literal file; detected before edits and replaced by the valid 83-entry backup.
2. PowerShell execution policy blocked `npm.ps1`; all Node checks used `npm.cmd`.
3. Sandbox initially blocked Vite/TypeScript/Cargo cache writes; commands were rerun with scoped approval.
4. Streaming test first used the Pronunciation venv and lacked `sounddevice`; rerun in Piper venv passed 15/15.
5. First screenshot process was DPI-unaware and falsely appeared clipped; DPI-aware Win32 capture proved correct rendering. Invalid temporary captures were removed.
6. Toggle round-trip automation clicked during the guarded busy state; the original value was then restored via the UI and verified read-only.

### FK. Product-readiness assessment

The platform base is ready for human acceptance testing of Phase Q. Shell, hierarchy, Settings, navigation, state feedback, error containment, onboarding and practical Windows scaling are materially consistent. All automated and offline regression gates pass, physical data is preserved, and only the explicitly human Narrator/active-voice UX smokes remain pending. Interactive Lessons can be considered later only after this acceptance; it was not started here.

## FL. Explicit protected-contract confirmations

- `voice_coach_v2.py` intact: SHA-256 `F56E16A71130C0BC4974DF13038D5937DA611AF3C90B2CE4C7891F28523D2E2D`.
- `voice_coach_v2_STABLE.py` intact: same SHA-256.
- Conversation Teacher prompt intact: `8B5E07911A50F18E23C6338F8521660BF4CEC652496C785F4B40A4B57056F19D`.
- Lesson Analyzer prompt/schema intact: prompt `6D4CB204B7D74C337546D466BCAB309A87C0859C6B38E1E067C3FA7A5D7C8C41`; schema/tests unchanged.
- Placement scoring intact: `5075B4FDA052E914B7F076A88223CB6F3E5750026E2F58C8E2CCC8A95E8BAB5C`.
- Placement bank intact: `0EB0117D3E465DB26A64218E8963B7C2BA6D514FC84B4328F0126D5ABD325581`.
- Placement Speaking Evaluator intact: `828B5B24ED3D88EAE9B315E22DAA30742499E2DC4A0B7FE4CA453081C9D7FF7A`.
- Student Profile semantics intact: repository `6D35248E7F552B572BB20867D6C1D8BB09512D4D96B563C5C8794373FF9993C6`.
- Learning Memory semantics intact: repository `4C6E67943D62C7AF303533D19D06AA52081B1013563B976D2FA543B7851A6C71`.
- Gamification XP Rule v1 intact: `7918347DE122BDB3F0CF42AD2412FBCA9088CA2384B6A4EB046CF96F8A8358D9`.
- Review queue semantics intact: `3E80CCE9B88C5E4FEFE40C8DFAC6DA1AFE432D9BBAD802BC76EC0997B639AF35`.
- Pronunciation Engine v1 scoring intact: core `0ED7D58735C64844D0B45EDB6455929DA437E3FD009557DE45C64D0057A8E71F`; model manifest `B9EFAEB12388B6BC446B228D8EB4EA3A5B3E6D2617F192A7BBD29EE101F3FC45`.
- Voice Streaming Runtime v1 semantics intact: `8A8BB8FB0CFAB51F37BABC6839FF012C8C051483DA7C57AA251C08CB79E2EAFE`.
- Backup/Restore semantics intact; reliability tests pass.
- Diagnostics semantics intact; physical All systems ready.
- Startup Recovery semantics intact; recovery/reliability tests pass.
- Whisper remains `ggml-small.en-q5_1.bin`.
- Whisper remains 12 threads.
- Conversation VAD remains 3.5 seconds.
- `qwen3.5:4b` remains unchanged.
- Piper remains `en_US-lessac-medium`.
- Pronunciation acoustic model unchanged.
- Normal Lesson pronunciation remains null.
- No new LLM call added.
- No model downloaded.
- No package/crate/plugin installed without authorization.
- No cloud added.
- No telemetry added.
- No cloud backup added.
- No user authentication added.
- No multi-profile added.
- No curriculum added.
- Interactive Lessons **NOT** implemented.
- Theory **NOT** implemented.
- Visual Vocabulary **NOT** implemented.
- Listening Lessons **NOT** implemented.
- Interactive Repeat **NOT** implemented.
- Speaking Gates **NOT** implemented.
- Structured Exercise Engine **NOT** implemented.
- Guided Interactive Conversation **NOT** implemented.
- PDF intelligent system **NOT** implemented.
- Installer **NOT** created.
- Auto-update **NOT** implemented.
- `setup-windows.ps1` **NOT** executed.
- `ollama pull` **NOT** executed.
- Git **NOT** initialized.
- Next phase **NOT** started.

**STOP: Phase Q ends with this report.**
