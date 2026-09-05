use crate::database;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Speaker {
    id: String,
    voice: String,
    accent: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Graphic {
    title: String,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Question {
    id: String,
    text: String,
    choices: Vec<String>,
    answer: String,
    r#type: String,
    skill: String,
    difficulty: String,
    explanation: String,
    wrong: BTreeMap<String, String>,
    evidence: String,
    notice: String,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Set {
    set_id: String,
    version: u32,
    difficulty: String,
    scenario: String,
    speakers: Vec<Speaker>,
    turns: Vec<(String, String)>,
    graphic: Option<Graphic>,
    questions: Vec<Question>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Form {
    form_id: String,
    form_version: u32,
    set_ids: Vec<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Raw {
    schema_version: u32,
    bank_id: String,
    forms: Vec<Form>,
    sets: Vec<Set>,
}
#[derive(Clone)]
pub struct Part4Bank {
    bank_id: String,
    forms: BTreeMap<String, Form>,
    sets: BTreeMap<String, Set>,
}
impl Part4Bank {
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let raw: Raw = serde_json::from_slice(
            &fs::read(path).map_err(|e| format!("Part 4 bank missing: {e}"))?,
        )
        .map_err(|e| format!("Part 4 bank invalid: {e}"))?;
        validate(&raw)?;
        Ok(Self {
            bank_id: raw.bank_id,
            forms: raw
                .forms
                .into_iter()
                .map(|x| (x.form_id.clone(), x))
                .collect(),
            sets: raw
                .sets
                .into_iter()
                .map(|x| (x.set_id.clone(), x))
                .collect(),
        })
    }
}
fn validate(raw: &Raw) -> Result<(), String> {
    if raw.schema_version != 1 {
        return Err("Unsupported Part 4 schema.".into());
    }
    let choices = BTreeSet::from(["A", "B", "C", "D"]);
    let difficulties = BTreeSet::from(["easy", "medium", "hard"]);
    let types = BTreeSet::from([
        "main_idea",
        "purpose",
        "detail",
        "location",
        "speaker_role",
        "problem",
        "request",
        "suggestion",
        "next_action",
        "future_action",
        "reason",
        "opinion",
        "inference",
        "implied_meaning",
        "paraphrase",
        "relationship",
        "schedule",
        "graphic_integration",
    ]);
    let voices = BTreeSet::from(["amy", "lessac"]);
    let mut set_ids = BTreeSet::new();
    let mut question_ids = BTreeSet::new();
    for s in &raw.sets {
        if !set_ids.insert(s.set_id.clone())
            || !s.set_id.starts_with("toeic-l-p4-set-")
            || s.version != 1
            || !difficulties.contains(s.difficulty.as_str())
            || s.speakers.len() != 1
            || s.turns.len() != 1
            || s.questions.len() != 3
            || s.scenario.trim().is_empty()
        {
            return Err(format!("Invalid Part 4 set {}.", s.set_id));
        }
        let speaker_ids = s
            .speakers
            .iter()
            .map(|x| x.id.as_str())
            .collect::<BTreeSet<_>>();
        if speaker_ids.len() != s.speakers.len()
            || s.speakers
                .iter()
                .any(|x| !voices.contains(x.voice.as_str()) || x.accent != "en-US")
            || s.turns
                .iter()
                .any(|(who, text)| !speaker_ids.contains(who.as_str()) || text.trim().len() < 5)
        {
            return Err(format!("Invalid speakers/turns in {}.", s.set_id));
        }
        if let Some(g) = &s.graphic {
            if g.title.trim().is_empty()
                || g.columns.len() < 2
                || g.rows.is_empty()
                || g.rows.iter().any(|r| r.len() != g.columns.len())
            {
                return Err(format!("Invalid graphic in {}.", s.set_id));
            }
        }
        for q in &s.questions {
            if !question_ids.insert(q.id.clone())
                || !q.id.starts_with(&format!("{}-q", s.set_id))
                || q.choices.len() != 4
                || q.choices
                    .iter()
                    .map(|x| x.to_lowercase())
                    .collect::<BTreeSet<_>>()
                    .len()
                    != 4
                || !choices.contains(q.answer.as_str())
                || !types.contains(q.r#type.as_str())
                || !difficulties.contains(q.difficulty.as_str())
                || q.skill.trim().is_empty()
                || q.explanation.len() < 20
                || q.evidence.trim().is_empty()
                || q.notice.trim().is_empty()
            {
                return Err(format!("Invalid question {}.", q.id));
            }
            let wrong = choices
                .iter()
                .filter(|x| **x != q.answer)
                .map(|x| x.to_string())
                .collect::<BTreeSet<_>>();
            if q.wrong.keys().cloned().collect::<BTreeSet<_>>() != wrong
                || q.wrong.values().any(|x| x.len() < 10)
            {
                return Err(format!("Invalid distractors in {}.", q.id));
            }
        }
    }
    for f in &raw.forms {
        if f.set_ids.len() != 10
            || f.set_ids.iter().collect::<BTreeSet<_>>().len() != 10
            || f.set_ids.iter().any(|x| !set_ids.contains(x))
        {
            return Err(format!("{} must contain 10 unique sets.", f.form_id));
        }
        let qs = f
            .set_ids
            .iter()
            .flat_map(|id| {
                raw.sets
                    .iter()
                    .find(|s| &s.set_id == id)
                    .unwrap()
                    .questions
                    .iter()
            })
            .collect::<Vec<_>>();
        if qs.len() != 30 {
            return Err("Part 4 form must contain 30 questions.".into());
        }
        let mut answer_counts = BTreeMap::new();
        for q in &qs {
            *answer_counts.entry(&q.answer).or_insert(0) += 1
        }
        if answer_counts.values().any(|n| *n < 5 || *n > 10)
            || qs.iter().map(|q| &q.r#type).collect::<BTreeSet<_>>().len() < 8
            || f.set_ids
                .iter()
                .map(|id| &raw.sets.iter().find(|s| &s.set_id == id).unwrap().scenario)
                .collect::<BTreeSet<_>>()
                .len()
                < 10
        {
            return Err("Suspicious Part 4 form distribution.".into());
        }
    }
    Ok(())
}

#[derive(Clone)]
pub struct Part4Repository {
    database: PathBuf,
    bank: Part4Bank,
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
pub struct PublicQuestion {
    pub question_id: String,
    pub text: String,
    pub choices: Vec<Choice>,
    pub answered: bool,
    pub selected_choice: Option<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    pub choice: String,
    pub text: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicSet {
    pub set_id: String,
    pub set_version: u32,
    pub set_number: u32,
    pub total_sets: u32,
    pub questions: Vec<PublicQuestion>,
    pub graphic: Option<Graphic>,
    pub initial_audio_completed: bool,
    pub initial_audio_interrupted: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub session_id: String,
    pub form_id: String,
    pub status: String,
    pub current_set_index: u32,
    pub answered_count: u32,
    pub completed_sets: u32,
    pub current_set: Option<PublicSet>,
    pub feedback: Option<SetFeedback>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Submit {
    pub session_id: String,
    pub question_id: String,
    pub selected_choice: String,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionFeedback {
    pub question_id: String,
    pub text: String,
    pub choices: Vec<Choice>,
    pub selected_choice: String,
    pub correct_answer: String,
    pub is_correct: bool,
    pub correct_explanation: String,
    pub selected_explanation: Option<String>,
    pub evidence: String,
    pub listening_skill: String,
    pub useful_language: String,
    pub notice_next_time: String,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptTurn {
    pub speaker: String,
    pub text: String,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetFeedback {
    pub set_id: String,
    pub questions: Vec<QuestionFeedback>,
    pub transcript: Vec<TranscriptTurn>,
}
pub struct AudioTurn {
    pub set_id: String,
    pub set_version: u32,
    pub turn_index: u32,
    pub turn_count: u32,
    pub text: String,
    pub voice: String,
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
pub struct ResultDto {
    pub session_id: String,
    pub form_id: String,
    pub correct: u32,
    pub total: u32,
    pub accuracy: u32,
    pub question_type_breakdown: Vec<Breakdown>,
    pub difficulty_breakdown: Vec<Breakdown>,
    pub scenario_breakdown: Vec<Breakdown>,
    pub skill_breakdown: Vec<Breakdown>,
    pub has_scaled_score: bool,
    pub score_message: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSet {
    pub set_number: u32,
    pub scenario: String,
    pub graphic: Option<Graphic>,
    pub feedback: SetFeedback,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    form_id: String,
    form_version: u32,
    set_ids: Vec<String>,
}

impl Part4Repository {
    pub fn new(database: PathBuf, bank: Part4Bank) -> Self {
        Self { database, bank }
    }
    pub fn overview(&self) -> Result<Overview, String> {
        let c = database::open(&self.database)?;
        Ok(Overview{bank_id:self.bank.bank_id.clone(),forms:self.bank.forms.values().map(|f|FormDto{form_id:f.form_id.clone(),form_version:f.form_version,title:title(&f.form_id),active_session_id:c.query_row("SELECT id FROM toeic_session WHERE form_id=?1 AND status='in_progress'",[&f.form_id],|r|r.get(0)).optional().ok().flatten()}).collect()})
    }
    pub fn start(&self, form_id: &str, version: u32) -> Result<Session, String> {
        let f = self
            .bank
            .forms
            .get(form_id)
            .filter(|f| f.form_version == version)
            .ok_or("Part 4 form not found.")?;
        let c = database::open(&self.database)?;
        if let Some(id)=c.query_row("SELECT id FROM toeic_session WHERE form_id=?1 AND form_version=?2 AND status='in_progress'",params![form_id,version],|r|r.get::<_,String>(0)).optional().map_err(err)?{return self.session(&id)}
        let id = uuid::Uuid::new_v4().to_string();
        let snapshot = Snapshot {
            form_id: f.form_id.clone(),
            form_version: f.form_version,
            set_ids: f.set_ids.clone(),
        };
        c.execute("INSERT INTO toeic_session(id,form_id,form_version,section,part,status,schema_version,form_snapshot_json,current_question_index,created_at,updated_at) VALUES(?1,?2,?3,'listening','part4_talk','in_progress',1,?4,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![id,form_id,version,serde_json::to_string(&snapshot).unwrap()]).map_err(err)?;
        self.session(&id)
    }
    pub fn session(&self, id: &str) -> Result<Session, String> {
        let c = database::open(&self.database)?;
        let(form,status,json,index)=c.query_row("SELECT form_id,status,form_snapshot_json,current_question_index FROM toeic_session WHERE id=?1 AND part='part4_talk'",[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,u32>(3)?))).optional().map_err(err)?.ok_or("Part 4 session not found.")?;
        let snap: Snapshot = serde_json::from_str(&json).map_err(|_| "Invalid Part 4 snapshot.")?;
        let answers = answers(&c, id)?;
        let current = if status == "in_progress" {
            snap.set_ids
                .get(index as usize)
                .and_then(|x| self.bank.sets.get(x))
        } else {
            None
        };
        let (set, feedback) = if let Some(s) = current {
            let (done, interrupted) = presentation(&c, id, &s.set_id)?;
            let public = PublicSet {
                set_id: s.set_id.clone(),
                set_version: s.version,
                set_number: index + 1,
                total_sets: 10,
                questions: s
                    .questions
                    .iter()
                    .map(|q| {
                        let a = answers.get(&q.id);
                        PublicQuestion {
                            question_id: q.id.clone(),
                            text: q.text.clone(),
                            choices: choices(q),
                            answered: a.is_some(),
                            selected_choice: a.map(|x| x.0.clone()),
                        }
                    })
                    .collect(),
                graphic: s.graphic.clone(),
                initial_audio_completed: done,
                initial_audio_interrupted: interrupted,
            };
            let complete = s.questions.iter().all(|q| answers.contains_key(&q.id));
            (Some(public), complete.then(|| set_feedback(s, &answers)))
        } else {
            (None, None)
        };
        Ok(Session {
            session_id: id.into(),
            form_id: form,
            status,
            current_set_index: index,
            answered_count: answers.len() as u32,
            completed_sets: index,
            current_set: set,
            feedback,
        })
    }
    pub fn begin_audio(
        &self,
        id: &str,
        turn: u32,
        presentation_id: Option<&str>,
    ) -> Result<AudioTurn, String> {
        let session = self.session(id)?;
        let public = session.current_set.ok_or("No current Part 4 set.")?;
        let set = &self.bank.sets[&public.set_id];
        let initial = session.feedback.is_none();
        if initial && public.initial_audio_completed {
            return Err("The initial talk can be played only once.".into());
        }
        if turn as usize >= set.turns.len() {
            return Err("Invalid talk turn.".into());
        }
        let c = database::open(&self.database)?;
        let pid = if turn == 0 {
            if presentation_id.is_some() {
                return Err("Unexpected presentation identity.".into());
            }
            if initial {
                c.execute("UPDATE toeic_presentation_attempt SET status='interrupted',interrupted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE session_id=?1 AND item_id=?2 AND status='started'",params![id,set.set_id]).map_err(err)?;
                let p = uuid::Uuid::new_v4().to_string();
                c.execute("INSERT INTO toeic_presentation_attempt(id,session_id,item_id,item_version,status,started_at) VALUES(?1,?2,?3,1,'started',strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![p,id,set.set_id]).map_err(err)?;
                Some(p)
            } else {
                None
            }
        } else {
            if initial {
                let p = presentation_id.ok_or("Missing Part 4 presentation identity.")?;
                let exists:i32=c.query_row("SELECT EXISTS(SELECT 1 FROM toeic_presentation_attempt WHERE id=?1 AND session_id=?2 AND item_id=?3 AND status='started')",params![p,id,set.set_id],|r|r.get(0)).map_err(err)?;
                if exists == 0 {
                    return Err("Stale Part 4 presentation.".into());
                }
                Some(p.to_owned())
            } else {
                None
            }
        };
        let (speaker, text) = &set.turns[turn as usize];
        let voice = set
            .speakers
            .iter()
            .find(|x| &x.id == speaker)
            .unwrap()
            .voice
            .clone();
        Ok(AudioTurn {
            set_id: set.set_id.clone(),
            set_version: 1,
            turn_index: turn,
            turn_count: set.turns.len() as u32,
            text: text.clone(),
            voice: format!("en_US-{voice}-medium"),
            presentation_id: pid,
            initial,
        })
    }
    pub fn finish_audio(
        &self,
        id: &str,
        set_id: &str,
        pid: Option<&str>,
        initial: bool,
    ) -> Result<(), String> {
        if initial {
            let p = pid.ok_or("Missing Part 4 presentation identity.")?;
            let n=database::open(&self.database)?.execute("UPDATE toeic_presentation_attempt SET status='completed',completed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND session_id=?2 AND item_id=?3 AND status='started'",params![p,id,set_id]).map_err(err)?;
            if n != 1 {
                return Err("Stale Part 4 playback.".into());
            }
        }
        Ok(())
    }
    pub fn interrupt(&self, pid: Option<&str>) -> Result<(), String> {
        if let Some(p) = pid {
            database::open(&self.database)?.execute("UPDATE toeic_presentation_attempt SET status='interrupted',interrupted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND status='started'",[p]).map_err(err)?;
        }
        Ok(())
    }
    pub fn submit(&self, r: Submit) -> Result<Session, String> {
        if !matches!(r.selected_choice.as_str(), "A" | "B" | "C" | "D") {
            return Err("Answer must be A, B, C, or D.".into());
        }
        let s = self.session(&r.session_id)?;
        let set_public = s.current_set.ok_or("No current Part 4 set.")?;
        if !set_public.initial_audio_completed {
            return Err("Listen to the complete talk first.".into());
        }
        let set = &self.bank.sets[&set_public.set_id];
        let q = set
            .questions
            .iter()
            .find(|q| q.id == r.question_id)
            .ok_or("Question is not in the current set.")?;
        let c = database::open(&self.database)?;
        let n=c.execute("INSERT OR IGNORE INTO toeic_answer(id,session_id,item_id,item_version,selected_choice,is_correct,first_attempt,answered_at) VALUES(?1,?2,?3,1,?4,?5,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![uuid::Uuid::new_v4().to_string(),r.session_id,r.question_id,r.selected_choice,u8::from(r.selected_choice==q.answer)]).map_err(err)?;
        if n != 1 {
            return Err("The first Part 4 answer is final.".into());
        }
        self.session(&r.session_id)
    }
    pub fn advance(&self, id: &str) -> Result<Session, String> {
        let s = self.session(id)?;
        if s.feedback.is_none() {
            return Err("Complete all three questions before continuing.".into());
        }
        let c = database::open(&self.database)?;
        if s.current_set_index == 9 {
            c.execute("UPDATE toeic_session SET status='completed',current_question_index=10,completed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[id]).map_err(err)?;
        } else {
            c.execute("UPDATE toeic_session SET current_question_index=current_question_index+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[id]).map_err(err)?;
        }
        self.session(id)
    }
    pub fn result(&self, id: &str) -> Result<ResultDto, String> {
        let c = database::open(&self.database)?;
        let(form,json)=c.query_row("SELECT form_id,form_snapshot_json FROM toeic_session WHERE id=?1 AND part='part4_talk'",[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).map_err(err)?;
        let snap: Snapshot = serde_json::from_str(&json).unwrap();
        let all = answers(&c, id)?;
        let correct = all.values().filter(|x| x.1).count() as u32;
        let (mut types, mut diffs, mut scenarios, mut skills) = (
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
        );
        for sid in snap.set_ids {
            let s = &self.bank.sets[&sid];
            for q in &s.questions {
                if let Some(a) = all.get(&q.id) {
                    add(&mut types, &q.r#type, a.1);
                    add(&mut diffs, &q.difficulty, a.1);
                    add(&mut scenarios, &s.scenario, a.1);
                    add(&mut skills, &q.skill, a.1)
                }
            }
        }
        Ok(ResultDto{session_id:id.into(),form_id:form,correct,total:30,accuracy:correct*100/30,question_type_breakdown:breakdown(types),difficulty_breakdown:breakdown(diffs),scenario_breakdown:breakdown(scenarios),skill_breakdown:breakdown(skills),has_scaled_score:false,score_message:"Part 4 raw performance only. A scaled estimate is produced only after a complete 100-question Listening simulation.".into()})
    }
    pub fn review(&self, id: &str, mistakes: bool) -> Result<Vec<ReviewSet>, String> {
        let c = database::open(&self.database)?;
        let(status,json)=c.query_row("SELECT status,form_snapshot_json FROM toeic_session WHERE id=?1 AND part='part4_talk'",[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).map_err(err)?;
        if status != "completed" {
            return Err("Complete Part 4 before review.".into());
        }
        let snap: Snapshot = serde_json::from_str(&json).unwrap();
        let all = answers(&c, id)?;
        Ok(snap
            .set_ids
            .iter()
            .enumerate()
            .filter_map(|(i, id)| {
                let s = &self.bank.sets[id];
                let mut f = set_feedback(s, &all);
                if mistakes && f.questions.iter().all(|q| q.is_correct) {
                    None
                } else {
                    if mistakes {
                        f.questions.retain(|q| !q.is_correct);
                    }
                    Some(ReviewSet {
                        set_number: i as u32 + 1,
                        scenario: s.scenario.clone(),
                        graphic: s.graphic.clone(),
                        feedback: f,
                    })
                }
            })
            .collect())
    }
}
fn answers(c: &rusqlite::Connection, id: &str) -> Result<BTreeMap<String, (String, bool)>, String> {
    let mut s = c
        .prepare("SELECT item_id,selected_choice,is_correct FROM toeic_answer WHERE session_id=?1")
        .map_err(err)?;
    let value = s
        .query_map([id], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err);
    value
}
fn presentation(c: &rusqlite::Connection, id: &str, item: &str) -> Result<(bool, bool), String> {
    c.query_row("SELECT EXISTS(SELECT 1 FROM toeic_presentation_attempt WHERE session_id=?1 AND item_id=?2 AND status='completed'),EXISTS(SELECT 1 FROM toeic_presentation_attempt WHERE session_id=?1 AND item_id=?2 AND status='interrupted')",params![id,item],|r|Ok((r.get(0)?,r.get(1)?))).map_err(err)
}
fn choices(q: &Question) -> Vec<Choice> {
    q.choices
        .iter()
        .enumerate()
        .map(|(i, text)| Choice {
            choice: ["A", "B", "C", "D"][i].into(),
            text: text.clone(),
        })
        .collect()
}
fn set_feedback(s: &Set, a: &BTreeMap<String, (String, bool)>) -> SetFeedback {
    SetFeedback {
        set_id: s.set_id.clone(),
        questions: s
            .questions
            .iter()
            .map(|q| {
                let x = &a[&q.id];
                QuestionFeedback {
                    question_id: q.id.clone(),
                    text: q.text.clone(),
                    choices: choices(q),
                    selected_choice: x.0.clone(),
                    correct_answer: q.answer.clone(),
                    is_correct: x.1,
                    correct_explanation: q.explanation.clone(),
                    selected_explanation: q.wrong.get(&x.0).cloned(),
                    evidence: q.evidence.clone(),
                    listening_skill: q.skill.clone(),
                    useful_language: q.evidence.clone(),
                    notice_next_time: q.notice.clone(),
                }
            })
            .collect(),
        transcript: s
            .turns
            .iter()
            .map(|(speaker, text)| TranscriptTurn {
                speaker: speaker.clone(),
                text: text.clone(),
            })
            .collect(),
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
fn title(id: &str) -> String {
    format!(
        "Part 4 Form {}",
        id.chars().last().unwrap_or('A').to_ascii_uppercase()
    )
}
fn err(e: rusqlite::Error) -> String {
    format!("Part 4 database error: {e}")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Part4Repository, PathBuf) {
        let db = std::env::temp_dir().join(format!("toeic-p4-{}.db", uuid::Uuid::new_v4()));
        database::migrate(&db).unwrap();
        let bank = Part4Bank::load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources/toeic/item-bank-v1/part4.json"),
        )
        .unwrap();
        (Part4Repository::new(db.clone(), bank), db)
    }
    fn hear(repo: &Part4Repository, s: &Session) {
        let set = s.current_set.as_ref().unwrap();
        let p = repo.begin_audio(&s.session_id, 0, None).unwrap();
        repo.finish_audio(
            &s.session_id,
            &set.set_id,
            p.presentation_id.as_deref(),
            true,
        )
        .unwrap();
    }
    #[test]
    fn production_forms_are_complete_and_valid() {
        let b = Part4Bank::load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources/toeic/item-bank-v1/part4.json"),
        )
        .unwrap();
        assert_eq!(b.forms.len(), 3);
        assert_eq!(b.sets.len(), 30);
        assert!(b.forms.values().all(|form| form.set_ids.len() == 10));
        assert_eq!(
            b.sets.values().map(|s| s.questions.len()).sum::<usize>(),
            90
        );
        assert!(b
            .sets
            .values()
            .all(|s| s.speakers.len() == 1 && s.turns.len() == 1));
    }
    #[test]
    fn forms_b_and_c_start_with_their_frozen_talks() {
        let (repo, db) = setup();
        for family in ["b", "c"] {
            let form_id = format!("toeic-part4-form-{family}");
            let session = repo.start(&form_id, 1).unwrap();
            assert_eq!(session.form_id, form_id);
            assert_eq!(
                session.current_set.as_ref().unwrap().set_id,
                format!("toeic-l-p4-set-{family}01")
            );
        }
        drop(repo);
        let _ = std::fs::remove_file(db);
    }
    #[test]
    fn public_payload_hides_script_keys_and_feedback_until_third_answer() {
        let (repo, db) = setup();
        let mut s = repo.start("toeic-part4-form-a", 1).unwrap();
        let json = serde_json::to_string(&s).unwrap();
        assert!(
            !json.contains("correctAnswer")
                && !json.contains("transcript")
                && !json.contains("evidence")
        );
        hear(&repo, &s);
        let ids = s
            .current_set
            .as_ref()
            .unwrap()
            .questions
            .iter()
            .take(2)
            .map(|q| q.question_id.clone())
            .collect::<Vec<_>>();
        for question_id in ids {
            s = repo
                .submit(Submit {
                    session_id: s.session_id.clone(),
                    question_id,
                    selected_choice: "A".into(),
                })
                .unwrap();
            assert!(s.feedback.is_none());
        }
        let q = s.current_set.as_ref().unwrap().questions[2]
            .question_id
            .clone();
        s = repo
            .submit(Submit {
                session_id: s.session_id.clone(),
                question_id: q,
                selected_choice: "A".into(),
            })
            .unwrap();
        assert!(s.feedback.is_some());
        drop(repo);
        let _ = std::fs::remove_file(db);
    }
    #[test]
    fn completes_exactly_thirty_and_resumes() {
        let (repo, db) = setup();
        let mut s = repo.start("toeic-part4-form-a", 1).unwrap();
        while s.status == "in_progress" {
            hear(&repo, &s);
            let ids = s
                .current_set
                .as_ref()
                .unwrap()
                .questions
                .iter()
                .map(|q| q.question_id.clone())
                .collect::<Vec<_>>();
            for id in ids {
                s = repo
                    .submit(Submit {
                        session_id: s.session_id.clone(),
                        question_id: id,
                        selected_choice: "A".into(),
                    })
                    .unwrap();
            }
            if s.status == "in_progress" {
                s = repo.advance(&s.session_id).unwrap();
            }
        }
        let result = repo.result(&s.session_id).unwrap();
        assert_eq!(result.total, 30);
        assert_eq!(repo.review(&s.session_id, false).unwrap().len(), 10);
        assert_eq!(repo.session(&s.session_id).unwrap().status, "completed");
        drop(repo);
        let _ = std::fs::remove_file(db);
    }
    #[test]
    fn interrupted_audio_can_restart_but_completed_audio_cannot() {
        let (repo, db) = setup();
        let s = repo.start("toeic-part4-form-a", 1).unwrap();
        let p = repo.begin_audio(&s.session_id, 0, None).unwrap();
        repo.interrupt(p.presentation_id.as_deref()).unwrap();
        hear(&repo, &repo.session(&s.session_id).unwrap());
        assert!(repo.begin_audio(&s.session_id, 0, None).is_err());
        drop(repo);
        let _ = std::fs::remove_file(db);
    }
}
