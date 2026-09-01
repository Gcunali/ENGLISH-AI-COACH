use crate::{database, interactive_lesson::*, sha256};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::BTreeSet, path::PathBuf};

pub const PRACTICE_SCHEMA_VERSION: u32 = 1;
pub const PRACTICE_SELECTION_VERSION: u32 = 1;
pub const PRACTICE_XP_RULE_VERSION: u32 = 1;
pub const PRACTICE_SESSION_XP: u32 = 20;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PracticeMode {
    Daily,
    Dictation,
    Shadowing,
    MistakeRepair,
    SpeakingRecall,
}

impl PracticeMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Dictation => "dictation",
            Self::Shadowing => "shadowing",
            Self::MistakeRepair => "mistake_repair",
            Self::SpeakingRecall => "speaking_recall",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum PracticeItem {
    Vocabulary {
        item_id: String,
        term: String,
        meaning: String,
        example: String,
        status: String,
        source_label: String,
    },
    Dictation {
        item_id: String,
        target_text: String,
        source_label: String,
    },
    Shadowing {
        item_id: String,
        target_text: String,
        hint: Option<String>,
        source_label: String,
    },
    MistakeRepair {
        item_id: String,
        mistake_id: String,
        original: String,
        corrected: String,
        explanation: String,
        source_label: String,
    },
    SpeakingRecall {
        item_id: String,
        situation: String,
        prompt: String,
        model_expression: String,
        source_label: String,
    },
}

