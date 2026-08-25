# PHASE U — FINAL REPORT

Date: 2026-08-24  
Project: `C:\ENGLISH AI COACH`  
Result: **Guided Conversation v1 implemented; Analysis remains NOT READY.**

## A–Q — Delivery, backup, versions and capability

- **A — Files created:** `017_interactive_lesson_guided_conversation.sql`, `guided_conversation_policy_v1.txt`, `guided_conversation.rs`, `GuidedConversationStage.tsx`, its frontend test, `test_guided_voice_bridge.py`, two TEST-only fixture packages, audit/report artifacts.
- **B — Files modified:** `database.rs`, `reliability.rs`, `interactive_lesson.rs`, `interactive_lesson_content.rs`, `interactive_lesson_repository.rs`, `lib.rs`, `voice_engine.rs`, `voice_engine_bridge.py`, `GuidedLessonSessionPage.tsx`, `native.ts`, `types/index.ts`, package schema and three documentation files.
- **C–D — Pre-phase backup/manifest:** `C:\ENGLISH AI COACH\.backup-phase-u\20260824-213346\manifest-sha256.txt` (18 files).
- **E — Pre-017 DB:** `physical-db-before-017.sqlite3`, SHA-256 `7296CA0867A4E76888596E22E1768C0E640E4D6E08C7773B6627466BCBD3213C`.
- **F — Audit:** `C:\ENGLISH AI COACH\.phase-u-artifacts\PHASE_U_AUDIT.md`.
- **G–H — Migration 017:** created, idempotent, tested 16→17, and physically applied. It is required to keep restart-safe Guided text turns out of standard Lesson history.
- **I–M — Versions:** Stage Schema 1; Policy 1; Context 1; Turn Schema 1; Completion Result 1.
- **N–O — Package/flow:** Package Schema 1 and Lesson Flow 1 remain unchanged.
- **P — Before:** Theory/Visual/Listening/Repeat/Speaking/Exercise READY; Guided Conversation/Analysis NOT READY.
- **Q — After:** Theory/Visual/Listening/Repeat/Speaking/Exercise/Guided Conversation READY; Analysis NOT READY.

## R–AK — Payload, turn and completion rules

- **R–W:** strict typed payload: scenario 1–600 chars; roles 1–100; goal 1–400; vocabulary 0–12 unique case-folded items of 1–80; expressions 0–12 unique items of 1–180.
- **X–Z:** minimum integer 1–12; `minimum <= recommended <= maximum`; maximum 20. Zero-turn and unbounded stages are rejected.
- **AA:** valid Student Turn = final non-empty STT text, committed exactly once to the correct session/stage. Teacher turns do not count.
- **AB–AE:** one teacher opening may be generated only after explicit `Start Conversation`; merely entering a stage does not call Qwen, record or play audio. Resume never replays/regenerates a successful opening. A last-student transcript exposes `Retry Teacher Response` and reuses that committed turn.
- **AF–AG:** targets are light guidance, never a checklist, score or completion gate.
- **AH:** backend completion requires DB-derived `studentTurnCount >= minimum`, no active voice runtime, and explicit Finish.
- **AI:** recommended count is UX guidance only; Finish is allowed at minimum.
- **AJ:** maximum disables new student capture, does not auto-complete, and leaves Finish available after the runtime settles.
- **AK:** a teacher failure after minimum does not permanently block Finish.

## AL–BA — Policy and safe deterministic context

