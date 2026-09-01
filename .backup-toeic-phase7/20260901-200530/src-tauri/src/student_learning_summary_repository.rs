use crate::{
    database,
    learning_memory_repository::{
        LearningMemoryRepository, VocabularyFilter, VocabularySort, VocabularyStatus,
    },
    lesson_analysis::{LessonAnalysisPayload, ANALYSIS_SCHEMA_VERSION},
};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const STUDENT_LEARNING_SUMMARY_SCHEMA_VERSION: u32 = 1;
pub const TEACHER_MEMORY_CONTEXT_VERSION: u32 = 1;
pub const TEACHER_MEMORY_CONTEXT_MAX_CHARS: usize = 3_000;
pub const LEARNING_MEMORY_SETTING_KEY: &str = "use_learning_memory_in_lessons";
const PROFILE_KEY: &str = "default";
const NOW_SQL: &str = "strftime('%Y-%m-%dT%H:%M:%fZ', 'now')";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningStrength {
    pub title: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningFocusArea {
    pub area: String,
    pub title: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmedRecurringMistakeMemory {
    pub id: String,
    pub title: String,
    pub category: String,
    pub lesson_count: u32,
    pub occurrence_count: u32,
    pub example_original: String,
    pub example_corrected: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentVocabularyMemory {
    pub id: String,
    pub text: String,
    pub meaning: String,
    pub status: VocabularyStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceSnapshot {
    pub lesson_id: String,
    pub overall: i32,
    pub fluency: i32,
    pub grammar: i32,
    pub vocabulary: i32,
    pub comprehension: i32,
    pub interaction: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentLearningSummary {
    pub schema_version: u32,
    pub generated_at: String,
    pub analyzed_lesson_count: u32,
    pub completed_lesson_count: u32,
    pub recent_strengths: Vec<LearningStrength>,
    pub current_focus_areas: Vec<LearningFocusArea>,
    pub confirmed_recurring_mistakes: Vec<ConfirmedRecurringMistakeMemory>,
    pub recent_vocabulary: Vec<RecentVocabularyMemory>,
    pub next_lesson_recommendations: Vec<String>,
    pub latest_performance_snapshot: Option<PerformanceSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeacherMemorySnapshot {
    pub enabled: bool,
    pub context_loaded: bool,
    pub context_version: Option<u32>,
    pub summary_schema_version: u32,
    pub analyzed_lesson_count_used: u32,
}

#[derive(Clone)]
pub struct StudentLearningSummaryRepository {
    database: PathBuf,
    memory: LearningMemoryRepository,
}

struct AnalysisRow {
    lesson_id: String,
    raw_json: String,
    overall: i32,
    fluency: i32,
    grammar: i32,
    vocabulary: i32,
    comprehension: i32,
    interaction: i32,
}

impl StudentLearningSummaryRepository {
    pub fn new(database: PathBuf) -> Self {
        Self {
            memory: LearningMemoryRepository::new(database.clone()),
            database,
        }
    }

    pub fn refresh_summary(&self) -> Result<StudentLearningSummary, String> {
        let mut built = self.build_summary()?;
        if let Some(existing) = self.read_summary_unchecked()? {
            if existing.schema_version == STUDENT_LEARNING_SUMMARY_SCHEMA_VERSION
                && logical_summary_eq(&existing, &built)
            {
                return Ok(existing);
            }
        }
        let connection = database::open(&self.database)?;
        built.generated_at = connection
            .query_row(&format!("SELECT {NOW_SQL}"), [], |row| row.get(0))
            .map_err(|error| format!("Could not timestamp student learning summary: {error}"))?;
        let content_json = serde_json::to_string(&built)
            .map_err(|error| format!("Could not serialize student learning summary: {error}"))?;
        connection
            .execute(
                &format!(
                    "INSERT INTO student_learning_summary (
                       profile_key, schema_version, generated_at, analyzed_lesson_count,
                       completed_lesson_count, content_json, created_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, {NOW_SQL}, {NOW_SQL})
                     ON CONFLICT(profile_key) DO UPDATE SET
                       schema_version = excluded.schema_version,
                       generated_at = excluded.generated_at,
                       analyzed_lesson_count = excluded.analyzed_lesson_count,
                       completed_lesson_count = excluded.completed_lesson_count,
                       content_json = excluded.content_json,
                       updated_at = {NOW_SQL}"
                ),
                params![
                    PROFILE_KEY,
                    built.schema_version,
                    built.generated_at,
                    built.analyzed_lesson_count,
                    built.completed_lesson_count,
                    content_json,
                ],
            )
            .map_err(|error| format!("Could not persist student learning summary: {error}"))?;
        Ok(built)
    }

    #[cfg(test)]
    pub fn get_summary(&self) -> Result<Option<StudentLearningSummary>, String> {
        let summary = self.read_summary_unchecked()?;
        match summary {
            Some(value) if value.schema_version != STUDENT_LEARNING_SUMMARY_SCHEMA_VERSION => {
                Err(format!(
                    "Unsupported student learning summary schema {}; refresh is required.",
                    value.schema_version
                ))
            }
            value => Ok(value),
        }
    }

    pub fn get_memory_enabled(&self) -> Result<bool, String> {
        let connection = database::open(&self.database)?;
        let raw: Option<String> = connection
            .query_row(
                "SELECT value_json FROM settings WHERE key = ?1",
                [LEARNING_MEMORY_SETTING_KEY],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| format!("Could not read learning memory setting: {error}"))?;
        match raw {
            None => Ok(true),
            Some(value) => serde_json::from_str::<bool>(&value)
                .map_err(|error| format!("Learning memory setting is invalid: {error}")),
        }
    }

    pub fn set_memory_enabled(&self, enabled: bool) -> Result<bool, String> {
        let connection = database::open(&self.database)?;
        let value = serde_json::to_string(&enabled)
            .map_err(|error| format!("Could not serialize learning memory setting: {error}"))?;
        connection
            .execute(
                &format!(
                    "INSERT INTO settings(key, value_json, updated_at)
                     VALUES (?1, ?2, {NOW_SQL})
                     ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json,
                       updated_at = {NOW_SQL}"
                ),
                params![LEARNING_MEMORY_SETTING_KEY, value],
            )
            .map_err(|error| format!("Could not persist learning memory setting: {error}"))?;
        Ok(enabled)
    }

    pub fn record_lesson_snapshot(
        &self,
        lesson_id: &str,
        snapshot: &TeacherMemorySnapshot,
    ) -> Result<(), String> {
        let connection = database::open(&self.database)?;
        connection
            .execute(
                &format!(
                    "INSERT INTO lesson_teacher_memory (
                       lesson_id, memory_enabled, context_loaded, context_version,
                       summary_schema_version, analyzed_lesson_count_used, created_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, {NOW_SQL})
                     ON CONFLICT(lesson_id) DO NOTHING"
                ),
                params![
                    lesson_id,
                    snapshot.enabled,
                    snapshot.context_loaded,
                    snapshot.context_version,
                    snapshot.summary_schema_version,
                    snapshot.analyzed_lesson_count_used,
                ],
            )
            .map_err(|error| format!("Could not record lesson memory snapshot: {error}"))?;
        Ok(())
    }

    pub fn build_summary(&self) -> Result<StudentLearningSummary, String> {
        let connection = database::open(&self.database)?;
        let completed_lesson_count = connection
            .query_row(
                "SELECT COUNT(*) FROM lesson WHERE status = 'completed' AND student_turn_count > 0",
                [],
                |row| row.get(0),
            )
            .map_err(|error| format!("Could not count completed lessons for summary: {error}"))?;
        let analyses: Vec<AnalysisRow> = connection
            .prepare(
                "SELECT la.lesson_id, la.raw_json, la.overall_score, la.fluency_score,
                        la.grammar_score, la.vocabulary_score, la.comprehension_score,
                        la.interaction_score
                 FROM lesson_analysis la JOIN lesson l ON l.id = la.lesson_id
                 WHERE la.status = 'completed' AND la.raw_json IS NOT NULL
                   AND la.overall_score IS NOT NULL AND la.fluency_score IS NOT NULL
                   AND la.grammar_score IS NOT NULL AND la.vocabulary_score IS NOT NULL
                   AND la.comprehension_score IS NOT NULL AND la.interaction_score IS NOT NULL
                 ORDER BY l.started_at DESC, la.id DESC",
            )
            .map_err(|error| format!("Could not prepare summary analyses: {error}"))?
            .query_map([], |row| {
                Ok(AnalysisRow {
                    lesson_id: row.get(0)?,
                    raw_json: row.get(1)?,
                    overall: row.get(2)?,
                    fluency: row.get(3)?,
                    grammar: row.get(4)?,
                    vocabulary: row.get(5)?,
                    comprehension: row.get(6)?,
                    interaction: row.get(7)?,
                })
            })
            .map_err(|error| format!("Could not query summary analyses: {error}"))?
            .collect::<Result<_, _>>()
            .map_err(|error| format!("Could not read summary analysis: {error}"))?;
        drop(connection);

        let mut strengths = Vec::new();
        let mut focus = Vec::new();
        let mut recommendations = Vec::new();
        for row in &analyses {
            let payload: LessonAnalysisPayload =
                serde_json::from_str(&row.raw_json).map_err(|error| {
                    format!("Completed analysis JSON is invalid for summary: {error}")
                })?;
            if payload.schema_version != ANALYSIS_SCHEMA_VERSION {
                return Err(format!(
                    "Unsupported analysis schema {} while building summary.",
                    payload.schema_version
                ));
            }
            for item in payload.strengths {
                push_unique(&mut strengths, item.title, 3, |value| normalize(value));
            }
            for item in payload.priority_improvements {
                let candidate = LearningFocusArea {
                    area: collapse_whitespace(&item.area),
                    title: collapse_whitespace(&item.title),
                };
                push_unique(&mut focus, candidate, 3, |value| {
                    format!("{}|{}", normalize(&value.area), normalize(&value.title))
                });
            }
            for item in payload.next_lesson_recommendations {
                push_unique(
                    &mut recommendations,
                    collapse_whitespace(&item),
                    3,
                    |value| normalize(value),
                );
            }
        }

        let recent_strengths = strengths
            .into_iter()
            .map(|title| LearningStrength { title })
            .collect();
        let mut vocabulary_candidates = self
            .memory
            .list_vocabulary(
                VocabularyFilter::All,
                "",
                VocabularySort::RecentlySeen,
                100,
                0,
            )?
            .items;
        vocabulary_candidates.sort_by(|left, right| {
            right
                .last_seen_at
                .cmp(&left.last_seen_at)
                .then_with(|| right.lesson_count.cmp(&left.lesson_count))
                .then_with(|| right.occurrence_count.cmp(&left.occurrence_count))
                .then_with(|| left.text.to_lowercase().cmp(&right.text.to_lowercase()))
        });
        let recent_vocabulary = vocabulary_candidates
            .into_iter()
            .filter(|item| {
                matches!(
                    item.status,
                    VocabularyStatus::New | VocabularyStatus::Learning
                )
            })
            .take(6)
            .map(|item| RecentVocabularyMemory {
                id: item.id,
                text: item.text,
                meaning: item.meaning,
                status: item.status,
            })
            .collect();
        let mut confirmed_recurring_mistakes = Vec::new();
        for mistake in self.memory.list_recurring_mistakes(3)? {
            let Some(details) = self.memory.get_recurring_mistake(&mistake.id)? else {
                continue;
            };
            let Some(example) = details.occurrences.first() else {
                continue;
            };
            confirmed_recurring_mistakes.push(ConfirmedRecurringMistakeMemory {
                id: mistake.id,
                title: mistake.title,
                category: mistake.category,
                lesson_count: mistake.lesson_count,
                occurrence_count: mistake.occurrence_count,
                example_original: example.original.clone(),
                example_corrected: example.corrected.clone(),
            });
        }
        let latest_performance_snapshot = analyses.first().map(|row| PerformanceSnapshot {
            lesson_id: row.lesson_id.clone(),
            overall: row.overall,
            fluency: row.fluency,
            grammar: row.grammar,
            vocabulary: row.vocabulary,
            comprehension: row.comprehension,
            interaction: row.interaction,
        });

        Ok(StudentLearningSummary {
            schema_version: STUDENT_LEARNING_SUMMARY_SCHEMA_VERSION,
            generated_at: String::new(),
            analyzed_lesson_count: analyses.len() as u32,
            completed_lesson_count,
            recent_strengths,
            current_focus_areas: focus,
            confirmed_recurring_mistakes,
            recent_vocabulary,
            next_lesson_recommendations: recommendations,
            latest_performance_snapshot,
        })
    }

    fn read_summary_unchecked(&self) -> Result<Option<StudentLearningSummary>, String> {
        let connection = database::open(&self.database)?;
        let row: Option<(u32, String, u32, u32, String)> = connection
            .query_row(
                "SELECT schema_version, generated_at, analyzed_lesson_count,
                        completed_lesson_count, content_json
                 FROM student_learning_summary WHERE profile_key = ?1",
                [PROFILE_KEY],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("Could not read student learning summary: {error}"))?;
        let Some((schema, generated_at, analyzed, completed, raw)) = row else {
            return Ok(None);
        };
        let summary: StudentLearningSummary = serde_json::from_str(&raw)
            .map_err(|error| format!("Student learning summary JSON is invalid: {error}"))?;
        if summary.schema_version != schema
            || summary.generated_at != generated_at
            || summary.analyzed_lesson_count != analyzed
            || summary.completed_lesson_count != completed
        {
            return Err("Student learning summary columns and JSON disagree.".to_owned());
        }
        Ok(Some(summary))
    }
}

pub fn build_teacher_memory_context(summary: &StudentLearningSummary) -> Option<String> {
    if summary.recent_strengths.is_empty()
        && summary.current_focus_areas.is_empty()
        && summary.confirmed_recurring_mistakes.is_empty()
        && summary.recent_vocabulary.is_empty()
        && summary.next_lesson_recommendations.is_empty()
    {
        return None;
    }
    let mut strengths: Vec<String> = summary
        .recent_strengths
        .iter()
        .take(2)
        .map(|item| format!("- {}", item.title))
        .collect();
    let mut focus: Vec<String> = summary
        .current_focus_areas
        .iter()
        .take(3)
        .map(|item| format!("- {}: {}", item.area, item.title))
        .collect();
    let mut recurring: Vec<String> = summary
        .confirmed_recurring_mistakes
        .iter()
        .take(3)
        .map(|item| {
            format!(
                "- {} (confirmed across {} lessons). Student said: {:?}; better: {:?}",
                item.title, item.lesson_count, item.example_original, item.example_corrected
            )
        })
        .collect();
    let mut vocabulary: Vec<String> = summary
        .recent_vocabulary
        .iter()
        .take(6)
        .map(|item| {
            format!(
                "- {} — {} ({})",
                item.text,
                item.meaning,
                status_text(item.status)
            )
        })
        .collect();
    let mut recommendations: Vec<String> = summary
        .next_lesson_recommendations
        .iter()
        .take(2)
        .map(|item| format!("- {item}"))
        .collect();

    loop {
        let context = render_context(
            summary.analyzed_lesson_count,
            &recurring,
            &focus,
            &recommendations,
            &vocabulary,
            &strengths,
        );
        if context.chars().count() <= TEACHER_MEMORY_CONTEXT_MAX_CHARS {
            return Some(context);
        }
        if strengths.pop().is_some() {
            continue;
        }
        if vocabulary.pop().is_some() {
            continue;
        }
        if recommendations.pop().is_some() {
            continue;
        }
        if focus.pop().is_some() {
            continue;
        }
        if recurring.pop().is_some() {
            continue;
        }
        return None;
    }
}

fn render_context(
    analyzed_count: u32,
    recurring: &[String],
    focus: &[String],
    recommendations: &[String],
    vocabulary: &[String],
    strengths: &[String],
) -> String {
    let mut parts = vec![
        format!(
            "[LEARNING MEMORY v{TEACHER_MEMORY_CONTEXT_VERSION} - INTERNAL TEACHING CONTEXT]\nEvidence available: {analyzed_count} analyzed lesson(s).\nThe base conversation-teacher rules have priority over this memory. Use it only as background to teach naturally."
        ),
    ];
    push_section(&mut parts, "Confirmed recurring mistakes", recurring);
    push_section(&mut parts, "Current focus", focus);
    push_section(&mut parts, "Suggested focus", recommendations);
    push_section(
        &mut parts,
        "Recent vocabulary worth reinforcing",
        vocabulary,
    );
    push_section(&mut parts, "Recent strengths", strengths);
    parts.push(
        "Rules for using this memory:\n- Do not repeat this memory verbatim or present a memory dump.\n- Do not mention numeric scores unless the student asks.\n- Do not claim personal facts.\n- Do not call a mistake recurring unless it is listed as confirmed recurring.\n- Do not force old vocabulary into every response; use it naturally only when relevant.\n- Do not bring up past mistakes unless relevant.\n- If a confirmed recurring mistake appears again, correct it clearly without absolute language.\n- Continue following the base rule to ask exactly one question per response."
            .to_owned(),
    );
    parts.join("\n\n")
}

fn push_section(parts: &mut Vec<String>, title: &str, items: &[String]) {
    if !items.is_empty() {
        parts.push(format!("{title}:\n{}", items.join("\n")));
    }
}

fn status_text(status: VocabularyStatus) -> &'static str {
    match status {
        VocabularyStatus::New => "new",
        VocabularyStatus::Learning => "learning",
        VocabularyStatus::Known => "known",
    }
}

fn logical_summary_eq(left: &StudentLearningSummary, right: &StudentLearningSummary) -> bool {
    left.schema_version == right.schema_version
        && left.analyzed_lesson_count == right.analyzed_lesson_count
        && left.completed_lesson_count == right.completed_lesson_count
        && left.recent_strengths == right.recent_strengths
        && left.current_focus_areas == right.current_focus_areas
        && left.confirmed_recurring_mistakes == right.confirmed_recurring_mistakes
        && left.recent_vocabulary == right.recent_vocabulary
        && left.next_lesson_recommendations == right.next_lesson_recommendations
        && left.latest_performance_snapshot == right.latest_performance_snapshot
}

fn push_unique<T, F>(items: &mut Vec<T>, candidate: T, limit: usize, key: F)
where
    F: Fn(&T) -> String,
{
    if items.len() >= limit {
        return;
    }
    let candidate_key = key(&candidate);
    if !items.iter().any(|existing| key(existing) == candidate_key) {
        items.push(candidate);
    }
}

fn normalize(value: &str) -> String {
    collapse_whitespace(value).to_lowercase()
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::Path;

    fn repository() -> (PathBuf, PathBuf, StudentLearningSummaryRepository) {
        let directory =
            std::env::temp_dir().join(format!("english-ai-coach-summary-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("summary.sqlite3");
        database::migrate(&path).unwrap();
        (
            directory,
            path.clone(),
            StudentLearningSummaryRepository::new(path),
        )
    }

    fn seed_analysis(path: &Path, suffix: &str, date: &str, duplicate: bool) -> String {
        let connection = database::open(path).unwrap();
        let lesson_id = format!("lesson-{suffix}");
        let analysis_id = format!("analysis-{suffix}");
        connection
            .execute(
                "INSERT INTO lesson (
               id, started_at, ended_at, status, mode, student_turn_count, teacher_turn_count,
               correction_count, whisper_model, whisper_threads, ollama_model, piper_voice,
               voice_engine_version, created_at, updated_at
             ) VALUES (?1, ?2, ?2, 'completed', 'free_conversation', 3, 3, 1,
               'whisper', 12, 'qwen3.5:4b', 'lessac', 'voice-v2', ?2, ?2)",
                params![lesson_id, date],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO transcript_message (
               id, lesson_id, sequence_index, turn_index, role, text, source,
               engine_event_type, created_at
             ) VALUES (?1, ?2, 1, 1, 'student',
               'I like fishing with my friend John in Brazil.', 'test', 'transcript', ?3)",
                params![format!("message-{suffix}"), lesson_id, date],
            )
            .unwrap();
        let title = if duplicate {
            "  PREPOSIÇÃO NATURAL "
        } else {
            "Preposição natural"
        };
        let payload = json!({
            "schemaVersion": 1,
            "scores": { "fluency": 85, "grammar": 70, "vocabulary": 80, "comprehension": 90, "interaction": 80, "pronunciation": null },
            "strengths": [{ "title": "Boa abertura e engajamento", "evidence": "private evidence is not imported" }],
            "priorityImprovements": [{ "area": "word_choice", "title": title, "explanation": "x", "exampleFromLesson": "private", "betterAlternative": "private" }],
            "corrections": [{ "original": "I am terrible cooking.", "corrected": "I'm terrible at cooking.", "explanation": "Use at.", "category": "preposition" }],
            "naturalAlternatives": [],
            "vocabulary": [{ "wordOrPhrase": "terrible at", "meaning": "muito ruim em", "example": "I'm terrible at math." }],
            "recurringPatterns": [],
            "nextLessonRecommendations": ["Praticar preposições."],
            "summary": "summary", "pronunciationAvailable": false
        });
        connection
            .execute(
                "INSERT INTO lesson_analysis (
               id, lesson_id, status, schema_version, prompt_version, analyzer_model,
               overall_score, fluency_score, grammar_score, vocabulary_score,
               comprehension_score, interaction_score, raw_json, created_at, updated_at
             ) VALUES (?1, ?2, 'completed', 1, 1, 'qwen3.5:4b', 81, 85, 70, 80,
               90, 80, ?3, ?4, ?4)",
                params![analysis_id, lesson_id, payload.to_string(), date],
            )
            .unwrap();
        analysis_id
    }

    #[test]
    fn empty_summary_is_singleton_idempotent_and_has_no_context() {
        let (directory, path, repository) = repository();
        let first = repository.refresh_summary().unwrap();
        let second = repository.refresh_summary().unwrap();
        assert_eq!(first, second);
        assert_eq!(first.analyzed_lesson_count, 0);
        assert!(build_teacher_memory_context(&first).is_none());
        let connection = database::open(&path).unwrap();
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM student_learning_summary", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn one_analysis_builds_only_pedagogical_memory_and_deduplicates() {
        let (directory, path, repository) = repository();
        seed_analysis(&path, "old", "2026-01-01", false);
        seed_analysis(&path, "new", "2026-01-02", true);
        let memory = LearningMemoryRepository::new(path.clone());
        memory.sync_all_completed_analyses().unwrap();
        let summary = repository.refresh_summary().unwrap();
        assert_eq!(summary.analyzed_lesson_count, 2);
        assert_eq!(summary.recent_strengths.len(), 1);
        assert_eq!(summary.current_focus_areas.len(), 1);
        assert_eq!(summary.next_lesson_recommendations.len(), 1);
        assert_eq!(summary.recent_vocabulary.len(), 1);
        let serialized = serde_json::to_string(&summary).unwrap();
        for forbidden in ["fishing", "John", "Brazil", "private evidence"] {
            assert!(!serialized.contains(forbidden), "leaked {forbidden}");
        }
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn confirmed_recurring_requires_two_lessons_and_uses_real_examples() {
        let (directory, path, repository) = repository();
        seed_analysis(&path, "one", "2026-01-01", false);
        let memory = LearningMemoryRepository::new(path.clone());
        memory.sync_all_completed_analyses().unwrap();
        assert!(repository
            .build_summary()
            .unwrap()
            .confirmed_recurring_mistakes
            .is_empty());
        seed_analysis(&path, "two", "2026-01-02", false);
        memory.sync_all_completed_analyses().unwrap();
        let summary = repository.refresh_summary().unwrap();
        assert_eq!(summary.confirmed_recurring_mistakes.len(), 1);
        assert_eq!(summary.confirmed_recurring_mistakes[0].lesson_count, 2);
        let context = build_teacher_memory_context(&summary).unwrap();
        assert!(context.contains("I am terrible cooking."));
        assert!(context.contains("I'm terrible at cooking."));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn vocabulary_manual_status_changes_teacher_memory() {
        let (directory, path, repository) = repository();
        seed_analysis(&path, "one", "2026-01-01", false);
        let memory = LearningMemoryRepository::new(path.clone());
        memory.sync_all_completed_analyses().unwrap();
        let first = repository.refresh_summary().unwrap();
        assert_eq!(first.recent_vocabulary[0].status, VocabularyStatus::New);
        let vocabulary_id = first.recent_vocabulary[0].id.clone();
        memory
            .update_vocabulary_status(&vocabulary_id, VocabularyStatus::Learning)
            .unwrap();
        assert_eq!(
            repository.refresh_summary().unwrap().recent_vocabulary[0].status,
            VocabularyStatus::Learning
        );
        memory
            .update_vocabulary_status(&vocabulary_id, VocabularyStatus::Known)
            .unwrap();
        let known = repository.refresh_summary().unwrap();
        assert!(known.recent_vocabulary.is_empty());
        assert!(!build_teacher_memory_context(&known)
            .unwrap()
            .contains("terrible at —"));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn context_is_deterministic_bounded_and_contains_no_scores_or_transcript() {
        let mut summary = StudentLearningSummary {
            schema_version: 1,
            generated_at: "ignored".to_owned(),
            analyzed_lesson_count: 12,
            completed_lesson_count: 12,
            recent_strengths: (0..3)
                .map(|index| LearningStrength {
                    title: format!("Strength {index} {}", "x".repeat(800)),
                })
                .collect(),
            current_focus_areas: (0..3)
                .map(|index| LearningFocusArea {
                    area: "grammar".to_owned(),
                    title: format!("Focus {index} {}", "y".repeat(800)),
                })
                .collect(),
            confirmed_recurring_mistakes: vec![],
            recent_vocabulary: (0..6)
                .map(|index| RecentVocabularyMemory {
                    id: index.to_string(),
                    text: format!("word {index}"),
                    meaning: "z".repeat(600),
                    status: VocabularyStatus::New,
                })
                .collect(),
            next_lesson_recommendations: (0..3)
                .map(|index| format!("Recommendation {index} {}", "r".repeat(700)))
                .collect(),
            latest_performance_snapshot: Some(PerformanceSnapshot {
                lesson_id: "lesson".to_owned(),
                overall: 81,
                fluency: 85,
                grammar: 70,
                vocabulary: 80,
                comprehension: 90,
                interaction: 80,
            }),
        };
        let first = build_teacher_memory_context(&summary).unwrap();
        summary.generated_at = "different".to_owned();
        let second = build_teacher_memory_context(&summary).unwrap();
        assert_eq!(first, second);
        assert!(first.chars().count() <= TEACHER_MEMORY_CONTEXT_MAX_CHARS);
        for forbidden in ["overall", "grammar score", "81", "transcript"] {
            assert!(!first.to_lowercase().contains(forbidden));
        }
        assert!(first.contains("base conversation-teacher rules have priority"));
        assert!(first.contains("exactly one question"));
    }

    #[test]
    fn setting_defaults_on_persists_off_and_snapshot_records_only_metadata() {
        let (directory, path, repository) = repository();
        assert!(repository.get_memory_enabled().unwrap());
        assert!(!repository.set_memory_enabled(false).unwrap());
        drop(repository);
        let reopened = StudentLearningSummaryRepository::new(path.clone());
        assert!(!reopened.get_memory_enabled().unwrap());
        let connection = database::open(&path).unwrap();
        connection
            .execute(
                "INSERT INTO lesson (
               id, started_at, status, mode, whisper_model, whisper_threads, ollama_model,
               piper_voice, voice_engine_version, created_at, updated_at
             ) VALUES ('snapshot-lesson', 'now', 'starting', 'free_conversation', 'whisper',
               12, 'qwen', 'piper', 'v2', 'now', 'now')",
                [],
            )
            .unwrap();
        drop(connection);
        reopened
            .record_lesson_snapshot(
                "snapshot-lesson",
                &TeacherMemorySnapshot {
                    enabled: false,
                    context_loaded: false,
                    context_version: None,
                    summary_schema_version: 1,
                    analyzed_lesson_count_used: 0,
                },
            )
            .unwrap();
        let connection = database::open(&path).unwrap();
        let values: (bool, bool, Option<u32>, u32) = connection
            .query_row(
                "SELECT memory_enabled, context_loaded, context_version, analyzed_lesson_count_used
             FROM lesson_teacher_memory WHERE lesson_id = 'snapshot-lesson'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(values, (false, false, None, 0));
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn unknown_persisted_summary_schema_is_rebuilt_from_sources() {
        let (directory, path, repository) = repository();
        let connection = database::open(&path).unwrap();
        connection.execute(
            "INSERT INTO student_learning_summary (
               profile_key, schema_version, generated_at, analyzed_lesson_count,
               completed_lesson_count, content_json, created_at, updated_at
             ) VALUES ('default', 99, 'old', 0, 0,
               '{\"schemaVersion\":99,\"generatedAt\":\"old\",\"analyzedLessonCount\":0,\"completedLessonCount\":0,\"recentStrengths\":[],\"currentFocusAreas\":[],\"confirmedRecurringMistakes\":[],\"recentVocabulary\":[],\"nextLessonRecommendations\":[],\"latestPerformanceSnapshot\":null}',
               'old', 'old')", [],
        ).unwrap();
        drop(connection);
        let rebuilt = repository.refresh_summary().unwrap();
        assert_eq!(
            rebuilt.schema_version,
            STUDENT_LEARNING_SUMMARY_SCHEMA_VERSION
        );
        assert_eq!(repository.get_summary().unwrap().unwrap(), rebuilt);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    #[ignore = "manual migration and Phase H audit against the user's physical SQLite database"]
    fn physical_phase_h_summary_setting_and_integrity() {
        let local_app_data = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .expect("LOCALAPPDATA");
        let path = local_app_data
            .join("com.englishaicoach.desktop")
            .join("database")
            .join("english-ai-coach.sqlite3");
        database::migrate(&path).expect("migrate physical database");
        let memory = LearningMemoryRepository::new(path.clone());
        memory
            .sync_all_completed_analyses()
            .expect("sync physical learning memory");
        let repository = StudentLearningSummaryRepository::new(path.clone());
        let summary = repository
            .refresh_summary()
            .expect("refresh physical learning summary");
        assert_eq!(
            summary.schema_version,
            STUDENT_LEARNING_SUMMARY_SCHEMA_VERSION
        );
        assert!(summary.analyzed_lesson_count >= 1);
        assert!(!summary.recent_strengths.is_empty());
        assert!(!summary.current_focus_areas.is_empty());
        assert!(!summary.next_lesson_recommendations.is_empty());
        assert!(summary
            .recent_vocabulary
            .iter()
            .any(|item| item.text.eq_ignore_ascii_case("terrible at")));
        assert!(summary.confirmed_recurring_mistakes.is_empty());

        let context = build_teacher_memory_context(&summary).expect("physical teacher context");
        assert!(context.chars().count() <= TEACHER_MEMORY_CONTEXT_MAX_CHARS);
        assert!(!context.contains("overallScore"));
        assert!(!context.contains("grammar_score"));

        let original_setting = repository.get_memory_enabled().expect("read setting");
        repository.set_memory_enabled(false).expect("persist off");
        assert!(!StudentLearningSummaryRepository::new(path.clone())
            .get_memory_enabled()
            .expect("reopen off"));
        repository.set_memory_enabled(true).expect("persist on");
        assert!(StudentLearningSummaryRepository::new(path.clone())
            .get_memory_enabled()
            .expect("reopen on"));
        repository
            .set_memory_enabled(original_setting)
            .expect("restore setting");

        let connection = database::open(&path).expect("open physical database");
        let row_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM student_learning_summary", [], |row| {
                row.get(0)
            })
            .expect("summary row count");
        let raw: String = connection
            .query_row(
                "SELECT content_json FROM student_learning_summary WHERE profile_key = 'default'",
                [],
                |row| row.get(0),
            )
            .expect("summary JSON");
        let _: StudentLearningSummary = serde_json::from_str(&raw).expect("typed summary JSON");
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .expect("integrity check");
        let foreign_key_errors: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .expect("foreign key check");
        assert_eq!(row_count, 1);
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_key_errors, 0);
        println!(
            "{}",
            json!({
                "schemaVersion": summary.schema_version,
                "analyzedLessonCount": summary.analyzed_lesson_count,
                "completedLessonCount": summary.completed_lesson_count,
                "strengths": summary.recent_strengths,
                "focus": summary.current_focus_areas,
                "recurringConfirmed": summary.confirmed_recurring_mistakes.len(),
                "vocabulary": summary.recent_vocabulary,
                "recommendations": summary.next_lesson_recommendations,
                "contextLength": context.chars().count(),
                "integrity": integrity,
                "foreignKeyErrors": foreign_key_errors,
            })
        );
    }
}