impl PracticeItem {
    fn id(&self) -> &str {
        match self {
            Self::Vocabulary { item_id, .. }
            | Self::Dictation { item_id, .. }
            | Self::Shadowing { item_id, .. }
            | Self::MistakeRepair { item_id, .. }
            | Self::SpeakingRecall { item_id, .. } => item_id,
        }
    }
    fn supports(&self, mode: PracticeMode) -> bool {
        mode == PracticeMode::Daily
            || matches!(
                (self, mode),
                (Self::Dictation { .. }, PracticeMode::Dictation)
                    | (Self::Shadowing { .. }, PracticeMode::Shadowing)
                    | (Self::MistakeRepair { .. }, PracticeMode::MistakeRepair)
                    | (Self::SpeakingRecall { .. }, PracticeMode::SpeakingRecall)
            )
    }
    fn kind_name(&self) -> &'static str {
        match self {
            Self::Vocabulary { .. } => "vocabulary",
            Self::Dictation { .. } => "dictation",
            Self::Shadowing { .. } => "shadowing",
            Self::MistakeRepair { .. } => "mistake_repair",
            Self::SpeakingRecall { .. } => "speaking_recall",
        }
    }

    fn daily_priority(&self) -> u8 {
        match self {
            Self::MistakeRepair { .. } => 0,
            Self::Vocabulary { status, .. } if status == "learning" => 1,
            Self::Dictation { .. } | Self::Shadowing { .. } | Self::SpeakingRecall { .. } => 2,
            Self::Vocabulary { .. } => 3,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeAvailabilityDto {
    pub schema_version: u32,
    pub daily_count: usize,
    pub dictation_count: usize,
    pub shadowing_count: usize,
    pub mistake_repair_count: usize,
    pub speaking_recall_count: usize,
    pub confirmed_mistake_count: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeSessionDto {
    pub id: String,
    pub mode: PracticeMode,
    pub status: String,
    pub schema_version: u32,
    pub selection_version: u32,
    pub items: Vec<PracticeItem>,
    pub completed_item_ids: Vec<String>,
    pub active_seconds: u32,
    pub xp_awarded: u32,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartPracticeRequest {
    pub mode: PracticeMode,
    pub item_count: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompletePracticeItemRequest {
    pub session_id: String,
    pub item_id: String,
    #[serde(default)]
    pub response: Option<String>,
    #[serde(default)]
    pub self_assessment: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeItemResultDto {
    pub session: PracticeSessionDto,
    pub item_id: String,
    pub exact_match: Option<bool>,
    pub similarity_percent: Option<u32>,
    pub expected_text: Option<String>,
    pub normalized_response: Option<String>,
    pub word_diff: Vec<WordDiffDto>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WordDiffDto {
    pub word: String,
    pub status: String,
}

#[derive(Clone)]
pub struct PracticeRepository {
    database: PathBuf,
}

impl PracticeRepository {
    pub fn new(database: PathBuf) -> Self {
        Self { database }
    }

    pub fn availability(&self) -> Result<PracticeAvailabilityDto, String> {
        let connection = database::open(&self.database)?;
        let items = candidates(&connection)?;
        let count = |mode| items.iter().filter(|item| item.supports(mode)).count();
        Ok(PracticeAvailabilityDto {
            schema_version: PRACTICE_SCHEMA_VERSION,
            daily_count: items.len(),
            dictation_count: count(PracticeMode::Dictation),
            shadowing_count: count(PracticeMode::Shadowing),
            mistake_repair_count: count(PracticeMode::MistakeRepair),
            speaking_recall_count: count(PracticeMode::SpeakingRecall),
            confirmed_mistake_count: count(PracticeMode::MistakeRepair),
        })
    }

    pub fn start(&self, request: StartPracticeRequest) -> Result<PracticeSessionDto, String> {
        if !(1..=20).contains(&request.item_count) {
            return Err("Practice item count must be between 1 and 20.".into());
        }
        let connection = database::open(&self.database)?;
        let today: String = connection
            .query_row("SELECT date('now','localtime')", [], |row| row.get(0))
            .map_err(db)?;
        let recent = recent_item_ids(&connection)?;
        let mut eligible = candidates(&connection)?
            .into_iter()
            .filter(|item| item.supports(request.mode))
            .collect::<Vec<_>>();
        eligible.sort_by_key(|item| {
            (
                if request.mode == PracticeMode::Daily {
                    item.daily_priority()
                } else {
                    0
                },
                sha256::bytes(format!("{today}|{}", item.id()).as_bytes()),
            )
        });
        let mut fresh = eligible
            .iter()
            .filter(|item| !recent.contains(item.id()))
            .cloned()
            .collect::<Vec<_>>();
        fresh.extend(
            eligible
                .into_iter()
                .filter(|item| recent.contains(item.id())),
        );
        let ordered = if request.mode == PracticeMode::Daily {
            interleave_kinds(fresh)
        } else {
            fresh
        };
        let mut seen = BTreeSet::new();
        let selected = ordered
            .into_iter()
            .filter(|item| seen.insert(item.id().to_owned()))
            .take(request.item_count as usize)
            .collect::<Vec<_>>();
        if selected.is_empty() {
            return Err(empty_message(request.mode).into());
        }
        let id = uuid::Uuid::new_v4().to_string();
        let items_json = serde_json::to_string(&selected).map_err(json_error)?;
        connection.execute(
            "INSERT INTO learning_practice_session
             (id,mode,status,schema_version,selection_version,requested_item_count,actual_item_count,items_json,started_at,updated_at)
             VALUES(?1,?2,'in_progress',?3,?4,?5,?6,?7,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            params![id, request.mode.as_str(), PRACTICE_SCHEMA_VERSION, PRACTICE_SELECTION_VERSION, request.item_count, selected.len() as u32, items_json],
        ).map_err(db)?;
        session_with(&connection, &id)
    }

    pub fn get(&self, id: &str) -> Result<PracticeSessionDto, String> {
        session_with(&database::open(&self.database)?, id)
    }

    pub fn audio_text(&self, session_id: &str, item_id: &str) -> Result<String, String> {
        let session = self.get(session_id)?;
        let item = session
            .items
            .iter()
            .find(|item| item.id() == item_id)
            .ok_or("Practice item does not belong to this session.")?;
        match item {
            PracticeItem::Dictation { target_text, .. }
            | PracticeItem::Shadowing { target_text, .. } => Ok(target_text.clone()),
            PracticeItem::SpeakingRecall {
                model_expression, ..
            } => Ok(model_expression.clone()),
            PracticeItem::MistakeRepair { corrected, .. } => Ok(corrected.clone()),
            PracticeItem::Vocabulary { term, .. } => Ok(term.clone()),
        }
    }

    pub fn record_time(
        &self,
        session_id: &str,
        event_id: &str,
        seconds: u32,
    ) -> Result<u32, String> {
        if event_id.trim().is_empty() || !(1..=30).contains(&seconds) {
            return Err("Practice time event is invalid.".into());
        }
        let connection = database::open(&self.database)?;
        let status: Option<String> = connection
            .query_row(
                "SELECT status FROM learning_practice_session WHERE id=?1",
                [session_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(db)?;
        if status.as_deref() != Some("in_progress") {
            return Err("Practice session is not active.".into());
        }
        connection.execute(
            "INSERT OR IGNORE INTO learning_practice_active_time_event(event_id,session_id,duration_seconds,recorded_at)
             VALUES(?1,?2,?3,strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            params![event_id, session_id, seconds],
        ).map_err(db)?;
        active_seconds(&connection, session_id)
    }

    pub fn complete_item(
        &self,
        request: CompletePracticeItemRequest,
    ) -> Result<PracticeItemResultDto, String> {
        let mut connection = database::open(&self.database)?;
        let transaction = connection.transaction().map_err(db)?;
        let session = session_with(&transaction, &request.session_id)?;
        if session.status != "in_progress" {
            return Err("Practice session is already closed.".into());
        }
        let item = session
            .items
            .iter()
            .find(|item| item.id() == request.item_id)
            .ok_or("Practice item does not belong to this session.")?;
        let (expected, exact, similarity, normalized_response, diff) = match item {
            PracticeItem::Dictation { target_text, .. } => {
                let response = request.response.as_deref().unwrap_or("");
                let target_words = words(target_text);
                let response_words = words(response);
                let matches = target_words
                    .iter()
                    .zip(&response_words)
                    .filter(|(a, b)| a == b)
                    .count();
                let denominator = target_words.len().max(response_words.len()).max(1);
                let percent = ((matches * 100) / denominator) as u32;
                let word_diff = target_words
                    .iter()
                    .enumerate()
                    .map(|(index, word)| WordDiffDto {
                        word: word.clone(),
                        status: if response_words.get(index) == Some(word) {
                            "match"
                        } else {
                            "review"
                        }
                        .into(),
                    })
                    .collect::<Vec<_>>();
                (
                    Some(target_text.clone()),
                    Some(target_words == response_words),
                    Some(percent),
                    Some(response_words.join(" ")),
                    word_diff,
                )
            }
            _ => (
                None,
                None,
                None,
                request.response.as_deref().map(normalize),
                Vec::new(),
            ),
        };
        let result_json = json!({
            "response": request.response,
            "selfAssessment": request.self_assessment,
            "exactMatch": exact,
            "similarityPercent": similarity
        })
        .to_string();
        transaction.execute(
            "INSERT OR IGNORE INTO learning_practice_item_result(id,session_id,item_id,result_schema_version,result_json,completed_at)
             VALUES(?1,?2,?3,1,?4,strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            params![uuid::Uuid::new_v4().to_string(), request.session_id, request.item_id, result_json],
        ).map_err(db)?;
        transaction.execute(
            "UPDATE learning_practice_session SET updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
            [&request.session_id],
        ).map_err(db)?;
        transaction.commit().map_err(db)?;
        Ok(PracticeItemResultDto {
            session: self.get(&request.session_id)?,
            item_id: request.item_id,
            exact_match: exact,
            similarity_percent: similarity,
            expected_text: expected,
            normalized_response,
            word_diff: diff,
        })
    }

    pub fn complete(&self, session_id: &str) -> Result<PracticeSessionDto, String> {
        let mut connection = database::open(&self.database)?;
        let transaction = connection.transaction().map_err(db)?;
        let session = session_with(&transaction, session_id)?;
        if session.status == "completed" {
            transaction.commit().map_err(db)?;
            return self.get(session_id);
        }
        if session.status != "in_progress"
            || session.completed_item_ids.len() != session.items.len()
        {
            return Err(
                "Complete every available practice item before finishing the session.".into(),
            );
        }
        transaction.execute(
            "UPDATE learning_practice_session SET status='completed',completed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND status='in_progress'",
            [session_id],
        ).map_err(db)?;
        transaction.execute(
            "INSERT OR IGNORE INTO learning_practice_xp_event(id,session_id,rule_version,xp_amount,activity_day,created_at)
             VALUES(?1,?2,?3,?4,date('now','localtime'),strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            params![uuid::Uuid::new_v4().to_string(), session_id, PRACTICE_XP_RULE_VERSION, PRACTICE_SESSION_XP],
        ).map_err(db)?;
        transaction.commit().map_err(db)?;
        self.get(session_id)
    }

    pub fn abandon(&self, session_id: &str) -> Result<(), String> {
        database::open(&self.database)?.execute(
            "UPDATE learning_practice_session SET status='abandoned',abandoned_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND status='in_progress'",
            [session_id],
        ).map_err(db)?;
        Ok(())
    }
}

fn candidates(connection: &rusqlite::Connection) -> Result<Vec<PracticeItem>, String> {
    let mut items = vocabulary_candidates(connection)?;
    items.extend(mistake_candidates(connection)?);
    let snapshots = connection
        .prepare(
            "SELECT lesson_id,package_snapshot_json FROM interactive_lesson_session
         WHERE status='completed' ORDER BY completed_at DESC,id DESC LIMIT 12",
        )
        .map_err(db)?
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(db)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db)?;
    for (lesson_id, raw) in snapshots {
        let Ok(package) = serde_json::from_str::<InteractiveLessonPackage>(&raw) else {
            continue;
        };
        for stage in package.stages {
            match stage.payload {
                StagePayload::Listening { segments, .. } => {
                    for value in segments {
                        items.push(PracticeItem::Dictation {
                            item_id: format!("dictation:{lesson_id}:{}", value.segment_id),
                            target_text: value.text,
                            source_label: package.title.clone(),
                        });
                    }
                }
                StagePayload::Repeat { targets } => {
                    for value in targets {
                        items.push(PracticeItem::Shadowing {
                            item_id: format!("shadowing:{lesson_id}:{}", value.target_id),
                            target_text: value.text,
                            hint: value.hint,
                            source_label: package.title.clone(),
                        });
                    }
                }
                StagePayload::SpeakingCheck { targets } => {
                    for value in targets {
                        items.push(PracticeItem::SpeakingRecall {
                            item_id: format!("recall:{lesson_id}:{}", value.target_id),
                            situation: value.instruction,
                            prompt: "Say the idea naturally in English.".into(),
                            model_expression: value.target_text,
                            source_label: package.title.clone(),
                        });
                    }
                }
                StagePayload::GuidedConversation {
                    scenario,
                    goal,
                    target_expressions,
                    ..
                } => {
                    for (index, expression) in target_expressions.into_iter().enumerate() {
                        items.push(PracticeItem::SpeakingRecall {
                            item_id: format!("recall:{lesson_id}:guided:{index}"),
                            situation: scenario.clone(),
                            prompt: goal.clone(),
                            model_expression: expression,
                            source_label: package.title.clone(),
                        });
                    }
                }
                _ => {}
            }
        }
    }
    Ok(items)
}

fn vocabulary_candidates(connection: &rusqlite::Connection) -> Result<Vec<PracticeItem>, String> {
    connection.prepare(
        "SELECT v.id,v.display_text,v.meaning,
         COALESCE((SELECT example FROM lesson_vocabulary lv WHERE lv.vocabulary_item_id=v.id ORDER BY lv.created_at DESC LIMIT 1),
                  (SELECT example FROM guided_session_vocabulary gv WHERE gv.vocabulary_item_id=v.id ORDER BY gv.created_at DESC LIMIT 1),''),
         v.status
         FROM vocabulary_item v WHERE v.status IN('learning','new')
         ORDER BY CASE v.status WHEN 'learning' THEN 0 ELSE 1 END,v.last_seen_at DESC,v.id"
    ).map_err(db)?.query_map([], |row| Ok(PracticeItem::Vocabulary {
        item_id: format!("vocabulary:{}", row.get::<_, String>(0)?), term: row.get(1)?, meaning: row.get(2)?, example: row.get(3)?, status: row.get(4)?, source_label: "Your vocabulary".into()
    })).map_err(db)?.collect::<Result<Vec<_>,_>>().map_err(db)
}

fn mistake_candidates(connection: &rusqlite::Connection) -> Result<Vec<PracticeItem>, String> {
    connection.prepare(
        "SELECT m.id,m.explanation,o.original,o.corrected,o.source_label FROM recurring_mistake m JOIN (
           SELECT recurring_mistake_id,original,corrected,'Conversation lesson' source_label,created_at FROM recurring_mistake_occurrence
           UNION ALL
           SELECT recurring_mistake_id,original,corrected,'Guided lesson' source_label,created_at FROM guided_recurring_mistake_occurrence
         ) o ON o.recurring_mistake_id=m.id
         WHERE m.lesson_count>=2 AND m.status IN('active','improving')
           AND o.created_at=(SELECT MAX(all_o.created_at) FROM (
             SELECT recurring_mistake_id,created_at FROM recurring_mistake_occurrence
             UNION ALL SELECT recurring_mistake_id,created_at FROM guided_recurring_mistake_occurrence
           ) all_o WHERE all_o.recurring_mistake_id=m.id)
         ORDER BY m.last_seen_at DESC,m.id"
    ).map_err(db)?.query_map([], |row| {
        let id: String = row.get(0)?;
        Ok(PracticeItem::MistakeRepair { item_id: format!("mistake:{id}"), mistake_id: id, explanation: row.get(1)?, original: row.get(2)?, corrected: row.get(3)?, source_label: row.get(4)? })
    }).map_err(db)?.collect::<Result<Vec<_>,_>>().map_err(db)
}

fn recent_item_ids(connection: &rusqlite::Connection) -> Result<BTreeSet<String>, String> {
    connection
        .prepare(
            "SELECT DISTINCT r.item_id FROM learning_practice_item_result r
         WHERE r.completed_at >= datetime('now','-3 days')",
        )
        .map_err(db)?
        .query_map([], |row| row.get(0))
        .map_err(db)?
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(db)
}

fn interleave_kinds(items: Vec<PracticeItem>) -> Vec<PracticeItem> {
    let order = [
        "mistake_repair",
        "vocabulary",
        "dictation",
        "shadowing",
        "speaking_recall",
    ];
    let mut remaining = items;
    let mut output = Vec::new();
    loop {
        let before = remaining.len();
        for kind in order {
            if let Some(index) = remaining.iter().position(|item| item.kind_name() == kind) {
                output.push(remaining.remove(index));
            }
        }
        if remaining.len() == before {
            break;
        }
    }
    output.extend(remaining);
    output
}

fn session_with(connection: &rusqlite::Connection, id: &str) -> Result<PracticeSessionDto, String> {
    let raw: Option<(String,String,u32,u32,String,String,Option<String>)> = connection.query_row(
        "SELECT mode,status,schema_version,selection_version,items_json,started_at,completed_at FROM learning_practice_session WHERE id=?1",
        [id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?))
    ).optional().map_err(db)?;
    let (mode, status, schema_version, selection_version, items_json, started_at, completed_at) =
        raw.ok_or("Practice session not found.")?;
    let completed_item_ids = connection.prepare(
        "SELECT item_id FROM learning_practice_item_result WHERE session_id=?1 ORDER BY completed_at,id"
    ).map_err(db)?.query_map([id], |row| row.get(0)).map_err(db)?
      .collect::<Result<Vec<_>,_>>().map_err(db)?;
    let xp_awarded: u32 = connection.query_row(
        "SELECT COALESCE(SUM(xp_amount),0) FROM learning_practice_xp_event WHERE session_id=?1 AND rule_version=1",
        [id], |row| row.get(0)).map_err(db)?;
    Ok(PracticeSessionDto {
        id: id.into(),
        mode: serde_json::from_str(&format!("\"{mode}\"")).map_err(json_error)?,
        status,
        schema_version,
        selection_version,
        items: serde_json::from_str(&items_json).map_err(json_error)?,
        completed_item_ids,
        active_seconds: active_seconds(connection, id)?,
        xp_awarded,
        started_at,
        completed_at,
    })
}

fn active_seconds(connection: &rusqlite::Connection, id: &str) -> Result<u32, String> {
    connection.query_row(
        "SELECT COALESCE(SUM(duration_seconds),0) FROM learning_practice_active_time_event WHERE session_id=?1",
        [id], |row| row.get(0)).map_err(db)
}

fn words(value: &str) -> Vec<String> {
    normalize(value)
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}
fn normalize(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .map(|c| {
            if matches!(c, '\u{2018}' | '\u{2019}' | '\u{02bc}') {
                '\''
            } else if ('\u{ff01}'..='\u{ff5e}').contains(&c) {
                char::from_u32(c as u32 - 0xfee0).unwrap_or(c)
            } else if c.is_alphanumeric() || c.is_whitespace() || c == '\'' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
fn empty_message(mode: PracticeMode) -> &'static str {
    match mode {
        PracticeMode::MistakeRepair => "No confirmed recurring mistakes yet. A mistake appears here only after it is confirmed across at least two lessons.",
        PracticeMode::Dictation | PracticeMode::Shadowing | PracticeMode::SpeakingRecall => "Complete a Guided Lesson first so this practice can use content you have really studied.",
        PracticeMode::Daily => "Complete a lesson or add vocabulary before starting Daily Practice.",
    }
}
fn db(error: rusqlite::Error) -> String {
    format!("Practice database error: {error}")
}
fn json_error(error: serde_json::Error) -> String {
    format!("Practice snapshot error: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    #[test]
    fn empty_state_is_honest_and_completion_is_idempotent() {
        let directory =
            std::env::temp_dir().join(format!("practice-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("test.sqlite3");
        database::migrate(&path).unwrap();
        let repository = PracticeRepository::new(path.clone());
        assert_eq!(repository.availability().unwrap().daily_count, 0);
        let connection = database::open(&path).unwrap();
        connection.execute("INSERT INTO vocabulary_item(id,canonical_text,display_text,meaning,first_seen_at,last_seen_at,lesson_count,occurrence_count,status,created_at,updated_at) VALUES('v1','hello','Hello','a greeting','2026-01-01','2026-01-01',1,1,'new','2026-01-01','2026-01-01')",[]).unwrap();
        drop(connection);
        let session = repository
            .start(StartPracticeRequest {
                mode: PracticeMode::Daily,
                item_count: 5,
            })
            .unwrap();
        let item_id = session.items[0].id().to_owned();
        let request = CompletePracticeItemRequest {
            session_id: session.id.clone(),
            item_id,
            response: None,
            self_assessment: Some("review".into()),
        };
        repository.complete_item(request.clone()).unwrap();
        repository.complete_item(request).unwrap();
        let completed = repository.complete(&session.id).unwrap();
        assert_eq!(completed.xp_awarded, 20);
        assert_eq!(repository.complete(&session.id).unwrap().xp_awarded, 20);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn dictation_normalization_is_case_punctuation_apostrophe_and_width_safe() {
        assert_eq!(normalize("  I'VE, been here!  "), "i've been here");
        assert_eq!(normalize("I\u{2019}ve been here."), "i've been here");
        assert_eq!(normalize("Ｉ'ｖｅ been here."), "i've been here");
        assert_eq!(words("one one two"), vec!["one", "one", "two"]);
    }

    #[test]
    fn daily_priority_starts_with_confirmed_mistakes_then_learning_vocabulary() {
        let mistake = PracticeItem::MistakeRepair {
            item_id: "m".into(),
            mistake_id: "m".into(),
            original: "in cooking".into(),
            corrected: "at cooking".into(),
            explanation: String::new(),
            source_label: String::new(),
        };
        let learning = PracticeItem::Vocabulary {
            item_id: "learning".into(),
            term: "term".into(),
            meaning: "meaning".into(),
            example: String::new(),
            status: "learning".into(),
            source_label: String::new(),
        };
        let new_word = PracticeItem::Vocabulary {
            item_id: "new".into(),
            term: "term".into(),
            meaning: "meaning".into(),
            example: String::new(),
            status: "new".into(),
            source_label: String::new(),
        };
        assert!(mistake.daily_priority() < learning.daily_priority());
        assert!(learning.daily_priority() < new_word.daily_priority());
    }
}