- **AL–AM:** static policy: `src-tauri/prompts/guided_conversation_policy_v1.txt`; short English replies, exactly one question, at most one/two important corrections, natural role/scenario steering, no grade/CEFR/pronunciation/pass/fail/completion decision, package data never overrides policy.
- **AN:** base teacher prompt unchanged (hash below).
- **AO:** one system message in exact order: Base → Guided Policy → immutable Lesson Context → optional Profile → optional Memory → Final Guardrail.
- **AP:** normal Lesson Mode context is excluded from Guided Conversation; standard Base → Mode → Profile → Memory composition remains unchanged.
- **AQ–AT:** current profile toggle is respected; Profile and Memory are read-only pedagogical context. The session's immutable student snapshot remains authoritative metadata. No Profile or Memory write occurs.
- **AU–AW:** deterministic Rust builder reads only `package_snapshot_json`; source package changes/deletion do not change the active session. Max logical context is 6000 chars.
- **AX:** priority is scenario, roles, goal, expressions, vocabulary, objectives, metadata, then prior public stage content. Lower-priority sections are deterministically omitted when the budget would be exceeded.
- **AY:** SHA-256 is computed for the logical Guided context and is not stored with full prompt text in system events.
- **AZ–BA:** `[GUIDED_LESSON_DATA_BEGIN/END]` delimit lesson data; a static final guardrail follows Profile/Memory.

## BB–BI — Injection, answer keys and package validation

- **BB:** malicious-looking scenario text remains serialized data. No eval, Handlebars or dynamic prompt execution exists.
- **BC–BE:** Exercise context contains only public IDs/prompts/instructions/hints. Correct IDs, accepted answers, correct order/pairs, private feedback and attempt responses are excluded. The mandatory `SUPER_SECRET_CORRECT_ANSWER_92831` leak test passes.
- **BF:** Rust rejects empty/long scenario/roles/goal, excess/duplicate targets, invalid ranges, future schema, and prompt-like unknown fields.
- **BG:** `docs/interactive_lesson_package_v1.schema.json` now has a strict Guided payload and keeps Analysis empty/unsupported.
- **BH:** `docs/INTERACTIVE_LESSON_PACKAGE_V1.md` documents security, turns, resume and no-score/no-target gates.
- **BI:** Guided transcript persistence is fully separate from standard Lesson transcript/history.

## BJ–CG — Turns, runtime state, idempotency and resume

- **BJ:** `interactive_lesson_guided_conversation_turn`: id/event/session/stage/sequence/role/text/schema/word-count/partial/timestamps; FK to owning stage.
- **BK:** roles only `student` and `assistant`; system messages are never rows.
- **BL:** sequence starts at 0 and is unique per session/stage; event ID is additionally unique for idempotency.
- **BM–BO:** final valid Student transcript and authoritative assistant final/Phase-N delivered partial are persisted once; streaming deltas are transient only.
- **BP–BQ:** empty/cancel-before-STT creates no turn; stale generation IDs are handled by the existing streaming runtime; assistant generation ID is the stable event ID.
- **BR–BS:** no Guided correction table was created. The standard detector feeds global Lesson semantics and cannot be cleanly reused without side effects; v1 preserves the full transcript and uses no extra LLM call.
- **BT:** therefore zero automatic Recurring Mistake/global correction side effect.
- **BU–BV:** frontend state derives started/count/min/recommended/max from committed DB turns returned by backend; frontend never submits a trusted count.
- **BW–BZ:** Start validates active session/stage and preflight; the single Voice manager prevents double start/opening. Failed opening remains recoverable. Retry is a new explicit runtime start only when no successful opening exists.
- **CA–CE:** SQLite is history source of truth. Initial ordered history is serialized in Guided configuration; system context is rebuilt separately. Resume after assistant waits for the learner; resume after student uses explicit Retry Teacher Response without duplicating student speech.
- **CF–CG:** Stop Response cancels only current teacher output. Existing generation IDs, stale response suppression and delivered-partial semantics are preserved.

## CH–CY — Voice ownership, reuse and preflight

