use crate::{
    database,
    toeic::ToeicRepository,
    toeic_listening_score::{self, ListeningEstimate},
    toeic_part2::Part2Repository,
    toeic_part3::Part3Repository,
    toeic_part4::Part4Repository,
};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Clone)]
pub struct FullListeningRepository {
    database: PathBuf,
    pub(crate) p1: ToeicRepository,
    pub(crate) p2: Part2Repository,
    pub(crate) p3: Part3Repository,
    pub(crate) p4: Part4Repository,
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
    pub estimate: Option<ListeningEstimate>,
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
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregatePart {
    pub part_number: u32,
    pub session_id: String,
    pub result: Option<Value>,
    pub review: Value,
}

impl FullListeningRepository {
    pub fn new(
        database: PathBuf,
        p1: ToeicRepository,
        p2: Part2Repository,
        p3: Part3Repository,
        p4: Part4Repository,
    ) -> Self {
        Self {
            database,
            p1,
            p2,
            p3,
            p4,
        }
    }
    pub fn start(&self, mode: &str) -> Result<FullSession, String> {
        self.start_with_family(mode, "A")
    }
    pub fn start_with_family(&self, mode: &str, family: &str) -> Result<FullSession, String> {
        if !matches!(mode, "simulation" | "learning") {
            return Err("Full Listening mode must be simulation or learning.".into());
        }
        let family = family.trim().to_ascii_uppercase();
        if !matches!(family.as_str(), "A" | "B" | "C") {
            return Err("Full Listening family must be A, B, or C.".into());
        }
        let suffix = family.to_ascii_lowercase();
        let c = database::open(&self.database)?;
        if let Some(id)=c.query_row("SELECT id FROM toeic_full_listening_session WHERE status='in_progress' AND mode=?1 AND family=?2 ORDER BY created_at DESC LIMIT 1",params![mode,family],|r|r.get::<_,String>(0)).optional().map_err(err)?{return self.session(&id)}
        let p1_form = format!("toeic-part1-form-{suffix}");
        let p2_form = format!("toeic-part2-form-{suffix}");
        let p3_form = format!("toeic-part3-form-{suffix}");
        let p4_form = format!("toeic-part4-form-{suffix}");
        let p1 = self.p1.start(&p1_form, 1)?;
        let p2 = self.p2.start(&p2_form, 1)?;
        let p3 = self.p3.start(&p3_form, 1)?;
        let p4 = self.p4.start(&p4_form, 1)?;
        let id = uuid::Uuid::new_v4().to_string();
        let composition = serde_json::json!({"family":family,"parts":[{"part":1,"form":p1_form,"version":1},{"part":2,"form":p2_form,"version":1},{"part":3,"form":p3_form,"version":1},{"part":4,"form":p4_form,"version":1}]});
        let tx = c.unchecked_transaction().map_err(err)?;
        tx.execute("INSERT INTO toeic_full_listening_session(id,family,mode,status,current_part,composition_json,created_at,updated_at) VALUES(?1,?2,?3,'in_progress',1,?4,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![id,family,mode,composition.to_string()]).map_err(err)?;
        for (part, sid, form) in [
            (1, p1.session_id, p1_form.as_str()),
            (2, p2.session_id, p2_form.as_str()),
            (3, p3.session_id, p3_form.as_str()),
            (4, p4.session_id, p4_form.as_str()),
        ] {
            tx.execute("INSERT INTO toeic_full_listening_part(full_session_id,part_number,toeic_session_id,form_id,form_version,status) VALUES(?1,?2,?3,?4,1,CASE WHEN ?2=1 THEN 'in_progress' ELSE 'pending' END)",params![id,part,sid,form]).map_err(err)?;
        }
        tx.commit().map_err(err)?;
        self.session(&id)
    }
    pub fn session(&self, id: &str) -> Result<FullSession, String> {
        let c = database::open(&self.database)?;
        let (family, mode, parent_status) = c
            .query_row(
                "SELECT family,mode,status FROM toeic_full_listening_session WHERE id=?1",
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
            .ok_or("Full Listening simulation not found.")?;
        let mut st=c.prepare("SELECT p.part_number,p.toeic_session_id,s.status FROM toeic_full_listening_part p JOIN toeic_session s ON s.id=p.toeic_session_id WHERE p.full_session_id=?1 ORDER BY p.part_number").map_err(err)?;
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
        if rows.len() != 4 {
            return Err("Full Listening composition is incomplete.".into());
        }
        let answered=c.query_row("SELECT COUNT(*) FROM toeic_answer a JOIN toeic_full_listening_part p ON p.toeic_session_id=a.session_id WHERE p.full_session_id=?1",[id],|r|r.get::<_,u32>(0)).map_err(err)?;
        let current = rows
            .iter()
            .find(|x| x.2 != "completed")
            .map(|x| x.0)
            .unwrap_or(4);
        let complete = rows.iter().all(|x| x.2 == "completed");
        let mut estimate = None;
        if complete {
            let raw=c.query_row("SELECT COALESCE(SUM(a.is_correct),0) FROM toeic_answer a JOIN toeic_full_listening_part p ON p.toeic_session_id=a.session_id WHERE p.full_session_id=?1",[id],|r|r.get::<_,u32>(0)).map_err(err)?;
            let e = toeic_listening_score::estimate(raw)?;
            c.execute("UPDATE toeic_full_listening_session SET status='completed',current_part=4,raw_correct=?2,estimated_score=?3,range_low=?4,range_high=?5,score_profile_id=?6,score_profile_version=?7,completed_at=COALESCE(completed_at,strftime('%Y-%m-%dT%H:%M:%fZ','now')),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![id,raw,e.estimated_score,e.range_low,e.range_high,e.profile_id,e.profile_version]).map_err(err)?;
            estimate = Some(e);
        } else {
            c.execute("UPDATE toeic_full_listening_session SET current_part=?2,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![id,current]).map_err(err)?;
        }
        let meta = [
            ("Photographs", 6, "/toeic/session/", "/toeic/results/"),
            (
                "Question–Response",
                25,
                "/toeic/part2/session/",
                "/toeic/part2/results/",
            ),
            (
                "Conversations",
                39,
                "/toeic/part3/session/",
                "/toeic/part3/results/",
            ),
            (
                "Talks",
                30,
                "/toeic/part4/session/",
                "/toeic/part4/results/",
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
                    format!("{}{}?fullListening={id}&mode={mode}", meta[i].2, sid)
                },
                session_id: sid,
                status,
            })
            .collect();
        Ok(FullSession {
            session_id: id.into(),
            family,
            mode,
            status: if complete {
                "completed".into()
            } else {
                parent_status
            },
            current_part: current,
            answered_count: answered,
            total_questions: 100,
            parts,
            estimate,
            disclaimer:
                "Unofficial practice estimate. This app is not affiliated with or endorsed by ETS."
                    .into(),
        })
    }
    pub fn history(&self) -> Result<Vec<FullHistory>, String> {
        let c = database::open(&self.database)?;
        let mut st=c.prepare("SELECT id,mode,status,raw_correct,estimated_score,range_low,range_high,created_at,completed_at FROM toeic_full_listening_session ORDER BY created_at DESC,id DESC").map_err(err)?;
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
                })
            })
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        Ok(rows)
    }
    pub fn aggregate(&self, id: &str, mistakes: bool) -> Result<Vec<AggregatePart>, String> {
        let c = database::open(&self.database)?;
        let mut st=c.prepare("SELECT part_number,toeic_session_id FROM toeic_full_listening_part WHERE full_session_id=?1 ORDER BY part_number").map_err(err)?;
        let rows = st
            .query_map([id], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?)))
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        if rows.len() != 4 {
            return Err("Full Listening composition is incomplete.".into());
        }
        rows.into_iter()
            .map(|(part, sid)| {
                let (result, review) = match part {
                    1 => (
                        self.p1.result(&sid).ok().map(to_value).transpose()?,
                        to_value(self.p1.review(&sid, mistakes)?)?,
                    ),
                    2 => (
                        self.p2.result(&sid).ok().map(to_value).transpose()?,
                        to_value(self.p2.review(&sid, mistakes)?)?,
                    ),
                    3 => (
                        self.p3.result(&sid).ok().map(to_value).transpose()?,
                        to_value(self.p3.review(&sid, mistakes)?)?,
                    ),
                    4 => (
                        self.p4.result(&sid).ok().map(to_value).transpose()?,
                        to_value(self.p4.review(&sid, mistakes)?)?,
                    ),
                    _ => return Err("Invalid Full Listening child part.".into()),
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
    serde_json::to_value(value).map_err(|e| format!("Full Listening serialization error: {e}"))
}
fn err(e: rusqlite::Error) -> String {
    format!("Full Listening database error: {e}")
}
