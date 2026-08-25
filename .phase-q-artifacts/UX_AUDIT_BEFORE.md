# Phase Q UX Audit — Before

Date: 2026-08-23  
Scope: React/Tauri product shell and every registered route.  
Method: source inspection, existing tests, semantic-pattern search, and Phase P evidence. Physical screenshots are pending until the pre-Q app build is launched.  
Accessibility wording: audited against relevant WCAG 2.2 AA criteria; this is not a formal certification.

## Route inventory

| Route | Screen | Baseline finding | Priority |
|---|---|---|---|
| `/` | Home + active voice lesson | Strong functionality but overloaded hierarchy; five zero metrics precede first-use guidance; streaming container announces every draft delta. | P1 |
| `/dashboard` | Redirect | Redirects to `/`; UI terminology should consistently use Home. | P2 |
| `/lesson/new` | New Lesson | Mode cards are native buttons and keyboard-operable, but shared header/form/button patterns are absent; difficulty descriptions and disabled-start explanation are missing. | P1 |
| `/history` | History | Pagination exists and loading precedes empty state. Empty state has no CTA; long title is truncated with no full-content access. | P1 |
| `/history/:lessonId` | Lesson Details | Predictable back link and secondary technical details. Sections lack shared hierarchy; missing-item state has no recovery CTA. Pronunciation is correctly not evaluated, never zero. | P2 |
| `/progress` | Progress | CEFR placement and Practice Level are separate. Empty state has no action; page header duplicated; chart text values exist but SVG focus semantics are brittle. | P1 |
| `/vocabulary` | Vocabulary | Search/filter/pagination work. Initial empty state explains provenance but has no lesson CTA; status meaning is not explained. | P1 |
| `/vocabulary/:vocabularyId` | Vocabulary Details | Predictable back link and autosaving status. Save success is silent, status description absent, missing-item recovery absent. | P2 |
| `/review` | Review | Semantically grouped radio controls. Start-over confirmation is inline and inconsistent with restore/abandon; empty-mode guidance is weak. | P1 |
| `/review/session/:sessionId` | Review Session | Clear progress and non-binary outcomes. Abandon confirmation lacks dialog focus/Escape/return-focus behavior. | P1 |
| `/placement` | Placement | Progress and recording states are explicit. Start-over abandons data without a true confirmation dialog; skip confirmation is inconsistent. | P1 |
| `/placement/results/:attemptId` | Placement Result | Overall estimate precedes domains and unassessed skills. Missing shared back/header patterns. | P2 |
| `/profile` | Student Profile | Estimated level and target are distinct; goals enforce three. Mixed Save behavior vs autosave elsewhere; difficulty choices lack descriptions. | P2 |
| `/achievements` | Achievements | Locked state uses icon and text. Progress bars lack semantic progress values; goal success and error share one status channel. | P1 |
| `/pronunciation` | Pronunciation Practice | Core states exist and breakdown uses buttons. Unique button/eyebrow classes are undefined; engine-unavailable and inline validation states need consolidation. | P0/P1 |
| `/settings` | Settings | Responsive toggle stacking exists. Voice, Learning, backup, memory and privacy are a flat sequence rather than grouped settings. | P1 |
| `/diagnostics` | System Diagnostics | Healthy/degraded cards include icons and text. Advanced JSON and events need stronger long-content handling. | P2 |
| `*` | Unknown route | Silently redirects to Home; no friendly 404. | P1 |

## P0 — blocks or materially impairs use

1. Pronunciation relies on `button-primary`, `button-secondary`, and `eyebrow` classes that are not defined in `styles.css`; primary recording actions can lose their intended affordance.
2. No React error boundary exists. A render exception can blank the route and may take the shell with it.

## P1 — important