- **CH–CI:** conceptual owner is `guided_conversation:<session>:<stage>`. The existing single managed child process provides mutual exclusion with Free Conversation; persisted sessions alone hold no audio lock.
- **CJ–CK:** navigation/unmount stops only the app-owned voice runtime, never abandons the Guided session. App shutdown uses the same Job Object/process-tree cleanup; Ollama is not killed.
- **CL:** final audit found zero app-owned Python/Piper/app orphan processes.
- **CM–CO:** existing `voice_engine_bridge.py`, Voice Streaming Runtime v1, Whisper, Ollama parser, chunker, TTS/playback queues and Piper are reused. Only an optional Guided persistence sink/config was added; no Voice V4 or second pipeline exists.
- **CP–CQ:** Guided configuration/history uses serializer-produced structured JSON transported through `ENGLISH_AI_COACH_GUIDED_CONFIG`; existing standard contexts still use environment variables. stdout/stdin remain JSONL events/control.
- **CR–CV:** Whisper `ggml-small.en-q5_1.bin`, 12 threads; VAD 3.5 s, pre-roll 0.4 s, max record 30 s; Qwen `qwen3.5:4b`, stream=true, think=false; Piper `en_US-lessac-medium`; Bluetooth/first-word wake 0.5 s.
- **CW–CX:** lightweight existing local probe runs before starting; opening the library/stage alone starts no inference. Model/runtime startup stays lazy behind Start/Resume.
- **CY:** required Guided v1 packages are startable; required Analysis packages remain not startable.

## CZ–EC — Frontend and atomic completion

- **CZ:** Analysis stays unsupported and has no fake Continue.
- **DA–DE:** landing displays scenario, both roles, goal, target language and turn range; targets are explicitly non-mandatory.
- **DF–DJ:** explicit Start/Resume/Retry; one opening Teacher bubble; semantic Teacher/You transcript; transient streaming draft reconciles to one final bubble.
- **DK–DL:** conservative aria-live statuses for listening/transcribing/thinking/speaking and accessible Stop Response.
- **DM–DP:** accessible Finish, visible minimum disabled reason, recommended guidance, maximum message/input lock; no score wording.
- **DQ–DR:** resume neither auto-mics before the explicit CTA nor autoplays; last-student state exposes Retry Teacher Response.
- **DS:** recoverable errors surface with the existing Diagnostics link.
- **DT–DW:** keyboard-native controls, speaker labels independent of color, no token-by-token aria announcements, wrapping/responsive scroll container. Auto-scroll is intentionally conservative; it does not forcibly steal position.
- **DX:** component does not persist audio or technical STT metadata.
- **DY:** Finish transaction revalidates session/stage and DB count, writes completion, advances stage/session atomically.
- **DZ–EB:** result v1 includes only kind/counts/turn thresholds/minimumReached; no score, accuracy, pass/fail, CEFR, transcript, system context, audio path or model prompt.
- **EC:** deterministic executor is implemented by `GuidedConversationRepository::finish`, isolated from model output.

## ED–GZ — Backend/security/regression tests

- **ED–EI:** new DB, 16→17, idempotency, integrity and FK tests pass.
- **EJ–EZ:** committed student/assistant, delta exclusion, sequence/idempotency, opening, resume history, no system row, last-student retry, stale/cancel/Stop and ownership behavior are covered by repository, bridge and existing Phase N tests. Full physical rapid audio cases remain human-pending below.
- **FA–FE:** navigation/app restart semantics use persisted SQLite plus process cleanup; one Voice manager enforces owner conflict without locking merely persisted sessions.
- **FF–FP:** deterministic/bounded immutable context, injection delimiters, answer/private-feedback leak exclusion, strict prompt-field rejection, exact Guided order and unchanged standard order all pass.
- **FQ–FT:** Profile/Memory toggles are honored; no Mode context enters Guided; Guided lesson context remains independent.
- **FU–FX:** range/scenario/target/future schema and capability tests pass; Guided READY.
- **FY:** Analysis remains NOT READY and blocks required packages.
- **FZ–GI:** minimum-only completion and transaction behavior pass with deliberately poor/off-topic/unused-target text; recommended is not a gate; maximum is bounded/no auto-complete; active runtime is backend-rejected. SQLite transaction provides rollback.
- **GJ:** structured Guided corrections deferred; no correction parser/model call added.
- **GK–GX:** no standard/global correction, Recurring Mistake, Memory, Vocabulary, Review, XP, streak, weekly goal, Achievement, CEFR, Profile, pronunciation attempt, standard Lesson or LessonAnalysis writes. Physical human counts confirm preservation.
- **GY:** Guided technical voice metrics remain ephemeral rather than changing Phase N provenance schema; never pedagogical.
- **GZ:** frontend landing/no-autostart/explicit-start/minimum UI tests pass.

