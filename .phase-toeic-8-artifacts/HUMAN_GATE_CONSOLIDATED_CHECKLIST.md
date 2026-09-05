# TOEIC Phase 8 — Consolidated Human Gate Checklist

Tester: ____________________  Date: ____________________  App build/path: ____________________

For each line record **PASS**, **FAIL**, or **N/A**, plus the exact Form, Part, question/set number, observed text/audio and a screenshot when relevant. Do not report only “it failed.”

## 1. Smoke, safety and navigation

- [ ] App opens normally and TOEIC Exam Center is reachable.
- [ ] Existing lessons/history/profile data are present; no reset or migration prompt appears.
- [ ] TOEIC History exposes standalone attempts, Full Listening, Full Reading and Full L&R parents without mixing child ownership.
- [ ] Closing and reopening the app preserves the active TOEIC session and exact Form family.
- [ ] No official ETS affiliation, official score or timed-official-exam claim appears.

## 2. Visual and interaction review

Test at the normal desktop size and at the smallest supported app window.

- [ ] Cards, choices, graphics/tables, progress, controls and feedback remain readable with no clipping or overlap.
- [ ] Keyboard focus is visible; Tab order is logical; Enter/Space activates the focused choice/button.
- [ ] Selected, locked, correct and incorrect states are visually distinguishable without relying only on color.
- [ ] Loading, disabled, retry, save/exit and resume states explain what is happening.
- [ ] Long passages in Parts 6/7 can be read and scrolled without losing the active question.

## 3. Listening human review — Parts 1–4

Sample at least **one complete set/sequence from every Form A/B/C in every Part**; for Parts 1/2, listen to at least 6 items per Form. Record every problematic item ID.

- [ ] Part 1 photographs: image and four statements are coherent; exactly one best answer; no missing asset.
- [ ] Part 2 question–response: prompt and responses sound natural; exactly one best response; no truncated or merged audio.
- [ ] Part 3 conversations: speaker changes are clear, pacing natural, questions answerable from audio/graphic, and replay/lock behavior is correct.
- [ ] Part 4 talks: delivery sounds like spoken English, not an essay; questions and graphics match the talk.
- [ ] Across Parts 1–4, voices are intelligible, volume is stable, and punctuation/names/numbers/dates are pronounced acceptably.
- [ ] Interrupt audio mid-play, leave/reopen, then replay: no duplicate scoring, answer leak or unrecoverable state.

## 4. Reading human review — Parts 5–7

Sample at least **10 Part 5 items, 2 complete Part 6 sets and 3 complete Part 7 sets per Form A/B/C**. In Part 7 include one single, one double and one triple set per Form.

- [ ] Part 5: only one defensible completion; grammar/vocabulary label and explanations are accurate.
- [ ] Part 6: answers genuinely depend on passage context/cohesion; inserted sentences fit both surrounding sides.
- [ ] Part 7 single passages: every correct answer is supported and distractors are plausible but wrong.
- [ ] Part 7 double/triple passages: cross-document questions truly require 2+ documents and evidence/explanations identify the correct relationship.
- [ ] No copied-looking, templated, contradictory, unnatural or culturally unsafe item is observed.
- [ ] Learning feedback does not appear early; simulation feedback stays hidden until completion.

## 5. Family snapshot and resume

Perform these in the real app for **Reading B, Reading C, Full L&R B and Full L&R C**.

- [ ] Start the family, answer at least two questions, save/exit, close the app, reopen and resume.
- [ ] Family letter, form IDs/content, item order, answered choices and progress remain unchanged.
- [ ] A second parent attempt does not inherit answers, review items or child sessions from the first.

## 6. Complete manual simulations

These are the mandatory long-run checks. Use intentional mixed correct/incorrect answers so both review paths can be inspected.

- [ ] Complete one Full Reading (100 questions) from start to finish.
- [ ] Confirm no Reading estimate before question 100 and an explicitly unofficial estimate after 100.
- [ ] Review All contains exactly 100 items; Review Mistakes contains only wrong items; grouped Part 6/7 sets contain only their wrong questions in Mistakes.
- [ ] Complete one Full TOEIC L&R (200 questions) from start to finish.
- [ ] Confirm no Total estimate at 199 and an explicitly unofficial total after 200.
- [ ] Confirm Total estimate equals displayed Listening + Reading estimates.
- [ ] Review All contains exactly 200 items; Mistakes contains exactly the wrong first attempts and preserves Part/set grouping.
- [ ] History shows the completed parent once, with correct family, raw/estimate fields and completion status.

## 7. Score and content calibration judgment

- [ ] Easy/medium/hard progression feels broadly sensible within each Part and comparable across Forms A/B/C (do not claim psychometric equivalence).
- [ ] Answer positions do not feel predictably patterned during use.
- [ ] The unofficial score range/label is understandable and never presented as an ETS conversion.
- [ ] Explanations are useful, accurate, concise and do not reveal private answer data before locking.

## 8. Consolidated result to send back

Overall Human Gate: **PASS / FAIL**

Failures (duplicate this block for each issue):

- Severity: BLOCKER / HIGH / MEDIUM / LOW
- Area: Part/Form/parent screen
- Item or set ID/number:
- Exact steps:
- Expected:
- Observed:
- Audio/visual/content judgment:
- Screenshot or recording path:
- Reproducible after restart: YES / NO

Final counts:

- BLOCKER: ___
- HIGH: ___
- MEDIUM: ___
- LOW: ___
- Items listened/reviewed per Form: A ___ / B ___ / C ___
- Full Reading completed: YES / NO
- Full L&R completed: YES / NO

Phase 8 cannot be marked complete until required human checks pass. Phase 9 must not begin before these real results are returned and resolved.
