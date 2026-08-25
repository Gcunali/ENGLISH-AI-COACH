use crate::{
    database,
    lesson_analysis::{
        LessonAnalysisCorrection, LessonAnalysisCorrectionCategory, LessonAnalysisPayload,
        ANALYSIS_SCHEMA_VERSION,
    },
};
use rusqlite::{params, OptionalExtension, Row, Transaction};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const NOW_SQL: &str = "strftime('%Y-%m-%dT%H:%M:%fZ', 'now')";
const MAX_PAGE_SIZE: u32 = 100;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VocabularyStatus {
    New,
    Learning,
    Known,
}

impl VocabularyStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Learning => "learning",
            Self::Known => "known",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VocabularyFilter {
    All,
    New,
    Learning,
    Known,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VocabularySort {
    RecentlySeen,
    FirstSeen,
    MostFrequent,
    Alphabetical,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularySummary {
    pub total: u32,
    pub new: u32,
    pub learning: u32,
    pub known: u32,
    pub contributing_lessons: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningMemorySummary {
    pub vocabulary_total: u32,
    pub vocabulary_new: u32,
    pub vocabulary_learning: u32,
    pub vocabulary_known: u32,
    pub lessons_contributing_vocabulary: u32,
    pub recurring_mistakes_confirmed: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyItemDto {
    pub id: String,
    pub text: String,
    pub meaning: String,
    pub status: VocabularyStatus,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub lesson_count: u32,
    pub occurrence_count: u32,
    pub latest_example: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyPage {
    pub items: Vec<VocabularyItemDto>,
    pub total: u32,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyOccurrenceDto {
    pub lesson_id: String,
    pub lesson_date: String,
    pub example: String,
    pub occurrence_count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyItemDetails {
    pub item: VocabularyItemDto,
    pub occurrences: Vec<VocabularyOccurrenceDto>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringMistakeDto {
    pub id: String,
    pub category: String,
    pub title: String,
    pub explanation: String,
    pub lesson_count: u32,
    pub occurrence_count: u32,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MistakeOccurrenceDto {
    pub lesson_id: String,
    pub lesson_date: String,
    pub original: String,
    pub corrected: String,
    pub explanation: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringMistakeDetails {
    pub mistake: RecurringMistakeDto,
    pub occurrences: Vec<MistakeOccurrenceDto>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningMemorySyncResult {
    pub synchronized: u32,
    pub failed: u32,
    pub errors: Vec<String>,
}

#[derive(Clone)]
pub struct LearningMemoryRepository {
    database: PathBuf,
}

struct AnalysisSource {
    id: String,
    lesson_id: String,
    lesson_started_at: String,
    schema_version: u32,
    raw_json: String,
}

struct AggregatedVocabulary {
    canonical: String,
    display: String,
    meaning: String,
    example: String,
    occurrences: u32,
}

impl LearningMemoryRepository {
    pub fn new(database: PathBuf) -> Self {
        Self { database }
    }

    pub fn sync_analysis(&self, analysis_id: &str) -> Result<(), String> {
        let mut connection = database::open(&self.database)?;
        let source = connection
            .query_row(
                "SELECT la.id, la.lesson_id, l.started_at, la.schema_version, la.raw_json
                 FROM lesson_analysis la
                 JOIN lesson l ON l.id = la.lesson_id
                 WHERE la.id = ?1 AND la.status = 'completed' AND la.raw_json IS NOT NULL",
                [analysis_id],
                |row| {
                    Ok(AnalysisSource {
                        id: row.get(0)?,
                        lesson_id: row.get(1)?,
                        lesson_started_at: row.get(2)?,
                        schema_version: row.get(3)?,
                        raw_json: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(|error| format!("Could not read completed analysis for memory sync: {error}"))?
            .ok_or_else(|| {
                "Only a completed analysis with validated JSON can be synchronized.".to_owned()
            })?;
        let payload: LessonAnalysisPayload = serde_json::from_str(&source.raw_json)
            .map_err(|error| format!("Completed analysis JSON is invalid: {error}"))?;
        if source.schema_version != ANALYSIS_SCHEMA_VERSION
            || payload.schema_version != ANALYSIS_SCHEMA_VERSION
        {
            return Err(format!(
                "Unsupported analysis schema for memory sync: database={}, payload={}, supported={}.",
                source.schema_version, payload.schema_version, ANALYSIS_SCHEMA_VERSION
            ));
        }

        let vocabulary = aggregate_vocabulary(&payload);
        let transaction = connection
            .transaction()
            .map_err(|error| format!("Could not start learning memory transaction: {error}"))?;
        sync_vocabulary(&transaction, &source, &vocabulary)?;
        sync_mistakes(&transaction, &source, &payload.corrections)?;
        transaction
            .commit()
            .map_err(|error| format!("Could not commit learning memory sync: {error}"))
    }

    pub fn sync_all_completed_analyses(&self) -> Result<LearningMemorySyncResult, String> {
        let connection = database::open(&self.database)?;
        let analysis_ids: Vec<String> = connection
            .prepare(
                "SELECT id FROM lesson_analysis
                 WHERE status = 'completed' AND raw_json IS NOT NULL
                 ORDER BY created_at, id",
            )
            .map_err(|error| format!("Could not prepare historical memory sync: {error}"))?
            .query_map([], |row| row.get(0))
            .map_err(|error| format!("Could not list historical analyses: {error}"))?
            .collect::<Result<_, _>>()
            .map_err(|error| format!("Could not read historical analysis id: {error}"))?;
        drop(connection);

        let mut result = LearningMemorySyncResult::default();
        for analysis_id in analysis_ids {
            match self.sync_analysis(&analysis_id) {
                Ok(()) => result.synchronized += 1,
                Err(error) => {
                    result.failed += 1;
                    result.errors.push(format!("{analysis_id}: {error}"));
                }
            }
        }
        Ok(result)
    }

    pub fn vocabulary_summary(&self) -> Result<VocabularySummary, String> {
        let connection = database::open(&self.database)?;
        connection
            .query_row(
                "SELECT COUNT(*),
                        COUNT(*) FILTER (WHERE status = 'new'),
                        COUNT(*) FILTER (WHERE status = 'learning'),
                        COUNT(*) FILTER (WHERE status = 'known'),
                        (SELECT COUNT(DISTINCT lesson_id) FROM lesson_vocabulary)
                 FROM vocabulary_item",
                [],
                |row| {
                    Ok(VocabularySummary {
                        total: row.get(0)?,
                        new: row.get(1)?,
                        learning: row.get(2)?,
                        known: row.get(3)?,
                        contributing_lessons: row.get(4)?,
                    })
                },
            )
            .map_err(|error| format!("Could not load vocabulary summary: {error}"))
    }

    pub fn summary(&self) -> Result<LearningMemorySummary, String> {
        let vocabulary = self.vocabulary_summary()?;
        let connection = database::open(&self.database)?;
        let recurring_mistakes_confirmed = connection
            .query_row(
                "SELECT COUNT(*) FROM recurring_mistake WHERE lesson_count >= 2",
                [],
                |row| row.get(0),
            )
            .map_err(|error| format!("Could not count confirmed recurring mistakes: {error}"))?;
        Ok(LearningMemorySummary {
            vocabulary_total: vocabulary.total,
            vocabulary_new: vocabulary.new,
            vocabulary_learning: vocabulary.learning,
            vocabulary_known: vocabulary.known,
            lessons_contributing_vocabulary: vocabulary.contributing_lessons,
            recurring_mistakes_confirmed,
        })
    }

    pub fn list_vocabulary(
        &self,
        filter: VocabularyFilter,
        search: &str,
        sort: VocabularySort,
        limit: u32,
        offset: u32,
    ) -> Result<VocabularyPage, String> {
        let connection = database::open(&self.database)?;
        let limit = limit.clamp(1, MAX_PAGE_SIZE);
        let search = escape_like(search.trim());
        let condition = vocabulary_condition(filter);
        let order = vocabulary_order(sort);
        let where_sql = format!(
            "{condition} AND (?1 = '' OR lower(v.display_text) LIKE '%' || lower(?1) || '%' ESCAPE '\\'
             OR lower(v.meaning) LIKE '%' || lower(?1) || '%' ESCAPE '\\')"
        );
        let total = connection
            .query_row(
                &format!("SELECT COUNT(*) FROM vocabulary_item v WHERE {where_sql}"),
                [&search],
                |row| row.get(0),
            )
            .map_err(|error| format!("Could not count vocabulary items: {error}"))?;
        let sql = format!(
            "SELECT v.id, v.display_text, v.meaning, v.status, v.first_seen_at, v.last_seen_at,
                    v.lesson_count, v.occurrence_count,
                    (SELECT lv.example FROM lesson_vocabulary lv
                     JOIN lesson l ON l.id = lv.lesson_id
                     WHERE lv.vocabulary_item_id = v.id
                     ORDER BY l.started_at DESC, lv.id DESC LIMIT 1)
             FROM vocabulary_item v WHERE {where_sql} ORDER BY {order} LIMIT ?2 OFFSET ?3"
        );
        let items = connection
            .prepare(&sql)
            .map_err(|error| format!("Could not prepare vocabulary list: {error}"))?
            .query_map(params![search, limit, offset], vocabulary_row)
            .map_err(|error| format!("Could not query vocabulary items: {error}"))?
            .collect::<Result<_, _>>()
            .map_err(|error| format!("Could not read vocabulary item: {error}"))?;
        Ok(VocabularyPage {
            items,
            total,
            limit,
            offset,
        })
    }

    pub fn get_vocabulary_item(
        &self,
        vocabulary_id: &str,
    ) -> Result<Option<VocabularyItemDetails>, String> {
        let connection = database::open(&self.database)?;
        let item = connection
            .query_row(
                "SELECT v.id, v.display_text, v.meaning, v.status, v.first_seen_at, v.last_seen_at,
                        v.lesson_count, v.occurrence_count,
                        (SELECT lv.example FROM lesson_vocabulary lv
                         JOIN lesson l ON l.id = lv.lesson_id
                         WHERE lv.vocabulary_item_id = v.id
                         ORDER BY l.started_at DESC, lv.id DESC LIMIT 1)
                 FROM vocabulary_item v WHERE v.id = ?1",
                [vocabulary_id],
                vocabulary_row,
            )
            .optional()
            .map_err(|error| format!("Could not load vocabulary item: {error}"))?;
        let Some(item) = item else { return Ok(None) };
        let occurrences = connection
            .prepare(
                "SELECT lv.lesson_id, l.started_at, lv.example, lv.occurrence_count
                 FROM lesson_vocabulary lv JOIN lesson l ON l.id = lv.lesson_id
                 WHERE lv.vocabulary_item_id = ?1
                 ORDER BY l.started_at DESC, lv.id DESC",
            )
            .map_err(|error| format!("Could not prepare vocabulary occurrences: {error}"))?
            .query_map([vocabulary_id], |row| {
                Ok(VocabularyOccurrenceDto {
                    lesson_id: row.get(0)?,
                    lesson_date: row.get(1)?,
                    example: row.get(2)?,
                    occurrence_count: row.get(3)?,
                })
            })
            .map_err(|error| format!("Could not query vocabulary occurrences: {error}"))?
            .collect::<Result<_, _>>()
            .map_err(|error| format!("Could not read vocabulary occurrence: {error}"))?;
        Ok(Some(VocabularyItemDetails { item, occurrences }))
    }

    pub fn update_vocabulary_status(
        &self,
        vocabulary_id: &str,
        status: VocabularyStatus,
    ) -> Result<VocabularyItemDto, String> {
        let connection = database::open(&self.database)?;
        let changed = connection
            .execute(
                &format!(
                    "UPDATE vocabulary_item SET status = ?2, updated_at = {NOW_SQL} WHERE id = ?1"
                ),
                params![vocabulary_id, status.as_str()],
            )
            .map_err(|error| format!("Could not update vocabulary status: {error}"))?;
        if changed == 0 {
            return Err("Vocabulary item was not found.".to_owned());
        }
        self.get_vocabulary_item(vocabulary_id)?
            .map(|details| details.item)
            .ok_or_else(|| "Updated vocabulary item could not be read back.".to_owned())
    }

    pub fn list_recurring_mistakes(&self, limit: u32) -> Result<Vec<RecurringMistakeDto>, String> {
        let connection = database::open(&self.database)?;
        let mistakes = connection
            .prepare(
                "SELECT id, category, title, explanation, lesson_count, occurrence_count,
                        first_seen_at, last_seen_at, status
                 FROM recurring_mistake WHERE lesson_count >= 2
                 ORDER BY lesson_count DESC, last_seen_at DESC, title ASC LIMIT ?1",
            )
            .map_err(|error| format!("Could not prepare recurring mistakes: {error}"))?
            .query_map([limit.clamp(1, MAX_PAGE_SIZE)], mistake_row)
            .map_err(|error| format!("Could not query recurring mistakes: {error}"))?
            .collect::<Result<_, _>>()
            .map_err(|error| format!("Could not read recurring mistake: {error}"))?;
        Ok(mistakes)
    }

    pub fn get_recurring_mistake(
        &self,
        mistake_id: &str,
    ) -> Result<Option<RecurringMistakeDetails>, String> {
        let connection = database::open(&self.database)?;
        let mistake = connection
            .query_row(
                "SELECT id, category, title, explanation, lesson_count, occurrence_count,
                        first_seen_at, last_seen_at, status
                 FROM recurring_mistake WHERE id = ?1 AND lesson_count >= 2",
                [mistake_id],
                mistake_row,
            )
            .optional()
            .map_err(|error| format!("Could not load recurring mistake: {error}"))?;
        let Some(mistake) = mistake else {
            return Ok(None);
        };
        let occurrences = connection
            .prepare(
                "SELECT o.lesson_id, l.started_at, o.original, o.corrected, o.explanation
                 FROM recurring_mistake_occurrence o JOIN lesson l ON l.id = o.lesson_id
                 WHERE o.recurring_mistake_id = ?1
                 ORDER BY l.started_at DESC, o.source_index",
            )
            .map_err(|error| format!("Could not prepare mistake occurrences: {error}"))?
            .query_map([mistake_id], |row| {
                Ok(MistakeOccurrenceDto {
                    lesson_id: row.get(0)?,
                    lesson_date: row.get(1)?,
                    original: row.get(2)?,
                    corrected: row.get(3)?,
                    explanation: row.get(4)?,
                })
            })
            .map_err(|error| format!("Could not query mistake occurrences: {error}"))?
            .collect::<Result<_, _>>()
            .map_err(|error| format!("Could not read mistake occurrence: {error}"))?;
        Ok(Some(RecurringMistakeDetails {
            mistake,
            occurrences,
        }))
    }
}

fn sync_vocabulary(
    transaction: &Transaction<'_>,
    source: &AnalysisSource,
    vocabulary: &[AggregatedVocabulary],
) -> Result<(), String> {
    for entry in vocabulary {
        transaction
            .execute(
                &format!(
                    "INSERT INTO vocabulary_item (
                       id, canonical_text, display_text, meaning, first_seen_at, last_seen_at,
                       lesson_count, occurrence_count, status, created_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?5, 0, 0, 'new', {NOW_SQL}, {NOW_SQL})
                     ON CONFLICT(canonical_text) DO NOTHING"
                ),
                params![
                    uuid::Uuid::new_v4().to_string(),
                    entry.canonical,
                    entry.display,
                    entry.meaning,
                    source.lesson_started_at,
                ],
            )
            .map_err(|error| format!("Could not upsert vocabulary item: {error}"))?;
        let vocabulary_id: String = transaction
            .query_row(
                "SELECT id FROM vocabulary_item WHERE canonical_text = ?1",
                [&entry.canonical],
                |row| row.get(0),
            )
            .map_err(|error| format!("Could not read synchronized vocabulary id: {error}"))?;
        transaction
            .execute(
                &format!(
                    "INSERT INTO lesson_vocabulary (
                       id, lesson_id, vocabulary_item_id, source_analysis_id, example,
                       occurrence_count, created_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, {NOW_SQL})
                     ON CONFLICT(lesson_id, vocabulary_item_id) DO UPDATE SET
                       source_analysis_id = excluded.source_analysis_id,
                       example = excluded.example,
                       occurrence_count = excluded.occurrence_count"
                ),
                params![
                    uuid::Uuid::new_v4().to_string(),
                    source.lesson_id,
                    vocabulary_id,
                    source.id,
                    entry.example,
                    entry.occurrences,
                ],
            )
            .map_err(|error| format!("Could not link lesson vocabulary: {error}"))?;
        recalculate_vocabulary(transaction, &vocabulary_id)?;
    }
    Ok(())
}

fn sync_mistakes(
    transaction: &Transaction<'_>,
    source: &AnalysisSource,
    corrections: &[LessonAnalysisCorrection],
) -> Result<(), String> {
    for (source_index, correction) in corrections.iter().enumerate() {
        let signature = derive_mistake_signature(correction);
        let category = correction_category(correction.category);
        let title = derive_mistake_title(correction);
        transaction
            .execute(
                &format!(
                    "INSERT INTO recurring_mistake (
                       id, signature, category, title, explanation, first_seen_at, last_seen_at,
                       lesson_count, occurrence_count, status, created_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, 0, 0, 'active', {NOW_SQL}, {NOW_SQL})
                     ON CONFLICT(signature) DO NOTHING"
                ),
                params![
                    uuid::Uuid::new_v4().to_string(),
                    signature,
                    category,
                    title,
                    correction.explanation,
                    source.lesson_started_at,
                ],
            )
            .map_err(|error| format!("Could not upsert mistake candidate: {error}"))?;
        let mistake_id: String = transaction
            .query_row(
                "SELECT id FROM recurring_mistake WHERE signature = ?1",
                [&signature],
                |row| row.get(0),
            )
            .map_err(|error| format!("Could not read synchronized mistake id: {error}"))?;
        transaction
            .execute(
                &format!(
                    "INSERT INTO recurring_mistake_occurrence (
                       id, recurring_mistake_id, lesson_id, analysis_id, source_index,
                       original, corrected, explanation, created_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, {NOW_SQL})
                     ON CONFLICT(analysis_id, source_index) DO NOTHING"
                ),
                params![
                    uuid::Uuid::new_v4().to_string(),
                    mistake_id,
                    source.lesson_id,
                    source.id,
                    source_index as u32,
                    correction.original,
                    correction.corrected,
                    correction.explanation,
                ],
            )
            .map_err(|error| format!("Could not persist mistake occurrence: {error}"))?;
        recalculate_mistake(transaction, &mistake_id)?;
    }
    Ok(())
}

fn recalculate_vocabulary(
    transaction: &Transaction<'_>,
    vocabulary_id: &str,
) -> Result<(), String> {
    transaction
        .execute(
            &format!(
                "UPDATE vocabulary_item SET
                   lesson_count = (SELECT COUNT(DISTINCT lesson_id) FROM lesson_vocabulary WHERE vocabulary_item_id = ?1),
                   occurrence_count = (SELECT COALESCE(SUM(occurrence_count), 0) FROM lesson_vocabulary WHERE vocabulary_item_id = ?1),
                   first_seen_at = (SELECT MIN(l.started_at) FROM lesson_vocabulary lv JOIN lesson l ON l.id = lv.lesson_id WHERE lv.vocabulary_item_id = ?1),
                   last_seen_at = (SELECT MAX(l.started_at) FROM lesson_vocabulary lv JOIN lesson l ON l.id = lv.lesson_id WHERE lv.vocabulary_item_id = ?1),
                   updated_at = {NOW_SQL}
                 WHERE id = ?1"
            ),
            [vocabulary_id],
        )
        .map_err(|error| format!("Could not recalculate vocabulary counters: {error}"))?;
    Ok(())
}

fn recalculate_mistake(transaction: &Transaction<'_>, mistake_id: &str) -> Result<(), String> {
    transaction
        .execute(
            &format!(
                "UPDATE recurring_mistake SET
                   lesson_count = (SELECT COUNT(DISTINCT lesson_id) FROM recurring_mistake_occurrence WHERE recurring_mistake_id = ?1),
                   occurrence_count = (SELECT COUNT(*) FROM recurring_mistake_occurrence WHERE recurring_mistake_id = ?1),
                   first_seen_at = (SELECT MIN(l.started_at) FROM recurring_mistake_occurrence o JOIN lesson l ON l.id = o.lesson_id WHERE o.recurring_mistake_id = ?1),
                   last_seen_at = (SELECT MAX(l.started_at) FROM recurring_mistake_occurrence o JOIN lesson l ON l.id = o.lesson_id WHERE o.recurring_mistake_id = ?1),
                   updated_at = {NOW_SQL}
                 WHERE id = ?1"
            ),
            [mistake_id],
        )
        .map_err(|error| format!("Could not recalculate mistake counters: {error}"))?;
    Ok(())
}

fn aggregate_vocabulary(payload: &LessonAnalysisPayload) -> Vec<AggregatedVocabulary> {
    let mut result: Vec<AggregatedVocabulary> = Vec::new();
    for item in &payload.vocabulary {
        let display = collapse_whitespace(&item.word_or_phrase);
        let canonical = normalize_vocabulary_key(&display);
        if let Some(existing) = result.iter_mut().find(|entry| entry.canonical == canonical) {
            existing.occurrences += 1;
            continue;
        }
        result.push(AggregatedVocabulary {
            canonical,
            display,
            meaning: item.meaning.trim().to_owned(),
            example: item.example.trim().to_owned(),
            occurrences: 1,
        });
    }
    result
}

pub(crate) fn normalize_vocabulary_key(value: &str) -> String {
    collapse_whitespace(value).to_lowercase()
}

pub(crate) fn derive_mistake_signature(correction: &LessonAnalysisCorrection) -> String {
    format!(
        "{}|{}|{}",
        correction_category(correction.category),
        normalize_signature_text(&correction.original),
        normalize_signature_text(&correction.corrected),
    )
}

fn normalize_signature_text(value: &str) -> String {
    collapse_whitespace(value).to_lowercase()
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn correction_category(category: LessonAnalysisCorrectionCategory) -> &'static str {
    match category {
        LessonAnalysisCorrectionCategory::Grammar => "grammar",
        LessonAnalysisCorrectionCategory::Vocabulary => "vocabulary",
        LessonAnalysisCorrectionCategory::WordChoice => "word_choice",
        LessonAnalysisCorrectionCategory::VerbTense => "verb_tense",
        LessonAnalysisCorrectionCategory::Preposition => "preposition",
        LessonAnalysisCorrectionCategory::Article => "article",
        LessonAnalysisCorrectionCategory::WordOrder => "word_order",
        LessonAnalysisCorrectionCategory::Naturalness => "naturalness",
        LessonAnalysisCorrectionCategory::Other => "other",
    }
}

fn derive_mistake_title(correction: &LessonAnalysisCorrection) -> String {
    let category = correction_category(correction.category).replace('_', " ");
    let corrected = collapse_whitespace(&correction.corrected);
    let concise = corrected.chars().take(100).collect::<String>();
    format!("{}: \"{}\"", humanize_ascii(&category), concise)
}

fn humanize_ascii(value: &str) -> String {
    let mut characters = value.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => String::new(),
    }
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn vocabulary_condition(filter: VocabularyFilter) -> &'static str {
    match filter {
        VocabularyFilter::All => "1 = 1",
        VocabularyFilter::New => "v.status = 'new'",
        VocabularyFilter::Learning => "v.status = 'learning'",
        VocabularyFilter::Known => "v.status = 'known'",
    }
}

fn vocabulary_order(sort: VocabularySort) -> &'static str {
    match sort {
        VocabularySort::RecentlySeen => "v.last_seen_at DESC, v.display_text COLLATE NOCASE ASC",
        VocabularySort::FirstSeen => "v.first_seen_at ASC, v.display_text COLLATE NOCASE ASC",
        VocabularySort::MostFrequent => {
            "v.lesson_count DESC, v.occurrence_count DESC, v.last_seen_at DESC"
        }
        VocabularySort::Alphabetical => "v.display_text COLLATE NOCASE ASC",
    }
}

fn vocabulary_row(row: &Row<'_>) -> rusqlite::Result<VocabularyItemDto> {
    let status: String = row.get(3)?;
    let status = match status.as_str() {
        "new" => VocabularyStatus::New,
        "learning" => VocabularyStatus::Learning,
        "known" => VocabularyStatus::Known,
        _ => {
            return Err(rusqlite::Error::InvalidColumnType(
                3,
                "status".to_owned(),
                rusqlite::types::Type::Text,
            ))
        }
    };
    Ok(VocabularyItemDto {
        id: row.get(0)?,
        text: row.get(1)?,
        meaning: row.get(2)?,
        status,
        first_seen_at: row.get(4)?,
        last_seen_at: row.get(5)?,
        lesson_count: row.get(6)?,
        occurrence_count: row.get(7)?,
        latest_example: row.get(8)?,
    })
}

fn mistake_row(row: &Row<'_>) -> rusqlite::Result<RecurringMistakeDto> {
    Ok(RecurringMistakeDto {
        id: row.get(0)?,
        category: row.get(1)?,
        title: row.get(2)?,
        explanation: row.get(3)?,
        lesson_count: row.get(4)?,
        occurrence_count: row.get(5)?,
        first_seen_at: row.get(6)?,
        last_seen_at: row.get(7)?,
        status: row.get(8)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::Path;

    fn repository() -> (PathBuf, PathBuf, LearningMemoryRepository) {
        let directory =
            std::env::temp_dir().join(format!("english-ai-coach-memory-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("memory.sqlite3");
        database::migrate(&path).unwrap();
        (directory, path.clone(), LearningMemoryRepository::new(path))
    }

    fn seed_analysis(
        path: &Path,
        suffix: &str,
        date: &str,
        vocabulary: serde_json::Value,
        corrections: serde_json::Value,
        schema: u32,
    ) -> String {
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
        let payload = json!({
            "schemaVersion": schema,
            "scores": { "fluency": 80, "grammar": 70, "vocabulary": 75, "comprehension": 90, "interaction": 85, "pronunciation": null },
            "strengths": [], "priorityImprovements": [], "corrections": corrections,
            "naturalAlternatives": [], "vocabulary": vocabulary, "recurringPatterns": [],
            "nextLessonRecommendations": [], "summary": "Test summary", "pronunciationAvailable": false
        });
        connection
            .execute(
                "INSERT INTO lesson_analysis (
               id, lesson_id, status, schema_version, prompt_version, analyzer_model,
               overall_score, raw_json, created_at, updated_at
             ) VALUES (?1, ?2, 'completed', ?3, 1, 'qwen3.5:4b', 80, ?4, ?5, ?5)",
                params![analysis_id, lesson_id, schema, payload.to_string(), date],
            )
            .unwrap();
        analysis_id
    }

    fn vocabulary(text: &str, meaning: &str, example: &str) -> serde_json::Value {
        json!([{ "wordOrPhrase": text, "meaning": meaning, "example": example }])
    }

    fn correction(original: &str, corrected: &str, category: &str) -> serde_json::Value {
        json!([{ "original": original, "corrected": corrected, "explanation": "Explicação real.", "category": category }])
    }

    #[test]
    fn vocabulary_normalization_is_cosmetic_and_preserves_distinct_phrases() {
        assert_eq!(normalize_vocabulary_key("Terrible at"), "terrible at");
        assert_eq!(normalize_vocabulary_key(" terrible   at "), "terrible at");
        assert_ne!(normalize_vocabulary_key("terrible"), "terrible at");
        assert_eq!(normalize_vocabulary_key("I'm used to"), "i'm used to");
    }

    #[test]
    fn vocabulary_sync_is_idempotent_and_preserves_manual_status() {
        let (directory, path, repository) = repository();
        let analysis = seed_analysis(
            &path,
            "one",
            "2026-01-01T00:00:00Z",
            vocabulary(" Terrible   at ", "muito ruim em", "I'm terrible at math."),
            json!([]),
            1,
        );
        repository.sync_analysis(&analysis).unwrap();
        repository.sync_analysis(&analysis).unwrap();
        let page = repository
            .list_vocabulary(
                VocabularyFilter::All,
                "",
                VocabularySort::RecentlySeen,
                25,
                0,
            )
            .unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].lesson_count, 1);
        assert_eq!(page.items[0].occurrence_count, 1);
        let item_id = page.items[0].id.clone();
        repository
            .update_vocabulary_status(&item_id, VocabularyStatus::Learning)
            .unwrap();
        repository.sync_analysis(&analysis).unwrap();
        assert_eq!(
            repository
                .get_vocabulary_item(&item_id)
                .unwrap()
                .unwrap()
                .item
                .status,
            VocabularyStatus::Learning
        );
        let connection = database::open(&path).unwrap();
        let links: i64 = connection
            .query_row("SELECT COUNT(*) FROM lesson_vocabulary", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(links, 1);
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn vocabulary_multi_lesson_recalculates_dates_counts_and_keeps_first_meaning() {
        let (directory, path, repository) = repository();
        let late = seed_analysis(
            &path,
            "late",
            "2026-02-01T00:00:00Z",
            vocabulary("Terrible at", "first imported meaning", "Late example"),
            json!([]),
            1,
        );
        let early = seed_analysis(
            &path,
            "early",
            "2026-01-01T00:00:00Z",
            vocabulary(" terrible  at ", "different later import", "Early example"),
            json!([]),
            1,
        );
        repository.sync_analysis(&late).unwrap();
        repository.sync_analysis(&early).unwrap();
        let item = repository
            .list_vocabulary(
                VocabularyFilter::All,
                "terrible",
                VocabularySort::MostFrequent,
                25,
                0,
            )
            .unwrap()
            .items
            .remove(0);
        assert_eq!(item.lesson_count, 2);
        assert_eq!(item.occurrence_count, 2);
        assert_eq!(item.first_seen_at, "2026-01-01T00:00:00Z");
        assert_eq!(item.last_seen_at, "2026-02-01T00:00:00Z");
        assert_eq!(item.meaning, "first imported meaning");
        let details = repository.get_vocabulary_item(&item.id).unwrap().unwrap();
        assert_eq!(details.occurrences.len(), 2);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn recurring_requires_two_lessons_and_same_category_alone_never_merges() {
        let (directory, path, repository) = repository();
        let first = seed_analysis(
            &path,
            "one",
            "2026-01-01T00:00:00Z",
            json!([]),
            correction(
                "I'm terrible cooking.",
                "I'm terrible at cooking.",
                "preposition",
            ),
            1,
        );
        repository.sync_analysis(&first).unwrap();
        assert!(repository.list_recurring_mistakes(20).unwrap().is_empty());
        let collision = seed_analysis(
            &path,
            "collision",
            "2026-01-02T00:00:00Z",
            json!([]),
            correction(
                "I'm interested music.",
                "I'm interested in music.",
                "preposition",
            ),
            1,
        );
        repository.sync_analysis(&collision).unwrap();
        assert!(repository.list_recurring_mistakes(20).unwrap().is_empty());
        let same = seed_analysis(
            &path,
            "same",
            "2026-01-03T00:00:00Z",
            json!([]),
            correction(
                "  I'M TERRIBLE   COOKING. ",
                "I'm terrible at cooking.",
                "preposition",
            ),
            1,
        );
        repository.sync_analysis(&same).unwrap();
        repository.sync_analysis(&same).unwrap();
        let mistakes = repository.list_recurring_mistakes(20).unwrap();
        assert_eq!(mistakes.len(), 1);
        assert_eq!(mistakes[0].lesson_count, 2);
        assert_eq!(mistakes[0].occurrence_count, 2);
        assert_eq!(
            repository
                .get_recurring_mistake(&mistakes[0].id)
                .unwrap()
                .unwrap()
                .occurrences
                .len(),
            2
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn signature_is_conservative_and_schema_mismatch_is_controlled() {
        let a: LessonAnalysisCorrection = serde_json::from_value(json!({
            "original": "I'm terrible cooking.", "corrected": "I'm terrible at cooking.",
            "explanation": "x", "category": "preposition"
        }))
        .unwrap();
        let b: LessonAnalysisCorrection = serde_json::from_value(json!({
            "original": "I'm terrible playing football.", "corrected": "I'm terrible at playing football.",
            "explanation": "x", "category": "preposition"
        })).unwrap();
        assert_ne!(derive_mistake_signature(&a), derive_mistake_signature(&b));

        let (directory, path, repository) = repository();
        let analysis = seed_analysis(&path, "schema", "2026-01-01", json!([]), json!([]), 2);
        let error = repository.sync_analysis(&analysis).unwrap_err();
        assert!(error.contains("Unsupported analysis schema"));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn transaction_rolls_back_every_derived_write_on_failure() {
        let (directory, path, repository) = repository();
        let analysis = seed_analysis(
            &path,
            "rollback",
            "2026-01-01T00:00:00Z",
            vocabulary("rollback phrase", "meaning", "example"),
            json!([]),
            1,
        );
        let connection = database::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TRIGGER force_memory_failure BEFORE INSERT ON lesson_vocabulary
             BEGIN SELECT RAISE(ABORT, 'forced memory failure'); END;",
            )
            .unwrap();
        drop(connection);
        assert!(repository.sync_analysis(&analysis).is_err());
        let connection = database::open(&path).unwrap();
        let items: i64 = connection
            .query_row("SELECT COUNT(*) FROM vocabulary_item", [], |row| row.get(0))
            .unwrap();
        let links: i64 = connection
            .query_row("SELECT COUNT(*) FROM lesson_vocabulary", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!((items, links), (0, 0));
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    #[ignore = "manual migration and learning-memory sync against the user's physical SQLite database"]
    fn physical_phase_g_syncs_real_completed_analysis() {
        let local_app_data = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .expect("LOCALAPPDATA");
        let database_path = local_app_data
            .join("com.englishaicoach.desktop")
            .join("database")
            .join("english-ai-coach.sqlite3");
        database::migrate(&database_path).expect("migrate physical database through v4");
        let repository = LearningMemoryRepository::new(database_path.clone());
        let sync = repository
            .sync_all_completed_analyses()
            .expect("sync physical completed analyses");
        assert_eq!(sync.failed, 0, "{:?}", sync.errors);
        let page = repository
            .list_vocabulary(
                VocabularyFilter::All,
                "terrible at",
                VocabularySort::RecentlySeen,
                25,
                0,
            )
            .expect("physical vocabulary");
        let item = page.items.first().expect("real analyzed vocabulary item");
        assert_eq!(normalize_vocabulary_key(&item.text), "terrible at");
        let details = repository
            .get_vocabulary_item(&item.id)
            .expect("physical vocabulary details")
            .expect("real vocabulary details");
        assert!(details
            .occurrences
            .iter()
            .any(|occurrence| { occurrence.lesson_id == "98d5e6f6-9c1a-47a1-8e1b-cba5421a0f34" }));
        let mistakes = repository
            .list_recurring_mistakes(100)
            .expect("physical recurring mistakes");
        let connection = database::open(&database_path).expect("open physical database");
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .expect("integrity check");
        let foreign_keys: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .expect("foreign key check");
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_keys, 0);
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "sync": sync,
                "summary": repository.summary().unwrap(),
                "item": item,
                "details": details,
                "confirmedRecurringMistakes": mistakes,
                "integrity": integrity,
                "foreignKeyViolations": foreign_keys,
            }))
            .unwrap()
        );
    }

    #[test]
    #[ignore = "manual persistence audit of vocabulary status against the user's physical SQLite database"]
    fn physical_vocabulary_status_survives_reopen_and_resync() {
        let local_app_data = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .expect("LOCALAPPDATA");
        let database_path = local_app_data
            .join("com.englishaicoach.desktop")
            .join("database")
            .join("english-ai-coach.sqlite3");
        let repository = LearningMemoryRepository::new(database_path.clone());
        let item = repository
            .list_vocabulary(
                VocabularyFilter::All,
                "terrible at",
                VocabularySort::RecentlySeen,
                25,
                0,
            )
            .expect("physical vocabulary")
            .items
            .into_iter()
            .next()
            .expect("real vocabulary item");
        let original_status = item.status;
        repository
            .update_vocabulary_status(&item.id, VocabularyStatus::Learning)
            .expect("mark real item learning");
        drop(repository);

        let reopened = LearningMemoryRepository::new(database_path);
        assert_eq!(
            reopened
                .get_vocabulary_item(&item.id)
                .unwrap()
                .unwrap()
                .item
                .status,
            VocabularyStatus::Learning
        );
        let sync = reopened.sync_all_completed_analyses().unwrap();
        assert_eq!(sync.failed, 0);
        assert_eq!(
            reopened
                .get_vocabulary_item(&item.id)
                .unwrap()
                .unwrap()
                .item
                .status,
            VocabularyStatus::Learning
        );
        reopened
            .update_vocabulary_status(&item.id, original_status)
            .expect("restore original physical vocabulary status");
    }
}
