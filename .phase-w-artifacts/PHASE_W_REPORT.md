# Phase W — Curriculum & Content Architecture

Final validation: 2026-08-25. Official project: `C:\ENGLISH AI COACH`.

## Delivery and versions

A. Files created. `docs/CURRICULUM_V1.md`, `docs/curriculum_v1.schema.json`, `src-tauri/src/curriculum.rs`, the two production `.keep` files, four test Curriculum manifests, four test Lesson packages, `src/pages/CoursePage.tsx`, `src/pages/CoursePage.test.tsx`, `src/components/AppLayoutCourse.test.tsx`, this report and Phase W audit/TEMP artifacts.

B. Files modified. `src-tauri/src/interactive_lesson.rs`, `interactive_lesson_content.rs`, `interactive_lesson_engine.rs`, `lib.rs`, `tauri.conf.json`, `src/App.tsx`, `App.test.tsx`, `components/AppLayout.tsx`, `services/native.ts`, and `types/index.ts`.

C. Pre-Phase W backup. `C:\ENGLISH AI COACH\.backup-phase-w\20260824-234214`.

D. SHA-256 backup manifest. `.backup-phase-w/20260824-234214/manifest-sha256.txt`; all entries physically hashed.

E. Physical DB backup path. `C:\Users\guicu\AppData\Local\com.englishaicoach.desktop\backups\EnglishAICoach-Physical-Backup-1787625685-604d2ef3.eacbackup`.

F. PHASE_W_AUDIT path. `.phase-w-artifacts/PHASE_W_AUDIT.md`.

G. Whether Migration 019 was created. No.

H. If yes, exact justification. Not applicable; all new state is bundled content or derived from existing tables.

I. Human DB schema before. 18.

J. Human DB schema after. 18.

K. Curriculum Schema Version. 1.

L. Curriculum Registry Version. 1.

M. Curriculum Progress Version. 1.

N. Curriculum Recommendation Version. 1.

O. Curriculum Taxonomy Version. 1.

## Content contract and registry

P. Curriculum content root architecture. Immediate package directories under sibling roots `resources/curriculum` and `resources/interactive-lessons`; production roots contain only `.keep`.

Q. Development resource resolution. `CARGO_MANIFEST_DIR/resources/{curriculum,interactive-lessons}`.

R. Packaged Tauri resource resolution. `resource_dir()/{curriculum,interactive-lessons}`, both declared in `bundle.resources`.

S. Curriculum manifest schema. Strict typed `curriculum.json`: identity/publication metadata plus ordered Levels, Units, and exact Lesson refs.

T. Curriculum ID rules. Unique lowercase slug, 1–64 characters.

U. Curriculum version rules. Positive integer; latest valid published version per ID wins deterministically.

V. Publication-state semantics. Only `published` is public; `draft` remains available only for validation/cross-reference rules.

W. Level schema. `levelId`, exact `cefrLevel`, title, description, order, and ordered Units.

X. Canonical CEFR order. A1, A2, B1, B2, C1, C2.

Y. Level/CEFR pairing rules. `levelId` must equal its `cefrLevel`; order must match canonical CEFR order.

Z. Level validation rules. Unique, canonical, nonempty, ordered, bounded; a Curriculum may expose any valid subset.

AA. Unit schema. Stable ID, title, description, objectives, controlled skill focuses/topic collections, and ordered Lesson refs.

AB. Unit ID semantics. Unique lowercase slug across the entire Curriculum.

AC. Unit objective limits. 1–10 plain-text objectives, each bounded to 240 characters.

AD. Skill Focus registry. `grammar`, `vocabulary`, `listening`, `pronunciation`, `speaking`, `interaction`.

AE. Grammar Topic rules. Plain metadata only, maximum 12 entries of 120 characters.

AF. Vocabulary Topic rules. Same bounded plain-metadata rule.

AG. Communicative Function rules. Same bounded plain-metadata rule.

AH. Lesson Reference schema. Exactly `lessonId` plus positive `contentVersion`.

AI. Exact contentVersion pinning. Registry retains and resolves the exact `(lessonId, contentVersion)`; requested versions never fall back.

