use crate::{
    database,
    toeic_part5::Part5Repository,
    toeic_part6::Part6Repository,
    toeic_part7::Part7Repository,
    toeic_reading_score::{self, ReadingEstimate},
};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use std::path::PathBuf;
#[derive(Clone)]
pub struct FullReadingRepository {
    database: PathBuf,
    p5: Part5Repository,
    p6: Part6Repository,
    p7: Part7Repository,
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
        if !matches!(mode, "simulation" | "learning") {
            return Err("Full Reading mode must be simulation or learning.".into());
        }
        let c = database::open(&self.database)?;
        if let Some(id)=c.query_row("SELECT id FROM toeic_full_reading_session WHERE status='in_progress' AND mode=?1 ORDER BY created_at DESC LIMIT 1",[mode],|r|r.get::<_,String>(0)).optional().map_err(err)?{return self.session(&id)}
        let p5 = self.p5.start("toeic-part5-form-a", 1, mode)?;
        let p6 = self.p6.start("toeic-part6-form-a", 1, mode)?;
        let p7 = self.p7.start("toeic-part7-form-a", 1, mode)?;
        if [&p5.mode, &p6.mode, &p7.mode]
            .iter()
            .any(|x| x.as_str() != mode)
        {
            return Err("Finish the active standalone Reading practice before starting a differently configured Full Reading simulation.".into());
        }
        let id = uuid::Uuid::new_v4().to_string();
        let composition = serde_json::json!({"family":"A","parts":[{"part":5,"form":"toeic-part5-form-a","version":1},{"part":6,"form":"toeic-part6-form-a","version":1},{"part":7,"form":"toeic-part7-form-a","version":1}]});
        let tx = c.unchecked_transaction().map_err(err)?;
        tx.execute("INSERT INTO toeic_full_reading_session(id,family,mode,status,current_part,composition_json,created_at,updated_at) VALUES(?1,'A',?2,'in_progress',5,?3,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![id,mode,composition.to_string()]).map_err(err)?;
        for (part, sid, form) in [
            (5, p5.session_id, "toeic-part5-form-a"),
            (6, p6.session_id, "toeic-part6-form-a"),
            (7, p7.session_id, "toeic-part7-form-a"),
        ] {
            tx.execute("INSERT INTO toeic_full_reading_part(full_session_id,part_number,toeic_session_id,form_id,form_version,status) VALUES(?1,?2,?3,?4,1,CASE WHEN ?2=5 THEN 'in_progress' ELSE 'pending' END)",params![id,part,sid,form]).map_err(err)?;
        }
        tx.commit().map_err(err)?;
        self.session(&id)
    }
    pub fn session(&self, id: &str) -> Result<FullSession, String> {
        let c = database::open(&self.database)?;
        let (mode, parent) = c
            .query_row(
                "SELECT mode,status FROM toeic_full_reading_session WHERE id=?1",
                [id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
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
        Ok(FullSession{session_id:id.into(),mode,status:if complete{"completed".into()}else{parent},current_part:current,answered_count:answered,total_questions:100,parts,estimate,disclaimer:"Untimed unofficial TOEIC-style Reading simulation. Official scores use ETS equating and may differ.".into()})
    }
    pub fn history(&self) -> Result<Vec<FullHistory>, String> {
        let c = database::open(&self.database)?;
        let mut st=c.prepare("SELECT id,mode,status,raw_correct,estimated_score,range_low,range_high,created_at,completed_at FROM toeic_full_reading_session ORDER BY created_at DESC,id DESC").map_err(err)?;
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
}
fn err(e: rusqlite::Error) -> String {
    format!("Full Reading database error: {e}")
}
