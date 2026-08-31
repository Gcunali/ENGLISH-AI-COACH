use crate::{
    database,
    learning_memory_repository::{update_vocabulary_status_in_transaction, VocabularyStatus},
    lesson_analysis::LessonAnalysisPayload,
    review::{
        build_queue, QueueCandidate, ReviewItemType, ReviewMode, ReviewOutcome,
        REVIEW_ITEM_SNAPSHOT_VERSION, REVIEW_QUEUE_VERSION, REVIEW_SYSTEM_SCHEMA_VERSION,
    },
};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const NOW_SQL: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";

#[derive(Clone)]
pub struct ReviewRepository {
    database: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartReviewSessionRequest {
    pub mode: ReviewMode,
    pub item_count: u32,
    #[serde(default)]
    pub start_over: bool,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitReviewItemRequest {
    pub session_id: String,
    pub item_id: String,
    pub outcome: ReviewOutcome,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewVocabularyCounts {
    pub new_count: u32,
    pub learning_count: u32,
    pub total_eligible_count: u32,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewMistakeCounts {
    pub confirmed_count: u32,
}
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewHistoryDto {
    pub completed_session_count: u32,
    pub reviewed_item_count: u32,
    pub reviewed_this_week: u32,
    pub vocabulary_reviewed: u32,
    pub mistakes_reviewed: u32,
    pub last_review_at: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewOverviewDto {
    pub schema_version: u32,
    pub active_session: Option<ReviewSessionSummaryDto>,
    pub vocabulary: ReviewVocabularyCounts,
    pub recurring_mistakes: ReviewMistakeCounts,
    pub review_history: ReviewHistoryDto,
    pub suggested_focus: Option<String>,
    pub recent_sessions: Vec<ReviewSessionSummaryDto>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSessionSummaryDto {
    pub id: String,
    pub status: String,
    pub mode: ReviewMode,
    pub requested_item_count: u32,
    pub actual_item_count: u32,
    pub reviewed_item_count: u32,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub abandoned_at: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCompletionSummaryDto {
    pub items_reviewed: u32,
    pub vocabulary_reviewed: u32,
    pub mistakes_reviewed: u32,
    pub vocabulary_marked_learning: u32,
    pub vocabulary_marked_known: u32,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSessionDto {
    pub id: String,
    pub status: String,
    pub mode: ReviewMode,
    pub requested_item_count: u32,
    pub actual_item_count: u32,
    pub reviewed_item_count: u32,
    pub current_index: u32,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub current_item: Option<ReviewItemDto>,
    pub completion_summary: Option<ReviewCompletionSummaryDto>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularySnapshot {
    pub schema_version: u32,
    pub display_text: String,
    pub meaning: String,
    pub example: Option<String>,
    pub status_at_start: String,
    pub lesson_count: u32,
    pub occurrence_count: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MistakeSnapshot {
    pub schema_version: u32,
    pub title: String,
    pub category: String,
    pub original: String,
    pub corrected: String,
    pub explanation: String,
    pub lesson_count: u32,
    pub occurrence_count: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum StoredSnapshot {
    Vocabulary(VocabularySnapshot),
    RecurringMistake(MistakeSnapshot),
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReviewItemDto {
    Vocabulary {
        id: String,
        sequence_index: u32,
        reviewed: bool,
        review_outcome: Option<ReviewOutcome>,
        reviewed_at: Option<String>,
        content: VocabularySnapshot,
    },
    RecurringMistake {
        id: String,
        sequence_index: u32,
        reviewed: bool,
        review_outcome: Option<ReviewOutcome>,
        reviewed_at: Option<String>,
        content: MistakeSnapshot,
    },
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSubmitResult {
    pub session: ReviewSessionDto,
    pub vocabulary_status_changed: bool,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewQueuePreviewDto {
    pub requested_item_count: u32,
    pub actual_item_count: u32,
    pub mistakes: u32,
    pub learning_vocabulary: u32,
    pub new_vocabulary: u32,
}

#[derive(Clone)]
struct CandidateSource {
    candidate: QueueCandidate,
    snapshot: StoredSnapshot,
}

impl ReviewRepository {
    pub fn new(database: PathBuf) -> Self {
        Self { database }
    }
    pub fn overview(&self) -> Result<ReviewOverviewDto, String> {
        let c = database::open(&self.database)?;
        let (new_count,learning_count):(u32,u32)=c.query_row("SELECT COUNT(*) FILTER(WHERE status='new'),COUNT(*) FILTER(WHERE status='learning') FROM vocabulary_item",[],|r|Ok((r.get(0)?,r.get(1)?))).map_err(db)?;
        let confirmed:u32=c.query_row("SELECT COUNT(*) FROM recurring_mistake WHERE lesson_count>=2 AND status IN('active','improving')",[],|r|r.get(0)).map_err(db)?;
        let history = history_with(&c)?;
        let active = active_with(&c)?;
        let recent = self.list_recent(5)?;
        let suggested_focus=c.query_row("SELECT raw_json FROM lesson_analysis WHERE status='completed' AND raw_json IS NOT NULL ORDER BY completed_at DESC,id DESC LIMIT 1",[],|r|r.get::<_,String>(0)).optional().map_err(db)?
    .and_then(|raw|serde_json::from_str::<LessonAnalysisPayload>(&raw).ok()).and_then(|p|p.next_lesson_recommendations.into_iter().next());
        Ok(ReviewOverviewDto {
            schema_version: REVIEW_SYSTEM_SCHEMA_VERSION,
            active_session: active,
            vocabulary: ReviewVocabularyCounts {
                new_count,
                learning_count,
                total_eligible_count: new_count + learning_count,
            },
            recurring_mistakes: ReviewMistakeCounts {
                confirmed_count: confirmed,
            },
            review_history: history,
            suggested_focus,
            recent_sessions: recent,
        })
    }
    pub fn preview(
        &self,
        mode: ReviewMode,
        item_count: u32,
    ) -> Result<ReviewQueuePreviewDto, String> {
        let c = database::open(&self.database)?;
        let sources = sources(&c)?;
        let queue = queue_from_sources(mode, item_count, &sources)?;
        Ok(preview_of(item_count, &queue))
    }
    pub fn start(&self, request: StartReviewSessionRequest) -> Result<ReviewSessionDto, String> {
        let mut c = database::open(&self.database)?;
        let tx = c.transaction().map_err(db)?;
        if let Some(active) = active_with(&tx)? {
            if !request.start_over {
                return Err(format!(
                    "Review session {} is already in progress.",
                    active.id
                ));
            }
            abandon_with(&tx, &active.id)?;
        }
        let sources = sources(&tx)?;
        let queue = queue_from_sources(request.mode, request.item_count, &sources)?;
        if queue.is_empty() {
            return Err("Nothing needs review yet.".into());
        }
        let id = uuid::Uuid::new_v4().to_string();
        let actual = queue.len() as u32;
        tx.execute(&format!("INSERT INTO review_session(id,status,mode,requested_item_count,actual_item_count,reviewed_item_count,queue_version,item_snapshot_version,started_at,created_at,updated_at) VALUES(?1,'in_progress',?2,?3,?4,0,?5,?6,{NOW_SQL},{NOW_SQL},{NOW_SQL})"),params![id,request.mode.as_str(),request.item_count,actual,REVIEW_QUEUE_VERSION,REVIEW_ITEM_SNAPSHOT_VERSION]).map_err(db)?;
        for (index, candidate) in queue.iter().enumerate() {
            let source = sources
                .iter()
                .find(|s| {
                    s.candidate.source_id == candidate.source_id
                        && s.candidate.item_type == candidate.item_type
                })
                .ok_or_else(|| "Review source disappeared while snapshotting.".to_owned())?;
            let json = serde_json::to_string(&source.snapshot)
                .map_err(|e| format!("Could not serialize review snapshot: {e}"))?;
            let (vocab, mistake) = match candidate.item_type {
                ReviewItemType::Vocabulary => (Some(&candidate.source_id), None),
                ReviewItemType::RecurringMistake => (None, Some(&candidate.source_id)),
            };
            tx.execute(&format!("INSERT INTO review_session_item(id,session_id,sequence_index,item_type,vocabulary_item_id,recurring_mistake_id,content_json,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,{NOW_SQL},{NOW_SQL})"),params![uuid::Uuid::new_v4().to_string(),id,index as u32,candidate.item_type.as_str(),vocab,mistake,json]).map_err(db)?;
        }
        tx.commit().map_err(db)?;
        self.get(&id)?
            .ok_or_else(|| "Created review session could not be read back.".into())
    }
    pub fn get(&self, id: &str) -> Result<Option<ReviewSessionDto>, String> {
        let c = database::open(&self.database)?;
        session_with(&c, id)
    }
    pub fn resume(&self, id: &str) -> Result<ReviewSessionDto, String> {
        let session = self
            .get(id)?
            .ok_or_else(|| "Review session not found.".to_owned())?;
        if session.status != "in_progress" {
            return Err("Only an in-progress Review session can be resumed.".into());
        }
        Ok(session)
    }
    pub fn abandon(&self, id: &str) -> Result<ReviewSessionDto, String> {
        let mut c = database::open(&self.database)?;
        let tx = c.transaction().map_err(db)?;
        abandon_with(&tx, id)?;
        tx.commit().map_err(db)?;
        self.get(id)?
            .ok_or_else(|| "Review session not found.".into())
    }
    pub fn submit(&self, request: SubmitReviewItemRequest) -> Result<ReviewSubmitResult, String> {
        let mut c = database::open(&self.database)?;
        let tx = c.transaction().map_err(db)?;
        let row:(String,String,Option<String>,Option<String>,String)=tx.query_row("SELECT i.item_type,i.content_json,i.reviewed_at,i.vocabulary_item_id,s.status FROM review_session_item i JOIN review_session s ON s.id=i.session_id WHERE i.id=?1 AND i.session_id=?2",params![request.item_id,request.session_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional().map_err(db)?.ok_or_else(||"Review item does not belong to this session.".to_owned())?;
        if row.4 != "in_progress" {
            return Err("Review session is not in progress.".into());
        }
        if row.2.is_some() {
            return Err("Review item was already submitted.".into());
        }
        let kind = match row.0.as_str() {
            "vocabulary" => ReviewItemType::Vocabulary,
            "recurring_mistake" => ReviewItemType::RecurringMistake,
            _ => return Err("Unsupported persisted review item type.".into()),
        };
        if !request.outcome.valid_for(kind) {
            return Err("Review outcome is not valid for this item type.".into());
        }
        validate_snapshot(&row.1, kind)?;
        let mut vocabulary_status_changed = false;
        if let Some(vocabulary_id) = row.3.as_deref() {
            let status = match request.outcome {
                ReviewOutcome::MarkLearning => Some(VocabularyStatus::Learning),
                ReviewOutcome::MarkKnown => Some(VocabularyStatus::Known),
                _ => None,
            };
            if let Some(status) = status {
                update_vocabulary_status_in_transaction(&tx, vocabulary_id, status)?;
                vocabulary_status_changed = true
            }
        }
        let changed=tx.execute(&format!("UPDATE review_session_item SET review_outcome=?3,reviewed_at={NOW_SQL},updated_at={NOW_SQL} WHERE id=?1 AND session_id=?2 AND reviewed_at IS NULL"),params![request.item_id,request.session_id,request.outcome.as_str()]).map_err(db)?;
        if changed != 1 {
            return Err("Review item was already submitted.".into());
        }
        tx.execute(&format!("UPDATE review_session SET reviewed_item_count=reviewed_item_count+1,updated_at={NOW_SQL} WHERE id=?1"),[&request.session_id]).map_err(db)?;
        tx.execute(&format!("UPDATE review_session SET status='completed',completed_at={NOW_SQL},updated_at={NOW_SQL} WHERE id=?1 AND reviewed_item_count=actual_item_count"),[&request.session_id]).map_err(db)?;
        tx.commit().map_err(db)?;
        let session = self
            .get(&request.session_id)?
            .ok_or_else(|| "Review session disappeared after submit.".to_owned())?;
        Ok(ReviewSubmitResult {
            session,
            vocabulary_status_changed,
        })
    }
    pub fn list_recent(&self, limit: u32) -> Result<Vec<ReviewSessionSummaryDto>, String> {
        let c = database::open(&self.database)?;
        let mut st=c.prepare("SELECT id,status,mode,requested_item_count,actual_item_count,reviewed_item_count,started_at,completed_at,abandoned_at FROM review_session ORDER BY started_at DESC,id DESC LIMIT ?1").map_err(db)?;
        let rows = st
            .query_map([limit.clamp(1, 50)], summary_row)
            .map_err(db)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db)?;
        Ok(rows)
    }
}

fn queue_from_sources(
    mode: ReviewMode,
    count: u32,
    sources: &[CandidateSource],
) -> Result<Vec<QueueCandidate>, String> {
    let v = sources
        .iter()
        .filter(|s| s.candidate.item_type == ReviewItemType::Vocabulary)
        .map(|s| s.candidate.clone())
        .collect();
    let m = sources
        .iter()
        .filter(|s| s.candidate.item_type == ReviewItemType::RecurringMistake)
        .map(|s| s.candidate.clone())
        .collect();
    build_queue(mode, count, v, m)
}
fn preview_of(requested: u32, queue: &[QueueCandidate]) -> ReviewQueuePreviewDto {
    ReviewQueuePreviewDto {
        requested_item_count: requested,
        actual_item_count: queue.len() as u32,
        mistakes: queue
            .iter()
            .filter(|x| x.item_type == ReviewItemType::RecurringMistake)
            .count() as u32,
        learning_vocabulary: queue
            .iter()
            .filter(|x| x.vocabulary_status.as_deref() == Some("learning"))
            .count() as u32,
        new_vocabulary: queue
            .iter()
            .filter(|x| x.vocabulary_status.as_deref() == Some("new"))
            .count() as u32,
    }
}

fn sources(c: &Connection) -> Result<Vec<CandidateSource>, String> {
    let mut result = Vec::new();
    {
        let mut st=c.prepare("SELECT v.id,v.display_text,v.meaning,v.status,v.lesson_count,v.occurrence_count,v.last_seen_at,(SELECT lv.example FROM lesson_vocabulary lv JOIN lesson l ON l.id=lv.lesson_id WHERE lv.vocabulary_item_id=v.id AND length(trim(lv.example))>0 ORDER BY l.started_at DESC,lv.id DESC LIMIT 1),COUNT(i.id),MAX(i.reviewed_at) FROM vocabulary_item v LEFT JOIN review_session_item i ON i.vocabulary_item_id=v.id AND i.reviewed_at IS NOT NULL WHERE v.status IN('new','learning') GROUP BY v.id ORDER BY v.id").map_err(db)?;
        let rows = st
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, u32>(4)?,
                    r.get::<_, u32>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, Option<String>>(7)?,
                    r.get::<_, u32>(8)?,
                    r.get::<_, Option<String>>(9)?,
                ))
            })
            .map_err(db)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db)?;
        for (
            id,
            text,
            meaning,
            status,
            lessons,
            occurrences,
            last_seen,
            example,
            reviews,
            last_review,
        ) in rows
        {
            result.push(CandidateSource {
                candidate: QueueCandidate {
                    source_id: id,
                    item_type: ReviewItemType::Vocabulary,
                    vocabulary_status: Some(status.clone()),
                    review_count: reviews,
                    last_reviewed_at: last_review,
                    occurrence_count: occurrences,
                    lesson_count: lessons,
                    last_seen_at: last_seen,
                },
                snapshot: StoredSnapshot::Vocabulary(VocabularySnapshot {
                    schema_version: REVIEW_ITEM_SNAPSHOT_VERSION,
                    display_text: text,
                    meaning,
                    example,
                    status_at_start: status,
                    lesson_count: lessons,
                    occurrence_count: occurrences,
                }),
            })
        }
    }
    {
        let mut st=c.prepare("SELECT m.id,m.title,m.category,m.explanation,m.lesson_count,m.occurrence_count,m.last_seen_at,o.original,o.corrected,COALESCE(NULLIF(trim(o.explanation),''),m.explanation),COUNT(i.id),MAX(i.reviewed_at) FROM recurring_mistake m JOIN recurring_mistake_occurrence o ON o.id=(SELECT o2.id FROM recurring_mistake_occurrence o2 JOIN lesson l ON l.id=o2.lesson_id WHERE o2.recurring_mistake_id=m.id ORDER BY l.started_at DESC,o2.source_index,o2.id LIMIT 1) LEFT JOIN review_session_item i ON i.recurring_mistake_id=m.id AND i.reviewed_at IS NOT NULL WHERE m.lesson_count>=2 AND m.status IN('active','improving') GROUP BY m.id ORDER BY m.id").map_err(db)?;
        let rows = st
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, u32>(4)?,
                    r.get::<_, u32>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                    r.get::<_, String>(8)?,
                    r.get::<_, String>(9)?,
                    r.get::<_, u32>(10)?,
                    r.get::<_, Option<String>>(11)?,
                ))
            })
            .map_err(db)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db)?;
        for (
            id,
            title,
            category,
            _base_explanation,
            lessons,
            occurrences,
            last_seen,
            original,
            corrected,
            explanation,
            reviews,
            last_review,
        ) in rows
        {
            result.push(CandidateSource {
                candidate: QueueCandidate {
                    source_id: id,
                    item_type: ReviewItemType::RecurringMistake,
                    vocabulary_status: None,
                    review_count: reviews,
                    last_reviewed_at: last_review,
                    occurrence_count: occurrences,
                    lesson_count: lessons,
                    last_seen_at: last_seen,
                },
                snapshot: StoredSnapshot::RecurringMistake(MistakeSnapshot {
                    schema_version: REVIEW_ITEM_SNAPSHOT_VERSION,
                    title,
                    category,
                    original,
                    corrected,
                    explanation,
                    lesson_count: lessons,
                    occurrence_count: occurrences,
                }),
            })
        }
    }
    Ok(result)
}

fn active_with(c: &Connection) -> Result<Option<ReviewSessionSummaryDto>, String> {
    c.query_row("SELECT id,status,mode,requested_item_count,actual_item_count,reviewed_item_count,started_at,completed_at,abandoned_at FROM review_session WHERE status='in_progress' ORDER BY started_at DESC LIMIT 1",[],summary_row).optional().map_err(db)
}
fn summary_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ReviewSessionSummaryDto> {
    let mode =
        ReviewMode::parse(&r.get::<_, String>(2)?).map_err(|_| rusqlite::Error::InvalidQuery)?;
    Ok(ReviewSessionSummaryDto {
        id: r.get(0)?,
        status: r.get(1)?,
        mode,
        requested_item_count: r.get(3)?,
        actual_item_count: r.get(4)?,
        reviewed_item_count: r.get(5)?,
        started_at: r.get(6)?,
        completed_at: r.get(7)?,
        abandoned_at: r.get(8)?,
    })
}
fn abandon_with(tx: &Transaction<'_>, id: &str) -> Result<(), String> {
    let changed=tx.execute(&format!("UPDATE review_session SET status='abandoned',abandoned_at={NOW_SQL},updated_at={NOW_SQL} WHERE id=?1 AND status='in_progress'"),[id]).map_err(db)?;
    if changed != 1 {
        return Err("Only an in-progress Review session can be abandoned.".into());
    }
    Ok(())
}

fn session_with(c: &Connection, id: &str) -> Result<Option<ReviewSessionDto>, String> {
    let summary=c.query_row("SELECT id,status,mode,requested_item_count,actual_item_count,reviewed_item_count,started_at,completed_at,abandoned_at FROM review_session WHERE id=?1",[id],summary_row).optional().map_err(db)?;
    let Some(s) = summary else { return Ok(None) };
    let item = if s.status == "in_progress" {
        c.query_row("SELECT id,sequence_index,item_type,content_json,review_outcome,reviewed_at FROM review_session_item WHERE session_id=?1 AND reviewed_at IS NULL ORDER BY sequence_index LIMIT 1",[id],item_row).optional().map_err(db)?
    } else {
        None
    };
    let completion = if s.status == "completed" {
        Some(completion_with(c, id)?)
    } else {
        None
    };
    Ok(Some(ReviewSessionDto {
        id: s.id,
        status: s.status,
        mode: s.mode,
        requested_item_count: s.requested_item_count,
        actual_item_count: s.actual_item_count,
        reviewed_item_count: s.reviewed_item_count,
        current_index: if item.is_some() {
            s.reviewed_item_count
        } else {
            s.actual_item_count
        },
        started_at: s.started_at,
        completed_at: s.completed_at,
        current_item: item,
        completion_summary: completion,
    }))
}
fn item_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ReviewItemDto> {
    let id: String = r.get(0)?;
    let sequence: u32 = r.get(1)?;
    let kind: String = r.get(2)?;
    let raw: String = r.get(3)?;
    let outcome_text: Option<String> = r.get(4)?;
    let reviewed_at: Option<String> = r.get(5)?;
    let outcome = outcome_text
        .as_deref()
        .map(parse_outcome)
        .transpose()
        .map_err(|_| rusqlite::Error::InvalidQuery)?;
    let snapshot: StoredSnapshot =
        serde_json::from_str(&raw).map_err(|_| rusqlite::Error::InvalidQuery)?;
    match (kind.as_str(), snapshot) {
        ("vocabulary", StoredSnapshot::Vocabulary(content))
            if content.schema_version == REVIEW_ITEM_SNAPSHOT_VERSION =>
        {
            Ok(ReviewItemDto::Vocabulary {
                id,
                sequence_index: sequence,
                reviewed: reviewed_at.is_some(),
                review_outcome: outcome,
                reviewed_at,
                content,
            })
        }
        ("recurring_mistake", StoredSnapshot::RecurringMistake(content))
            if content.schema_version == REVIEW_ITEM_SNAPSHOT_VERSION =>
        {
            Ok(ReviewItemDto::RecurringMistake {
                id,
                sequence_index: sequence,
                reviewed: reviewed_at.is_some(),
                review_outcome: outcome,
                reviewed_at,
                content,
            })
        }
        _ => Err(rusqlite::Error::InvalidQuery),
    }
}
fn validate_snapshot(raw: &str, kind: ReviewItemType) -> Result<(), String> {
    let value: StoredSnapshot =
        serde_json::from_str(raw).map_err(|e| format!("Invalid persisted review snapshot: {e}"))?;
    match (value, kind) {
        (StoredSnapshot::Vocabulary(s), ReviewItemType::Vocabulary)
            if s.schema_version == REVIEW_ITEM_SNAPSHOT_VERSION =>
        {
            Ok(())
        }
        (StoredSnapshot::RecurringMistake(s), ReviewItemType::RecurringMistake)
            if s.schema_version == REVIEW_ITEM_SNAPSHOT_VERSION =>
        {
            Ok(())
        }
        _ => Err("Unsupported review item snapshot version or type.".into()),
    }
}
fn parse_outcome(v: &str) -> Result<ReviewOutcome, String> {
    match v {
        "keep_practicing" => Ok(ReviewOutcome::KeepPracticing),
        "mark_learning" => Ok(ReviewOutcome::MarkLearning),
        "mark_known" => Ok(ReviewOutcome::MarkKnown),
        "review_again" => Ok(ReviewOutcome::ReviewAgain),
        "reviewed" => Ok(ReviewOutcome::Reviewed),
        _ => Err("Unsupported review outcome.".into()),
    }
}
fn completion_with(c: &Connection, id: &str) -> Result<ReviewCompletionSummaryDto, String> {
    c.query_row("SELECT COUNT(*),COUNT(*) FILTER(WHERE item_type='vocabulary'),COUNT(*) FILTER(WHERE item_type='recurring_mistake'),COUNT(*) FILTER(WHERE review_outcome='mark_learning'),COUNT(*) FILTER(WHERE review_outcome='mark_known') FROM review_session_item WHERE session_id=?1 AND reviewed_at IS NOT NULL",[id],|r|Ok(ReviewCompletionSummaryDto{items_reviewed:r.get(0)?,vocabulary_reviewed:r.get(1)?,mistakes_reviewed:r.get(2)?,vocabulary_marked_learning:r.get(3)?,vocabulary_marked_known:r.get(4)?})).map_err(db)
}
fn history_with(c: &Connection) -> Result<ReviewHistoryDto, String> {
    c.query_row("SELECT (SELECT COUNT(*) FROM review_session WHERE status='completed'),COUNT(*),COUNT(*) FILTER(WHERE date(reviewed_at,'localtime')>=date('now','localtime',printf('-%d days',(CAST(strftime('%w','now','localtime') AS INTEGER)+6)%7))),COUNT(*) FILTER(WHERE item_type='vocabulary'),COUNT(*) FILTER(WHERE item_type='recurring_mistake'),MAX(reviewed_at) FROM review_session_item WHERE reviewed_at IS NOT NULL",[],|r|Ok(ReviewHistoryDto{completed_session_count:r.get(0)?,reviewed_item_count:r.get(1)?,reviewed_this_week:r.get(2)?,vocabulary_reviewed:r.get(3)?,mistakes_reviewed:r.get(4)?,last_review_at:r.get(5)?})).map_err(db)
}
fn db(e: rusqlite::Error) -> String {
    format!("Review database error: {e}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;
    fn repo() -> (PathBuf, ReviewRepository) {
        let d = std::env::temp_dir().join(format!("review-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("db.sqlite3");
        database::migrate(&p).unwrap();
        (d, ReviewRepository::new(p))
    }
    fn seed(r: &ReviewRepository) {
        let c = database::open(&r.database).unwrap();
        for (i, status) in ["new", "learning", "known"].iter().enumerate() {
            c.execute("INSERT INTO vocabulary_item(id,canonical_text,display_text,meaning,first_seen_at,last_seen_at,lesson_count,occurrence_count,status,created_at,updated_at) VALUES(?1,?1,?1,?2,'2026-08-01','2026-08-20',2,3,?3,'x','x')",params![format!("v{i}"),format!("meaning{i}"),status]).unwrap();
        }
        c.execute("INSERT INTO lesson(id,started_at,status,mode,whisper_model,whisper_threads,ollama_model,piper_voice,voice_engine_version,created_at,updated_at) VALUES('l','2026-08-20','completed','free_conversation','w',1,'o','p','v','x','x')",[]).unwrap();
        c.execute("INSERT INTO lesson_analysis(id,lesson_id,status,schema_version,prompt_version,analyzer_model,created_at,updated_at) VALUES('a','l','completed',1,1,'o','x','x')",[]).unwrap();
        c.execute("INSERT INTO lesson_vocabulary(id,lesson_id,vocabulary_item_id,source_analysis_id,example,created_at) VALUES('lv','l','v0','a','real example','x')",[]).unwrap();
    }
    fn seed_mistake(r: &ReviewRepository) {
        let c = database::open(&r.database).unwrap();
        c.execute("INSERT INTO recurring_mistake(id,signature,category,title,explanation,first_seen_at,last_seen_at,lesson_count,occurrence_count,status,created_at,updated_at) VALUES('m','sig','preposition','Preposition pattern','Base explanation','2026-08-01','2026-08-20',2,2,'active','x','x')",[]).unwrap();
        c.execute("INSERT INTO recurring_mistake_occurrence(id,recurring_mistake_id,lesson_id,analysis_id,source_index,original,corrected,explanation,created_at) VALUES('mo','m','l','a',0,'I am terrible cooking.','I am terrible at cooking.','Use at before an activity.','x')",[]).unwrap();
    }
    #[test]
    fn zero_items_does_not_create_session() {
        let (d, r) = repo();
        assert!(r
            .start(StartReviewSessionRequest {
                mode: ReviewMode::Mixed,
                item_count: 10,
                start_over: false
            })
            .is_err());
        assert_eq!(
            database::open(&r.database)
                .unwrap()
                .query_row("SELECT COUNT(*) FROM review_session", [], |x| x
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        std::fs::remove_dir_all(d).unwrap()
    }
    #[test]
    fn snapshots_resume_outcomes_completion_and_known_exclusion() {
        let (d, r) = repo();
        seed(&r);
        let s = r
            .start(StartReviewSessionRequest {
                mode: ReviewMode::Vocabulary,
                item_count: 5,
                start_over: false,
            })
            .unwrap();
        assert_eq!(s.actual_item_count, 2);
        let first = s.current_item.unwrap();
        let (first_id, first_meaning) = match first {
            ReviewItemDto::Vocabulary { id, content, .. } => (id, content.meaning),
            _ => panic!(),
        };
        let c = database::open(&r.database).unwrap();
        c.execute("UPDATE vocabulary_item SET meaning='changed'", [])
            .unwrap();
        drop(c);
        let resumed = r.resume(&s.id).unwrap();
        match resumed.current_item.unwrap() {
            ReviewItemDto::Vocabulary { content, .. } => assert_eq!(content.meaning, first_meaning),
            _ => panic!(),
        };
        assert!(r
            .start(StartReviewSessionRequest {
                mode: ReviewMode::Mixed,
                item_count: 5,
                start_over: false
            })
            .is_err());
        let one = r
            .submit(SubmitReviewItemRequest {
                session_id: s.id.clone(),
                item_id: first_id.clone(),
                outcome: ReviewOutcome::MarkKnown,
            })
            .unwrap();
        assert!(r
            .submit(SubmitReviewItemRequest {
                session_id: s.id.clone(),
                item_id: first_id.clone(),
                outcome: ReviewOutcome::KeepPracticing
            })
            .is_err());
        let second_id = match one.session.current_item.unwrap() {
            ReviewItemDto::Vocabulary { id, .. } => id,
            _ => panic!(),
        };
        assert!(r
            .submit(SubmitReviewItemRequest {
                session_id: s.id.clone(),
                item_id: second_id.clone(),
                outcome: ReviewOutcome::Reviewed
            })
            .is_err());
        let done = r
            .submit(SubmitReviewItemRequest {
                session_id: s.id.clone(),
                item_id: second_id,
                outcome: ReviewOutcome::KeepPracticing,
            })
            .unwrap();
        assert_eq!(done.session.status, "completed");
        assert_eq!(done.session.completion_summary.unwrap().items_reviewed, 2);
        assert_eq!(
            r.preview(ReviewMode::Vocabulary, 5)
                .unwrap()
                .actual_item_count,
            1
        );
        std::fs::remove_dir_all(d).unwrap()
    }

    #[test]
    fn mistake_outcomes_never_resolve_and_wrong_type_is_atomic() {
        let (d, r) = repo();
        seed(&r);
        seed_mistake(&r);
        let first = r
            .start(StartReviewSessionRequest {
                mode: ReviewMode::Mistakes,
                item_count: 5,
                start_over: false,
            })
            .unwrap();
        let item = match first.current_item.unwrap() {
            ReviewItemDto::RecurringMistake { id, content, .. } => {
                assert_eq!(content.original, "I am terrible cooking.");
                id
            }
            _ => panic!(),
        };
        assert!(r
            .submit(SubmitReviewItemRequest {
                session_id: first.id.clone(),
                item_id: item.clone(),
                outcome: ReviewOutcome::MarkKnown
            })
            .is_err());
        let c = database::open(&r.database).unwrap();
        let reviewed: Option<String> = c
            .query_row(
                "SELECT reviewed_at FROM review_session_item WHERE id=?1",
                [&item],
                |x| x.get(0),
            )
            .unwrap();
        assert!(reviewed.is_none());
        drop(c);
        r.submit(SubmitReviewItemRequest {
            session_id: first.id,
            item_id: item,
            outcome: ReviewOutcome::Reviewed,
        })
        .unwrap();
        let c = database::open(&r.database).unwrap();
        let status: String = c
            .query_row(
                "SELECT status FROM recurring_mistake WHERE id='m'",
                [],
                |x| x.get(0),
            )
            .unwrap();
        assert_eq!(status, "active");
        drop(c);
        let second = r
            .start(StartReviewSessionRequest {
                mode: ReviewMode::Mistakes,
                item_count: 5,
                start_over: false,
            })
            .unwrap();
        let item = match second.current_item.unwrap() {
            ReviewItemDto::RecurringMistake { id, .. } => id,
            _ => panic!(),
        };
        r.submit(SubmitReviewItemRequest {
            session_id: second.id,
            item_id: item,
            outcome: ReviewOutcome::ReviewAgain,
        })
        .unwrap();
        assert_eq!(
            database::open(&r.database)
                .unwrap()
                .query_row(
                    "SELECT status FROM recurring_mistake WHERE id='m'",
                    [],
                    |x| x.get::<_, String>(0)
                )
                .unwrap(),
            "active"
        );
        std::fs::remove_dir_all(d).unwrap()
    }

    #[test]
    fn vocabulary_update_failure_rolls_back_review_atomically() {
        let (d, r) = repo();
        seed(&r);
        let s = r
            .start(StartReviewSessionRequest {
                mode: ReviewMode::Vocabulary,
                item_count: 5,
                start_over: false,
            })
            .unwrap();
        let item = match s.current_item.unwrap() {
            ReviewItemDto::Vocabulary { id, .. } => id,
            _ => panic!(),
        };
        let c = database::open(&r.database).unwrap();
        c.execute_batch("CREATE TRIGGER reject_review_status BEFORE UPDATE OF status ON vocabulary_item BEGIN SELECT RAISE(ABORT,'simulated status failure'); END;").unwrap();
        drop(c);
        assert!(r
            .submit(SubmitReviewItemRequest {
                session_id: s.id.clone(),
                item_id: item.clone(),
                outcome: ReviewOutcome::MarkKnown
            })
            .is_err());
        let c = database::open(&r.database).unwrap();
        let(row_reviewed,session_count):(Option<String>,u32)=c.query_row("SELECT i.reviewed_at,s.reviewed_item_count FROM review_session_item i JOIN review_session s ON s.id=i.session_id WHERE i.id=?1",[item],|x|Ok((x.get(0)?,x.get(1)?))).unwrap();
        assert!(row_reviewed.is_none());
        assert_eq!(session_count, 0);
        drop(c);
        std::fs::remove_dir_all(d).unwrap()
    }

    #[test]
    fn review_isolated_from_lessons_analysis_gamification_placement_and_profile() {
        let (d, r) = repo();
        seed(&r);
        let c = database::open(&r.database).unwrap();
        let before:(i64,i64,i64,i64,i64)=c.query_row("SELECT (SELECT COUNT(*) FROM lesson),(SELECT COUNT(*) FROM lesson_analysis),(SELECT COUNT(*) FROM gamification_xp_event),(SELECT COUNT(*) FROM achievement_unlock),(SELECT COUNT(*) FROM placement_attempt)",[],|x|Ok((x.get(0)?,x.get(1)?,x.get(2)?,x.get(3)?,x.get(4)?))).unwrap();
        let profile_before:(Option<String>,String,String)=c.query_row("SELECT target_cefr_level,learning_goals_json,default_lesson_difficulty FROM student_learning_profile WHERE profile_key='default'",[],|x|Ok((x.get(0)?,x.get(1)?,x.get(2)?))).unwrap();
        drop(c);
        let s = r
            .start(StartReviewSessionRequest {
                mode: ReviewMode::Vocabulary,
                item_count: 5,
                start_over: false,
            })
            .unwrap();
        let item = match s.current_item.unwrap() {
            ReviewItemDto::Vocabulary { id, .. } => id,
            _ => panic!(),
        };
        r.submit(SubmitReviewItemRequest {
            session_id: s.id,
            item_id: item,
            outcome: ReviewOutcome::KeepPracticing,
        })
        .unwrap();
        let c = database::open(&r.database).unwrap();
        let after:(i64,i64,i64,i64,i64)=c.query_row("SELECT (SELECT COUNT(*) FROM lesson),(SELECT COUNT(*) FROM lesson_analysis),(SELECT COUNT(*) FROM gamification_xp_event),(SELECT COUNT(*) FROM achievement_unlock),(SELECT COUNT(*) FROM placement_attempt)",[],|x|Ok((x.get(0)?,x.get(1)?,x.get(2)?,x.get(3)?,x.get(4)?))).unwrap();
        let profile_after:(Option<String>,String,String)=c.query_row("SELECT target_cefr_level,learning_goals_json,default_lesson_difficulty FROM student_learning_profile WHERE profile_key='default'",[],|x|Ok((x.get(0)?,x.get(1)?,x.get(2)?))).unwrap();
        assert_eq!(before, after);
        assert_eq!(profile_before, profile_after);
        drop(c);
        std::fs::remove_dir_all(d).unwrap()
    }
    #[test]
    fn start_over_abandons_without_erasing_review_history() {
        let (d, r) = repo();
        seed(&r);
        let first = r
            .start(StartReviewSessionRequest {
                mode: ReviewMode::Vocabulary,
                item_count: 5,
                start_over: false,
            })
            .unwrap();
        let item = match first.current_item.unwrap() {
            ReviewItemDto::Vocabulary { id, .. } => id,
            _ => panic!(),
        };
        r.submit(SubmitReviewItemRequest {
            session_id: first.id.clone(),
            item_id: item,
            outcome: ReviewOutcome::KeepPracticing,
        })
        .unwrap();
        let second = r
            .start(StartReviewSessionRequest {
                mode: ReviewMode::Vocabulary,
                item_count: 5,
                start_over: true,
            })
            .unwrap();
        assert_eq!(r.get(&first.id).unwrap().unwrap().status, "abandoned");
        assert_eq!(r.overview().unwrap().review_history.reviewed_item_count, 1);
        assert_eq!(r.resume(&second.id).unwrap().reviewed_item_count, 0);
        std::fs::remove_dir_all(d).unwrap()
    }

    #[test]
    #[ignore = "manual migration and read-only Review preview against the user's physical SQLite database"]
    fn physical_phase_m_migrates_previews_and_preserves_existing_data() {
        use crate::gamification_repository::GamificationRepository;
        let path = PathBuf::from(std::env::var_os("LOCALAPPDATA").expect("LOCALAPPDATA"))
            .join("com.englishaicoach.desktop")
            .join("database")
            .join("english-ai-coach.sqlite3");
        let before_connection = database::open(&path).expect("open physical v9 database");
        let before_counts:(i64,i64,i64,i64,i64,i64,i64)=before_connection.query_row("SELECT (SELECT COUNT(*) FROM lesson),(SELECT COUNT(*) FROM lesson_analysis),(SELECT COUNT(*) FROM vocabulary_item),(SELECT COUNT(*) FROM recurring_mistake),(SELECT COUNT(*) FROM gamification_xp_event),(SELECT COUNT(*) FROM achievement_unlock),(SELECT COUNT(*) FROM placement_attempt)",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?))).unwrap();
        let profile_before:(Option<String>,String,String,i64)=before_connection.query_row("SELECT target_cefr_level,learning_goals_json,default_lesson_difficulty,use_profile_in_lessons FROM student_learning_profile WHERE profile_key='default'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap();
        drop(before_connection);
        let gamification = GamificationRepository::new(path.clone());
        let game_before = gamification.overview().unwrap();
        let achievements_before = gamification
            .achievements()
            .unwrap()
            .into_iter()
            .filter(|a| a.unlocked)
            .map(|a| a.id)
            .collect::<Vec<_>>();
        database::migrate(&path).expect("apply migration 010");
        let repository = ReviewRepository::new(path.clone());
        let overview = repository.overview().unwrap();
        let preview = repository.preview(ReviewMode::Mixed, 10).unwrap();
        let connection = database::open(&path).unwrap();
        let after_counts:(i64,i64,i64,i64,i64,i64,i64)=connection.query_row("SELECT (SELECT COUNT(*) FROM lesson),(SELECT COUNT(*) FROM lesson_analysis),(SELECT COUNT(*) FROM vocabulary_item),(SELECT COUNT(*) FROM recurring_mistake),(SELECT COUNT(*) FROM gamification_xp_event),(SELECT COUNT(*) FROM achievement_unlock),(SELECT COUNT(*) FROM placement_attempt)",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?))).unwrap();
        let profile_after:(Option<String>,String,String,i64)=connection.query_row("SELECT target_cefr_level,learning_goals_json,default_lesson_difficulty,use_profile_in_lessons FROM student_learning_profile WHERE profile_key='default'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap();
        let statuses:(i64,i64,i64)=connection.query_row("SELECT COUNT(*) FILTER(WHERE status='new'),COUNT(*) FILTER(WHERE status='learning'),COUNT(*) FILTER(WHERE status='known') FROM vocabulary_item",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
        let sessions: i64 = connection
            .query_row("SELECT COUNT(*) FROM review_session", [], |r| r.get(0))
            .unwrap();
        let items: i64 = connection
            .query_row("SELECT COUNT(*) FROM review_session_item", [], |r| r.get(0))
            .unwrap();
        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migration", [], |r| {
                r.get(0)
            })
            .unwrap();
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        let foreign_keys: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
                r.get(0)
            })
            .unwrap();
        drop(connection);
        let game_after = gamification.overview().unwrap();
        let achievements_after = gamification
            .achievements()
            .unwrap()
            .into_iter()
            .filter(|a| a.unlocked)
            .map(|a| a.id)
            .collect::<Vec<_>>();
        assert_eq!(before_counts, after_counts);
        assert_eq!(profile_before, profile_after);
        assert_eq!(sessions, 0);
        assert_eq!(items, 0);
        assert_eq!(version, 10);
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_keys, 0);
        assert_eq!(
            (
                game_before.total_xp,
                game_before.practice_level,
                game_before.current_streak_days,
                game_before.weekly_goal.practiced_minutes
            ),
            (
                game_after.total_xp,
                game_after.practice_level,
                game_after.current_streak_days,
                game_after.weekly_goal.practiced_minutes
            )
        );
        assert_eq!(achievements_before, achievements_after);
        database::migrate(&path).expect("reopen migration 010");
        assert_eq!(
            database::open(&path)
                .unwrap()
                .query_row("SELECT COUNT(*) FROM review_session", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        println!("PHASE_M vocabulary_new={} vocabulary_learning={} vocabulary_known={} eligible_vocabulary={} confirmed_mistakes={} mixed10_actual={} mixed10_mistakes={} mixed10_learning={} mixed10_new={} review_sessions={} review_items={} xp={} level={} streak={} weekly_minutes={} achievements={:?}",statuses.0,statuses.1,statuses.2,overview.vocabulary.total_eligible_count,overview.recurring_mistakes.confirmed_count,preview.actual_item_count,preview.mistakes,preview.learning_vocabulary,preview.new_vocabulary,sessions,items,game_after.total_xp,game_after.practice_level,game_after.current_streak_days,game_after.weekly_goal.practiced_minutes,achievements_after);
    }
}