AJ. Why titles/descriptions are not duplicated in curriculum. Lesson presentation metadata is resolved from the referenced package, preventing divergence.

AK. Stable Lesson ID semantics. It is the logical completion key across content updates.

AL. contentVersion update semantics. New content can show `Updated` while preserving completion for the stable ID.

AM. Rule for significant lesson redesign requiring new lessonId. Use a new ID when the learning objective/identity changes; use a new version for revisions of the same lesson.

AN. Published Curriculum → Published Lesson cross-validation. Exact package must exist, be published, valid, and have matching CEFR.

AO. Duplicate Lesson semantics. Forbidden inside one Curriculum; the same Lesson may be referenced by different Curricula.

AP. Curriculum size limits. Manifest ≤1 MiB, ≤500 total refs, ≤6 Levels, ≤50 Units per Level, and 1–30 Lessons per Unit.

AQ. Strict-field validation. `deny_unknown_fields` throughout; future/unknown schemas are isolated.

AR. Curriculum security restrictions. No prompts, URLs, commands, code, paths, HTML/control characters, private answer material, or symlink traversal.

AS. Curriculum Registry architecture. Startup-loaded immutable registry with strict parse, exact Lesson cross-validation, deterministic public catalog, and typed hash.

AT. Invalid Curriculum isolation. One invalid package is excluded without crashing or hiding valid siblings.

AU. Curriculum caching. Registry is loaded once at app startup; browsing does not rescan disk.

AV. Curriculum hash architecture. SHA-256 over canonical typed manifest plus ordered exact Lesson package hashes.

AW. Whether referenced Lesson package hashes are included. Yes.

AX. Multi-curriculum support. Yes; landing shows cards when more than one published Curriculum exists.

AY. Safe public DTO architecture. Only public curriculum/lesson metadata, progress, recommendation, target, and active-session identity are serialized.

AZ. Lesson metadata resolution. From the exact validated Guided Lesson package.

BA. Confirmation Exercise answer keys are not exposed. Confirmed.

BB. Confirmation Curriculum does not contain prompts. Confirmed and validated.

BC. Confirmation Curriculum does not duplicate Guided Lesson Engine. Confirmed.

## Sessions, progress, and recommendation

BD. Relationship between Course and Guided Lessons library. Separate navigation experiences over the same package and session foundation.

BE. Start Lesson flow from Course. Calls official Guided start with exact ID/version, then routes to the official session page.

BF. Active Guided Session handling. Catalog exposes safe active identity; Course offers explicit resume and blocks a competing start.

BG. No-silent-abandon confirmation. Confirmed; official engine also rejects the second start.

BH. Curriculum Progress source of truth. One query over existing `interactive_lesson_session`; no Curriculum progress table.

BI. Lesson status semantics. Completed wins; otherwise matching active is in-progress; otherwise not-started.

BJ. Abandoned-session semantics. Never completes progress.

BK. Failed-session semantics. Never completes progress.

BL. Multiple-completion semantics. Counts the stable Lesson once.

BM. Stable lessonId progress key. Confirmed.

BN. Cross-contentVersion completion semantics. Any completed version preserves logical completion.

BO. Updated-content detection. Referenced version greater than the maximum completed version sets `updatedContentAvailable`.

BP. Active old-version session behavior. Resume immutable original snapshot; no upgrade or abandonment.

BQ. Unit progress formula. Completed unique Lessons / referenced Lessons.

BR. Level progress formula. Completed unique Lessons / all referenced Lessons in Level.

BS. Course progress formula. Completed unique Lessons / all referenced Lessons in Course.

BT. Progress rounding. Nearest whole percent.

BU. Confirmation Exercise Accuracy does not affect progress. Confirmed.

BV. Confirmation Pronunciation does not affect progress. Confirmed.

BW. Confirmation Conversation scores do not affect progress. Confirmed.

BX. Confirmation Analysis partial still allows completed-session progress. Confirmed; only session status matters.

BY. Placement recommendation architecture. Deterministic view-only badge derived from official current completed Placement.