1. No shared PageShell/PageHeader/SectionHeader primitives; nearly every page hand-builds different header spacing and hierarchy.
2. Button variants, sizes, disabled/busy treatment, notices, metrics, forms, and destructive confirmations are duplicated.
3. Restore dialog has no focus trap, Escape handling, or focus return. Review/Placement confirmations use unrelated inline patterns.
4. The transcript region has `aria-live="polite"`, so streaming teacher deltas can be announced repeatedly.
5. There is no skip link. Main and nav landmarks exist; active-link semantics are not explicitly tested.
6. Unknown routes redirect silently instead of rendering a friendly 404.
7. New-user Dashboard shows dense zero metrics before useful first actions.
8. No persisted, non-blocking first-run welcome or existing-user bypass logic exists.
9. Empty states generally explain absence but do not offer the next action.
10. Status colors and surfaces use ad-hoc raw values; semantic success/warning/error/info usage is inconsistent.
11. Focus rules omit textarea and generic switch controls. Disabled states often do not explain why.
12. Sidebar is scrollable but ungrouped; eleven items have equal weight.
13. Long titles may truncate without full access; diagnostic and pronunciation content need wrapping/min-width protection.
14. Async action guards exist, but busy labels and `aria-busy` feedback differ.

## P2 — polish

1. Global max width is 1500 px; forms and detail pages have no narrower shared container.
2. Radius and padding values vary without semantic meaning.
3. Typography alternates among raw uppercase labels, `eyebrow`, and local title sizes.
4. Date formatting is environment-default and Pronunciation duplicates it directly.
5. Metric/Summary/Detail/Fact cards are locally reimplemented.
6. Review and Pronunciation files use compressed long lines, increasing maintenance cost.
7. Disabled Hear Target relies on a `title`; its explanation should be visible.
8. There is no shared toast/live-region policy.

## P3 — deliberately future work

1. Formal assistive-technology certification and complete Narrator matrix.
2. Automated visual-regression tooling; none is installed and Phase Q forbids adding it without approval.
3. Full mobile layout; desktop viewports are the product target.
4. Internationalization and dark/light theme system.
5. Data virtualization unless fixture tests reveal a measurable issue.

## Baseline design tokens and patterns

- Palette: `--bg`, `--panel`, `--panel-solid`, `--border`, `--muted`, `--text`, `--accent`, `--accent-soft`, `--danger`, `--warning`.
- Missing named tokens: success, info, focus, layered surfaces, spacing, motion, and semantic radii.
- Surface: `.glass` plus many bespoke `bg-white/[...]` values.
- Typography: system Segoe/Inter stack; no shared page-title, description, section-title, metadata classes.
- Focus: 2 px accent outline for buttons, inputs, selects, links, summaries and `[tabindex=0]`; textarea omitted.
- Motion: pulse and audio-bar animations are gated behind `prefers-reduced-motion: no-preference`; transition utilities elsewhere are not globally neutralized.
- Breakpoints: mobile nav at 760 px; primary practical minimum remains to be validated near 1024×640.

## State, accessibility, responsive, and performance baseline

- Shared Loading/Error/Empty components exist, but Empty has no action and Error always exposes raw detail.
- Several components still implement private states. Loading generally precedes empty correctly.
- Busy booleans guard most mutations; consistent `aria-busy` and loading labels are absent.
- Present accessibility: semantic controls, main landmark, labeled nav, fieldsets, mostly visible focus, color plus text, reduced-motion gating.
- Missing/inconsistent: skip link, error boundary, dialog focus lifecycle, final-only streaming announcement, semantic progress, textarea focus, 404, systematic headings, accessible icon-help patterns.
- Phase P added sidebar scroll, `min-width: 0`, horizontal clipping, route scroll reset, and stacked Settings toggles. Risks remain in long flex content and compact action rows.
- History and Vocabulary are paginated; no virtualization is justified. Dashboard polling is limited to the active-lesson clock; no unnecessary data polling was found.
- Physical keyboard, Narrator, viewport, and screenshot checks remain pending and will not be fabricated.

## Planned incremental response

1. Add small shared UI primitives and semantic CSS tokens without a dependency.
2. Harden shell navigation, skip link, route titles, route error boundary, and 404.
3. Persist one UX onboarding setting in the existing key/value table with no migration and bypass existing users.
4. Consolidate Settings into named sections and link to Profile/Diagnostics rather than duplicate controls.
5. Replace high-value duplicated headers/states/actions and standardize destructive dialogs.
6. Fix streaming accessibility and long-content/viewport behavior.
7. Extend tests for shared primitives, navigation, onboarding, 404, error boundary, settings, empty states, and critical page semantics.

No protected AI, prompt, scoring, pedagogy, voice, placement-bank, review-queue, pronunciation-core, or migration file was changed by this audit.
