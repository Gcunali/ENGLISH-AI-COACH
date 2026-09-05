use crate::database;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Choice {
    choice: String,
    text: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Question {
    item_id: String,
    item_version: u32,
    publication_state: String,
    question_type: String,
    blank_id: String,
    choices: Vec<Choice>,
    correct_answer: String,
    correct_explanation: String,
    distractor_explanations: BTreeMap<String, String>,
    completed_context: String,
    skill_category: String,
    difficulty: String,
    useful_pattern: Option<String>,
    extra_example: Option<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TextSet {
    text_set_id: String,
    version: u32,
    publication_state: String,
    document_type: String,
    title: String,
    passage: String,
    difficulty: String,
    domain: String,
    questions: Vec<Question>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Form {
    form_id: String,
    form_version: u32,
    title: String,
    publication_state: String,
    text_set_ids: Vec<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Raw {
    schema_version: u32,
    bank_id: String,
    forms: Vec<Form>,
    text_sets: Vec<TextSet>,
}
#[derive(Clone)]
pub struct Part7Bank {
    bank_id: String,
    forms: BTreeMap<String, Form>,
    sets: BTreeMap<String, TextSet>,
}

impl Part7Bank {
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let raw: Raw = serde_json::from_slice(
            &fs::read(path).map_err(|e| format!("Part 7 bank missing: {e}"))?,
        )
        .map_err(|e| format!("Part 7 bank invalid: {e}"))?;
        validate(&raw)?;
        Ok(Self {
            bank_id: raw.bank_id,
            forms: raw
                .forms
                .into_iter()
                .map(|x| (x.form_id.clone(), x))
                .collect(),
            sets: raw
                .text_sets
                .into_iter()
                .map(|x| (x.text_set_id.clone(), x))
                .collect(),
        })
    }
}
fn validate(raw: &Raw) -> Result<(), String> {
    if raw.schema_version != 1 {
        return Err("Unsupported Part 7 schema.".into());
    }
    let labels = BTreeSet::from(["A", "B", "C", "D"]);
    let diffs = BTreeSet::from(["easy", "medium", "hard"]);
    let mut set_ids = BTreeSet::new();
    let mut item_ids = BTreeSet::new();
    for set in &raw.text_sets {
        if !set_ids.insert(set.text_set_id.clone())
            || !set.text_set_id.starts_with("toeic-r-p7-set-")
            || set.version != 1
            || set.publication_state != "published"
            || !diffs.contains(set.difficulty.as_str())
            || set.passage.len() < 120
            || !(2..=5).contains(&set.questions.len())
        {
            return Err(format!("Invalid Part 7 text set {}.", set.text_set_id));
        }
        for q in &set.questions {
            let choice_labels = q
                .choices
                .iter()
                .map(|x| x.choice.as_str())
                .collect::<BTreeSet<_>>();
            let choice_text = q
                .choices
                .iter()
                .map(|x| x.text.to_lowercase())
                .collect::<BTreeSet<_>>();
            let wrong = q
                .distractor_explanations
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            let expected_wrong = labels
                .iter()
                .filter(|x| **x != q.correct_answer)
                .copied()
                .collect::<BTreeSet<_>>();
            if !item_ids.insert(q.item_id.clone())
                || !q.item_id.starts_with("toeic-r-p7-")
                || q.item_version != 1
                || q.publication_state != "published"
                || q.question_type.trim().is_empty()
                || q.skill_category.trim().is_empty()
                || !diffs.contains(q.difficulty.as_str())
                || q.choices.len() != 4
                || choice_labels != labels
                || choice_text.len() != 4
                || !labels.contains(q.correct_answer.as_str())
                || wrong != expected_wrong
                || q.correct_explanation.len() < 35
                || q.completed_context.len() < 20
                || q.distractor_explanations.values().any(|x| x.len() < 25)
            {
                return Err(format!("Invalid Part 7 question {}.", q.item_id));
            }
        }
    }
    for form in &raw.forms {
        if form.form_version != 1
            || form.publication_state != "published"
            || form.text_set_ids.len() != 15
            || form.text_set_ids.iter().collect::<BTreeSet<_>>().len() != 15
            || form.text_set_ids.iter().any(|x| !set_ids.contains(x))
        {
            return Err(format!(
                "{} must contain fifteen unique sets.",
                form.form_id
            ));
        }
        let qs = form
            .text_set_ids
            .iter()
            .flat_map(|id| {
                raw.text_sets
                    .iter()
                    .find(|x| &x.text_set_id == id)
                    .unwrap()
                    .questions
                    .iter()
            })
            .collect::<Vec<_>>();
        let mut answers = BTreeMap::new();
        for q in qs {
            *answers.entry(q.correct_answer.as_str()).or_insert(0usize) += 1;
        }
        let sets = form
            .text_set_ids
            .iter()
            .map(|id| raw.text_sets.iter().find(|x| &x.text_set_id == id).unwrap())
            .collect::<Vec<_>>();
        let single = sets
            .iter()
            .filter(|x| x.text_set_id.contains("set-s"))
            .collect::<Vec<_>>();
        let multiple = sets
            .iter()
            .filter(|x| x.text_set_id.contains("set-m"))
            .collect::<Vec<_>>();
        if single.len() != 10
            || single.iter().map(|x| x.questions.len()).sum::<usize>() != 29
            || multiple.len() != 5
            || multiple.iter().any(|x| x.questions.len() != 5)
            || answers.values().any(|n| *n < 12 || *n > 15)
        {
            return Err(format!(
                "Suspicious Part 7 distribution in {}.",
                form.form_id
            ));
        }
    }
    Ok(())
}

#[derive(Clone)]
pub struct Part7Repository {
    database: PathBuf,
    bank: Part7Bank,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub bank_id: String,
    pub forms: Vec<FormDto>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormDto {
    pub form_id: String,
    pub form_version: u32,
    pub title: String,
    pub active_session_id: Option<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicChoice {
    pub choice: String,
    pub text: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicQuestion {
    pub item_id: String,
    pub item_version: u32,
    pub question_number: u32,
    pub blank_id: String,
    pub question_type: String,
    pub choices: Vec<PublicChoice>,
    pub selected_choice: Option<String>,
    pub locked: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicSet {
    pub text_set_id: String,
    pub set_number: u32,
    pub total_sets: u32,
    pub title: String,
    pub document_type: String,
    pub passage: String,
    pub questions: Vec<PublicQuestion>,
    pub active_question_index: u32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChoiceExplanation {
    pub choice: String,
    pub text: String,
    pub explanation: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionFeedback {
    pub question_number: u32,
    pub blank_id: String,
    pub selected_choice: String,
    pub is_correct: bool,
    pub correct_choice: String,
    pub completed_context: String,
    pub correct_explanation: String,
    pub selected_distractor_explanation: Option<String>,
    pub other_distractor_explanations: Vec<ChoiceExplanation>,
    pub skill_category: String,
    pub useful_pattern: Option<String>,
    pub extra_example: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetFeedback {
    pub text_set_id: String,
    pub completed_text: String,
    pub questions: Vec<QuestionFeedback>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub session_id: String,
    pub form_id: String,
    pub form_title: String,
    pub mode: String,
    pub status: String,
    pub current_set_index: u32,
    pub answered_count: u32,
    pub current_set: Option<PublicSet>,
    pub set_feedback: Option<SetFeedback>,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Submit {
    pub session_id: String,
    pub item_id: String,
    pub item_version: u32,
    pub selected_choice: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Breakdown {
    pub label: String,
    pub correct: u32,
    pub total: u32,
    pub accuracy: u32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultDto {
    pub session_id: String,
    pub form_id: String,
    pub correct: u32,
    pub total: u32,
    pub accuracy: u32,
    pub skill_breakdown: Vec<Breakdown>,
    pub difficulty_breakdown: Vec<Breakdown>,
    pub document_breakdown: Vec<Breakdown>,
    pub needs_attention: Vec<String>,
    pub has_scaled_score: bool,
    pub score_message: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSet {
    pub text_set_id: String,
    pub title: String,
    pub document_type: String,
    pub original_passage: String,
    pub feedback: SetFeedback,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    form_id: String,
    form_version: u32,
    title: String,
    mode: String,
    text_sets: Vec<TextSet>,
}

impl Part7Repository {
    pub fn new(database: PathBuf, bank: Part7Bank) -> Self {
        Self { database, bank }
    }
    pub fn overview(&self) -> Result<Overview, String> {
        let c = database::open(&self.database)?;
        let forms=self.bank.forms.values().map(|f|{let active=c.query_row("SELECT id FROM toeic_session WHERE form_id=?1 AND form_version=?2 AND part='part7_text_completion' AND status='in_progress' ORDER BY created_at DESC LIMIT 1",params![f.form_id,f.form_version],|r|r.get(0)).optional().map_err(err)?;Ok(FormDto{form_id:f.form_id.clone(),form_version:f.form_version,title:f.title.clone(),active_session_id:active})}).collect::<Result<Vec<_>,String>>()?;
        Ok(Overview {
            bank_id: self.bank.bank_id.clone(),
            forms,
        })
    }
    pub fn start(&self, form_id: &str, version: u32, mode: &str) -> Result<Session, String> {
        if !matches!(mode, "learning" | "simulation") {
            return Err("Part 7 mode must be learning or simulation.".into());
        }
        let c = database::open(&self.database)?;
        if let Some(id)=c.query_row("SELECT id FROM toeic_session WHERE form_id=?1 AND form_version=?2 AND status='in_progress'",params![form_id,version],|r|r.get::<_,String>(0)).optional().map_err(err)?{return self.session(&id)}
        let f = self
            .bank
            .forms
            .get(form_id)
            .filter(|x| x.form_version == version)
            .ok_or("Part 7 form not found.")?;
        let snap = Snapshot {
            form_id: f.form_id.clone(),
            form_version: f.form_version,
            title: f.title.clone(),
            mode: mode.into(),
            text_sets: f
                .text_set_ids
                .iter()
                .map(|id| self.bank.sets[id].clone())
                .collect(),
        };
        let id = uuid::Uuid::new_v4().to_string();
        c.execute("INSERT INTO toeic_session(id,form_id,form_version,section,part,status,schema_version,form_snapshot_json,current_question_index,created_at,updated_at) VALUES(?1,?2,?3,'reading','part7_reading_comprehension','in_progress',1,?4,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![id,form_id,version,serde_json::to_string(&snap).unwrap()]).map_err(err)?;
        self.session(&id)
    }
    pub fn session(&self, id: &str) -> Result<Session, String> {
        let c = database::open(&self.database)?;
        let(status,json,index,created,updated)=c.query_row("SELECT status,form_snapshot_json,current_question_index,created_at,updated_at FROM toeic_session WHERE id=?1 AND part='part7_reading_comprehension'",[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,u32>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?))).optional().map_err(err)?.ok_or("Part 7 session not found.")?;
        let snap: Snapshot = serde_json::from_str(&json).map_err(|_| "Invalid Part 7 snapshot.")?;
        let answers = answers(&c, id)?;
        let set = snap.text_sets.get(index as usize);
        let set_done = set
            .map(|x| x.questions.iter().all(|q| answers.contains_key(&q.item_id)))
            .unwrap_or(false);
        let feedback = if snap.mode == "learning" && set_done {
            set.map(|x| set_feedback(x, &answers, index))
        } else {
            None
        };
        let current_set = set.map(|x| public_set(x, index, &answers));
        Ok(Session {
            session_id: id.into(),
            form_id: snap.form_id,
            form_title: snap.title,
            mode: snap.mode,
            status,
            current_set_index: index,
            answered_count: answers.len() as u32,
            current_set,
            set_feedback: feedback,
            created_at: created,
            updated_at: updated,
        })
    }
    pub fn submit(&self, r: Submit) -> Result<Session, String> {
        if !matches!(r.selected_choice.as_str(), "A" | "B" | "C" | "D") {
            return Err("Answer must be A, B, C, or D.".into());
        }
        let s = self.session(&r.session_id)?;
        if s.status != "in_progress" {
            return Err("Part 7 session is complete.".into());
        }
        if s.set_feedback.is_some() {
            return Err("Continue to the next text before answering.".into());
        }
        let set = s.current_set.ok_or("No current Part 7 text.")?;
        let q = set
            .questions
            .iter()
            .find(|q| !q.locked)
            .ok_or("Text set is already complete.")?;
        if q.item_id != r.item_id || q.item_version != r.item_version {
            return Err("Stale Part 7 question.".into());
        }
        let c = database::open(&self.database)?;
        let json: String = c
            .query_row(
                "SELECT form_snapshot_json FROM toeic_session WHERE id=?1",
                [&r.session_id],
                |x| x.get(0),
            )
            .map_err(err)?;
        let snap: Snapshot = serde_json::from_str(&json).map_err(|_| "Invalid Part 7 snapshot.")?;
        let item = &snap.text_sets[s.current_set_index as usize].questions
            [set.active_question_index as usize];
        let n=c.execute("INSERT OR IGNORE INTO toeic_answer(id,session_id,item_id,item_version,selected_choice,is_correct,first_attempt,answered_at) VALUES(?1,?2,?3,?4,?5,?6,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![uuid::Uuid::new_v4().to_string(),r.session_id,r.item_id,r.item_version,r.selected_choice,u8::from(r.selected_choice==item.correct_answer)]).map_err(err)?;
        if n != 1 {
            return Err("The first Part 7 answer is final.".into());
        }
        c.execute(
            "UPDATE toeic_session SET updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
            [&r.session_id],
        )
        .map_err(err)?;
        let after = self.session(&r.session_id)?;
        if after.mode == "simulation"
            && after
                .current_set
                .as_ref()
                .map(|x| x.questions.iter().all(|q| q.locked))
                .unwrap_or(false)
        {
            self.advance(&r.session_id)
        } else {
            Ok(after)
        }
    }
    pub fn advance(&self, id: &str) -> Result<Session, String> {
        let s = self.session(id)?;
        let set = s.current_set.as_ref().ok_or("No current Part 7 text.")?;
        if !set.questions.iter().all(|q| q.locked) {
            return Err("Answer all four questions before continuing.".into());
        }
        let c = database::open(&self.database)?;
        if s.current_set_index == 14 {
            c.execute("UPDATE toeic_session SET status='completed',current_question_index=15,completed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[id]).map_err(err)?;
        } else {
            c.execute("UPDATE toeic_session SET current_question_index=current_question_index+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[id]).map_err(err)?;
        }
        self.session(id)
    }
    pub fn result(&self, id: &str) -> Result<ResultDto, String> {
        let (snap, ans) = self.completed(id)?;
        let mut skills = BTreeMap::new();
        let mut diffs = BTreeMap::new();
        let mut docs = BTreeMap::new();
        for set in &snap.text_sets {
            for q in &set.questions {
                let ok = ans.get(&q.item_id).map(|x| x.1).unwrap_or(false);
                add(&mut skills, &q.skill_category, ok);
                add(&mut diffs, &q.difficulty, ok);
                add(&mut docs, &set.document_type, ok)
            }
        }
        let correct = ans.values().filter(|x| x.1).count() as u32;
        let skill_breakdown = breakdown(skills);
        let needs_attention = skill_breakdown
            .iter()
            .filter(|x| x.accuracy < 70)
            .map(|x| x.label.clone())
            .collect();
        Ok(ResultDto{session_id:id.into(),form_id:snap.form_id,correct,total:54,accuracy:correct*100/54,skill_breakdown,difficulty_breakdown:breakdown(diffs),document_breakdown:breakdown(docs),needs_attention,has_scaled_score:false,score_message:"Part 7 raw performance only. A Reading estimate is produced only by a completed 100-question Full Reading simulation.".into()})
    }
    pub fn review(&self, id: &str, mistakes: bool) -> Result<Vec<ReviewSet>, String> {
        let (snap, ans) = self.completed(id)?;
        Ok(snap
            .text_sets
            .iter()
            .enumerate()
            .filter(|(_, s)| !mistakes || s.questions.iter().any(|q| !ans[&q.item_id].1))
            .map(|(i, s)| ReviewSet {
                text_set_id: s.text_set_id.clone(),
                title: s.title.clone(),
                document_type: s.document_type.clone(),
                original_passage: s.passage.clone(),
                feedback: set_feedback(s, &ans, i as u32),
            })
            .collect())
    }
    fn completed(&self, id: &str) -> Result<(Snapshot, BTreeMap<String, (String, bool)>), String> {
        let c = database::open(&self.database)?;
        let(status,json)=c.query_row("SELECT status,form_snapshot_json FROM toeic_session WHERE id=?1 AND part='part7_reading_comprehension'",[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).map_err(err)?;
        if status != "completed" {
            return Err("Complete Part 7 before viewing results.".into());
        }
        Ok((
            serde_json::from_str(&json).map_err(|_| "Invalid Part 7 snapshot.")?,
            answers(&c, id)?,
        ))
    }
}
fn public_set(s: &TextSet, index: u32, ans: &BTreeMap<String, (String, bool)>) -> PublicSet {
    let active = s
        .questions
        .iter()
        .position(|q| !ans.contains_key(&q.item_id))
        .unwrap_or(s.questions.len()) as u32;
    PublicSet {
        text_set_id: s.text_set_id.clone(),
        set_number: index + 1,
        total_sets: 15,
        title: s.title.clone(),
        document_type: s.document_type.clone(),
        passage: s.passage.clone(),
        active_question_index: active,
        questions: s
            .questions
            .iter()
            .enumerate()
            .map(|(i, q)| PublicQuestion {
                item_id: q.item_id.clone(),
                item_version: q.item_version,
                question_number: question_offset(index) + i as u32 + 1,
                blank_id: q.blank_id.clone(),
                question_type: q.question_type.clone(),
                choices: q
                    .choices
                    .iter()
                    .map(|c| PublicChoice {
                        choice: c.choice.clone(),
                        text: c.text.clone(),
                    })
                    .collect(),
                selected_choice: ans.get(&q.item_id).map(|x| x.0.clone()),
                locked: ans.contains_key(&q.item_id),
            })
            .collect(),
    }
}
fn question_offset(set_index: u32) -> u32 {
    if set_index <= 9 {
        set_index * 3
    } else {
        29 + (set_index - 10) * 5
    }
}
fn set_feedback(s: &TextSet, ans: &BTreeMap<String, (String, bool)>, index: u32) -> SetFeedback {
    let completed = s.passage.clone();
    SetFeedback {
        text_set_id: s.text_set_id.clone(),
        completed_text: completed,
        questions: s
            .questions
            .iter()
            .enumerate()
            .map(|(i, q)| {
                let a = &ans[&q.item_id];
                QuestionFeedback {
                    question_number: question_offset(index) + i as u32 + 1,
                    blank_id: q.blank_id.clone(),
                    selected_choice: a.0.clone(),
                    is_correct: a.1,
                    correct_choice: q.correct_answer.clone(),
                    completed_context: q.completed_context.clone(),
                    correct_explanation: q.correct_explanation.clone(),
                    selected_distractor_explanation: if a.1 {
                        None
                    } else {
                        q.distractor_explanations.get(&a.0).cloned()
                    },
                    other_distractor_explanations: q
                        .choices
                        .iter()
                        .filter(|c| c.choice != q.correct_answer && c.choice != a.0)
                        .map(|c| ChoiceExplanation {
                            choice: c.choice.clone(),
                            text: c.text.clone(),
                            explanation: q.distractor_explanations[&c.choice].clone(),
                        })
                        .collect(),
                    skill_category: q.skill_category.clone(),
                    useful_pattern: q.useful_pattern.clone(),
                    extra_example: q.extra_example.clone(),
                }
            })
            .collect(),
    }
}
fn answers(c: &rusqlite::Connection, id: &str) -> Result<BTreeMap<String, (String, bool)>, String> {
    let mut st = c
        .prepare("SELECT item_id,selected_choice,is_correct FROM toeic_answer WHERE session_id=?1")
        .map_err(err)?;
    let out = st
        .query_map([id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, bool>(2)?,
            ))
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?
        .into_iter()
        .map(|(i, c, o)| (i, (c, o)))
        .collect();
    Ok(out)
}
fn add(m: &mut BTreeMap<String, (u32, u32)>, k: &str, ok: bool) {
    let e = m.entry(k.into()).or_default();
    e.1 += 1;
    if ok {
        e.0 += 1
    }
}
fn breakdown(m: BTreeMap<String, (u32, u32)>) -> Vec<Breakdown> {
    m.into_iter()
        .map(|(label, (correct, total))| Breakdown {
            label,
            correct,
            total,
            accuracy: if total == 0 { 0 } else { correct * 100 / total },
        })
        .collect()
}
fn err(e: rusqlite::Error) -> String {
    format!("Part 7 database error: {e}")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Part7Repository, PathBuf) {
        let db = std::env::temp_dir().join(format!("toeic-p6-{}.db", uuid::Uuid::new_v4()));
        database::migrate(&db).unwrap();
        let bank = Part7Bank::load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources/toeic/item-bank-v1/part7.json"),
        )
        .unwrap();
        (Part7Repository::new(db.clone(), bank), db)
    }
    #[test]
    fn production_form_is_complete_and_balanced() {
        let (r, db) = setup();
        let o = r.overview().unwrap();
        assert_eq!(o.forms.len(), 1);
        let s = r.start(&o.forms[0].form_id, 1, "learning").unwrap();
        assert_eq!(s.current_set.unwrap().questions.len(), 3);
        let _ = fs::remove_file(db);
    }
    #[test]
    fn feedback_does_not_leak_before_fourth_answer() {
        let (r, db) = setup();
        let s = r.start("toeic-part7-form-a", 1, "learning").unwrap();
        for n in 0..2 {
            let q = &s.current_set.as_ref().unwrap().questions[n];
            let next = r
                .submit(Submit {
                    session_id: s.session_id.clone(),
                    item_id: q.item_id.clone(),
                    item_version: 1,
                    selected_choice: "A".into(),
                })
                .unwrap();
            assert!(next.set_feedback.is_none())
        }
        let _ = fs::remove_file(db);
    }
    #[test]
    fn first_attempt_is_immutable_and_set_feedback_unlocks() {
        let (r, db) = setup();
        let mut s = r.start("toeic-part7-form-a", 1, "learning").unwrap();
        for _ in 0..3 {
            let q = &s.current_set.as_ref().unwrap().questions
                [s.current_set.as_ref().unwrap().active_question_index as usize];
            s = r
                .submit(Submit {
                    session_id: s.session_id.clone(),
                    item_id: q.item_id.clone(),
                    item_version: 1,
                    selected_choice: "A".into(),
                })
                .unwrap()
        }
        assert_eq!(s.set_feedback.as_ref().unwrap().questions.len(), 3);
        let q = &s.current_set.as_ref().unwrap().questions[0];
        assert!(r
            .submit(Submit {
                session_id: s.session_id.clone(),
                item_id: q.item_id.clone(),
                item_version: 1,
                selected_choice: "B".into()
            })
            .is_err());
        let _ = fs::remove_file(db);
    }
    #[test]
    fn resume_mid_set_preserves_locks_without_feedback() {
        let (r, db) = setup();
        let mut s = r.start("toeic-part7-form-a", 1, "learning").unwrap();
        for _ in 0..2 {
            let set = s.current_set.as_ref().unwrap();
            let q = &set.questions[set.active_question_index as usize];
            s = r
                .submit(Submit {
                    session_id: s.session_id.clone(),
                    item_id: q.item_id.clone(),
                    item_version: 1,
                    selected_choice: "B".into(),
                })
                .unwrap()
        }
        let resumed = r.session(&s.session_id).unwrap();
        assert_eq!(resumed.answered_count, 2);
        assert_eq!(resumed.current_set.unwrap().active_question_index, 2);
        assert!(resumed.set_feedback.is_none());
        let _ = fs::remove_file(db);
    }
    #[test]
    fn simulation_hides_feedback_and_completes_sixteen() {
        let (r, db) = setup();
        let mut s = r.start("toeic-part7-form-a", 1, "simulation").unwrap();
        while s.status == "in_progress" {
            let set = s.current_set.as_ref().unwrap();
            let q = &set.questions[set.active_question_index as usize];
            s = r
                .submit(Submit {
                    session_id: s.session_id.clone(),
                    item_id: q.item_id.clone(),
                    item_version: 1,
                    selected_choice: "A".into(),
                })
                .unwrap();
            assert!(s.set_feedback.is_none())
        }
        let result = r.result(&s.session_id).unwrap();
        assert_eq!(result.total, 54);
        assert_eq!(r.review(&s.session_id, false).unwrap().len(), 15);
        let _ = fs::remove_file(db);
    }
}
