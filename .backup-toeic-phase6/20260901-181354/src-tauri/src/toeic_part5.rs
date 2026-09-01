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
struct Item {
    item_id: String,
    item_version: u32,
    publication_state: String,
    section: String,
    part: u32,
    item_type: String,
    sentence: String,
    choices: Vec<Choice>,
    correct_answer: String,
    complete_sentence: String,
    correct_explanation: String,
    distractor_explanations: BTreeMap<String, String>,
    skill_category: String,
    skill_subcategory: String,
    difficulty: String,
    domain: String,
    useful_pattern: Option<String>,
    extra_example: Option<String>,
    tags: Vec<String>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Form {
    form_id: String,
    form_version: u32,
    title: String,
    publication_state: String,
    item_ids: Vec<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Raw {
    schema_version: u32,
    bank_id: String,
    forms: Vec<Form>,
    items: Vec<Item>,
}
#[derive(Clone)]
pub struct Part5Bank {
    bank_id: String,
    forms: BTreeMap<String, Form>,
    items: BTreeMap<String, Item>,
}

impl Part5Bank {
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let raw: Raw = serde_json::from_slice(
            &fs::read(path).map_err(|e| format!("Part 5 bank missing: {e}"))?,
        )
        .map_err(|e| format!("Part 5 bank invalid: {e}"))?;
        validate(&raw)?;
        Ok(Self {
            bank_id: raw.bank_id,
            forms: raw
                .forms
                .into_iter()
                .map(|x| (x.form_id.clone(), x))
                .collect(),
            items: raw
                .items
                .into_iter()
                .map(|x| (x.item_id.clone(), x))
                .collect(),
        })
    }
}

fn validate(raw: &Raw) -> Result<(), String> {
    if raw.schema_version != 1 {
        return Err("Unsupported Part 5 schema.".into());
    }
    let labels = BTreeSet::from(["A", "B", "C", "D"]);
    let difficulties = BTreeSet::from(["easy", "medium", "hard"]);
    let categories = BTreeSet::from(["grammar", "vocabulary"]);
    let grammar = BTreeSet::from([
        "verb_tense",
        "verb_form",
        "subject_verb_agreement",
        "gerund_infinitive",
        "participles",
        "passive_voice",
        "modals",
        "conditionals",
        "noun",
        "pronoun",
        "relative_pronoun",
        "determiner",
        "article",
        "adjective",
        "adverb",
        "comparative",
        "superlative",
        "preposition",
        "conjunction",
        "connector",
        "relative_clause",
        "subordinate_clause",
        "word_order",
        "sentence_structure",
        "quantity",
        "agreement",
        "parallel_structure",
        "word_family",
    ]);
    let vocabulary = BTreeSet::from([
        "word_family",
        "word_choice",
        "collocation",
        "phrasal_verb",
        "business_vocabulary",
        "workplace_vocabulary",
        "context_vocabulary",
        "fixed_expression",
        "prepositional_phrase",
    ]);
    let mut ids = BTreeSet::new();
    let mut sentences = BTreeSet::new();
    for item in &raw.items {
        let choice_labels = item
            .choices
            .iter()
            .map(|x| x.choice.as_str())
            .collect::<BTreeSet<_>>();
        let choice_text = item
            .choices
            .iter()
            .map(|x| x.text.to_lowercase())
            .collect::<BTreeSet<_>>();
        let correct = item
            .choices
            .iter()
            .find(|x| x.choice == item.correct_answer)
            .map(|x| x.text.as_str())
            .unwrap_or("");
        let expected = item.sentence.replacen("_____", correct, 1);
        let required_wrong = labels
            .iter()
            .filter(|x| **x != item.correct_answer)
            .map(|x| x.to_string())
            .collect::<BTreeSet<_>>();
        let taxonomy_ok = match item.skill_category.as_str() {
            "grammar" => grammar.contains(item.skill_subcategory.as_str()),
            "vocabulary" => vocabulary.contains(item.skill_subcategory.as_str()),
            _ => false,
        };
        if !ids.insert(item.item_id.clone())
            || !item.item_id.starts_with("toeic-r-p5-")
            || item.item_version != 1
            || item.publication_state != "published"
            || item.section != "reading"
            || item.part != 5
            || item.item_type != "part5_incomplete_sentence"
            || item.sentence.matches("_____").count() != 1
            || item.choices.len() != 4
            || choice_labels != labels
            || choice_text.len() != 4
            || correct.is_empty()
            || item.complete_sentence != expected
            || item.complete_sentence.contains("_____")
            || item.correct_explanation.len() < 35
            || item
                .distractor_explanations
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>()
                != required_wrong
            || item.distractor_explanations.values().any(|x| x.len() < 25)
            || !categories.contains(item.skill_category.as_str())
            || !taxonomy_ok
            || !difficulties.contains(item.difficulty.as_str())
            || item.domain.trim().is_empty()
            || !sentences.insert(item.sentence.to_lowercase())
        {
            return Err(format!("Invalid Part 5 item {}.", item.item_id));
        }
    }
    let all_ids = raw
        .items
        .iter()
        .map(|x| x.item_id.as_str())
        .collect::<BTreeSet<_>>();
    for form in &raw.forms {
        if form.publication_state != "published"
            || form.form_version != 1
            || form.item_ids.len() != 30
            || form.item_ids.iter().collect::<BTreeSet<_>>().len() != 30
            || form.item_ids.iter().any(|x| !all_ids.contains(x.as_str()))
        {
            return Err(format!(
                "{} must contain 30 unique published items.",
                form.form_id
            ));
        }
        let items = form
            .item_ids
            .iter()
            .map(|id| raw.items.iter().find(|x| &x.item_id == id).unwrap())
            .collect::<Vec<_>>();
        let mut answers = BTreeMap::new();
        let mut diff = BTreeMap::new();
        let mut cats = BTreeMap::new();
        let mut subs = BTreeSet::new();
        for x in &items {
            *answers.entry(x.correct_answer.as_str()).or_insert(0usize) += 1;
            *diff.entry(x.difficulty.as_str()).or_insert(0usize) += 1;
            *cats.entry(x.skill_category.as_str()).or_insert(0usize) += 1;
            subs.insert(x.skill_subcategory.as_str());
        }
        if answers.values().any(|n| *n < 6 || *n > 9)
            || diff.get("easy").copied().unwrap_or(0) < 7
            || diff.get("medium").copied().unwrap_or(0) < 13
            || diff.get("hard").copied().unwrap_or(0) < 6
            || !(17..=20).contains(&cats.get("grammar").copied().unwrap_or(0))
            || subs.len() < 12
        {
            return Err(format!(
                "Suspicious Part 5 distribution in {}.",
                form.form_id
            ));
        }
        let sequence = items
            .windows(5)
            .any(|w| w.iter().all(|x| x.correct_answer == w[0].correct_answer));
        if sequence {
            return Err(format!("Predictable answer sequence in {}.", form.form_id));
        }
    }
    Ok(())
}

#[derive(Clone)]
pub struct Part5Repository {
    database: PathBuf,
    bank: Part5Bank,
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
#[derive(Serialize)]
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
    pub total_questions: u32,
    pub sentence: String,
    pub choices: Vec<PublicChoice>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Feedback {
    pub selected_choice: String,
    pub is_correct: bool,
    pub correct_choice: String,
    pub complete_sentence: String,
    pub correct_explanation: String,
    pub selected_distractor_explanation: Option<String>,
    pub other_distractor_explanations: Vec<ChoiceExplanation>,
    pub skill_category: String,
    pub skill_subcategory: String,
    pub useful_pattern: Option<String>,
    pub extra_example: Option<String>,
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
pub struct Session {
    pub session_id: String,
    pub form_id: String,
    pub form_title: String,
    pub mode: String,
    pub status: String,
    pub current_question_index: u32,
    pub answered_count: u32,
    pub current_question: Option<PublicQuestion>,
    pub feedback: Option<Feedback>,
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
    pub grammar: Breakdown,
    pub vocabulary: Breakdown,
    pub subcategory_breakdown: Vec<Breakdown>,
    pub difficulty_breakdown: Vec<Breakdown>,
    pub needs_attention: Vec<String>,
    pub has_scaled_score: bool,
    pub score_message: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewItem {
    pub question_number: u32,
    pub sentence: String,
    pub choices: Vec<PublicChoice>,
    pub feedback: Feedback,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    form_id: String,
    form_version: u32,
    title: String,
    mode: String,
    items: Vec<Item>,
}

impl Part5Repository {
    pub fn new(database: PathBuf, bank: Part5Bank) -> Self {
        Self { database, bank }
    }
    pub fn overview(&self) -> Result<Overview, String> {
        let c = database::open(&self.database)?;
        let forms=self.bank.forms.values().map(|f|{let active=c.query_row("SELECT id FROM toeic_session WHERE form_id=?1 AND form_version=?2 AND part='part5_incomplete_sentence' AND status='in_progress' ORDER BY created_at DESC LIMIT 1",params![f.form_id,f.form_version],|r|r.get(0)).optional().map_err(err)?;Ok(FormDto{form_id:f.form_id.clone(),form_version:f.form_version,title:f.title.clone(),active_session_id:active})}).collect::<Result<Vec<_>,String>>()?;
        Ok(Overview {
            bank_id: self.bank.bank_id.clone(),
            forms,
        })
    }
    pub fn start(&self, form_id: &str, version: u32, mode: &str) -> Result<Session, String> {
        if !matches!(mode, "learning" | "simulation") {
            return Err("Part 5 mode must be learning or simulation.".into());
        }
        let c = database::open(&self.database)?;
        if let Some(id)=c.query_row("SELECT id FROM toeic_session WHERE form_id=?1 AND form_version=?2 AND status='in_progress'",params![form_id,version],|r|r.get::<_,String>(0)).optional().map_err(err)?{return self.session(&id)}
        let form = self
            .bank
            .forms
            .get(form_id)
            .filter(|x| x.form_version == version)
            .ok_or("Part 5 form not found.")?;
        let snapshot = Snapshot {
            form_id: form.form_id.clone(),
            form_version: form.form_version,
            title: form.title.clone(),
            mode: mode.into(),
            items: form
                .item_ids
                .iter()
                .map(|id| self.bank.items[id].clone())
                .collect(),
        };
        let id = uuid::Uuid::new_v4().to_string();
        c.execute("INSERT INTO toeic_session(id,form_id,form_version,section,part,status,schema_version,form_snapshot_json,current_question_index,created_at,updated_at) VALUES(?1,?2,?3,'reading','part5_incomplete_sentence','in_progress',1,?4,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![id,form_id,version,serde_json::to_string(&snapshot).unwrap()]).map_err(err)?;
        self.session(&id)
    }
    pub fn session(&self, id: &str) -> Result<Session, String> {
        let c = database::open(&self.database)?;
        let(status,json,index,created,updated)=c.query_row("SELECT status,form_snapshot_json,current_question_index,created_at,updated_at FROM toeic_session WHERE id=?1 AND part='part5_incomplete_sentence'",[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,u32>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?))).optional().map_err(err)?.ok_or("Part 5 session not found.")?;
        let snap: Snapshot = serde_json::from_str(&json).map_err(|_| "Invalid Part 5 snapshot.")?;
        let answered = c
            .query_row(
                "SELECT COUNT(*) FROM toeic_answer WHERE session_id=?1",
                [id],
                |r| r.get::<_, u32>(0),
            )
            .map_err(err)?;
        let item = snap.items.get(index as usize);
        let answer = if let Some(x) = item {
            answer(&c, id, &x.item_id, x.item_version)?
        } else {
            None
        };
        let feedback = if snap.mode == "learning" {
            item.zip(answer.as_ref())
                .map(|(x, a)| feedback(x, &a.0, a.1))
        } else {
            None
        };
        Ok(Session {
            session_id: id.into(),
            form_id: snap.form_id,
            form_title: snap.title,
            mode: snap.mode,
            status,
            current_question_index: index,
            answered_count: answered,
            current_question: item.map(|x| public(x, index)),
            feedback,
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
            return Err("Part 5 session is complete.".into());
        }
        let q = s.current_question.ok_or("No current Part 5 question.")?;
        if q.item_id != r.item_id || q.item_version != r.item_version {
            return Err("Stale Part 5 question.".into());
        }
        let c = database::open(&self.database)?;
        let json: String = c
            .query_row(
                "SELECT form_snapshot_json FROM toeic_session WHERE id=?1",
                [&r.session_id],
                |row| row.get(0),
            )
            .map_err(err)?;
        let snap: Snapshot = serde_json::from_str(&json).unwrap();
        let item = &snap.items[s.current_question_index as usize];
        let n=c.execute("INSERT OR IGNORE INTO toeic_answer(id,session_id,item_id,item_version,selected_choice,is_correct,first_attempt,answered_at) VALUES(?1,?2,?3,?4,?5,?6,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![uuid::Uuid::new_v4().to_string(),r.session_id,r.item_id,r.item_version,r.selected_choice,u8::from(r.selected_choice==item.correct_answer)]).map_err(err)?;
        if n != 1 {
            return Err("The first Part 5 answer is final.".into());
        }
        c.execute(
            "UPDATE toeic_session SET updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
            [&r.session_id],
        )
        .map_err(err)?;
        if s.mode == "simulation" {
            self.advance(&r.session_id)
        } else {
            self.session(&r.session_id)
        }
    }
    pub fn advance(&self, id: &str) -> Result<Session, String> {
        let s = self.session(id)?;
        let q = s
            .current_question
            .as_ref()
            .ok_or("No current Part 5 question.")?;
        let c = database::open(&self.database)?;
        if answer(&c, id, &q.item_id, q.item_version)?.is_none() {
            return Err("Submit an answer before continuing.".into());
        }
        if s.current_question_index == 29 {
            c.execute("UPDATE toeic_session SET status='completed',current_question_index=30,completed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[id]).map_err(err)?;
        } else {
            c.execute("UPDATE toeic_session SET current_question_index=current_question_index+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[id]).map_err(err)?;
        }
        self.session(id)
    }
    pub fn result(&self, id: &str) -> Result<ResultDto, String> {
        let (c, snap, answers) = self.completed(id)?;
        drop(c);
        let mut cats = BTreeMap::new();
        let mut subs = BTreeMap::new();
        let mut diffs = BTreeMap::new();
        for item in &snap.items {
            let ok = answers.get(&item.item_id).map(|x| x.1).unwrap_or(false);
            add(&mut cats, &item.skill_category, ok);
            add(&mut subs, &item.skill_subcategory, ok);
            add(&mut diffs, &item.difficulty, ok)
        }
        let correct = answers.values().filter(|x| x.1).count() as u32;
        let grammar = one("grammar", *cats.get("grammar").unwrap_or(&(0, 0)));
        let vocabulary = one("vocabulary", *cats.get("vocabulary").unwrap_or(&(0, 0)));
        let subcategory_breakdown = breakdown(subs);
        let needs_attention = subcategory_breakdown
            .iter()
            .filter(|x| x.total >= 2 && x.accuracy < 70)
            .take(4)
            .map(|x| x.label.clone())
            .collect();
        Ok(ResultDto{session_id:id.into(),form_id:snap.form_id,correct,total:30,accuracy:correct*100/30,grammar,vocabulary,subcategory_breakdown,difficulty_breakdown:breakdown(diffs),needs_attention,has_scaled_score:false,score_message:"Part 5 raw performance only. Reading Parts 6 and 7 are not implemented, so no Reading scaled score is estimated.".into()})
    }
    pub fn review(&self, id: &str, mistakes: bool) -> Result<Vec<ReviewItem>, String> {
        let (_c, snap, answers) = self.completed(id)?;
        Ok(snap
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                answers
                    .get(&item.item_id)
                    .filter(|a| !mistakes || !a.1)
                    .map(|a| ReviewItem {
                        question_number: i as u32 + 1,
                        sentence: item.sentence.clone(),
                        choices: item
                            .choices
                            .iter()
                            .map(|x| PublicChoice {
                                choice: x.choice.clone(),
                                text: x.text.clone(),
                            })
                            .collect(),
                        feedback: feedback(item, &a.0, a.1),
                    })
            })
            .collect())
    }
    fn completed(
        &self,
        id: &str,
    ) -> Result<
        (
            rusqlite::Connection,
            Snapshot,
            BTreeMap<String, (String, bool)>,
        ),
        String,
    > {
        let c = database::open(&self.database)?;
        let(status,json)=c.query_row("SELECT status,form_snapshot_json FROM toeic_session WHERE id=?1 AND part='part5_incomplete_sentence'",[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).map_err(err)?;
        if status != "completed" {
            return Err("Complete Part 5 before viewing results.".into());
        }
        let snap = serde_json::from_str(&json).map_err(|_| "Invalid Part 5 snapshot.")?;
        let mut st = c
            .prepare(
                "SELECT item_id,selected_choice,is_correct FROM toeic_answer WHERE session_id=?1",
            )
            .map_err(err)?;
        let answers = st
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
            .map(|(id, choice, ok)| (id, (choice, ok)))
            .collect();
        drop(st);
        Ok((c, snap, answers))
    }
}
fn public(x: &Item, index: u32) -> PublicQuestion {
    PublicQuestion {
        item_id: x.item_id.clone(),
        item_version: x.item_version,
        question_number: index + 1,
        total_questions: 30,
        sentence: x.sentence.clone(),
        choices: x
            .choices
            .iter()
            .map(|c| PublicChoice {
                choice: c.choice.clone(),
                text: c.text.clone(),
            })
            .collect(),
    }
}
fn feedback(x: &Item, selected: &str, ok: bool) -> Feedback {
    Feedback {
        selected_choice: selected.into(),
        is_correct: ok,
        correct_choice: x.correct_answer.clone(),
        complete_sentence: x.complete_sentence.clone(),
        correct_explanation: x.correct_explanation.clone(),
        selected_distractor_explanation: if ok {
            None
        } else {
            x.distractor_explanations.get(selected).cloned()
        },
        other_distractor_explanations: x
            .choices
            .iter()
            .filter(|c| c.choice != x.correct_answer && c.choice != selected)
            .map(|c| ChoiceExplanation {
                choice: c.choice.clone(),
                text: c.text.clone(),
                explanation: x.distractor_explanations[&c.choice].clone(),
            })
            .collect(),
        skill_category: x.skill_category.clone(),
        skill_subcategory: x.skill_subcategory.clone(),
        useful_pattern: x.useful_pattern.clone(),
        extra_example: x.extra_example.clone(),
    }
}
fn answer(
    c: &rusqlite::Connection,
    sid: &str,
    item: &str,
    version: u32,
) -> Result<Option<(String, bool)>, String> {
    c.query_row("SELECT selected_choice,is_correct FROM toeic_answer WHERE session_id=?1 AND item_id=?2 AND item_version=?3",params![sid,item,version],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(err)
}
fn add(m: &mut BTreeMap<String, (u32, u32)>, key: &str, ok: bool) {
    let e = m.entry(key.into()).or_default();
    e.1 += 1;
    if ok {
        e.0 += 1
    }
}
fn one(label: &str, (correct, total): (u32, u32)) -> Breakdown {
    Breakdown {
        label: label.into(),
        correct,
        total,
        accuracy: if total == 0 { 0 } else { correct * 100 / total },
    }
}
fn breakdown(m: BTreeMap<String, (u32, u32)>) -> Vec<Breakdown> {
    m.into_iter().map(|(k, v)| one(&k, v)).collect()
}
fn err(e: rusqlite::Error) -> String {
    format!("Part 5 database error: {e}")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Part5Repository, PathBuf) {
        let db = std::env::temp_dir().join(format!("toeic-p5-{}.db", uuid::Uuid::new_v4()));
        database::migrate(&db).unwrap();
        let bank = Part5Bank::load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources/toeic/item-bank-v1/part5.json"),
        )
        .unwrap();
        (Part5Repository::new(db.clone(), bank), db)
    }
    fn respond(repo: &Part5Repository, s: &Session, choice: &str) -> Session {
        let q = s.current_question.as_ref().unwrap();
        repo.submit(Submit {
            session_id: s.session_id.clone(),
            item_id: q.item_id.clone(),
            item_version: q.item_version,
            selected_choice: choice.into(),
        })
        .unwrap()
    }
    #[test]
    fn production_form_is_complete_balanced_and_valid() {
        let b = Part5Bank::load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources/toeic/item-bank-v1/part5.json"),
        )
        .unwrap();
        assert_eq!(b.items.len(), 30);
        assert_eq!(b.forms["toeic-part5-form-a"].item_ids.len(), 30)
    }
    #[test]
    fn public_dto_hides_keys_and_authored_explanations() {
        let (repo, db) = setup();
        let s = repo.start("toeic-part5-form-a", 1, "learning").unwrap();
        let json = serde_json::to_string(&s.current_question).unwrap();
        assert!(
            !json.contains("correctAnswer")
                && !json.contains("completeSentence")
                && !json.contains("Explanation")
        );
        drop(repo);
        let _ = fs::remove_file(db);
    }
    #[test]
    fn first_answer_is_final_and_wrong_feedback_is_complete() {
        let (repo, db) = setup();
        let s = repo.start("toeic-part5-form-a", 1, "learning").unwrap();
        let q = s.current_question.as_ref().unwrap();
        let wrong = q
            .choices
            .iter()
            .find(|x| x.choice != "A")
            .unwrap()
            .choice
            .clone();
        let mut answered = respond(&repo, &s, &wrong);
        if answered.feedback.as_ref().unwrap().is_correct {
            answered = repo.advance(&answered.session_id).unwrap();
            let q = answered.current_question.as_ref().unwrap();
            let wrong = q
                .choices
                .iter()
                .find(|x| x.choice != "B")
                .unwrap()
                .choice
                .clone();
            answered = respond(&repo, &answered, &wrong)
        }
        let f = answered.feedback.unwrap();
        assert!(
            !f.is_correct
                && f.selected_distractor_explanation.is_some()
                && !f.complete_sentence.contains("_____")
        );
        let q = repo
            .session(&s.session_id)
            .unwrap()
            .current_question
            .unwrap();
        assert!(repo
            .submit(Submit {
                session_id: s.session_id.clone(),
                item_id: q.item_id,
                item_version: q.item_version,
                selected_choice: "D".into()
            })
            .is_err());
        drop(repo);
        let _ = fs::remove_file(db);
    }
    #[test]
    fn resumes_after_eleven_and_full_form_has_no_scaled_score() {
        let (repo, db) = setup();
        let mut s = repo.start("toeic-part5-form-a", 1, "learning").unwrap();
        for _ in 0..11 {
            s = respond(&repo, &s, "A");
            s = repo.advance(&s.session_id).unwrap()
        }
        let resumed = repo.session(&s.session_id).unwrap();
        assert_eq!(
            (
                resumed.answered_count,
                resumed.current_question.as_ref().unwrap().question_number
            ),
            (11, 12)
        );
        s = resumed;
        while s.status == "in_progress" {
            s = respond(&repo, &s, "A");
            s = repo.advance(&s.session_id).unwrap()
        }
        let result = repo.result(&s.session_id).unwrap();
        assert_eq!(result.total, 30);
        assert_eq!(result.grammar.total + result.vocabulary.total, 30);
        assert!(!result.has_scaled_score);
        drop(repo);
        let _ = fs::remove_file(db);
    }
    #[test]
    fn simulation_auto_advances_without_feedback() {
        let (repo, db) = setup();
        let s = repo.start("toeic-part5-form-a", 1, "simulation").unwrap();
        let s = respond(&repo, &s, "A");
        assert_eq!(s.current_question.unwrap().question_number, 2);
        assert!(s.feedback.is_none());
        drop(repo);
        let _ = fs::remove_file(db);
    }
    #[test]
    fn deterministic_full_form_outcomes_zero_fifteen_twenty_four_and_thirty() {
        for expected in [0usize, 15, 24, 30] {
            let (repo, db) = setup();
            let mut s = repo.start("toeic-part5-form-a", 1, "learning").unwrap();
            for index in 0..30 {
                let item = &repo.bank.items[&format!("toeic-r-p5-{:04}", index + 1)];
                let choice = if index < expected {
                    item.correct_answer.clone()
                } else {
                    ["A", "B", "C", "D"]
                        .into_iter()
                        .find(|x| *x != item.correct_answer)
                        .unwrap()
                        .into()
                };
                s = respond(&repo, &s, &choice);
                s = repo.advance(&s.session_id).unwrap()
            }
            let result = repo.result(&s.session_id).unwrap();
            assert_eq!(
                (result.correct, result.accuracy),
                (expected as u32, expected as u32 * 100 / 30)
            );
            drop(repo);
            let _ = fs::remove_file(db);
        }
    }
}