BZ. Placement → Suggested Level mapping. Exact CEFR match when that Level exists.

CA. No-Placement behavior. No suggestion.

CB. Confirmation no silent A1 recommendation. Confirmed.

CC. Target Level behavior. Separate Profile badge; informational only.

CD. Confirmation Placement does not complete curriculum. Confirmed.

CE. Confirmation Target Level does not complete curriculum. Confirmed.

CF. Confirmation CEFR does not lock Levels. Confirmed; every valid Level is linked.

CG. Course Next Step behavior, if implemented. Not implemented; deliberately outside the deterministic foundation requirement.

CH. Confirmation no AI Recommendation Engine. Confirmed.

CI. Confirmation no prerequisite hard gates. Confirmed.

## Routes and UI

CJ. Course routes. `/course`, `/course/:curriculumId`, `/course/:curriculumId/:levelId`, `/course/:curriculumId/:levelId/:unitId`.

CK. Course landing UI. Honest empty state, direct overview for one Curriculum, selection cards for multiple.

CL. Zero-curriculum empty state. “No course content is installed yet.”; route remains safe and sidebar hides Course.

CM. Multi-curriculum UI behavior. Deterministic cards link to each Curriculum.

CN. Course overview UI. Recommendation/target context, Course progress, and all accessible Level cards.

CO. Level cards. CEFR/title/description/progress/status with accessible links.

CP. Placement recommendation badge. Informational “Suggested from Placement”.

CQ. Target Level badge. Separate informational target.

CR. Level page. Generic data-driven route with Level progress and Units.

CS. Unit cards. Metadata, skills, lesson count, and progress.

CT. Unit page. Generic route with objectives/topics/functions and Lesson list.

CU. Lesson cards. Resolved title/description/CEFR/version/status/update/action.

CV. Lesson status UI. Text plus visual badge: Not started, In progress, Completed.

CW. Updated-content UI. Completed remains completed and receives `Updated`.

CX. Start CTA. Starts the exact pinned version.

CY. Resume CTA. Resumes the existing official session.

CZ. Review CTA. Starts exact content as review; new revision reads `Review Update`.

DA. Sidebar visibility rule. Course appears only when at least one valid published Curriculum exists.

DB. Confirmation Guided Lessons navigation remains available. Existing independent visibility rule is preserved.

DC. Accessibility. Semantic headings/nav/links/buttons, status text, labels, focusable native controls, and honest errors.

DD. Progress accessibility. Every bar has `role=progressbar`, label, min/max/current, and visible percentage.

DE. Keyboard behavior. Native links/buttons preserve Enter/Space/tab behavior; no custom trap.

DF. Responsive behavior. Wrapping grids, flexible cards, and Phase Q container classes avoid fixed desktop-only layouts.

DG. Phase Q design-system reuse. Existing surface, muted, accent, spacing, button, empty/error, and route foundation reused.

DH. Curriculum authoring guide path. `docs/CURRICULUM_V1.md`.

DI. Curriculum JSON Schema path. `docs/curriculum_v1.schema.json`.

DJ. Authoring workflow. Author/test exact Guided packages, author strict Curriculum refs, validate, publish both, then restart to refresh startup cache.

DK. Confirmation no lesson-specific code. Confirmed.

DL. Confirmation no unit-specific code. Confirmed.

DM. Confirmation no separate A1/A2/B1 page implementations. Confirmed; one generic page handles all routes.

## Fixtures and tests

DN. Test Curriculum path. `src-tauri/test-fixtures/curriculum-phase-w`.

DO. Test Guided Lesson root. `src-tauri/test-fixtures/interactive-lessons-phase-w`.

DP. TEMP DB path. `.phase-w-artifacts/phase-w-temp-validation.sqlite3` only.

DQ. Draft visibility tests. Passed.

DR. Invalid Curriculum tests. Passed, including sibling isolation.

DS. Schema-version tests. Passed.

DT. Canonical-order tests. Passed.

DU. Level-pair tests. Passed.

DV. Duplicate-level tests. Passed.

DW. Duplicate-unit tests. Passed.

