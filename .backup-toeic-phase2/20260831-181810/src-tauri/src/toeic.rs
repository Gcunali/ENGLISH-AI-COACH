use crate::{database, toeic_item_bank::*};
use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

const SESSION_SCHEMA_VERSION: u32 = 1;
const FORM_LENGTH: usize = 6;

/// Reserved contract for a future calibrated conversion. Phase 1 intentionally
/// ships no profile and never converts six raw answers to a 5–495 score.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToeicScoreProfile {
    pub score_profile_id: String,
    pub version: u32,
    pub section: ToeicSection,
    pub form_family: String,
    pub calibration_method: String,
    pub conversion_table: BTreeMap<u32, u32>,
    pub confidence_metadata: String,
}

#[derive(Clone)]
pub struct ToeicRepository {
    database: PathBuf,
    bank: ToeicItemBank,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToeicPartAvailabilityDto {
    pub part: String,
    pub title: String,
    pub question_count: u32,
    pub runtime_available: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToeicFormDto {
    pub form_id: String,
    pub form_version: u32,
    pub title: String,
    pub question_count: u32,
    pub active_session_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToeicHistoryEntryDto {
    pub session_id: String,
    pub form_id: String,
    pub form_title: String,
    pub status: String,
    pub correct: u32,
    pub answered: u32,
    pub total: u32,
    pub accuracy: u32,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToeicOverviewDto {
    pub bank_id: String,
    pub untimed: bool,
    pub parts: Vec<ToeicPartAvailabilityDto>,
    pub forms: Vec<ToeicFormDto>,
    pub active_sessions: Vec<ToeicHistoryEntryDto>,
    pub recent_history: Vec<ToeicHistoryEntryDto>,
    pub disclaimer: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToeicPublicQuestionDto {
    pub item_id: String,
    pub item_version: u32,
    pub question_number: u32,
    pub total_questions: u32,
    pub image_base64: String,
    pub image_mime_type: String,
    pub choices: Vec<String>,
    pub initial_audio_completed: bool,
    pub initial_audio_interrupted: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToeicFeedbackDto {
    pub selected_choice: String,
    pub is_correct: bool,
    pub correct_answer: String,
    pub statements: Vec<ToeicTranscriptDto>,
    pub correct_explanation: String,
    pub selected_explanation: Option<String>,
    pub language_focus: Vec<String>,
    pub useful_vocabulary: Vec<String>,
    pub answered_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToeicTranscriptDto {
    pub choice: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToeicSessionDto {
    pub session_id: String,
    pub form_id: String,
    pub form_version: u32,
    pub form_title: String,
    pub status: String,
    pub untimed: bool,
    pub current_question_index: u32,
    pub answered_count: u32,
    pub current_question: Option<ToeicPublicQuestionDto>,
    pub feedback: Option<ToeicFeedbackDto>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToeicSubmitAnswerRequest {
    pub session_id: String,
    pub item_id: String,
    pub item_version: u32,
    pub selected_choice: String,
}

#[derive(Clone, Debug)]
pub struct ToeicAudioContext {
    pub item_id: String,
    pub item_version: u32,
    pub script: String,
    pub presentation_id: Option<String>,
    pub initial: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToeicBreakdownDto {
    pub label: String,
    pub correct: u32,
    pub total: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToeicDistractorCountDto {
    pub distractor_type: String,
    pub count: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToeicResultDto {
    pub session_id: String,
    pub form_id: String,
    pub form_title: String,
    pub status: String,
    pub correct: u32,
    pub total: u32,
    pub accuracy: u32,
    pub skill_breakdown: Vec<ToeicBreakdownDto>,
    pub difficulty_breakdown: Vec<ToeicBreakdownDto>,
    pub common_distractors: Vec<ToeicDistractorCountDto>,
    pub has_scaled_score: bool,
    pub score_message: String,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToeicReviewItemDto {
    pub item_id: String,
    pub question_number: u32,
    pub image_base64: String,
    pub image_mime_type: String,
    pub difficulty: String,
    pub skill_tags: Vec<String>,
    pub feedback: ToeicFeedbackDto,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FormSnapshot {
    form_id: String,
    form_version: u32,
    items: Vec<FormSnapshotItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FormSnapshotItem {
    item_id: String,
    item_version: u32,
}

impl ToeicRepository {
    pub fn new(database: PathBuf, bank: ToeicItemBank) -> Result<Self, String> {
        let result = Self { database, bank };
        result.recover_interrupted_presentations()?;
        Ok(result)
    }

    pub fn overview(&self) -> Result<ToeicOverviewDto, String> {
        let active = self.history_query(Some("in_progress"), 20)?;
        let history = self.history_query(Some("completed"), 20)?;
        let forms = self
            .bank
            .published_forms()
            .into_iter()
            .map(|form| {
                let active_session_id = active
                    .iter()
                    .find(|entry| entry.form_id == form.form_id)
                    .map(|entry| entry.session_id.clone());
                ToeicFormDto {
                    title: form_title(&form.form_id),
                    form_id: form.form_id,
                    form_version: form.form_version,
                    question_count: form.items.len() as u32,
                    active_session_id,
                }
            })
            .collect();
        Ok(ToeicOverviewDto {
            bank_id: self.bank.bank_id().to_owned(), untimed: true, parts: part_availability(), forms,
            active_sessions: active, recent_history: history,
            disclaimer: "Independent TOEIC-style preparation. Not affiliated with or endorsed by ETS. TOEIC is a trademark of ETS; internal results are not official TOEIC scores.".into(),
        })
    }

    pub fn start(&self, form_id: &str, form_version: u32) -> Result<ToeicSessionDto, String> {
        let form = self
            .bank
            .form(form_id, form_version)
            .ok_or("Published TOEIC form not found.")?;
        if !form.part.runtime_available() || form.items.len() != FORM_LENGTH {
            return Err("This TOEIC form is not available in Phase 1.".into());
        }
        let connection = database::open(&self.database)?;
        if let Some(id) = connection.query_row("SELECT id FROM toeic_session WHERE form_id=?1 AND form_version=?2 AND status='in_progress'", params![form_id, form_version], |row| row.get::<_, String>(0)).optional().map_err(db_error)? {
            return self.session(&id);
        }
        let snapshot = FormSnapshot {
            form_id: form.form_id.clone(),
            form_version: form.form_version,
            items: form
                .items
                .iter()
                .map(|item| FormSnapshotItem {
                    item_id: item.item_id.clone(),
                    item_version: item.item_version,
                })
                .collect(),
        };
        let id = uuid::Uuid::new_v4().to_string();
        connection.execute("INSERT INTO toeic_session(id,form_id,form_version,section,part,status,schema_version,form_snapshot_json,current_question_index,created_at,updated_at) VALUES(?1,?2,?3,'listening','part1_photograph','in_progress',?4,?5,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'))", params![id, form.form_id, form.form_version, SESSION_SCHEMA_VERSION, serde_json::to_string(&snapshot).map_err(|_| "Could not snapshot TOEIC form.")?]).map_err(db_error)?;
        self.session(&id)
    }

    pub fn session(&self, session_id: &str) -> Result<ToeicSessionDto, String> {
        let connection = database::open(&self.database)?;
        let row = connection.query_row("SELECT form_id,form_version,status,form_snapshot_json,current_question_index,created_at,updated_at FROM toeic_session WHERE id=?1", [session_id], |row| Ok((row.get::<_,String>(0)?,row.get::<_,u32>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?,row.get::<_,u32>(4)?,row.get::<_,String>(5)?,row.get::<_,String>(6)?))).optional().map_err(db_error)?.ok_or("TOEIC session not found.")?;
        let snapshot = parse_snapshot(&row.3)?;
        let answered_count = connection
            .query_row(
                "SELECT COUNT(*) FROM toeic_answer WHERE session_id=?1",
                [session_id],
                |r| r.get::<_, u32>(0),
            )
            .map_err(db_error)?;
        let current = if row.2 == "in_progress" {
            snapshot.items.get(row.4 as usize)
        } else {
            None
        };
        let (question, feedback) = if let Some(reference) = current {
            let item = self
                .bank
                .item(&reference.item_id, reference.item_version)
                .ok_or("The snapshotted TOEIC item version is unavailable.")?;
            let answer = answer_row(&connection, session_id, &item.item_id, item.item_version)?;
            let presentation =
                presentation_state(&connection, session_id, &item.item_id, item.item_version)?;
            (
                Some(self.public_question(&item, row.4, presentation.0, presentation.1)?),
                answer.map(|answer| feedback(&item, answer)),
            )
        } else {
            (None, None)
        };
        Ok(ToeicSessionDto {
            session_id: session_id.into(),
            form_id: row.0.clone(),
            form_version: row.1,
            form_title: form_title(&row.0),
            status: row.2,
            untimed: true,
            current_question_index: row.4,
            answered_count,
            current_question: question,
            feedback,
            created_at: row.5,
            updated_at: row.6,
        })
    }

    fn public_question(
        &self,
        item: &ToeicItem,
        index: u32,
        audio_completed: bool,
        audio_interrupted: bool,
    ) -> Result<ToeicPublicQuestionDto, String> {
        Ok(ToeicPublicQuestionDto {
            item_id: item.item_id.clone(),
            item_version: item.item_version,
            question_number: index + 1,
            total_questions: FORM_LENGTH as u32,
            image_base64: STANDARD.encode(self.bank.image_bytes(item)?),
            image_mime_type: "image/png".into(),
            choices: vec!["A".into(), "B".into(), "C".into(), "D".into()],
            initial_audio_completed: audio_completed,
            initial_audio_interrupted: audio_interrupted,
        })
    }

    pub fn begin_audio(&self, session_id: &str) -> Result<ToeicAudioContext, String> {
        let session = self.session(session_id)?;
        if session.status != "in_progress" {
            return Err("This TOEIC session is not active.".into());
        }
        let question = session
            .current_question
            .ok_or("No current TOEIC question.")?;
        let item = self
            .bank
            .item(&question.item_id, question.item_version)
            .ok_or("TOEIC item unavailable.")?;
        let connection = database::open(&self.database)?;
        let answered =
            answer_row(&connection, session_id, &item.item_id, item.item_version)?.is_some();
        let initial = !answered;
        let presentation_id = if initial {
            if question.initial_audio_completed {
                return Err(
                    "The initial statements can be played only once before answering.".into(),
                );
            }
            connection.execute("UPDATE toeic_presentation_attempt SET status='interrupted',interrupted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE session_id=?1 AND item_id=?2 AND item_version=?3 AND status='started'", params![session_id,item.item_id,item.item_version]).map_err(db_error)?;
            let id = uuid::Uuid::new_v4().to_string();
            connection.execute("INSERT INTO toeic_presentation_attempt(id,session_id,item_id,item_version,status,started_at) VALUES(?1,?2,?3,?4,'started',strftime('%Y-%m-%dT%H:%M:%fZ','now'))", params![id,session_id,item.item_id,item.item_version]).map_err(db_error)?;
            Some(id)
        } else {
            None
        };
        let script = audio_script(&item);
        Ok(ToeicAudioContext {
            item_id: item.item_id,
            item_version: item.item_version,
            script,
            presentation_id,
            initial,
        })
    }

    pub fn interrupt_audio(&self, presentation_id: Option<&str>) -> Result<(), String> {
        if let Some(id) = presentation_id {
            database::open(&self.database)?.execute("UPDATE toeic_presentation_attempt SET status='interrupted',interrupted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND status='started'", [id]).map_err(db_error)?;
        }
        Ok(())
    }

    pub fn complete_audio(
        &self,
        session_id: &str,
        item_id: &str,
        item_version: u32,
        presentation_id: Option<&str>,
    ) -> Result<(), String> {
        if let Some(id) = presentation_id {
            let changed = database::open(&self.database)?.execute("UPDATE toeic_presentation_attempt SET status='completed',completed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND session_id=?2 AND item_id=?3 AND item_version=?4 AND status='started'", params![id,session_id,item_id,item_version]).map_err(db_error)?;
            if changed != 1 {
                return Err("Stale TOEIC audio completion was ignored.".into());
            }
        }
        Ok(())
    }

    pub fn submit(&self, request: ToeicSubmitAnswerRequest) -> Result<ToeicSessionDto, String> {
        if !matches!(request.selected_choice.as_str(), "A" | "B" | "C" | "D") {
            return Err("Answer must be A, B, C, or D.".into());
        }
        let session = self.session(&request.session_id)?;
        let question = session
            .current_question
            .ok_or("No active TOEIC question.")?;
        if question.item_id != request.item_id || question.item_version != request.item_version {
            return Err("The submitted TOEIC question is stale.".into());
        }
        if !question.initial_audio_completed {
            return Err("Listen to all four statements before answering.".into());
        }
        let item = self
            .bank
            .item(&request.item_id, request.item_version)
            .ok_or("TOEIC item unavailable.")?;
        let connection = database::open(&self.database)?;
        let inserted = connection.execute("INSERT OR IGNORE INTO toeic_answer(id,session_id,item_id,item_version,selected_choice,is_correct,first_attempt,answered_at) VALUES(?1,?2,?3,?4,?5,?6,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'))", params![uuid::Uuid::new_v4().to_string(),request.session_id,request.item_id,request.item_version,request.selected_choice,u8::from(request.selected_choice==item.correct_answer)]).map_err(db_error)?;
        if inserted != 1 {
            return Err("The first answer is final and this question is already locked.".into());
        }
        connection.execute("UPDATE toeic_session SET updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND status='in_progress'", [&request.session_id]).map_err(db_error)?;
        self.session(&request.session_id)
    }

    pub fn advance(&self, session_id: &str) -> Result<ToeicSessionDto, String> {
        let session = self.session(session_id)?;
        let question = session
            .current_question
            .ok_or("No active TOEIC question.")?;
        if session.feedback.is_none() {
            return Err("Answer the current question before continuing.".into());
        }
        let connection = database::open(&self.database)?;
        if question.question_number as usize == FORM_LENGTH {
            connection.execute("UPDATE toeic_session SET status='completed',current_question_index=?2,completed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND status='in_progress'", params![session_id,FORM_LENGTH as u32]).map_err(db_error)?;
        } else {
            connection.execute("UPDATE toeic_session SET current_question_index=current_question_index+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND status='in_progress'", [session_id]).map_err(db_error)?;
        }
        self.session(session_id)
    }

    pub fn abandon(&self, session_id: &str) -> Result<(), String> {
        database::open(&self.database)?.execute("UPDATE toeic_session SET status='abandoned',abandoned_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND status='in_progress'", [session_id]).map_err(db_error)?;
        Ok(())
    }

    pub fn record_active_time(
        &self,
        session_id: &str,
        event_id: &str,
        seconds: u32,
    ) -> Result<u32, String> {
        if event_id.len() > 100 || !(1..=30).contains(&seconds) {
            return Err("Invalid TOEIC active-time event.".into());
        }
        let connection = database::open(&self.database)?;
        let exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM toeic_session WHERE id=?1 AND status='in_progress')",
                [session_id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        if !exists {
            return Err("Active TOEIC session not found.".into());
        }
        connection.execute("INSERT OR IGNORE INTO toeic_active_time_event(event_id,session_id,duration_seconds,recorded_at) VALUES(?1,?2,?3,strftime('%Y-%m-%dT%H:%M:%fZ','now'))", params![event_id,session_id,seconds]).map_err(db_error)?;
        connection.query_row("SELECT COALESCE(SUM(duration_seconds),0) FROM toeic_active_time_event WHERE session_id=?1", [session_id], |r| r.get(0)).map_err(db_error)
    }

    pub fn result(&self, session_id: &str) -> Result<ToeicResultDto, String> {
        let connection = database::open(&self.database)?;
        let (form_id,status,snapshot_json,completed_at) = connection.query_row("SELECT form_id,status,form_snapshot_json,completed_at FROM toeic_session WHERE id=?1", [session_id], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,Option<String>>(3)?))).optional().map_err(db_error)?.ok_or("TOEIC session not found.")?;
        let snapshot = parse_snapshot(&snapshot_json)?;
        let answers = all_answers(&connection, session_id)?;
        let correct = answers.values().filter(|answer| answer.is_correct).count() as u32;
        let mut skills: BTreeMap<String, (u32, u32)> = BTreeMap::new();
        let mut difficulty: BTreeMap<String, (u32, u32)> = BTreeMap::new();
        let mut distractors: BTreeMap<String, u32> = BTreeMap::new();
        for reference in &snapshot.items {
            let item = self
                .bank
                .item(&reference.item_id, reference.item_version)
                .ok_or("Snapshotted TOEIC item unavailable.")?;
            if let Some(answer) = answers.get(&(reference.item_id.clone(), reference.item_version))
            {
                for tag in &item.skill_tags {
                    let count = skills.entry(tag.clone()).or_default();
                    count.1 += 1;
                    if answer.is_correct {
                        count.0 += 1;
                    }
                }
                let count = difficulty
                    .entry(item.difficulty.as_str().into())
                    .or_default();
                count.1 += 1;
                if answer.is_correct {
                    count.0 += 1;
                }
                if !answer.is_correct {
                    if let Some(kind) = item
                        .statements
                        .iter()
                        .find(|s| s.choice == answer.selected_choice)
                        .and_then(|s| s.distractor_type.clone())
                    {
                        *distractors.entry(kind).or_default() += 1;
                    }
                }
            }
        }
        Ok(ToeicResultDto { session_id:session_id.into(),form_id:form_id.clone(),form_title:form_title(&form_id),status,correct,total:FORM_LENGTH as u32,accuracy:accuracy(correct,FORM_LENGTH as u32),skill_breakdown:to_breakdown(skills),difficulty_breakdown:to_breakdown(difficulty),common_distractors:distractors.into_iter().map(|(distractor_type,count)|ToeicDistractorCountDto{distractor_type,count}).collect(),has_scaled_score:false,score_message:"Part 1 performance only. Six questions cannot produce an official or estimated TOEIC Listening score.".into(),completed_at })
    }

    pub fn review(
        &self,
        session_id: &str,
        mistakes_only: bool,
    ) -> Result<Vec<ToeicReviewItemDto>, String> {
        let connection = database::open(&self.database)?;
        let (status, snapshot_json) = connection
            .query_row(
                "SELECT status,form_snapshot_json FROM toeic_session WHERE id=?1",
                [session_id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(db_error)?
            .ok_or("TOEIC session not found.")?;
        if status != "completed" {
            return Err("Complete the Part 1 form before reviewing it.".into());
        }
        let snapshot = parse_snapshot(&snapshot_json)?;
        let answers = all_answers(&connection, session_id)?;
        let mut result = Vec::new();
        for (index, reference) in snapshot.items.iter().enumerate() {
            let Some(answer) = answers.get(&(reference.item_id.clone(), reference.item_version))
            else {
                continue;
            };
            if mistakes_only && answer.is_correct {
                continue;
            }
            let item = self
                .bank
                .item(&reference.item_id, reference.item_version)
                .ok_or("Snapshotted TOEIC item unavailable.")?;
            result.push(ToeicReviewItemDto {
                item_id: item.item_id.clone(),
                question_number: index as u32 + 1,
                image_base64: STANDARD.encode(self.bank.image_bytes(&item)?),
                image_mime_type: "image/png".into(),
                difficulty: item.difficulty.as_str().into(),
                skill_tags: item.skill_tags.clone(),
                feedback: feedback(&item, answer.clone()),
            });
        }
        Ok(result)
    }

    pub fn history(&self) -> Result<Vec<ToeicHistoryEntryDto>, String> {
        self.history_query(None, 100)
    }

    fn history_query(
        &self,
        status: Option<&str>,
        limit: u32,
    ) -> Result<Vec<ToeicHistoryEntryDto>, String> {
        let connection = database::open(&self.database)?;
        let mut sql="SELECT s.id,s.form_id,s.status,s.created_at,s.updated_at,s.completed_at,COUNT(a.id),COALESCE(SUM(a.is_correct),0) FROM toeic_session s LEFT JOIN toeic_answer a ON a.session_id=s.id".to_owned();
        if status.is_some() {
            sql.push_str(" WHERE s.status=?1");
        }
        sql.push_str(" GROUP BY s.id ORDER BY s.created_at DESC LIMIT ");
        sql.push_str(&limit.min(100).to_string());
        let mut statement = connection.prepare(&sql).map_err(db_error)?;
        let mapper = |r: &rusqlite::Row<'_>| -> rusqlite::Result<ToeicHistoryEntryDto> {
            let correct = r.get::<_, u32>(7)?;
            let answered = r.get::<_, u32>(6)?;
            let form_id = r.get::<_, String>(1)?;
            Ok(ToeicHistoryEntryDto {
                session_id: r.get(0)?,
                form_title: form_title(&form_id),
                form_id,
                status: r.get(2)?,
                correct,
                answered,
                total: FORM_LENGTH as u32,
                accuracy: accuracy(correct, answered),
                created_at: r.get(3)?,
                updated_at: r.get(4)?,
                completed_at: r.get(5)?,
            })
        };
        let rows = if let Some(value) = status {
            statement.query_map([value], mapper)
        } else {
            statement.query_map([], mapper)
        }
        .map_err(db_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
    }

    fn recover_interrupted_presentations(&self) -> Result<u32, String> {
        let connection = database::open(&self.database)?;
        connection.execute("UPDATE toeic_presentation_attempt SET status='interrupted',interrupted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE status='started'",[]).map(|n|n as u32).map_err(db_error)
    }
}

#[derive(Clone)]
struct AnswerRow {
    selected_choice: String,
    is_correct: bool,
    answered_at: String,
}
fn answer_row(
    connection: &rusqlite::Connection,
    session_id: &str,
    item_id: &str,
    item_version: u32,
) -> Result<Option<AnswerRow>, String> {
    connection.query_row("SELECT selected_choice,is_correct,answered_at FROM toeic_answer WHERE session_id=?1 AND item_id=?2 AND item_version=?3",params![session_id,item_id,item_version],|r|Ok(AnswerRow{selected_choice:r.get(0)?,is_correct:r.get(1)?,answered_at:r.get(2)?})).optional().map_err(db_error)
}
fn all_answers(
    connection: &rusqlite::Connection,
    session_id: &str,
) -> Result<BTreeMap<(String, u32), AnswerRow>, String> {
    let mut statement=connection.prepare("SELECT item_id,item_version,selected_choice,is_correct,answered_at FROM toeic_answer WHERE session_id=?1").map_err(db_error)?;
    let rows = statement
        .query_map([session_id], |r| {
            Ok((
                (r.get(0)?, r.get(1)?),
                AnswerRow {
                    selected_choice: r.get(2)?,
                    is_correct: r.get(3)?,
                    answered_at: r.get(4)?,
                },
            ))
        })
        .map_err(db_error)?;
    rows.collect::<Result<BTreeMap<_, _>, _>>()
        .map_err(db_error)
}
fn presentation_state(
    connection: &rusqlite::Connection,
    session_id: &str,
    item_id: &str,
    item_version: u32,
) -> Result<(bool, bool), String> {
    let (completed,interrupted)=connection.query_row("SELECT EXISTS(SELECT 1 FROM toeic_presentation_attempt WHERE session_id=?1 AND item_id=?2 AND item_version=?3 AND status='completed'),EXISTS(SELECT 1 FROM toeic_presentation_attempt WHERE session_id=?1 AND item_id=?2 AND item_version=?3 AND status='interrupted')",params![session_id,item_id,item_version],|r|Ok((r.get(0)?,r.get(1)?))).map_err(db_error)?;
    Ok((completed, interrupted))
}
fn parse_snapshot(json: &str) -> Result<FormSnapshot, String> {
    let snapshot: FormSnapshot = serde_json::from_str(json)
        .map_err(|_| "Stored TOEIC form snapshot is invalid.".to_owned())?;
    if snapshot.items.len() != FORM_LENGTH {
        return Err("Stored TOEIC form snapshot has an invalid item count.".into());
    }
    Ok(snapshot)
}
fn audio_script(item: &ToeicItem) -> String {
    item.statements
        .iter()
        .map(|s| format!("{}. {}", s.choice, s.text))
        .collect::<Vec<_>>()
        .join("  ")
}
fn feedback(item: &ToeicItem, answer: AnswerRow) -> ToeicFeedbackDto {
    ToeicFeedbackDto {
        selected_choice: answer.selected_choice.clone(),
        is_correct: answer.is_correct,
        correct_answer: item.correct_answer.clone(),
        statements: item
            .statements
            .iter()
            .map(|s| ToeicTranscriptDto {
                choice: s.choice.clone(),
                text: s.text.clone(),
            })
            .collect(),
        correct_explanation: item.correct_explanation.clone(),
        selected_explanation: if answer.is_correct {
            None
        } else {
            item.distractor_explanations
                .get(&answer.selected_choice)
                .cloned()
        },
        language_focus: item.language_focus.clone(),
        useful_vocabulary: item.useful_vocabulary.clone(),
        answered_at: answer.answered_at,
    }
}
fn accuracy(correct: u32, total: u32) -> u32 {
    if total == 0 {
        0
    } else {
        ((correct as f64 / total as f64) * 100.0).round() as u32
    }
}
fn to_breakdown(map: BTreeMap<String, (u32, u32)>) -> Vec<ToeicBreakdownDto> {
    map.into_iter()
        .map(|(label, (correct, total))| ToeicBreakdownDto {
            label,
            correct,
            total,
        })
        .collect()
}
fn form_title(id: &str) -> String {
    id.strip_prefix("toeic-part1-form-")
        .map(|suffix| format!("Part 1 Form {}", suffix.to_uppercase()))
        .unwrap_or_else(|| id.to_owned())
}
fn db_error(error: rusqlite::Error) -> String {
    format!("TOEIC database operation failed: {error}")
}
fn part_availability() -> Vec<ToeicPartAvailabilityDto> {
    vec![
        (ToeicPart::Part1Photograph, "Photographs", 6),
        (ToeicPart::Part2QuestionResponse, "Question–Response", 25),
        (ToeicPart::Part3Conversation, "Conversations", 39),
        (ToeicPart::Part4Talk, "Talks", 30),
        (
            ToeicPart::Part5IncompleteSentence,
            "Incomplete Sentences",
            30,
        ),
        (ToeicPart::Part6TextCompletion, "Text Completion", 16),
        (
            ToeicPart::Part7ReadingComprehension,
            "Reading Comprehension",
            54,
        ),
    ]
    .into_iter()
    .map(|(part, title, count)| ToeicPartAvailabilityDto {
        part: part.as_str().into(),
        title: title.into(),
        question_count: count,
        runtime_available: part.runtime_available(),
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (ToeicRepository, PathBuf) {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let directory = std::env::temp_dir().join(format!("toeic-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let database = directory.join("test.sqlite3");
        crate::database::migrate(&database).unwrap();
        let bank = ToeicItemBank::load(root.join("resources/toeic/item-bank-v1")).unwrap();
        (ToeicRepository::new(database, bank).unwrap(), directory)
    }
    fn complete_audio(repo: &ToeicRepository, session: &ToeicSessionDto) {
        let q = session.current_question.as_ref().unwrap();
        let context = repo.begin_audio(&session.session_id).unwrap();
        repo.complete_audio(
            &session.session_id,
            &q.item_id,
            q.item_version,
            context.presentation_id.as_deref(),
        )
        .unwrap();
    }
    #[test]
    fn public_question_serialization_never_leaks_answers_or_explanations() {
        let (repo, dir) = setup();
        let session = repo.start("toeic-part1-form-a", 1).unwrap();
        let json = serde_json::to_string(&session.current_question.unwrap()).unwrap();
        assert!(!json.contains("correctAnswer"));
        assert!(!json.contains("Explanation"));
        assert!(!json.contains("statement"));
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn first_answer_is_immutable_and_wrong_answer_stays_wrong() {
        let (repo, dir) = setup();
        let session = repo.start("toeic-part1-form-a", 1).unwrap();
        complete_audio(&repo, &session);
        let q = session.current_question.unwrap();
        let request = ToeicSubmitAnswerRequest {
            session_id: session.session_id,
            item_id: q.item_id,
            item_version: q.item_version,
            selected_choice: "A".into(),
        };
        let answered = repo.submit(request.clone()).unwrap();
        assert!(!answered.feedback.unwrap().is_correct);
        assert!(repo
            .submit(ToeicSubmitAnswerRequest {
                selected_choice: "B".into(),
                ..request
            })
            .is_err());
        assert_eq!(repo.result(&answered.session_id).unwrap().correct, 0);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn interruption_allows_restart_without_penalty() {
        let (repo, dir) = setup();
        let session = repo.start("toeic-part1-form-a", 1).unwrap();
        let context = repo.begin_audio(&session.session_id).unwrap();
        repo.interrupt_audio(context.presentation_id.as_deref())
            .unwrap();
        let resumed = repo.session(&session.session_id).unwrap();
        assert!(resumed.current_question.unwrap().initial_audio_interrupted);
        assert!(repo.begin_audio(&session.session_id).is_ok());
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn session_resumes_hours_later_with_order_and_answers_intact() {
        let (repo, dir) = setup();
        let mut session = repo.start("toeic-part1-form-a", 1).unwrap();
        for choice in ["B", "C"] {
            complete_audio(&repo, &session);
            let q = session.current_question.clone().unwrap();
            session = repo
                .submit(ToeicSubmitAnswerRequest {
                    session_id: session.session_id.clone(),
                    item_id: q.item_id,
                    item_version: q.item_version,
                    selected_choice: choice.into(),
                })
                .unwrap();
            session = repo.advance(&session.session_id).unwrap();
        }
        database::open(&repo.database)
            .unwrap()
            .execute(
                "UPDATE toeic_session SET updated_at='2026-08-31T01:00:00.000Z' WHERE id=?1",
                [&session.session_id],
            )
            .unwrap();
        let resumed = repo.session(&session.session_id).unwrap();
        assert_eq!(resumed.current_question_index, 2);
        assert_eq!(resumed.answered_count, 2);
        assert_eq!(resumed.current_question.unwrap().item_id, "toeic-l-p1-0003");
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn result_math_supports_zero_one_three_and_six_without_scaled_score() {
        for target in [0, 1, 3, 6] {
            let (repo, dir) = setup();
            let mut session = repo.start("toeic-part1-form-a", 1).unwrap();
            for index in 0..6 {
                complete_audio(&repo, &session);
                let q = session.current_question.clone().unwrap();
                let item = repo.bank.item(&q.item_id, q.item_version).unwrap();
                let selected = if index < target {
                    item.correct_answer
                } else {
                    item.statements
                        .iter()
                        .find(|s| s.choice != item.correct_answer)
                        .unwrap()
                        .choice
                        .clone()
                };
                session = repo
                    .submit(ToeicSubmitAnswerRequest {
                        session_id: session.session_id.clone(),
                        item_id: q.item_id,
                        item_version: q.item_version,
                        selected_choice: selected,
                    })
                    .unwrap();
                session = repo.advance(&session.session_id).unwrap();
            }
            let result = repo.result(&session.session_id).unwrap();
            assert_eq!(result.correct, target);
            assert_eq!(result.accuracy, accuracy(target, 6));
            assert!(!result.has_scaled_score);
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn completed_form_exposes_separate_history_and_mistake_review() {
        let (repo, dir) = setup();
        let mut session = repo.start("toeic-part1-form-b", 1).unwrap();
        for _ in 0..6 {
            complete_audio(&repo, &session);
            let question = session.current_question.clone().unwrap();
            let item = repo
                .bank
                .item(&question.item_id, question.item_version)
                .unwrap();
            let wrong = item
                .statements
                .iter()
                .find(|value| value.choice != item.correct_answer)
                .unwrap()
                .choice
                .clone();
            session = repo
                .submit(ToeicSubmitAnswerRequest {
                    session_id: session.session_id.clone(),
                    item_id: question.item_id,
                    item_version: question.item_version,
                    selected_choice: wrong,
                })
                .unwrap();
            session = repo.advance(&session.session_id).unwrap();
        }
        assert_eq!(session.status, "completed");
        assert_eq!(repo.review(&session.session_id, true).unwrap().len(), 6);
        assert_eq!(repo.review(&session.session_id, false).unwrap().len(), 6);
        let history = repo.history().unwrap();
        assert_eq!(history[0].correct, 0);
        assert_eq!(history[0].status, "completed");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn session_survives_repository_reopen_like_an_app_restart() {
        let (repo, dir) = setup();
        let database = repo.database.clone();
        let session = repo.start("toeic-part1-form-c", 1).unwrap();
        complete_audio(&repo, &session);
        let question = session.current_question.clone().unwrap();
        let answered = repo
            .submit(ToeicSubmitAnswerRequest {
                session_id: session.session_id.clone(),
                item_id: question.item_id,
                item_version: question.item_version,
                selected_choice: "A".into(),
            })
            .unwrap();
        drop(repo);
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let reopened = ToeicRepository::new(
            database,
            ToeicItemBank::load(root.join("resources/toeic/item-bank-v1")).unwrap(),
        )
        .unwrap();
        let restored = reopened.session(&answered.session_id).unwrap();
        assert_eq!(restored.answered_count, 1);
        assert!(restored.feedback.is_some());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