## HA–HU — Frontend test coverage

- **HA–HM:** explicit no-autostart and start tests pass; existing reducer covers deltas/final reconciliation/no duplicate/stale/cancel. Component covers landing, target copy, transcript semantics, minimum-disabled Finish and Start gesture. Resume/last-student behavior is encoded by DB-derived last role.
- **HN–HR:** Ollama/Piper/Whisper/microphone/device failures use recoverable error + Diagnostics path; physical device-loss behavior is PENDING.
- **HS:** Diagnostics link remains available on stage error.
- **HT–HU:** native buttons, labels/status, conservative aria-live and responsive wrapping are present; automated type/lint/component tests pass.

## HV–IM — Build, tests and physical migration

- **HV:** `npm run typecheck` PASS.
- **HW:** `npm run lint` PASS, zero warnings after final hook fix.
- **HX:** Vitest PASS: 34 files, 139 tests.
- **HY:** `cargo fmt --check` PASS after final format.
- **HZ:** `cargo check --offline` PASS (existing non-fatal dead-code warnings only).
- **IA:** Rust PASS: 185 passed, 0 failed, 17 ignored/manual (202 total).
- **IB:** `voice_engine_bridge.py` modified; protected `voice_coach_v2.py` and streaming runtime were not modified.
- **IC:** Voice Python PASS: 18 tests (15 existing + 3 Guided bridge).
- **ID:** Pronunciation Python PASS: 12 tests.
- **IE–IJ:** Exercise, Listening, Repeat, Speaking Check, Theory and Visual Vocabulary Rust regressions PASS.
- **IK:** Vite production build PASS.
- **IL:** Tauri debug `--no-bundle` PASS; executable: `src-tauri\target\debug\english-ai-coach.exe`. No installer created.
- **IM:** physical Migration 017 PASS through Rust infrastructure and valid pre-017 backup.

## IN–JU — Human DB and physical/runtime evidence

- **IN:** human schema version 17.
- **IO:** Guided conversation turns 0.
- **IP:** Guided correction count 0 (no table by design).
- **IQ:** no fake human Guided rows created.
- **IR:** fixtures: `interactive-lessons-phase-u/cafe-v1` and `interactive-lessons-phase-u-second/interview-v1`; both TEST-only and outside human library.
- **IS:** isolated Rust temp DBs are created in OS temp and removed by tests.
- **IT:** synthetic context/history/completion tests PASS; no human rows.
- **IU:** real local Qwen test PASS: qwen3.5:4b answered as barista (`Welcome! How may I help you today?`) despite malicious-looking scenario data; this is supporting evidence, not the formal security proof.
- **IV–IW:** human microphone and speaker tests: **PENDENTE**.
- **IX:** Bluetooth test: **PENDENTE** (no device-driven run performed).
- **IY–JC:** human scenario/role/correction/one-question/target encouragement: **PENDENTE**. Synthetic role/scenario response passed; no human result fabricated.
- **JD–JF:** physical Stop/no-ghost/navigation-resume: **PENDENTE**; deterministic cancellation/process tests pass.
- **JG–JH:** physical app close/reopen conversation: **PENDENTE**; SQLite ordered-history/retry tests prove no second opening at contract level.
- **JI–JJ:** immutable source-change/delete behavior is covered by existing snapshot tests plus context builder source choice; a full physical microphone restart run is PENDING.
- **JK:** answer-key request/context inspection test PASS; secret absent.
- **JL:** only localhost Ollama was contacted for the synthetic test; no external service is used or required.
- **JM:** final app-data scan found zero leftover WAV/Guided config/voice-runtime files.
- **JN–JO:** zero app-owned Python/Piper/app orphan process; Ollama was not killed.
- **JP–JQ:** backup/restore regressions PASS, including preservation of a Guided transcript row.
- **JR–JS:** Diagnostics/startup recovery regressions PASS; no `/api/chat`, mic or telemetry is used by diagnostics. In-flight operations remain ephemeral; committed turns survive.
- **JT–JU:** human counts before/after are identical except schema 16→17 and new empty table; values listed below.