DX. Skill-enum tests. Passed, including unknown skill.

DY. Lesson-ref tests. Passed.

DZ. Missing-Lesson tests. Passed.

EA. Wrong-contentVersion tests. Passed; exact version is required.

EB. Draft-Lesson reference tests. Passed; published Curriculum cannot reference draft.

EC. Duplicate-Lesson tests. Passed.

ED. Manifest-size tests. Passed for byte and 500-reference limits.

EE. Unknown-field tests. Passed.

EF. Prompt-field rejection tests. Passed.

EG. Hash-determinism tests. Passed across JSON formatting and exact package hash changes.

EH. Registry-isolation tests. Passed.

EI. Cache tests. Passed; post-load filesystem change does not alter instance.

EJ. Progress not-started tests. Passed.

EK. Progress in-progress tests. Passed.

EL. Progress completed tests. Passed.

EM. Abandoned-session tests. Passed.

EN. Failed-session tests. Passed.

EO. Multiple-completion tests. Passed.

EP. contentVersion progress tests. Passed.

EQ. Updated-content tests. Passed.

ER. Active-old-version tests. Passed in unit and physical TEMP validation.

ES. Unit-progress tests. Passed.

ET. Level-progress tests. Passed.

EU. Course-progress tests. Passed; physical result 50%.

EV. Score-not-progress tests. Passed.

EW. Placement recommendation tests. Passed; physical result B1.

EX. No-Placement tests. Passed.

EY. No-silent-A1 tests. Passed.

EZ. No-level-lock tests. Passed.

FA. Cross-level-access tests. Passed; physical fixture exposed both A1 and B1.

FB. Course Next Step tests if implemented. Not applicable; not implemented.

FC. N+1 query audit. Passed architecturally: all sessions are aggregated by one SQL query, then mapped in memory.

FD. No-Qwen tests. Browsing path has no model dependency/call; source audit passed.

FE. No-Whisper tests. Same; passed.

FF. No-Piper tests. Same; passed.

FG. No-Pronunciation-worker tests. Same; passed.

FH. No-audio tests. Course browsing has no audio path; passed.

FI. Empty-state frontend tests. Passed.

FJ. Sidebar visibility tests. 2/2 passed.

FK. One-Curriculum frontend tests. Passed.

FL. Multiple-Curricula frontend tests. Passed.

FM. Six-Level ordering tests. Passed with canonical A1–C2 fixture DTO.

FN. Recommendation badge tests. Passed.

FO. Level-progress UI tests. Passed.

FP. Unit UI tests. Passed.

FQ. Lesson-status UI tests. Passed.

FR. Updated-content UI tests. Passed.

FS. Start CTA tests. Passed.

FT. Resume CTA tests. Passed.

FU. Review CTA tests. Passed.

FV. No-silent-abandon tests. Passed.

FW. Guided Lesson Library regression. Full Rust/frontend suites passed.

FX. Guided History regression. Full suites passed; repository/session behavior unchanged.

FY. Interactive Analysis regression. Full suites passed; protected hash intact.

FZ. Guided Conversation regression. Full suites passed; protected engine/prompt hashes intact.

GA. Exercise regression. Passed; protected hash intact.

GB. Listening regression. Passed in full Guided engine suite.

GC. Repeat regression. Passed in full Guided engine suite.

GD. Speaking Check regression. Passed in full Guided engine suite.

GE. Theory regression. Passed in full Guided engine suite.

GF. Visual Vocabulary regression. Passed in full Guided content/engine suite.

GG. Free Conversation regression. Passed in existing lesson-mode/frontend suites.

GH. Placement regression. Passed; bank/scoring/evaluator behavior covered and source timestamps predate W.

GI. Profile regression. Passed in Rust and frontend suites.

GJ. Memory regression. Passed in Rust suite.

GK. Gamification regression. Passed in Rust/frontend suites.

GL. Review regression. Passed in Rust/frontend suites.

GM. Pronunciation regression. Rust suite passed plus Python 12/12.

GN. Voice Performance regression. Rust/frontend suites plus voice Python 18/18 passed.

