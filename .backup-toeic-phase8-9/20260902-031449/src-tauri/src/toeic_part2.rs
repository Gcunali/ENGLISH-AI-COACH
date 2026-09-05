use crate::database;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Part2Item {
    pub item_id: String,
    pub item_version: u32,
    pub difficulty: String,
    pub prompt_type: String,
    pub prompt: String,
    pub responses: Vec<String>,
    pub correct_answer: String,
    pub correct_explanation: String,
    pub explanations: BTreeMap<String, String>,
    pub listening_focus: Vec<String>,
    pub useful_pattern: Option<String>,
    pub tags: Vec<String>,
    pub distractor_types: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Form {
    form_id: String,
    form_version: u32,
    items: Vec<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Raw {
    schema_version: u32,
    bank_id: String,
    items: Vec<Part2Item>,
    forms: Vec<Form>,
}
#[derive(Clone)]
pub struct Part2Bank {
    pub bank_id: String,
    items: BTreeMap<String, Part2Item>,
    forms: BTreeMap<String, Form>,
}
impl Part2Bank {
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let raw: Raw = serde_json::from_slice(
            &fs::read(path).map_err(|_| "TOEIC Part 2 bank is missing.".to_owned())?,
        )
        .map_err(|e| format!("TOEIC Part 2 bank is invalid: {e}"))?;
        validate(&raw)?;
        Ok(Self {
            bank_id: raw.bank_id,
            items: raw
                .items
                .into_iter()
                .map(|i| (i.item_id.clone(), i))
                .collect(),
            forms: raw
                .forms
                .into_iter()
                .map(|f| (f.form_id.clone(), f))
                .collect(),
        })
    }
}
fn validate(raw: &Raw) -> Result<(), String> {
    if raw.schema_version != 1 {
        return Err("Unsupported Part 2 schema.".into());
    }
    let choices = BTreeSet::from(["A", "B", "C"]);
    let prompt_types = BTreeSet::from([
        "who",
        "what",
        "when",
        "where",
        "why",
        "how",
        "which",
        "yes_no",
        "choice",
        "request",
        "offer",
        "suggestion",
        "statement",
        "indirect",
        "negative",
        "confirmation",
    ]);
    let difficulties = BTreeSet::from(["easy", "medium", "hard"]);
    let distractors = BTreeSet::from([
        "wrong_wh_category",
        "wrong_time",
        "wrong_location",
        "wrong_person",
        "yes_no_trap",
        "similar_sound",
        "same_word_trap",
        "semantic_association",
        "irrelevant_but_plausible",
        "wrong_tense",
        "wrong_function",
        "literal_response",
        "topic_related_but_wrong",
    ]);
    let mut ids = BTreeSet::new();
    for i in &raw.items {
        if !ids.insert(i.item_id.clone())
            || !i.item_id.starts_with("toeic-l-p2-")
            || i.item_version != 1
        {
            return Err("Invalid or duplicate Part 2 item identity.".into());
        }
        if !difficulties.contains(i.difficulty.as_str())
            || !prompt_types.contains(i.prompt_type.as_str())
            || i.prompt.trim().len() < 5
            || i.prompt.len() > 240
            || i.responses.len() != 3
            || i.responses
                .iter()
                .any(|r| r.trim().len() < 2 || r.len() > 240)
            || i.responses
                .iter()
                .map(|r| r.to_lowercase())
                .collect::<BTreeSet<_>>()
                .len()
                != 3
        {
            return Err(format!("{} has invalid prompt/responses.", i.item_id));
        }
        if !choices.contains(i.correct_answer.as_str())
            || i.correct_explanation.len() < 20
            || i.listening_focus.is_empty()
            || i.tags.is_empty()
        {
            return Err(format!("{} has incomplete authored feedback.", i.item_id));
        }
        let expected = choices
            .iter()
            .filter(|c| **c != i.correct_answer)
            .map(|s| s.to_string())
            .collect::<BTreeSet<_>>();
        if i.explanations.keys().cloned().collect::<BTreeSet<_>>() != expected
            || i.distractor_types.keys().cloned().collect::<BTreeSet<_>>() != expected
            || i.explanations.values().any(|v| v.len() < 15)
            || i.distractor_types
                .values()
                .any(|v| !distractors.contains(v.as_str()))
        {
            return Err(format!("{} has invalid distractor feedback.", i.item_id));
        }
    }
    for f in &raw.forms {
        if f.items.len() != 25
            || f.items.iter().collect::<BTreeSet<_>>().len() != 25
            || f.items.iter().any(|id| !ids.contains(id))
        {
            return Err(format!("{} must reference 25 unique items.", f.form_id));
        }
        let form_items = f
            .items
            .iter()
            .map(|id| raw.items.iter().find(|i| &i.item_id == id).unwrap())
            .collect::<Vec<_>>();
        let diff = |d: &str| form_items.iter().filter(|i| i.difficulty == d).count();
        if !(6..=8).contains(&diff("easy"))
            || !(10..=13).contains(&diff("medium"))
            || !(5..=7).contains(&diff("hard"))
        {
            return Err("Part 2 difficulty distribution is invalid.".into());
        }
        let mut answers = BTreeMap::new();
        for i in &form_items {
            *answers.entry(&i.correct_answer).or_insert(0) += 1
        }
        if answers.values().any(|n| *n < 7 || *n > 10)
            || form_items
                .iter()
                .map(|i| &i.prompt_type)
                .collect::<BTreeSet<_>>()
                .len()
                < 8
        {
            return Err("Part 2 answer or prompt diversity is suspicious.".into());
        }
    }
    Ok(())
}

#[derive(Clone)]
pub struct Part2Repository {
    database: PathBuf,
    bank: Part2Bank,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Part2Overview {
    pub bank_id: String,
    pub forms: Vec<Part2FormDto>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Part2FormDto {
    pub form_id: String,
    pub form_version: u32,
    pub title: String,
    pub active_session_id: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicQuestion {
    pub item_id: String,
    pub item_version: u32,
    pub question_number: u32,
    pub total_questions: u32,
    pub choices: Vec<String>,
    pub initial_audio_completed: bool,
    pub initial_audio_interrupted: bool,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Feedback {
    pub selected_choice: String,
    pub is_correct: bool,
    pub correct_answer: String,
    pub prompt: String,
    pub responses: Vec<Response>,
    pub correct_explanation: String,
    pub selected_explanation: Option<String>,
    pub listening_focus: Vec<String>,
    pub useful_pattern: Option<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub choice: String,
    pub text: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Part2Session {
    pub session_id: String,
    pub form_id: String,
    pub status: String,
    pub current_question_index: u32,
    pub answered_count: u32,
    pub current_question: Option<PublicQuestion>,
    pub feedback: Option<Feedback>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Submit {
    pub session_id: String,
    pub item_id: String,
    pub item_version: u32,
    pub selected_choice: String,
}
pub struct AudioContext {
    pub item_id: String,
    pub item_version: u32,
    pub script: String,
    pub presentation_id: Option<String>,
    pub initial: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Breakdown {
    pub label: String,
    pub correct: u32,
    pub total: u32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Part2Result {
    pub session_id: String,
    pub form_id: String,
    pub correct: u32,
    pub total: u32,
    pub accuracy: u32,
    pub prompt_breakdown: Vec<Breakdown>,
    pub difficulty_breakdown: Vec<Breakdown>,
    pub distractor_breakdown: Vec<Breakdown>,
    pub has_scaled_score: bool,
    pub score_message: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewItem {
    pub question_number: u32,
    pub difficulty: String,
    pub tags: Vec<String>,
    pub feedback: Feedback,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    form_id: String,
    form_version: u32,
    items: Vec<SnapItem>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SnapItem {
    item_id: String,
    item_version: u32,
}
impl Part2Repository {
    pub fn new(database: PathBuf, bank: Part2Bank) -> Self {
        Self { database, bank }
    }
    pub fn overview(&self) -> Result<Part2Overview, String> {
        let c = database::open(&self.database)?;
        Ok(Part2Overview{bank_id:self.bank.bank_id.clone(),forms:self.bank.forms.values().map(|f|Part2FormDto{form_id:f.form_id.clone(),form_version:f.form_version,title:form_title(&f.form_id),active_session_id:c.query_row("SELECT id FROM toeic_session WHERE form_id=?1 AND status='in_progress'",[&f.form_id],|r|r.get(0)).optional().ok().flatten()}).collect()})
    }
    pub fn start(&self, form_id: &str, version: u32) -> Result<Part2Session, String> {
        let f = self
            .bank
            .forms
            .get(form_id)
            .filter(|f| f.form_version == version)
            .ok_or("Part 2 form not found.")?;
        let c = database::open(&self.database)?;
        if let Some(id)=c.query_row("SELECT id FROM toeic_session WHERE form_id=?1 AND form_version=?2 AND status='in_progress'",params![form_id,version],|r|r.get::<_,String>(0)).optional().map_err(err)?{return self.session(&id)}
        let id = uuid::Uuid::new_v4().to_string();
        let snap = Snapshot {
            form_id: f.form_id.clone(),
            form_version: f.form_version,
            items: f
                .items
                .iter()
                .map(|id| SnapItem {
                    item_id: id.clone(),
                    item_version: 1,
                })
                .collect(),
        };
        c.execute("INSERT INTO toeic_session(id,form_id,form_version,section,part,status,schema_version,form_snapshot_json,current_question_index,created_at,updated_at) VALUES(?1,?2,?3,'listening','part2_question_response','in_progress',1,?4,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![id,form_id,version,serde_json::to_string(&snap).unwrap()]).map_err(err)?;
        self.session(&id)
    }
    pub fn session(&self, id: &str) -> Result<Part2Session, String> {
        let c = database::open(&self.database)?;
        let (form,status,json,index)=c.query_row("SELECT form_id,status,form_snapshot_json,current_question_index FROM toeic_session WHERE id=?1 AND part='part2_question_response'",[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,u32>(3)?))).optional().map_err(err)?.ok_or("Part 2 session not found.")?;
        let s: Snapshot = serde_json::from_str(&json).map_err(|_| "Invalid Part 2 snapshot.")?;
        let answered = c
            .query_row(
                "SELECT COUNT(*) FROM toeic_answer WHERE session_id=?1",
                [id],
                |r| r.get(0),
            )
            .map_err(err)?;
        let current = if status == "in_progress" {
            s.items.get(index as usize)
        } else {
            None
        };
        let (q, feedback) = if let Some(x) = current {
            let item = self
                .bank
                .items
                .get(&x.item_id)
                .ok_or("Snapshotted Part 2 item unavailable.")?;
            let answer = answer(&c, id, &x.item_id)?;
            let (done, interrupted) = presentation(&c, id, &x.item_id)?;
            (
                Some(PublicQuestion {
                    item_id: x.item_id.clone(),
                    item_version: 1,
                    question_number: index + 1,
                    total_questions: 25,
                    choices: vec!["A".into(), "B".into(), "C".into()],
                    initial_audio_completed: done,
                    initial_audio_interrupted: interrupted,
                }),
                answer.map(|a| feedback(item, a)),
            )
        } else {
            (None, None)
        };
        Ok(Part2Session {
            session_id: id.into(),
            form_id: form,
            status,
            current_question_index: index,
            answered_count: answered,
            current_question: q,
            feedback,
        })
    }
    pub fn begin_audio(&self, id: &str) -> Result<AudioContext, String> {
        let s = self.session(id)?;
        let q = s.current_question.ok_or("No current Part 2 question.")?;
        let item = self.bank.items.get(&q.item_id).unwrap();
        let initial = s.feedback.is_none();
        if initial && q.initial_audio_completed {
            return Err("Initial Part 2 audio can be played only once.".into());
        }
        let c = database::open(&self.database)?;
        let pid = if initial {
            c.execute("UPDATE toeic_presentation_attempt SET status='interrupted',interrupted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE session_id=?1 AND item_id=?2 AND status='started'",params![id,q.item_id]).map_err(err)?;
            let p = uuid::Uuid::new_v4().to_string();
            c.execute("INSERT INTO toeic_presentation_attempt(id,session_id,item_id,item_version,status,started_at) VALUES(?1,?2,?3,1,'started',strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![p,id,q.item_id]).map_err(err)?;
            Some(p)
        } else {
            None
        };
        let script = format!(
            "{}  A. {}  B. {}  C. {}",
            item.prompt, item.responses[0], item.responses[1], item.responses[2]
        );
        Ok(AudioContext {
            item_id: q.item_id,
            item_version: 1,
            script,
            presentation_id: pid,
            initial,
        })
    }
    pub fn interrupt(&self, pid: Option<&str>) -> Result<(), String> {
        if let Some(p) = pid {
            database::open(&self.database)?.execute("UPDATE toeic_presentation_attempt SET status='interrupted',interrupted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND status='started'",[p]).map_err(err)?;
        }
        Ok(())
    }
    pub fn complete_audio(&self, id: &str, item: &str, pid: Option<&str>) -> Result<(), String> {
        if let Some(p) = pid {
            let n=database::open(&self.database)?.execute("UPDATE toeic_presentation_attempt SET status='completed',completed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND session_id=?2 AND item_id=?3 AND status='started'",params![p,id,item]).map_err(err)?;
            if n != 1 {
                return Err("Stale Part 2 playback.".into());
            }
        }
        Ok(())
    }
    pub fn submit(&self, r: Submit) -> Result<Part2Session, String> {
        if !matches!(r.selected_choice.as_str(), "A" | "B" | "C") {
            return Err("Answer must be A, B, or C.".into());
        }
        let s = self.session(&r.session_id)?;
        let q = s.current_question.ok_or("No current Part 2 question.")?;
        if q.item_id != r.item_id || q.item_version != r.item_version || !q.initial_audio_completed
        {
            return Err("Listen to the complete current question first.".into());
        }
        let item = self.bank.items.get(&r.item_id).unwrap();
        let c = database::open(&self.database)?;
        let n=c.execute("INSERT OR IGNORE INTO toeic_answer(id,session_id,item_id,item_version,selected_choice,is_correct,first_attempt,answered_at) VALUES(?1,?2,?3,1,?4,?5,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![uuid::Uuid::new_v4().to_string(),r.session_id,r.item_id,r.selected_choice,u8::from(r.selected_choice==item.correct_answer)]).map_err(err)?;
        if n != 1 {
            return Err("The first Part 2 answer is final.".into());
        }
        self.session(&r.session_id)
    }
    pub fn advance(&self, id: &str) -> Result<Part2Session, String> {
        let s = self.session(id)?;
        if s.feedback.is_none() {
            return Err("Answer before continuing.".into());
        }
        let c = database::open(&self.database)?;
        if s.current_question_index == 24 {
            c.execute("UPDATE toeic_session SET status='completed',current_question_index=25,completed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[id]).map_err(err)?;
        } else {
            c.execute("UPDATE toeic_session SET current_question_index=current_question_index+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[id]).map_err(err)?;
        }
        self.session(id)
    }
    pub fn result(&self, id: &str) -> Result<Part2Result, String> {
        let c = database::open(&self.database)?;
        let (form,json)=c.query_row("SELECT form_id,form_snapshot_json FROM toeic_session WHERE id=?1 AND part='part2_question_response'",[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).map_err(err)?;
        let s: Snapshot = serde_json::from_str(&json).unwrap();
        let answers = answers(&c, id)?;
        let correct = answers.values().filter(|a| a.1).count() as u32;
        let mut prompts = BTreeMap::new();
        let mut diffs = BTreeMap::new();
        let mut traps = BTreeMap::new();
        for x in s.items {
            let i = &self.bank.items[&x.item_id];
            if let Some(a) = answers.get(&x.item_id) {
                add(&mut prompts, &i.prompt_type, a.1);
                add(&mut diffs, &i.difficulty, a.1);
                if !a.1 {
                    if let Some(t) = i.distractor_types.get(&a.0) {
                        add(&mut traps, t, false)
                    }
                }
            }
        }
        Ok(Part2Result{session_id:id.into(),form_id:form,correct,total:25,accuracy:correct*100/25,prompt_breakdown:breakdown(prompts),difficulty_breakdown:breakdown(diffs),distractor_breakdown:breakdown(traps),has_scaled_score:false,score_message:"Part 2 raw performance only. Parts 1 and 2 cover 31 of 100 Listening questions; no scaled score is produced.".into()})
    }
    pub fn review(&self, id: &str, mistakes: bool) -> Result<Vec<ReviewItem>, String> {
        let c = database::open(&self.database)?;
        let (status, json) = c
            .query_row(
                "SELECT status,form_snapshot_json FROM toeic_session WHERE id=?1",
                [id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .map_err(err)?;
        if status != "completed" {
            return Err("Complete Part 2 before review.".into());
        }
        let s: Snapshot = serde_json::from_str(&json).unwrap();
        let all = answers(&c, id)?;
        Ok(s.items
            .iter()
            .enumerate()
            .filter_map(|(n, x)| {
                let a = all.get(&x.item_id)?;
                if mistakes && a.1 {
                    return None;
                }
                let i = &self.bank.items[&x.item_id];
                Some(ReviewItem {
                    question_number: n as u32 + 1,
                    difficulty: i.difficulty.clone(),
                    tags: i.tags.clone(),
                    feedback: feedback(i, a.clone()),
                })
            })
            .collect())
    }
}
fn answer(
    c: &rusqlite::Connection,
    id: &str,
    item: &str,
) -> Result<Option<(String, bool)>, String> {
    c.query_row(
        "SELECT selected_choice,is_correct FROM toeic_answer WHERE session_id=?1 AND item_id=?2",
        params![id, item],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
    .map_err(err)
}
fn answers(c: &rusqlite::Connection, id: &str) -> Result<BTreeMap<String, (String, bool)>, String> {
    let mut s = c
        .prepare("SELECT item_id,selected_choice,is_correct FROM toeic_answer WHERE session_id=?1")
        .map_err(err)?;
    let result = s
        .query_map([id], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err);
    result
}
fn presentation(c: &rusqlite::Connection, id: &str, item: &str) -> Result<(bool, bool), String> {
    c.query_row("SELECT EXISTS(SELECT 1 FROM toeic_presentation_attempt WHERE session_id=?1 AND item_id=?2 AND status='completed'),EXISTS(SELECT 1 FROM toeic_presentation_attempt WHERE session_id=?1 AND item_id=?2 AND status='interrupted')",params![id,item],|r|Ok((r.get(0)?,r.get(1)?))).map_err(err)
}
fn feedback(i: &Part2Item, a: (String, bool)) -> Feedback {
    Feedback {
        selected_choice: a.0.clone(),
        is_correct: a.1,
        correct_answer: i.correct_answer.clone(),
        prompt: i.prompt.clone(),
        responses: i
            .responses
            .iter()
            .enumerate()
            .map(|(n, t)| Response {
                choice: ["A", "B", "C"][n].into(),
                text: t.clone(),
            })
            .collect(),
        correct_explanation: i.correct_explanation.clone(),
        selected_explanation: i.explanations.get(&a.0).cloned(),
        listening_focus: i.listening_focus.clone(),
        useful_pattern: i.useful_pattern.clone(),
    }
}
fn add(m: &mut BTreeMap<String, (u32, u32)>, k: &str, ok: bool) {
    let x = m.entry(k.into()).or_default();
    x.1 += 1;
    if ok {
        x.0 += 1
    }
}
fn breakdown(m: BTreeMap<String, (u32, u32)>) -> Vec<Breakdown> {
    m.into_iter()
        .map(|(label, (correct, total))| Breakdown {
            label,
            correct,
            total,
        })
        .collect()
}
fn form_title(id: &str) -> String {
    format!(
        "Part 2 Form {}",
        id.chars().last().unwrap_or('A').to_ascii_uppercase()
    )
}
fn err(e: rusqlite::Error) -> String {
    format!("Part 2 database error: {e}")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Part2Repository, PathBuf) {
        let directory =
            std::env::temp_dir().join(format!("toeic-p2-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let database = directory.join("test.sqlite3");
        crate::database::migrate(&database).unwrap();
        let bank = Part2Bank::load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources/toeic/item-bank-v1/part2.json"),
        )
        .unwrap();
        (Part2Repository::new(database, bank), directory)
    }
    fn answer_current(
        repo: &Part2Repository,
        session: &Part2Session,
        correct: bool,
    ) -> Part2Session {
        let q = session.current_question.as_ref().unwrap();
        let context = repo.begin_audio(&session.session_id).unwrap();
        repo.complete_audio(
            &session.session_id,
            &q.item_id,
            context.presentation_id.as_deref(),
        )
        .unwrap();
        let item = &repo.bank.items[&q.item_id];
        let choice = if correct {
            item.correct_answer.clone()
        } else {
            ["A", "B", "C"]
                .into_iter()
                .find(|x| *x != item.correct_answer)
                .unwrap()
                .into()
        };
        repo.submit(Submit {
            session_id: session.session_id.clone(),
            item_id: q.item_id.clone(),
            item_version: q.item_version,
            selected_choice: choice,
        })
        .unwrap()
    }
    #[test]
    fn production_bank_has_valid_complete_forms() {
        let b = Part2Bank::load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources/toeic/item-bank-v1/part2.json"),
        )
        .unwrap();
        assert_eq!(b.items.len(), 75);
        assert_eq!(b.forms.len(), 3);
        assert!(b.forms.values().all(|form| form.items.len() == 25));
    }
    #[test]
    fn forms_b_and_c_are_startable_with_frozen_snapshots() {
        let (repo, dir) = setup();
        for family in ["b", "c"] {
            let form_id = format!("toeic-part2-form-{family}");
            let session = repo.start(&form_id, 1).unwrap();
            assert_eq!(session.form_id, form_id);
            assert_eq!(
                session.current_question.as_ref().unwrap().total_questions,
                25
            );
            let first_number = if family == "b" { "0026" } else { "0051" };
            assert_eq!(
                session.current_question.as_ref().unwrap().item_id,
                format!("toeic-l-p2-{first_number}")
            );
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn public_dto_has_no_transcripts_or_keys() {
        let q = PublicQuestion {
            item_id: "x".into(),
            item_version: 1,
            question_number: 1,
            total_questions: 25,
            choices: vec!["A".into(), "B".into(), "C".into()],
            initial_audio_completed: false,
            initial_audio_interrupted: false,
        };
        let j = serde_json::to_string(&q).unwrap();
        assert!(!j.contains("correct"));
        assert!(!j.contains("prompt"));
        assert!(!j.contains("response"));
    }
    #[test]
    fn first_answer_is_immutable_and_interrupted_audio_can_restart() {
        let (repo, dir) = setup();
        let s = repo.start("toeic-part2-form-a", 1).unwrap();
        let c = repo.begin_audio(&s.session_id).unwrap();
        repo.interrupt(c.presentation_id.as_deref()).unwrap();
        assert!(
            repo.session(&s.session_id)
                .unwrap()
                .current_question
                .unwrap()
                .initial_audio_interrupted
        );
        let answered = answer_current(&repo, &s, false);
        let q = s.current_question.unwrap();
        assert!(repo
            .submit(Submit {
                session_id: s.session_id,
                item_id: q.item_id,
                item_version: 1,
                selected_choice: "A".into()
            })
            .is_err());
        assert!(!answered.feedback.unwrap().is_correct);
        std::fs::remove_dir_all(dir).unwrap()
    }
    #[test]
    fn resumes_after_seven_answers() {
        let (repo, dir) = setup();
        let mut s = repo.start("toeic-part2-form-a", 1).unwrap();
        for _ in 0..7 {
            s = answer_current(&repo, &s, true);
            s = repo.advance(&s.session_id).unwrap()
        }
        let resumed = repo.session(&s.session_id).unwrap();
        assert_eq!(resumed.answered_count, 7);
        assert_eq!(resumed.current_question_index, 7);
        std::fs::remove_dir_all(dir).unwrap()
    }
    #[test]
    fn deterministic_results_support_zero_ten_eighteen_and_twenty_five() {
        for target in [0, 10, 18, 25] {
            let (repo, dir) = setup();
            let mut s = repo.start("toeic-part2-form-a", 1).unwrap();
            for index in 0..25 {
                s = answer_current(&repo, &s, index < target);
                s = repo.advance(&s.session_id).unwrap()
            }
            let r = repo.result(&s.session_id).unwrap();
            assert_eq!(r.correct, target);
            assert_eq!(r.accuracy, target * 100 / 25);
            assert!(!r.has_scaled_score);
            std::fs::remove_dir_all(dir).unwrap()
        }
    }
}
