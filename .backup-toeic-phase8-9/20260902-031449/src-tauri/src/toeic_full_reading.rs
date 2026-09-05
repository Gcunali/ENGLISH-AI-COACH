use crate::{
    database,
    toeic_part5::Part5Repository,
    toeic_part6::Part6Repository,
    toeic_part7::Part7Repository,
    toeic_reading_score::{self, ReadingEstimate},
};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;
#[derive(Clone)]
pub struct FullReadingRepository {
    database: PathBuf,
    pub(crate) p5: Part5Repository,
    pub(crate) p6: Part6Repository,
    pub(crate) p7: Part7Repository,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullPart {
    pub part_number: u32,
    pub title: String,
    pub question_count: u32,
    pub session_id: String,
    pub status: String,
    pub route: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullSession {
    pub session_id: String,
    pub family: String,
    pub mode: String,
    pub status: String,
    pub current_part: u32,
    pub answered_count: u32,
    pub total_questions: u32,
    pub parts: Vec<FullPart>,
    pub estimate: Option<ReadingEstimate>,
    pub disclaimer: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullHistory {
    pub session_id: String,
    pub mode: String,
    pub status: String,
    pub raw_correct: Option<u32>,
    pub estimated_score: Option<u32>,
    pub range_low: Option<u32>,
    pub range_high: Option<u32>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub family: String,
    pub answered_count: u32,
    pub current_part: u32,
    pub score_profile_version: Option<u32>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregatePart {
    pub part_number: u32,
    pub session_id: String,
    pub result: Option<Value>,
    pub review: Value,
}
impl FullReadingRepository {
    pub fn new(
        database: PathBuf,
        p5: Part5Repository,
        p6: Part6Repository,
        p7: Part7Repository,
    ) -> Self {
        Self {
            database,
            p5,
            p6,
            p7,
        }
    }
    pub fn start(&self, mode: &str) -> Result<FullSession, String> {
        self.start_with_family(mode, "A")
    }
    pub fn start_with_family(&self, mode: &str, family: &str) -> Result<FullSession, String> {
        if !matches!(mode, "simulation" | "learning") {
            return Err("Full Reading mode must be simulation or learning.".into());
        }
        let family = family.trim().to_ascii_uppercase();
        if !matches!(family.as_str(), "A" | "B" | "C") {
            return Err("Full Reading family must be A, B, or C.".into());
        }
        let suffix = family.to_ascii_lowercase();
        let c = database::open(&self.database)?;
        if let Some(id)=c.query_row("SELECT id FROM toeic_full_reading_session WHERE status='in_progress' AND mode=?1 AND family=?2 ORDER BY created_at DESC LIMIT 1",params![mode,family],|r|r.get::<_,String>(0)).optional().map_err(err)?{return self.session(&id)}
        let p5_form = format!("toeic-part5-form-{suffix}");
        let p6_form = format!("toeic-part6-form-{suffix}");
        let p7_form = format!("toeic-part7-form-{suffix}");
        let p5 = self.p5.start(&p5_form, 1, mode)?;
        let p6 = self.p6.start(&p6_form, 1, mode)?;
        let p7 = self.p7.start(&p7_form, 1, mode)?;
        if [&p5.mode, &p6.mode, &p7.mode]
            .iter()
            .any(|x| x.as_str() != mode)
        {
            return Err("Finish the active standalone Reading practice before starting a differently configured Full Reading simulation.".into());
        }
        let id = uuid::Uuid::new_v4().to_string();
        let composition = serde_json::json!({"family":family,"parts":[{"part":5,"form":p5_form,"version":1},{"part":6,"form":p6_form,"version":1},{"part":7,"form":p7_form,"version":1}]});
        let tx = c.unchecked_transaction().map_err(err)?;
        tx.execute("INSERT INTO toeic_full_reading_session(id,family,mode,status,current_part,composition_json,created_at,updated_at) VALUES(?1,?2,?3,'in_progress',5,?4,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![id,family,mode,composition.to_string()]).map_err(err)?;
        for (part, sid, form) in [
            (5, p5.session_id, p5_form.as_str()),
            (6, p6.session_id, p6_form.as_str()),
            (7, p7.session_id, p7_form.as_str()),
        ] {
            tx.execute("INSERT INTO toeic_full_reading_part(full_session_id,part_number,toeic_session_id,form_id,form_version,status) VALUES(?1,?2,?3,?4,1,CASE WHEN ?2=5 THEN 'in_progress' ELSE 'pending' END)",params![id,part,sid,form]).map_err(err)?;
        }
        tx.commit().map_err(err)?;
        self.session(&id)
    }
    pub fn session(&self, id: &str) -> Result<FullSession, String> {
        let c = database::open(&self.database)?;
        let (family, mode, parent) = c
            .query_row(
                "SELECT family,mode,status FROM toeic_full_reading_session WHERE id=?1",
                [id],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(err)?
            .ok_or("Full Reading simulation not found.")?;
        let mut st=c.prepare("SELECT p.part_number,p.toeic_session_id,s.status FROM toeic_full_reading_part p JOIN toeic_session s ON s.id=p.toeic_session_id WHERE p.full_session_id=?1 ORDER BY p.part_number").map_err(err)?;
        let rows = st
            .query_map([id], |r| {
                Ok((
                    r.get::<_, u32>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        if rows.len() != 3 {
            return Err("Full Reading composition is incomplete.".into());
        }
        let answered=c.query_row("SELECT COUNT(*) FROM toeic_answer a JOIN toeic_full_reading_part p ON p.toeic_session_id=a.session_id WHERE p.full_session_id=?1",[id],|r|r.get::<_,u32>(0)).map_err(err)?;
        let current = rows
            .iter()
            .find(|x| x.2 != "completed")
            .map(|x| x.0)
            .unwrap_or(7);
        let complete = rows.iter().all(|x| x.2 == "completed");
        let mut estimate = None;
        if complete {
            let raw=c.query_row("SELECT COALESCE(SUM(a.is_correct),0) FROM toeic_answer a JOIN toeic_full_reading_part p ON p.toeic_session_id=a.session_id WHERE p.full_session_id=?1",[id],|r|r.get::<_,u32>(0)).map_err(err)?;
            let e = toeic_reading_score::estimate(raw)?;
            c.execute("UPDATE toeic_full_reading_session SET status='completed',current_part=7,raw_correct=?2,estimated_score=?3,range_low=?4,range_high=?5,score_profile_id=?6,score_profile_version=?7,completed_at=COALESCE(completed_at,strftime('%Y-%m-%dT%H:%M:%fZ','now')),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![id,raw,e.estimated_score,e.range_low,e.range_high,e.profile_id,e.profile_version]).map_err(err)?;
            estimate = Some(e)
        } else {
            c.execute("UPDATE toeic_full_reading_session SET current_part=?2,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![id,current]).map_err(err)?;
        }
        let meta = [
            (
                "Incomplete Sentences",
                30,
                "/toeic/part5/session/",
                "/toeic/part5/results/",
            ),
            (
                "Text Completion",
                16,
                "/toeic/part6/session/",
                "/toeic/part6/results/",
            ),
            (
                "Reading Comprehension",
                54,
                "/toeic/part7/session/",
                "/toeic/part7/results/",
            ),
        ];
        let parts = rows
            .into_iter()
            .enumerate()
            .map(|(i, (n, sid, status))| FullPart {
                part_number: n,
                title: meta[i].0.into(),
                question_count: meta[i].1,
                route: if complete {
                    format!("{}{}", meta[i].3, sid)
                } else {
                    format!("{}{}?fullReading={id}&mode={mode}", meta[i].2, sid)
                },
                session_id: sid,
                status,
            })
            .collect();
        Ok(FullSession{session_id:id.into(),family,mode,status:if complete{"completed".into()}else{parent},current_part:current,answered_count:answered,total_questions:100,parts,estimate,disclaimer:"Untimed unofficial TOEIC-style Reading simulation. Official scores use ETS equating and may differ.".into()})
    }
    pub fn history(&self) -> Result<Vec<FullHistory>, String> {
        let c = database::open(&self.database)?;
        let mut st=c.prepare("SELECT f.id,f.mode,f.status,f.raw_correct,f.estimated_score,f.range_low,f.range_high,f.created_at,f.completed_at,f.family,(SELECT COUNT(*) FROM toeic_answer a JOIN toeic_full_reading_part p ON p.toeic_session_id=a.session_id WHERE p.full_session_id=f.id),f.current_part,f.score_profile_version FROM toeic_full_reading_session f ORDER BY f.created_at DESC,f.id DESC").map_err(err)?;
        let rows = st
            .query_map([], |r| {
                Ok(FullHistory {
                    session_id: r.get(0)?,
                    mode: r.get(1)?,
                    status: r.get(2)?,
                    raw_correct: r.get(3)?,
                    estimated_score: r.get(4)?,
                    range_low: r.get(5)?,
                    range_high: r.get(6)?,
                    created_at: r.get(7)?,
                    completed_at: r.get(8)?,
                    family: r.get(9)?,
                    answered_count: r.get(10)?,
                    current_part: r.get(11)?,
                    score_profile_version: r.get(12)?,
                })
            })
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        Ok(rows)
    }
    pub fn aggregate(&self, id: &str, mistakes: bool) -> Result<Vec<AggregatePart>, String> {
        let c = database::open(&self.database)?;
        let mut st=c.prepare("SELECT part_number,toeic_session_id FROM toeic_full_reading_part WHERE full_session_id=?1 ORDER BY part_number").map_err(err)?;
        let rows = st
            .query_map([id], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?)))
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        if rows.len() != 3 {
            return Err("Full Reading composition is incomplete.".into());
        }
        rows.into_iter()
            .map(|(part, sid)| {
                let (result, review) = match part {
                    5 => (
                        self.p5.result(&sid).ok().map(to_value).transpose()?,
                        to_value(self.p5.review(&sid, mistakes)?)?,
                    ),
                    6 => (
                        self.p6.result(&sid).ok().map(to_value).transpose()?,
                        to_value(self.p6.review(&sid, mistakes)?)?,
                    ),
                    7 => (
                        self.p7.result(&sid).ok().map(to_value).transpose()?,
                        to_value(self.p7.review(&sid, mistakes)?)?,
                    ),
                    _ => return Err("Invalid Full Reading child part.".into()),
                };
                Ok(AggregatePart {
                    part_number: part,
                    session_id: sid,
                    result,
                    review,
                })
            })
            .collect()
    }
}
fn to_value<T: Serialize>(value: T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|e| format!("Full Reading serialization error: {e}"))
}
fn err(e: rusqlite::Error) -> String {
    format!("Full Reading database error: {e}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        toeic_part5::{Part5Bank, Submit as Submit5},
        toeic_part6::{Part6Bank, Submit as Submit6},
        toeic_part7::{Part7Bank, Submit as Submit7},
    };
    fn setup() -> (FullReadingRepository, PathBuf) {
        let db =
            std::env::temp_dir().join(format!("toeic-full-reading-{}.db", uuid::Uuid::new_v4()));
        database::migrate(&db).unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/toeic/item-bank-v1");
        let p5 = Part5Repository::new(
            db.clone(),
            Part5Bank::load(root.join("part5.json")).unwrap(),
        );
        let p6 = Part6Repository::new(
            db.clone(),
            Part6Bank::load(root.join("part6.json")).unwrap(),
        );
        let p7 = Part7Repository::new(
            db.clone(),
            Part7Bank::load(root.join("part7.json")).unwrap(),
        );
        (FullReadingRepository::new(db.clone(), p5, p6, p7), db)
    }
    #[test]
    fn parent_completes_exactly_100_and_aggregates_history_review_and_resume() {
        let (r, db) = setup();
        let parent = r.start("simulation").unwrap();
        let p5id = parent.parts[0].session_id.clone();
        let p6id = parent.parts[1].session_id.clone();
        let p7id = parent.parts[2].session_id.clone();
        let mut s5 = r.p5.session(&p5id).unwrap();
        while s5.status == "in_progress" {
            let q = s5.current_question.as_ref().unwrap();
            s5 =
                r.p5.submit(Submit5 {
                    session_id: p5id.clone(),
                    item_id: q.item_id.clone(),
                    item_version: q.item_version,
                    selected_choice: "A".into(),
                })
                .unwrap()
        }
        let mut s6 = r.p6.session(&p6id).unwrap();
        while s6.status == "in_progress" {
            let set = s6.current_set.as_ref().unwrap();
            let q = &set.questions[set.active_question_index as usize];
            s6 =
                r.p6.submit(Submit6 {
                    session_id: p6id.clone(),
                    item_id: q.item_id.clone(),
                    item_version: q.item_version,
                    selected_choice: "B".into(),
                })
                .unwrap()
        }
        let halfway = r.session(&parent.session_id).unwrap();
        assert_eq!(halfway.answered_count, 46);
        assert!(halfway.estimate.is_none());
        let mut s7 = r.p7.session(&p7id).unwrap();
        while s7.status == "in_progress" {
            let set = s7.current_set.as_ref().unwrap();
            let q = &set.questions[set.active_question_index as usize];
            s7 =
                r.p7.submit(Submit7 {
                    session_id: p7id.clone(),
                    item_id: q.item_id.clone(),
                    item_version: q.item_version,
                    selected_choice: "C".into(),
                })
                .unwrap()
        }
        let done = r.session(&parent.session_id).unwrap();
        assert_eq!(done.answered_count, 100);
        assert_eq!(done.status, "completed");
        assert!(done.estimate.is_some());
        let h = r.history().unwrap();
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].answered_count, 100);
        let all = r.aggregate(&parent.session_id, false).unwrap();
        assert_eq!(all.len(), 3);
        let total: usize = all
            .iter()
            .map(|p| {
                p.review
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| {
                        x.get("feedback")
                            .and_then(|f| f.get("questions"))
                            .and_then(|q| q.as_array())
                            .map_or(1, |q| q.len())
                    })
                    .sum::<usize>()
            })
            .sum();
        assert_eq!(total, 100);
        let c = database::open(&db).unwrap();
        let answers: u32 = c
            .query_row("SELECT COUNT(*) FROM toeic_answer", [], |x| x.get(0))
            .unwrap();
        assert_eq!(answers, 100);
        drop(c);
        let _ = std::fs::remove_file(db);
    }
    #[test]
    fn families_b_and_c_freeze_matching_forms_and_resume_without_drift() {
        let (r, db) = setup();
        for family in ["B", "C"] {
            let parent = r.start_with_family("simulation", family).unwrap();
            assert_eq!(parent.family, family);
            assert_eq!(parent.total_questions, 100);
            let resumed = r.start_with_family("simulation", family).unwrap();
            assert_eq!(resumed.session_id, parent.session_id);
            assert_eq!(resumed.family, family);
            let c = database::open(&db).unwrap();
            let forms = c
                .prepare("SELECT form_id FROM toeic_full_reading_part WHERE full_session_id=?1 ORDER BY part_number")
                .unwrap()
                .query_map([&parent.session_id], |row| row.get::<_, String>(0))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let suffix = family.to_ascii_lowercase();
            assert_eq!(
                forms,
                vec![
                    format!("toeic-part5-form-{suffix}"),
                    format!("toeic-part6-form-{suffix}"),
                    format!("toeic-part7-form-{suffix}"),
                ]
            );
        }
        drop(r);
        let _ = std::fs::remove_file(db);
    }
}