GO. Accessibility tests. Course and Phase Q product tests passed.

GP. Responsive tests. Course responsive class/UI assertions and existing Phase Q suite passed.

GQ. Typecheck result. Passed (`tsc -b`).

GR. Lint result. Passed (`oxlint`).

GS. Frontend test result. 37 files, 153 tests passed, zero failures.

GT. Rust fmt result. Passed (`cargo fmt --all -- --check`).

GU. Rust check result. Passed offline; only pre-existing dead-code warnings.

GV. Rust test result. 204 passed, 20 explicit manual/physical tests ignored, zero failed; Phase W focused 8 active passed plus physical TEMP 1/1.

GW. Python modification status. No Python file modified.

GX. Voice Python regression result. 18/18 passed.

GY. Pronunciation Python regression result. 12/12 passed.

GZ. Vite build. Passed; 1,864 modules transformed.

HA. Tauri debug build. Passed with `--debug --no-bundle`; output: `src-tauri/target/debug/english-ai-coach.exe`. A first attempt found PID 25024 holding the old executable; because the human DB had zero active Guided sessions, the window accepted a normal `CloseMainWindow` request and the clean retry succeeded. No process was force-killed.

HB. Confirmation no installer. Confirmed; `--no-bundle` only.

HC. Confirmation no new dependency. Confirmed; Cargo/npm manifests and locks equal backup hashes.

HD. Confirmation no npm/npx download. Confirmed; no install or npx command executed.

HE. package.json status. Unchanged; SHA-256 `572A27D52E3A998A6A22F4BF642A39B6D855708925E7BF9DB2EAB7E8AFDA5E9D`.

HF. package-lock.json status. Unchanged; SHA-256 `5CE35927AFEB8FEA1C35A4DC1E87439553BC48545BEDAB715A79A97628990317`.

## Physical data and preservation

HG. Migration 019 physical status. Absent and not applied.

HH. Human DB schema final version. 18.

HI. Human Curriculum-specific row count if any new table exists. No new table; not applicable.

HJ. Confirmation no fake human curriculum progress. Confirmed; human Guided session count remains 0.

HK. Confirmation test curriculum not exposed to human library. Confirmed; production roots contain only `.keep`.

HL. TEMP physical Curriculum navigation result. Passed with 2 published test Curricula and safe nested navigation DTO.

HM. TEMP Placement recommendation result. B1.

HN. TEMP cross-level access result. Both test Levels accessible; no lock.

HO. TEMP Guided Lesson start result. Exact pinned contentVersion 1 started through official engine.

HP. TEMP progress update after completion. 1 completed Lesson; Course progress 50%.

HQ. TEMP contentVersion-update result. Completion preserved and `updatedContentAvailable=true` for version 2.

HR. TEMP active-old-version resume result. Resumed immutable snapshot contentVersion 1.

HS. Curriculum source-update result. Registry hash includes exact package hashes; a fresh load observes source updates, an existing cached instance does not mutate.

HT. Curriculum-delete/session-independence result. Session snapshots/repository contain the started content independently; deleting bundled definitions cannot rewrite existing sessions. Existing reliability tests validate snapshot preservation.

HU. Backup regression. Passed existing manifest/hash and Guided state backup tests.

HV. Restore regression. Passed current/older schema, Guided stages, and Interactive Analysis restore tests.

HW. Diagnostics regression. Full suite passed; diagnostics protected behavior unchanged.

HX. Startup Recovery regression. Full suite passed; no Curriculum write/recovery state was introduced.

HY. Human DB counts before. achievement 3; system event 0; conversation 0; corrections 7; gamification profile/events 1/5; Guided session/stage/runtime/exercise/conversation/pronunciation/analysis all 0; lessons 13; analyses 9; config/profile/memory snapshots 6/4/8; lesson vocabulary 3; Placement answer/attempt/speaking 27/4/3; pronunciation attempt/words 1/3; recurring mistake/occurrence 6/6; review session/items 0/0; settings 3; profile 1; summary 1; transcript 84; vocabulary 3; voice performance 2.

HZ. Human DB counts after. Exactly equal to HY.

