use crate::{
    database, toeic_full_listening::FullListeningRepository,
    toeic_full_reading::FullReadingRepository,
};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;
#[derive(Clone)]
pub struct FullLrRepository {
    database: PathBuf,
    listening: FullListeningRepository,
    reading: FullReadingRepository,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullLrSession {
    pub session_id: String,
    pub family: String,
    pub status: String,
    pub current_section: String,
    pub listening_session_id: String,
    pub reading_session_id: String,
    pub listening_raw: Option<u32>,
    pub reading_raw: Option<u32>,
    pub total_raw: Option<u32>,
    pub listening_estimate: Option<u32>,
    pub reading_estimate: Option<u32>,
    pub total_estimate: Option<u32>,
    pub range_low: Option<u32>,
    pub range_high: Option<u32>,
    pub listening_route: String,
    pub reading_route: String,
    pub disclaimer: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullLrHistory {
    pub session_id: String,
    pub family: String,
    pub status: String,
    pub listening_raw: Option<u32>,
    pub reading_raw: Option<u32>,
    pub total_raw: Option<u32>,
    pub listening_estimate: Option<u32>,
    pub reading_estimate: Option<u32>,
    pub total_estimate: Option<u32>,
    pub range_low: Option<u32>,
    pub range_high: Option<u32>,
    pub listening_profile_version: Option<u32>,
    pub reading_profile_version: Option<u32>,
    pub created_at: String,
    pub completed_at: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullLrAggregate {
    pub session_id: String,
    pub listening: Value,
    pub reading: Value,
}
impl FullLrRepository {
    pub fn new(
        database: PathBuf,
        listening: FullListeningRepository,
        reading: FullReadingRepository,
    ) -> Self {
        Self {
            database,
            listening,
            reading,
        }
    }
    pub fn start(&self) -> Result<FullLrSession, String> {
        self.start_with_family("A")
    }
    pub fn start_with_family(&self, family: &str) -> Result<FullLrSession, String> {
        let family = family.trim().to_ascii_uppercase();
        if !matches!(family.as_str(), "A" | "B" | "C") {
            return Err("Full TOEIC L&R family must be A, B, or C.".into());
        }
        let c = database::open(&self.database)?;
        if let Some(id)=c.query_row("SELECT id FROM toeic_full_lr_session WHERE status='in_progress' AND family=?1 ORDER BY created_at DESC LIMIT 1",[&family],|r|r.get::<_,String>(0)).optional().map_err(err)?{return self.session(&id)}
        let l = self.listening.start_with_family("simulation", &family)?;
        let r = self.reading.start_with_family("simulation", &family)?;
        let id = uuid::Uuid::new_v4().to_string();
        c.execute("INSERT INTO toeic_full_lr_session(id,family,status,listening_session_id,reading_session_id,created_at,updated_at) VALUES(?1,?2,'in_progress',?3,?4,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![id,family,l.session_id,r.session_id]).map_err(err)?;
        self.session(&id)
    }
    pub fn session(&self, id: &str) -> Result<FullLrSession, String> {
        let c = database::open(&self.database)?;
        let(family,lid,rid)=c.query_row("SELECT family,listening_session_id,reading_session_id FROM toeic_full_lr_session WHERE id=?1",[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).optional().map_err(err)?.ok_or("Full TOEIC L&R simulation not found.")?;
        let l = self.listening.session(&lid)?;
        let r = self.reading.session(&rid)?;
        let done = l.status == "completed" && r.status == "completed";
        let (le, re, lr, rr) = (
            l.estimate.as_ref().map(|x| x.estimated_score),
            r.estimate.as_ref().map(|x| x.estimated_score),
            l.estimate.as_ref().map(|x| (x.range_low, x.range_high)),
            r.estimate.as_ref().map(|x| (x.range_low, x.range_high)),
        );
        let (raw_l, raw_r) = (
            l.estimate.as_ref().map(|x| x.raw_correct),
            r.estimate.as_ref().map(|x| x.raw_correct),
        );
        if done {
            c.execute("UPDATE toeic_full_lr_session SET status='completed',listening_raw=?2,reading_raw=?3,total_raw=?4,listening_estimate=?5,reading_estimate=?6,total_estimate=?7,range_low=?8,range_high=?9,listening_profile_version=1,reading_profile_version=1,completed_at=COALESCE(completed_at,strftime('%Y-%m-%dT%H:%M:%fZ','now')),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![id,raw_l,raw_r,raw_l.zip(raw_r).map(|x|x.0+x.1),le,re,le.zip(re).map(|x|x.0+x.1),lr.zip(rr).map(|x|x.0.0+x.1.0),lr.zip(rr).map(|x|x.0.1+x.1.1)]).map_err(err)?;
        }
        Ok(FullLrSession{session_id:id.into(),family,status:if done{"completed".into()}else{"in_progress".into()},current_section:if l.status!="completed"{"listening".into()}else if r.status!="completed"{"reading".into()}else{"complete".into()},listening_session_id:lid.clone(),reading_session_id:rid.clone(),listening_raw:raw_l,reading_raw:raw_r,total_raw:raw_l.zip(raw_r).map(|x|x.0+x.1),listening_estimate:le,reading_estimate:re,total_estimate:le.zip(re).map(|x|x.0+x.1),range_low:lr.zip(rr).map(|x|x.0.0+x.1.0),range_high:lr.zip(rr).map(|x|x.0.1+x.1.1),listening_route:format!("/toeic/listening/{lid}"),reading_route:format!("/toeic/reading/{rid}"),disclaimer:"Unofficial untimed practice estimate. The total is the Listening and Reading central estimates added together; official ETS scores may differ.".into()})
    }
    pub fn history(&self) -> Result<Vec<FullLrHistory>, String> {
        let c = database::open(&self.database)?;
        let mut st=c.prepare("SELECT id,family,status,listening_raw,reading_raw,total_raw,listening_estimate,reading_estimate,total_estimate,range_low,range_high,listening_profile_version,reading_profile_version,created_at,completed_at FROM toeic_full_lr_session ORDER BY created_at DESC,id DESC").map_err(err)?;
        let rows = st
            .query_map([], |r| {
                Ok(FullLrHistory {
                    session_id: r.get(0)?,
                    family: r.get(1)?,
                    status: r.get(2)?,
                    listening_raw: r.get(3)?,
                    reading_raw: r.get(4)?,
                    total_raw: r.get(5)?,
                    listening_estimate: r.get(6)?,
                    reading_estimate: r.get(7)?,
                    total_estimate: r.get(8)?,
                    range_low: r.get(9)?,
                    range_high: r.get(10)?,
                    listening_profile_version: r.get(11)?,
                    reading_profile_version: r.get(12)?,
                    created_at: r.get(13)?,
                    completed_at: r.get(14)?,
                })
            })
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        Ok(rows)
    }
    pub fn aggregate(&self, id: &str, mistakes: bool) -> Result<FullLrAggregate, String> {
        let c = database::open(&self.database)?;
        let(lid,rid)=c.query_row("SELECT listening_session_id,reading_session_id FROM toeic_full_lr_session WHERE id=?1",[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).optional().map_err(err)?.ok_or("Full TOEIC L&R simulation not found.")?;
        Ok(FullLrAggregate {
            session_id: id.into(),
            listening: serde_json::to_value(self.listening.aggregate(&lid, mistakes)?)
                .map_err(json_err)?,
            reading: serde_json::to_value(self.reading.aggregate(&rid, mistakes)?)
                .map_err(json_err)?,
        })
    }
}
fn json_err(e: serde_json::Error) -> String {
    format!("Full TOEIC L&R serialization error: {e}")
}
fn err(e: rusqlite::Error) -> String {
    format!("Full TOEIC L&R database error: {e}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        toeic::{ToeicRepository, ToeicSubmitAnswerRequest},
        toeic_item_bank::ToeicItemBank,
        toeic_part2::{Part2Bank, Part2Repository, Submit as Submit2},
        toeic_part3::{Part3Bank, Part3Repository, Submit as Submit3},
        toeic_part4::{Part4Bank, Part4Repository, Submit as Submit4},
        toeic_part5::{Part5Bank, Part5Repository, Submit as Submit5},
        toeic_part6::{Part6Bank, Part6Repository, Submit as Submit6},
        toeic_part7::{Part7Bank, Part7Repository, Submit as Submit7},
    };
    fn setup() -> (FullLrRepository, PathBuf) {
        let dir = std::env::temp_dir().join(format!("toeic-full-lr-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = dir.join("test.sqlite3");
        database::migrate(&db).unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/toeic/item-bank-v1");
        let p1 =
            ToeicRepository::new(db.clone(), ToeicItemBank::load(root.clone()).unwrap()).unwrap();
        let p2 = Part2Repository::new(
            db.clone(),
            Part2Bank::load(root.join("part2.json")).unwrap(),
        );
        let p3 = Part3Repository::new(
            db.clone(),
            Part3Bank::load(root.join("part3.json")).unwrap(),
        );
        let p4 = Part4Repository::new(
            db.clone(),
            Part4Bank::load(root.join("part4.json")).unwrap(),
        );
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
        let listening = FullListeningRepository::new(db.clone(), p1, p2, p3, p4);
        let reading = FullReadingRepository::new(db.clone(), p5, p6, p7);
        (FullLrRepository::new(db, listening, reading), dir)
    }
    fn complete_listening(r: &FullLrRepository, id: &str) {
        let full = r.listening.session(id).unwrap();
        let ids = full
            .parts
            .iter()
            .map(|p| p.session_id.clone())
            .collect::<Vec<_>>();
        let mut s = r.listening.p1.session(&ids[0]).unwrap();
        while s.status == "in_progress" {
            let q = s.current_question.as_ref().unwrap();
            let a = r.listening.p1.begin_audio(&s.session_id).unwrap();
            r.listening
                .p1
                .complete_audio(
                    &s.session_id,
                    &q.item_id,
                    q.item_version,
                    a.presentation_id.as_deref(),
                )
                .unwrap();
            s = r
                .listening
                .p1
                .submit(ToeicSubmitAnswerRequest {
                    session_id: s.session_id.clone(),
                    item_id: q.item_id.clone(),
                    item_version: q.item_version,
                    selected_choice: "A".into(),
                })
                .unwrap();
            if s.status == "in_progress" {
                s = r.listening.p1.advance(&s.session_id).unwrap()
            }
        }
        let mut s = r.listening.p2.session(&ids[1]).unwrap();
        while s.status == "in_progress" {
            let q = s.current_question.as_ref().unwrap();
            let a = r.listening.p2.begin_audio(&s.session_id).unwrap();
            r.listening
                .p2
                .complete_audio(&s.session_id, &q.item_id, a.presentation_id.as_deref())
                .unwrap();
            s = r
                .listening
                .p2
                .submit(Submit2 {
                    session_id: s.session_id.clone(),
                    item_id: q.item_id.clone(),
                    item_version: q.item_version,
                    selected_choice: "B".into(),
                })
                .unwrap();
            if s.status == "in_progress" {
                s = r.listening.p2.advance(&s.session_id).unwrap()
            }
        }
        let mut s = r.listening.p3.session(&ids[2]).unwrap();
        while s.status == "in_progress" {
            let a = r.listening.p3.begin_audio(&s.session_id, 0, None).unwrap();
            r.listening
                .p3
                .finish_audio(&s.session_id, &a.set_id, a.presentation_id.as_deref(), true)
                .unwrap();
            let qs = s
                .current_set
                .as_ref()
                .unwrap()
                .questions
                .iter()
                .map(|q| q.question_id.clone())
                .collect::<Vec<_>>();
            for question_id in qs {
                s = r
                    .listening
                    .p3
                    .submit(Submit3 {
                        session_id: s.session_id.clone(),
                        question_id,
                        selected_choice: "C".into(),
                    })
                    .unwrap()
            }
            if s.status == "in_progress" {
                s = r.listening.p3.advance(&s.session_id).unwrap()
            }
        }
        let mut s = r.listening.p4.session(&ids[3]).unwrap();
        while s.status == "in_progress" {
            let a = r.listening.p4.begin_audio(&s.session_id, 0, None).unwrap();
            r.listening
                .p4
                .finish_audio(&s.session_id, &a.set_id, a.presentation_id.as_deref(), true)
                .unwrap();
            let qs = s
                .current_set
                .as_ref()
                .unwrap()
                .questions
                .iter()
                .map(|q| q.question_id.clone())
                .collect::<Vec<_>>();
            for question_id in qs {
                s = r
                    .listening
                    .p4
                    .submit(Submit4 {
                        session_id: s.session_id.clone(),
                        question_id,
                        selected_choice: "D".into(),
                    })
                    .unwrap()
            }
            if s.status == "in_progress" {
                s = r.listening.p4.advance(&s.session_id).unwrap()
            }
        }
        let done = r.listening.session(id).unwrap();
        assert_eq!(done.answered_count, 100);
        assert!(done.estimate.is_some());
    }
    fn complete_reading(r: &FullLrRepository, id: &str, stop_at_99: bool) {
        let full = r.reading.session(id).unwrap();
        let ids = full
            .parts
            .iter()
            .map(|p| p.session_id.clone())
            .collect::<Vec<_>>();
        let mut s = r.reading.p5.session(&ids[0]).unwrap();
        while s.status == "in_progress" {
            let q = s.current_question.as_ref().unwrap();
            s = r
                .reading
                .p5
                .submit(Submit5 {
                    session_id: s.session_id.clone(),
                    item_id: q.item_id.clone(),
                    item_version: q.item_version,
                    selected_choice: "A".into(),
                })
                .unwrap()
        }
        let mut s = r.reading.p6.session(&ids[1]).unwrap();
        while s.status == "in_progress" {
            let set = s.current_set.as_ref().unwrap();
            let q = &set.questions[set.active_question_index as usize];
            s = r
                .reading
                .p6
                .submit(Submit6 {
                    session_id: s.session_id.clone(),
                    item_id: q.item_id.clone(),
                    item_version: q.item_version,
                    selected_choice: "B".into(),
                })
                .unwrap()
        }
        let mut s = r.reading.p7.session(&ids[2]).unwrap();
        while s.status == "in_progress" {
            if stop_at_99 && s.answered_count == 53 {
                break;
            }
            let set = s.current_set.as_ref().unwrap();
            let q = &set.questions[set.active_question_index as usize];
            s = r
                .reading
                .p7
                .submit(Submit7 {
                    session_id: s.session_id.clone(),
                    item_id: q.item_id.clone(),
                    item_version: q.item_version,
                    selected_choice: "C".into(),
                })
                .unwrap()
        }
    }
    fn finish_last_reading(r: &FullLrRepository, id: &str) {
        let full = r.reading.session(id).unwrap();
        let sid = &full.parts[2].session_id;
        let s = r.reading.p7.session(sid).unwrap();
        let set = s.current_set.as_ref().unwrap();
        let q = &set.questions[set.active_question_index as usize];
        r.reading
            .p7
            .submit(Submit7 {
                session_id: sid.clone(),
                item_id: q.item_id.clone(),
                item_version: q.item_version,
                selected_choice: "C".into(),
            })
            .unwrap();
        r.reading.session(id).unwrap();
    }
    fn review_count(v: &Value) -> usize {
        v.as_array()
            .map(|a| {
                a.iter()
                    .map(|x| {
                        x.get("feedback")
                            .and_then(|f| f.get("questions"))
                            .and_then(Value::as_array)
                            .map_or(1, Vec::len)
                    })
                    .sum()
            })
            .unwrap_or(0)
    }
    fn part_scores(parts: &Value) -> Vec<(u64, u64, u64)> {
        parts
            .as_array()
            .unwrap()
            .iter()
            .map(|part| {
                (
                    part["partNumber"].as_u64().unwrap(),
                    part["result"]["correct"].as_u64().unwrap(),
                    part["result"]["total"].as_u64().unwrap(),
                )
            })
            .collect()
    }
    #[test]
    fn parent_completes_200_has_exact_ownership_history_review_and_score_gate() {
        let (r, dir) = setup();
        let p = r.start().unwrap();
        complete_listening(&r, &p.listening_session_id);
        complete_reading(&r, &p.reading_session_id, true);
        let at199 = r.session(&p.session_id).unwrap();
        assert_eq!(at199.status, "in_progress");
        assert!(at199.total_estimate.is_none());
        finish_last_reading(&r, &p.reading_session_id);
        let done = r.session(&p.session_id).unwrap();
        assert_eq!(done.status, "completed");
        assert!(done.total_estimate.is_some());
        assert_eq!(
            done.total_estimate,
            done.listening_estimate
                .zip(done.reading_estimate)
                .map(|x| x.0 + x.1)
        );
        let h = r.history().unwrap();
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].total_raw, done.total_raw);
        let all = r.aggregate(&p.session_id, false).unwrap();
        assert_eq!(
            part_scores(&all.listening),
            vec![(1, 1, 6), (2, 8, 25), (3, 12, 39), (4, 7, 30)]
        );
        assert_eq!(
            part_scores(&all.reading),
            vec![(5, 8, 30), (6, 4, 16), (7, 13, 54)]
        );
        assert_eq!(done.listening_raw, Some(28));
        assert_eq!(done.reading_raw, Some(25));
        assert_eq!(done.total_raw, Some(53));
        let count = all
            .listening
            .as_array()
            .unwrap()
            .iter()
            .map(|x| review_count(&x["review"]))
            .sum::<usize>()
            + all
                .reading
                .as_array()
                .unwrap()
                .iter()
                .map(|x| review_count(&x["review"]))
                .sum::<usize>();
        assert_eq!(count, 200);
        let mistakes = r.aggregate(&p.session_id, true).unwrap();
        let mistake_count = mistakes
            .listening
            .as_array()
            .unwrap()
            .iter()
            .map(|x| review_count(&x["review"]))
            .sum::<usize>()
            + mistakes
                .reading
                .as_array()
                .unwrap()
                .iter()
                .map(|x| review_count(&x["review"]))
                .sum::<usize>();
        assert_eq!(mistake_count, 147);
        let c = database::open(&dir.join("test.sqlite3")).unwrap();
        assert_eq!(
            c.query_row("SELECT COUNT(*) FROM toeic_answer", [], |x| x
                .get::<_, u32>(0))
                .unwrap(),
            200
        );
        drop(c);
        let _ = std::fs::remove_dir_all(dir);
    }
    #[test]
    fn two_parents_keep_exact_child_ownership_in_history_and_aggregate() {
        let (r, dir) = setup();
        let a = r.start().unwrap();
        complete_listening(&r, &a.listening_session_id);
        complete_reading(&r, &a.reading_session_id, false);
        let a = r.session(&a.session_id).unwrap();
        let b = r.start().unwrap();
        complete_listening(&r, &b.listening_session_id);
        complete_reading(&r, &b.reading_session_id, false);
        let b = r.session(&b.session_id).unwrap();
        assert_ne!(a.session_id, b.session_id);
        assert_ne!(a.listening_session_id, b.listening_session_id);
        assert_ne!(a.reading_session_id, b.reading_session_id);
        let aa = r.aggregate(&a.session_id, false).unwrap();
        let bb = r.aggregate(&b.session_id, false).unwrap();
        let a_children = aa
            .listening
            .as_array()
            .unwrap()
            .iter()
            .chain(aa.reading.as_array().unwrap())
            .map(|x| x["sessionId"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        let b_children = bb
            .listening
            .as_array()
            .unwrap()
            .iter()
            .chain(bb.reading.as_array().unwrap())
            .map(|x| x["sessionId"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert!(a_children.is_disjoint(&b_children));
        assert_eq!(a_children.len(), 7);
        assert_eq!(b_children.len(), 7);
        assert_eq!(r.history().unwrap().len(), 2);
        let c = database::open(&dir.join("test.sqlite3")).unwrap();
        let pairs=c.query_row("SELECT COUNT(DISTINCT listening_session_id || ':' || reading_session_id) FROM toeic_full_lr_session",[],|x|x.get::<_,u32>(0)).unwrap();
        assert_eq!(pairs, 2);
        drop(c);
        let _ = std::fs::remove_dir_all(dir);
    }
    #[test]
    fn full_lr_families_b_and_c_own_matching_listening_and_reading_snapshots() {
        let (r, dir) = setup();
        let db = dir.join("test.sqlite3");
        for family in ["B", "C"] {
            let parent = r.start_with_family(family).unwrap();
            assert_eq!(parent.family, family);
            let resumed = r.start_with_family(family).unwrap();
            assert_eq!(resumed.session_id, parent.session_id);
            let c = database::open(&db).unwrap();
            let listening_family: String = c
                .query_row(
                    "SELECT family FROM toeic_full_listening_session WHERE id=?1",
                    [&parent.listening_session_id],
                    |row| row.get(0),
                )
                .unwrap();
            let reading_family: String = c
                .query_row(
                    "SELECT family FROM toeic_full_reading_session WHERE id=?1",
                    [&parent.reading_session_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(listening_family, family);
            assert_eq!(reading_family, family);
            let suffix = family.to_ascii_lowercase();
            let listening_forms = c
                .prepare("SELECT form_id FROM toeic_full_listening_part WHERE full_session_id=?1 ORDER BY part_number")
                .unwrap()
                .query_map([&parent.listening_session_id], |row| row.get::<_, String>(0))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let reading_forms = c
                .prepare("SELECT form_id FROM toeic_full_reading_part WHERE full_session_id=?1 ORDER BY part_number")
                .unwrap()
                .query_map([&parent.reading_session_id], |row| row.get::<_, String>(0))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert_eq!(
                listening_forms,
                (1..=4)
                    .map(|part| format!("toeic-part{part}-form-{suffix}"))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                reading_forms,
                (5..=7)
                    .map(|part| format!("toeic-part{part}-form-{suffix}"))
                    .collect::<Vec<_>>()
            );
        }
        drop(r);
        let _ = std::fs::remove_dir_all(dir);
    }
}
