# Curriculum Manifest V1

Curriculum is bundled local metadata that organizes existing Guided Lesson Packages into Course → CEFR Level → Unit → exact Lesson reference. It is not lesson content, a prompt, a second lesson engine, learner CEFR, or progress storage.

## File structure

Production manifests live under `src-tauri/resources/curriculum/<curriculum-directory>/curriculum.json`. Referenced packages live independently under `src-tauri/resources/interactive-lessons/<package-directory>/lesson.json`. Test roots are separate under `src-tauri/test-fixtures` and are never loaded by the normal app.

All v1 constants are 1: Curriculum Schema, Registry, Progress, Recommendation and Taxonomy. Manifests are strict, at most 1 MiB and contain 1–6 Levels, at most 30 Units per Level, at most 30 Lessons per Unit and at most 500 Lesson references total.

## Identity and publication

- `curriculumId` and `unitId` are stable lowercase slugs using `a-z`, `0-9` and hyphens. Do not repurpose IDs.
- `curriculumVersion` is an integer ≥1. Increment it when a published structure or pinned Lesson version changes.
- `publicationState` is `draft` or `published`. Draft curricula do not appear in the human UI.
- A published Curriculum may reference only an installed, valid, published Guided Lesson at the exact `lessonId` and `contentVersion`.
- One invalid manifest is isolated and does not hide other valid curricula.

## Levels and Units

Levels use canonical pairs and ascending subset order only: `a1/A1`, `a2/A2`, `b1/B1`, `b2/B2`, `c1/C1`, `c2/C2`. Duplicate Levels are invalid. A Level has 1–30 Units and up to 12 plain-text objectives.

Unit IDs are unique across the entire Curriculum. A Unit contains 1–30 Lesson refs, up to 10 objectives, and controlled `skillFocus` values: `grammar`, `vocabulary`, `listening`, `pronunciation`, `speaking`, `interaction`. Grammar topics, vocabulary topics and communicative functions each allow up to 12 plain-text entries of at most 120 characters. Skill tags indicate predominant practice, never certification or assessment support.

## Exact Lesson references

Each ref contains only:

```json
{ "lessonId": "a1-saying-hello", "contentVersion": 1 }
```

Title, description, CEFR, duration, objectives, stages, assets, prompts, answers and answer keys remain sourced from the Guided Lesson Package. The same stable Lesson ID may not occur twice within one Curriculum, but different curricula may reuse it.

A published Curriculum pins an exact version so installing Lesson v2 cannot silently change Curriculum v1. To adopt v2, edit the draft manifest, validate it, increment `curriculumVersion`, then publish. A wording/content improvement may increment `contentVersion` while preserving `lessonId`; a significant redesign with different central objectives requires a new `lessonId`.

## Progress and Placement separation

Progress is derived from `interactive_lesson_session`, keyed by stable `lessonId`, and is not stored separately:

- any completed session → Completed;
- otherwise a matching active session → In progress;
- abandoned/failed sessions do not complete;
- multiple completions count once in percentage;
- every Lesson has weight 1; percentage uses nearest whole percent;
- Exercise, Pronunciation, Conversation and Analysis scores never affect progress;
- an officially completed session with partial Analysis counts;
- completion of v1 remains when Curriculum later pins v2; `Updated content available` may be shown;
- an active old-version session always resumes its immutable original snapshot.

Placement CEFR is an estimated learner level and only suggests the matching Course Level. Curriculum CEFR classifies content. Target Level is only a goal. Neither creates progress nor locks any Level. With no Placement there is no silent A1 recommendation.

## Authoring workflow

1. Create or update a Guided Lesson Package.
2. Validate and publish its exact `contentVersion`.
3. Add the exact ref to a draft Curriculum.
4. Validate IDs, canonical CEFR order, limits, duplicates and all cross-references.
5. Increment `curriculumVersion` for a changed published Course.
6. Change `publicationState` to `published` only when every exact ref is published and runtime-ready.

Adding Levels, Units or Lesson refs is a content-only change. Do not add Lesson-specific Rust/React branches, separate A1/A2 pages, prerequisites, locks, exams, certificates, AI recommendations, XP or learning-memory integrations.

## Common validation errors

Unsupported schema/version, invalid slug, noncanonical CEFR order or pair, duplicate Level/Unit/Lesson, unknown skill, collection/text limit, empty published hierarchy, missing exact Lesson, wrong contentVersion, draft Lesson referenced by a published Curriculum, CEFR mismatch, unknown fields or prompt/URL/asset fields cause that Curriculum to be omitted safely.