IA. Standard data preservation. 13 lessons, 9 analyses, 84 transcript rows preserved.

IB. Guided Session preservation. 0 before and after; no fabrication.

IC. Interactive Analysis preservation. 0 before and after; no fabrication.

ID. Gamification preservation. Profile 1, XP events 5, achievements 3.

IE. CEFR preservation. Placement attempts/answers/speaking 4/27/3.

IF. Student Profile preservation. 1 row.

IG. Learning Memory preservation. Summary 1 and teacher-memory snapshots 8.

IH. Vocabulary preservation. Global 3 and lesson vocabulary 3.

II. Recurring Mistakes preservation. 6 mistakes and 6 occurrences.

IJ. Review preservation. 0/0, unchanged.

IK. Pronunciation preservation. 1 attempt and 3 word results.

IL. Voice Performance preservation. 2 rows.

IM. SQLite integrity. `ok` before and after.

IN. Foreign-key result. 0 violations before and after, including TEMP DB.

IO. Offline status. All build/test commands used local installed tools; Rust resolution used `--offline`.

IP. External network audit. No Curriculum runtime network client, cloud, telemetry, URL fetch, or external call exists; test-only forbidden URL field is rejected.

IQ. Curriculum discovery timing if measured. Focused nine-test run (including registry/security/hash/progress, excluding physical test) completed in 0.12 s; no production benchmark claimed.

IR. Progress aggregation timing if measured. Not independently benchmarked; physical end-to-end TEMP test completed in 0.32 s and uses one SQL query.

IS. Problems encountered. First manual physical invocation omitted its required TEMP environment guard and was safely rejected; rerun passed. First Tauri link retry met an open debug executable; its window closed normally and the next build passed.

IT. Future debts. Real authored A1 content remains a separate approved content-production phase; no Foundation v1 code debt blocks it.

IU. Readiness for real A1 content production. Curriculum Foundation v1 is ready for controlled authoring; the final unlocked-executable build passed and no real course content was mass-generated.

## IV. Explicit confirmation

IV. Explicitly confirm:

- `voice_coach_v2.py`, `voice_coach_v2_STABLE.py`, Voice Streaming Runtime v1, Conversation Teacher prompt, normal Lesson Mode, Standard Lesson Analyzer, Placement bank/scoring/evaluator, Student Profile, Learning Memory, Gamification XP Rule v1, Review, Pronunciation Engine/Score v1, Backup/Restore, Diagnostics, Startup Recovery and Phase Q UX/accessibility remain intact.
- Guided Package Registry public latest-version behavior and Guided Session/Snapshot architecture are preserved; exact-version retention is additive.
- Theory, Visual Vocabulary, Listening, Repeat, Speaking Check, Exercise, Guided Conversation and Interactive Lesson Analysis remain READY; all eight stages remain READY.
- Curriculum Foundation v1 is implemented as bundled/local data, not a database source of truth, and does not duplicate Lesson content.
- Exact version, publication, existence, CEFR, deterministic ordering and canonical A1–C2 rules are enforced.
- Content CEFR, Placement estimate and actual Curriculum completion remain separate; Placement/Target never complete or lock Levels; no prerequisites exist.
- Progress comes only from completed Guided sessions keyed by stable Lesson ID; abandoned/failed sessions do not count, repeated completions count once, revisions preserve completion and can signal Updated, and active snapshots never mutate.
- Accuracy, pronunciation, conversation and Analysis scores do not affect Curriculum progress.
- Placement suggestion is deterministic, absent without Placement, informational and nonlocking; no AI/model starts during Course browsing.
- Course and Guided Lessons remain distinct experiences over the one official Guided engine/session type/start semantics; existing active sessions are never silently abandoned.
- The implementation is fully data-driven: no Lesson/Unit/CEFR-specific Rust or React page code.
- No production Curriculum, fake human content/progress, exam, certificate, Guided XP/streak/achievement, memory, vocabulary, Review, recommendation AI, PDF, cloud or telemetry was added.
- No model was downloaded and no package, crate or plugin was installed; no missing tool was fetched by npx.