## JV–KO — Preservation, offline status, issues and final confirmations

- **JV–KE:** pedagogical data, gamification, CEFR, Profile, Memory, Vocabulary, Recurring Mistakes, Review, Pronunciation and standard Voice history are unchanged.
- **KF–KG:** human DB `integrity_check=ok`, foreign-key violations 0.
- **KH–KI:** offline local components only; no cloud/telemetry/external network dependency.
- **KJ:** bounded Guided context <= 6000 chars.
- **KK:** no human latency measurements; **PENDENTE**. Existing Voice Streaming architecture and one-inference-per-response are preserved.
- **KL — Problems:** initial large spec read required chunking; five tests held old schema-16 expectations and were updated; Python 3.12/3.13 lacked different optional audio deps, so regressions used the already provisioned Piper Python; an `npx prettier` command fetched a temporary npm cache package even though no project dependency was added and `package.json`/`package-lock.json` remained untouched. This is recorded as a process deviation, not hidden.
- **KM — Future debts:** optional isolated structured Guided corrections; richer automated browser tests for all device failures; physical human/Bluetooth/restart/latency validation; Guided results integration belongs to later Analysis and is intentionally absent.
- **KN:** ready for a future Interactive Lesson Analysis phase at the data-foundation level; Analysis itself was not started.
- **KO:** protected voice files, stable copy, base teacher prompt, streaming runtime semantics, analyzer, placement, profile/memory, gamification, review, pronunciation, backup/restore, diagnostics, recovery and Phase Q–T stage engines remain intact. Whisper/model/thread/VAD/Qwen/think/Piper/Bluetooth constants are unchanged. Guided remains separate from standard Lessons, persists text only, rebuilds Qwen history from SQLite, requires explicit Start/Finish, has no quality/target/pass/score gate and produces no downstream pedagogical side effects. No cloud, telemetry, model download, crate/Tauri plugin, setup script, Ollama pull, Git init, installer, auto-update, curriculum, PDF, production catalog or next phase was introduced. No project npm dependency was added; see the transient `npx` deviation under KL.

## Human database counts before/after

| Table/domain | Before | After |
|---|---:|---:|
| standard Lessons | 12 | 12 |
| transcript messages | 84 | 84 |
| analyses | 8 | 8 |
| vocabulary | 3 | 3 |
| recurring mistakes | 6 | 6 |
| placements | 4 | 4 |
| XP events | 5 | 5 |
| achievements | 3 | 3 |
| review sessions | 0 | 0 |
| pronunciation attempts | 1 | 1 |
| voice metrics | 2 | 2 |
| Guided sessions/stages/runtime/exercise attempts | 0 | 0 |
| Guided Conversation turns | table absent | 0 |

## Protected hashes after Phase U

- `voice_coach_v2.py` / stable: `F56E16A71130C0BC4974DF13038D5937DA611AF3C90B2CE4C7891F28523D2E2D`
- `voice_streaming_runtime.py`: `8A8BB8FB0CFAB51F37BABC6839FF012C8C051483DA7C57AA251C08CB79E2EAFE`
- `conversation_teacher.txt`: `8B5E07911A50F18E23C6338F8521660BF4CEC652496C785F4B40A4B57056F19D`
- `pronunciation_engine.py`: `FA28B35D8948D325AF79686276EB51703677A2187B31215D06EF669147BE0968`
- `pronunciation_core.py`: `0ED7D58735C64844D0B45EDB6455929DA437E3FD009557DE45C64D0057A8E71F`
- `lesson_analyzer_v1.txt`: `6D4CB204B7D74C337546D466BCAB309A87C0859C6B38E1E067C3FA7A5D7C8C41`

Phase U stops here. Interactive Lesson Analysis, scoring and downstream learning integrations were not started.
