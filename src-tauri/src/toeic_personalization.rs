use crate::{
    database, toeic::ToeicRepository, toeic_part2::Part2Repository, toeic_part3::Part3Repository,
    toeic_part4::Part4Repository, toeic_part5::Part5Repository, toeic_part6::Part6Repository,
    toeic_part7::Part7Repository,
};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

const TARGET_KEY: &str = "toeic_target_score";
const BANK_ITEM_COUNT: u32 = 600;

#[derive(Clone)]
pub struct ToeicPersonalizationRepository {
    database: PathBuf,
    p1: ToeicRepository,
    p2: Part2Repository,
    p3: Part3Repository,
    p4: Part4Repository,
    p5: Part5Repository,
    p6: Part6Repository,
    p7: Part7Repository,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeaknessDto {
    pub part_number: u32,
    pub skill: String,
    pub correct: u32,
    pub total: u32,
    pub accuracy: u32,
    pub label: String,
    pub sufficient_sample: bool,
    pub last_seen_at: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExposureDto {
    pub unique_seen: u32,
    pub bank_items: u32,
    pub total_answers: u32,
    pub unseen: u32,
    pub repeated_items: u32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendPointDto {
    pub session_id: String,
    pub family: String,
    pub completed_at: String,
    pub listening_raw: Option<u32>,
    pub reading_raw: Option<u32>,
    pub total_raw: Option<u32>,
    pub listening_estimate: Option<u32>,
    pub reading_estimate: Option<u32>,
    pub total_estimate: Option<u32>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriorityDto {
    pub rank: u32,
    pub part_number: u32,
    pub skill: String,
    pub reason: String,
    pub route: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationDto {
    pub title: String,
    pub description: String,
    pub route: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardDto {
    pub target_score: u32,
    pub latest_listening_estimate: Option<u32>,
    pub latest_reading_estimate: Option<u32>,
    pub latest_total_estimate: Option<u32>,
    pub latest_range_low: Option<u32>,
    pub latest_range_high: Option<u32>,
    pub estimated_gap: Option<i32>,
    pub weaknesses: Vec<WeaknessDto>,
    pub exposure: ExposureDto,
    pub trends: Vec<TrendPointDto>,
    pub priorities: Vec<PriorityDto>,
    pub recommendations: Vec<RecommendationDto>,
    pub active_practice: Option<PracticeSessionDto>,
    pub recent_practice: Vec<PracticeSessionDto>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartPracticeRequest {
    pub kind: String,
    pub question_count: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeStepDto {
    pub step_number: u32,
    pub part_number: u32,
    pub form_id: String,
    pub session_id: String,
    pub quota: u32,
    pub answered: u32,
    pub correct: u32,
    pub status: String,
    pub route: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeSessionDto {
    pub session_id: String,
    pub kind: String,
    pub requested_count: u32,
    pub answered_count: u32,
    pub correct_count: Option<u32>,
    pub accuracy: Option<u32>,
    pub status: String,
    pub focus: Vec<String>,
    pub steps: Vec<PracticeStepDto>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Default)]
struct SkillStats {
    outcomes: Vec<(bool, String)>,
}

impl ToeicPersonalizationRepository {
    pub fn new(
        database: PathBuf,
        p1: ToeicRepository,
        p2: Part2Repository,
        p3: Part3Repository,
        p4: Part4Repository,
        p5: Part5Repository,
        p6: Part6Repository,
        p7: Part7Repository,
    ) -> Self {
        Self {
            database,
            p1,
            p2,
            p3,
            p4,
            p5,
            p6,
            p7,
        }
    }

    pub fn target(&self) -> Result<u32, String> {
        let c = database::open(&self.database)?;
        let raw: Option<String> = c
            .query_row(
                "SELECT value_json FROM settings WHERE key=?1",
                [TARGET_KEY],
                |r| r.get(0),
            )
            .optional()
            .map_err(err)?;
        let target = raw
            .and_then(|x| serde_json::from_str::<u32>(&x).ok())
            .unwrap_or(750);
        validate_target(target)?;
        Ok(target)
    }

    pub fn set_target(&self, target: u32) -> Result<DashboardDto, String> {
        validate_target(target)?;
        let c = database::open(&self.database)?;
        c.execute("INSERT INTO settings(key,value_json,updated_at) VALUES(?1,?2,strftime('%Y-%m-%dT%H:%M:%fZ','now')) ON CONFLICT(key) DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at", params![TARGET_KEY, target.to_string()]).map_err(err)?;
        self.dashboard()
    }

    pub fn dashboard(&self) -> Result<DashboardDto, String> {
        let target = self.target()?;
        let weaknesses = self.weaknesses()?;
        let exposure = self.exposure()?;
        let trends = self.trends()?;
        let latest = trends.last();
        let latest_total = latest.and_then(|x| x.total_estimate);
        let (latest_listening, latest_reading, range_low, range_high) = if let Some(point) = latest
        {
            let (low, high) = self.latest_full_lr_range()?;
            (point.listening_estimate, point.reading_estimate, low, high)
        } else {
            self.latest_section_estimates()?
        };
        let priorities = build_priorities(&weaknesses);
        let recommendations = build_recommendations(latest_total, &priorities);
        let active_practice = self.active_practice()?;
        Ok(DashboardDto {
            target_score: target,
            latest_listening_estimate: latest_listening,
            latest_reading_estimate: latest_reading,
            latest_total_estimate: latest_total,
            latest_range_low: range_low,
            latest_range_high: range_high,
            estimated_gap: latest_total.map(|score| target as i32 - score as i32),
            weaknesses,
            exposure,
            trends,
            priorities,
            recommendations,
            active_practice,
            recent_practice: self
                .practice_history()?
                .into_iter()
                .filter(|x| x.status == "completed")
                .take(5)
                .collect(),
        })
    }

    pub fn start_practice(
        &self,
        request: StartPracticeRequest,
    ) -> Result<PracticeSessionDto, String> {
        let kind = request.kind.as_str();
        if !matches!(kind, "smart" | "recent_mistakes" | "daily") {
            return Err("TOEIC practice kind must be smart, recent_mistakes, or daily.".into());
        }
        let requested = if kind == "daily" {
            12
        } else {
            request.question_count.unwrap_or(15)
        };
        if (kind == "daily" && requested != 12)
            || (kind != "daily" && !matches!(requested, 10 | 15 | 20))
        {
            return Err(
                "Smart practice supports 10, 15, or 20 questions; Daily Practice uses 12.".into(),
            );
        }
        let c = database::open(&self.database)?;
        if let Some(id) = c.query_row("SELECT id FROM toeic_personalized_practice_session WHERE kind=?1 AND status='in_progress' ORDER BY created_at DESC LIMIT 1", [kind], |r| r.get::<_, String>(0)).optional().map_err(err)? {
            return self.practice(&id);
        }
        let weaknesses = self.weaknesses()?;
        let focus = practice_focus(kind, &weaknesses, &c)?;
        let plan = if kind == "daily" {
            vec![
                (choose_part(&weaknesses, true), 5),
                (choose_part(&weaknesses, false), 7),
            ]
        } else {
            vec![(focus.first().map(|x| x.0).unwrap_or(5), requested)]
        };
        let id = uuid::Uuid::new_v4().to_string();
        c.execute("INSERT INTO toeic_personalized_practice_session(id,kind,requested_count,status,focus_json,created_at,updated_at) VALUES(?1,?2,?3,'in_progress',?4,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'))", params![id,kind,requested,serde_json::to_string(&focus.iter().map(|x| x.1.clone()).collect::<Vec<_>>()).unwrap()]).map_err(err)?;
        drop(c);
        for (i, (part, quota)) in plan.into_iter().enumerate() {
            if let Err(error) = self.create_step(&id, i as u32 + 1, part, quota) {
                let c = database::open(&self.database)?;
                let _ = c.execute(
                    "DELETE FROM toeic_personalized_practice_session WHERE id=?1",
                    [&id],
                );
                return Err(error);
            }
        }
        self.practice(&id)
    }

    pub fn practice(&self, id: &str) -> Result<PracticeSessionDto, String> {
        self.refresh(id)?;
        let c = database::open(&self.database)?;
        let (kind, requested, status, focus_json, answered, correct, created, completed) = c.query_row("SELECT kind,requested_count,status,focus_json,answered,correct,created_at,completed_at FROM toeic_personalized_practice_session WHERE id=?1", [id], |r| Ok((r.get::<_,String>(0)?,r.get::<_,u32>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,u32>(4)?,r.get::<_,Option<u32>>(5)?,r.get::<_,String>(6)?,r.get::<_,Option<String>>(7)?))).optional().map_err(err)?.ok_or("TOEIC personalized practice session not found.")?;
        let focus: Vec<String> = serde_json::from_str(&focus_json)
            .map_err(|_| "Invalid TOEIC practice focus snapshot.")?;
        let steps = self.steps(&c, id)?;
        Ok(PracticeSessionDto {
            session_id: id.into(),
            kind,
            requested_count: requested,
            answered_count: answered,
            correct_count: correct,
            accuracy: correct.map(|x| if answered == 0 { 0 } else { x * 100 / answered }),
            status,
            focus,
            steps,
            created_at: created,
            completed_at: completed,
        })
    }

    pub fn practice_history(&self) -> Result<Vec<PracticeSessionDto>, String> {
        let c = database::open(&self.database)?;
        let mut st = c.prepare("SELECT id FROM toeic_personalized_practice_session ORDER BY created_at DESC,id DESC LIMIT 50").map_err(err)?;
        let ids = st
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        drop(st);
        drop(c);
        ids.iter().map(|id| self.practice(id)).collect()
    }

    fn create_step(
        &self,
        practice_id: &str,
        step_number: u32,
        part: u32,
        quota: u32,
    ) -> Result<(), String> {
        let form_id = self.fresh_form(part)?;
        let (session_id, baseline) = self.start_child(part, &form_id)?;
        let c = database::open(&self.database)?;
        let snapshot: String = c
            .query_row(
                "SELECT form_snapshot_json FROM toeic_session WHERE id=?1",
                [&session_id],
                |r| r.get(0),
            )
            .map_err(err)?;
        let value: Value =
            serde_json::from_str(&snapshot).map_err(|_| "Invalid TOEIC child snapshot.")?;
        let mut ordered = Vec::new();
        collect_item_ids(&value, &mut ordered);
        let answered_ids = answer_ids(&c, &session_id)?;
        ordered.retain(|x| !answered_ids.contains(x));
        ordered.dedup();
        if ordered.len() < quota as usize {
            return Err(format!(
                "Part {part} does not have enough fresh questions for this practice size."
            ));
        }
        ordered.truncate(quota as usize);
        c.execute("INSERT INTO toeic_personalized_practice_step(practice_session_id,step_number,part_number,form_id,form_version,toeic_session_id,quota,baseline_answered,frozen_item_ids_json,status) VALUES(?1,?2,?3,?4,1,?5,?6,?7,?8,CASE WHEN ?2=1 THEN 'in_progress' ELSE 'pending' END)",params![practice_id,step_number,part,form_id,session_id,quota,baseline,serde_json::to_string(&ordered).unwrap()]).map_err(err)?;
        Ok(())
    }

    fn fresh_form(&self, part: u32) -> Result<String, String> {
        let c = database::open(&self.database)?;
        let mut ranked = Vec::new();
        for family in ["a", "b", "c"] {
            let form = format!("toeic-part{part}-form-{family}");
            let active: bool = c.query_row("SELECT EXISTS(SELECT 1 FROM toeic_session WHERE form_id=?1 AND status='in_progress')", [&form], |r| r.get(0)).map_err(err)?;
            let count: u32 = c.query_row("SELECT COUNT(*) FROM toeic_answer a JOIN toeic_session s ON s.id=a.session_id WHERE s.form_id=?1", [&form], |r| r.get(0)).map_err(err)?;
            ranked.push((active, count, form));
        }
        ranked.sort();
        ranked.into_iter().find(|x| !x.0).map(|x| x.2).ok_or("Finish or abandon the active standalone Forms for this Part before starting personalized practice.".into())
    }

    fn start_child(&self, part: u32, form: &str) -> Result<(String, u32), String> {
        let session_id = match part {
            1 => self.p1.start(form, 1)?.session_id,
            2 => self.p2.start(form, 1)?.session_id,
            3 => self.p3.start(form, 1)?.session_id,
            4 => self.p4.start(form, 1)?.session_id,
            5 => self.p5.start(form, 1, "learning")?.session_id,
            6 => self.p6.start(form, 1, "learning")?.session_id,
            7 => self.p7.start(form, 1, "learning")?.session_id,
            _ => return Err("TOEIC Part must be between 1 and 7.".into()),
        };
        let c = database::open(&self.database)?;
        let baseline = c
            .query_row(
                "SELECT COUNT(*) FROM toeic_answer WHERE session_id=?1",
                [&session_id],
                |r| r.get::<_, u32>(0),
            )
            .map_err(err)?;
        Ok((session_id, baseline))
    }

    fn refresh(&self, id: &str) -> Result<(), String> {
        let c = database::open(&self.database)?;
        let mut st = c.prepare("SELECT step_number,toeic_session_id,quota,frozen_item_ids_json FROM toeic_personalized_practice_step WHERE practice_session_id=?1 ORDER BY step_number").map_err(err)?;
        let rows = st
            .query_map([id], |r| {
                Ok((
                    r.get::<_, u32>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, u32>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        drop(st);
        let mut answered = 0u32;
        let mut correct = 0u32;
        let mut all_done = !rows.is_empty();
        for (number, sid, quota, json) in &rows {
            let ids: BTreeSet<String> = serde_json::from_str::<Vec<String>>(json)
                .map_err(|_| "Invalid frozen TOEIC practice items.")?
                .into_iter()
                .collect();
            let mut ast=c.prepare("SELECT item_id,is_correct FROM toeic_answer WHERE session_id=?1 ORDER BY answered_at,id").map_err(err)?;
            let values = ast
                .query_map([sid], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, bool>(1)?))
                })
                .map_err(err)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(err)?;
            let selected = values
                .into_iter()
                .filter(|x| ids.contains(&x.0))
                .take(*quota as usize)
                .collect::<Vec<_>>();
            let n = selected.len() as u32;
            let ok = selected.iter().filter(|x| x.1).count() as u32;
            answered += n;
            correct += ok;
            let done = n >= *quota;
            all_done &= done;
            c.execute("UPDATE toeic_personalized_practice_step SET status=CASE WHEN ?3 THEN 'completed' WHEN ?1=1 THEN 'in_progress' ELSE status END WHERE practice_session_id=?2 AND step_number=?1",params![number,id,done]).map_err(err)?;
            if done {
                c.execute("UPDATE toeic_session SET status=CASE WHEN status='in_progress' THEN 'abandoned' ELSE status END,abandoned_at=CASE WHEN status='in_progress' THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE abandoned_at END WHERE id=?1",[sid]).map_err(err)?;
            }
        }
        if let Some(next)=rows.iter().find(|(n,_,_,_)|c.query_row("SELECT status FROM toeic_personalized_practice_step WHERE practice_session_id=?1 AND step_number=?2",params![id,n],|r|r.get::<_,String>(0)).unwrap_or_default()!="completed") {
            c.execute("UPDATE toeic_personalized_practice_step SET status='in_progress' WHERE practice_session_id=?1 AND step_number=?2",params![id,next.0]).map_err(err)?;
        }
        c.execute("UPDATE toeic_personalized_practice_session SET answered=MIN(?2,requested_count),correct=CASE WHEN ?3 THEN ?4 ELSE NULL END,status=CASE WHEN ?3 THEN 'completed' ELSE status END,completed_at=CASE WHEN ?3 THEN COALESCE(completed_at,strftime('%Y-%m-%dT%H:%M:%fZ','now')) ELSE completed_at END,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![id,answered,all_done,correct]).map_err(err)?;
        Ok(())
    }

    fn steps(&self, c: &rusqlite::Connection, id: &str) -> Result<Vec<PracticeStepDto>, String> {
        let mut st=c.prepare("SELECT step_number,part_number,form_id,toeic_session_id,quota,frozen_item_ids_json,status FROM toeic_personalized_practice_step WHERE practice_session_id=?1 ORDER BY step_number").map_err(err)?;
        let rows = st
            .query_map([id], |r| {
                Ok((
                    r.get::<_, u32>(0)?,
                    r.get::<_, u32>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, u32>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                ))
            })
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        rows.into_iter()
            .map(|(n, part, form, sid, quota, json, status)| {
                let ids: BTreeSet<String> = serde_json::from_str::<Vec<String>>(&json)
                    .map_err(|_| "Invalid TOEIC practice snapshot.")?
                    .into_iter()
                    .collect();
                let mut ast = c
                    .prepare("SELECT item_id,is_correct FROM toeic_answer WHERE session_id=?1")
                    .map_err(err)?;
                let values = ast
                    .query_map([&sid], |r| {
                        Ok((r.get::<_, String>(0)?, r.get::<_, bool>(1)?))
                    })
                    .map_err(err)?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(err)?;
                let chosen = values
                    .into_iter()
                    .filter(|x| ids.contains(&x.0))
                    .take(quota as usize)
                    .collect::<Vec<_>>();
                let answered = chosen.len() as u32;
                let correct = chosen.iter().filter(|x| x.1).count() as u32;
                Ok(PracticeStepDto {
                    step_number: n,
                    part_number: part,
                    form_id: form,
                    session_id: sid.clone(),
                    quota,
                    answered,
                    correct,
                    status,
                    route: part_route(part, &sid, id, quota),
                })
            })
            .collect()
    }

    fn active_practice(&self) -> Result<Option<PracticeSessionDto>, String> {
        let c = database::open(&self.database)?;
        let id=c.query_row("SELECT id FROM toeic_personalized_practice_session WHERE status='in_progress' ORDER BY created_at DESC LIMIT 1",[],|r|r.get::<_,String>(0)).optional().map_err(err)?;
        drop(c);
        id.map(|x| self.practice(&x)).transpose()
    }

    fn exposure(&self) -> Result<ExposureDto, String> {
        let c = database::open(&self.database)?;
        let(total,unique,repeated)=c.query_row("SELECT COUNT(*),COUNT(DISTINCT item_id||':'||item_version),COUNT(*)-COUNT(DISTINCT item_id||':'||item_version) FROM toeic_answer",[],|r|Ok((r.get::<_,u32>(0)?,r.get::<_,u32>(1)?,r.get::<_,u32>(2)?))).map_err(err)?;
        Ok(ExposureDto {
            unique_seen: unique,
            bank_items: BANK_ITEM_COUNT,
            total_answers: total,
            unseen: BANK_ITEM_COUNT.saturating_sub(unique),
            repeated_items: repeated,
        })
    }

    fn weaknesses(&self) -> Result<Vec<WeaknessDto>, String> {
        let c = database::open(&self.database)?;
        let mut st=c.prepare("SELECT part,form_snapshot_json,item_id,is_correct,answered_at FROM (SELECT s.part,s.form_snapshot_json,a.item_id,a.is_correct,a.answered_at,a.id,ROW_NUMBER() OVER(PARTITION BY a.item_id,a.item_version ORDER BY a.answered_at,a.id) AS attempt_number FROM toeic_answer a JOIN toeic_session s ON s.id=a.session_id) WHERE attempt_number=1 ORDER BY answered_at,id").map_err(err)?;
        let rows = st
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, bool>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        let mut groups: BTreeMap<(u32, String), SkillStats> = BTreeMap::new();
        for (part, json, item, ok, at) in rows {
            let part_number = parse_part(&part);
            let value: Value = serde_json::from_str(&json).unwrap_or(Value::Null);
            let skill = find_skill(&value, &item).unwrap_or_else(|| default_skill(part_number));
            groups
                .entry((part_number, humanize(&skill)))
                .or_default()
                .outcomes
                .push((ok, at));
        }
        let mut out = groups
            .into_iter()
            .map(|((part, skill), stats)| {
                let total = stats.outcomes.len() as u32;
                let recent_start = stats.outcomes.len().saturating_sub(5);
                let mut weighted_total = 0u32;
                let mut weighted_correct = 0u32;
                for (i, (ok, _)) in stats.outcomes.iter().enumerate() {
                    let w = if i >= recent_start { 2 } else { 1 };
                    weighted_total += w;
                    if *ok {
                        weighted_correct += w
                    }
                }
                let accuracy = if weighted_total == 0 {
                    0
                } else {
                    weighted_correct * 100 / weighted_total
                };
                let sufficient = total >= 5;
                let label = if !sufficient {
                    "Insufficient Data"
                } else if accuracy >= 85 {
                    "Strong"
                } else if accuracy >= 70 {
                    "Stable"
                } else if accuracy >= 50 {
                    "Needs Practice"
                } else {
                    "Priority"
                };
                WeaknessDto {
                    part_number: part,
                    skill,
                    correct: stats.outcomes.iter().filter(|x| x.0).count() as u32,
                    total,
                    accuracy,
                    label: label.into(),
                    sufficient_sample: sufficient,
                    last_seen_at: stats
                        .outcomes
                        .last()
                        .map(|x| x.1.clone())
                        .unwrap_or_default(),
                }
            })
            .collect::<Vec<_>>();
        out.sort_by_key(|x| {
            (
                !x.sufficient_sample,
                x.accuracy,
                std::cmp::Reverse(x.total),
                x.part_number,
            )
        });
        Ok(out)
    }

    fn trends(&self) -> Result<Vec<TrendPointDto>, String> {
        let c = database::open(&self.database)?;
        let mut st=c.prepare("SELECT id,family,completed_at,listening_raw,reading_raw,total_raw,listening_estimate,reading_estimate,total_estimate FROM toeic_full_lr_session WHERE status='completed' AND completed_at IS NOT NULL ORDER BY completed_at,id").map_err(err)?;
        let trends = st
            .query_map([], |r| {
                Ok(TrendPointDto {
                    session_id: r.get(0)?,
                    family: r.get(1)?,
                    completed_at: r.get(2)?,
                    listening_raw: r.get(3)?,
                    reading_raw: r.get(4)?,
                    total_raw: r.get(5)?,
                    listening_estimate: r.get(6)?,
                    reading_estimate: r.get(7)?,
                    total_estimate: r.get(8)?,
                })
            })
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        Ok(trends)
    }

    fn latest_section_estimates(
        &self,
    ) -> Result<(Option<u32>, Option<u32>, Option<u32>, Option<u32>), String> {
        let c = database::open(&self.database)?;
        let l=c.query_row("SELECT estimated_score,range_low,range_high FROM toeic_full_listening_session WHERE status='completed' ORDER BY completed_at DESC,id DESC LIMIT 1",[],|r|Ok((r.get::<_,u32>(0)?,r.get::<_,u32>(1)?,r.get::<_,u32>(2)?))).optional().map_err(err)?;
        let rd=c.query_row("SELECT estimated_score,range_low,range_high FROM toeic_full_reading_session WHERE status='completed' ORDER BY completed_at DESC,id DESC LIMIT 1",[],|r|Ok((r.get::<_,u32>(0)?,r.get::<_,u32>(1)?,r.get::<_,u32>(2)?))).optional().map_err(err)?;
        let listening = l.as_ref().map(|x| x.0);
        let reading = rd.as_ref().map(|x| x.0);
        let low = l.as_ref().zip(rd.as_ref()).map(|(a, b)| a.1 + b.1);
        let high = l.as_ref().zip(rd.as_ref()).map(|(a, b)| a.2 + b.2);
        Ok((listening, reading, low, high))
    }

    fn latest_full_lr_range(&self) -> Result<(Option<u32>, Option<u32>), String> {
        let c = database::open(&self.database)?;
        c.query_row(
            "SELECT range_low,range_high FROM toeic_full_lr_session WHERE status='completed' ORDER BY completed_at DESC,id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map(|value| value.unwrap_or((None, None)))
        .map_err(err)
    }
}

fn validate_target(target: u32) -> Result<(), String> {
    if !(10..=990).contains(&target) {
        Err("Target TOEIC score must be between 10 and 990.".into())
    } else {
        Ok(())
    }
}
fn parse_part(part: &str) -> u32 {
    part.strip_prefix("part")
        .and_then(|x| x.chars().next())
        .and_then(|x| x.to_digit(10))
        .unwrap_or(0)
}
fn default_skill(part: u32) -> String {
    format!("Part {part} overall")
}
fn humanize(value: &str) -> String {
    value
        .replace(['_', '-'], " ")
        .split_whitespace()
        .map(|x| {
            let mut c = x.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn find_skill(value: &Value, item: &str) -> Option<String> {
    match value {
        Value::Object(m) => {
            let id = m
                .get("itemId")
                .or_else(|| m.get("questionId"))
                .or_else(|| {
                    if m.contains_key("choices") {
                        m.get("id")
                    } else {
                        None
                    }
                })
                .and_then(Value::as_str);
            if id == Some(item) {
                return [
                    "skillCategory",
                    "skill",
                    "category",
                    "questionType",
                    "intent",
                    "task",
                ]
                .iter()
                .find_map(|k| m.get(*k).and_then(Value::as_str).map(str::to_owned));
            }
            m.values().find_map(|v| find_skill(v, item))
        }
        Value::Array(a) => a.iter().find_map(|v| find_skill(v, item)),
        _ => None,
    }
}
fn collect_item_ids(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(m) => {
            let id = m
                .get("itemId")
                .or_else(|| m.get("questionId"))
                .or_else(|| {
                    if m.contains_key("choices") {
                        m.get("id")
                    } else {
                        None
                    }
                })
                .and_then(Value::as_str);
            if let Some(id) = id {
                out.push(id.into())
            }
            for v in m.values() {
                collect_item_ids(v, out)
            }
        }
        Value::Array(a) => {
            for v in a {
                collect_item_ids(v, out)
            }
        }
        _ => {}
    }
}
fn answer_ids(c: &rusqlite::Connection, id: &str) -> Result<BTreeSet<String>, String> {
    let mut st = c
        .prepare("SELECT item_id FROM toeic_answer WHERE session_id=?1")
        .map_err(err)?;
    let ids = st
        .query_map([id], |r| r.get::<_, String>(0))
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?
        .into_iter()
        .collect();
    Ok(ids)
}
fn choose_part(w: &[WeaknessDto], listening: bool) -> u32 {
    w.iter()
        .find(|x| {
            x.sufficient_sample
                && if listening {
                    x.part_number <= 4
                } else {
                    x.part_number >= 5
                }
        })
        .map(|x| x.part_number)
        .unwrap_or(if listening { 2 } else { 5 })
}
fn practice_focus(
    kind: &str,
    w: &[WeaknessDto],
    c: &rusqlite::Connection,
) -> Result<Vec<(u32, String)>, String> {
    if kind == "recent_mistakes" {
        if let Some((part,json,item))=c.query_row("SELECT s.part,s.form_snapshot_json,a.item_id FROM toeic_answer a JOIN toeic_session s ON s.id=a.session_id WHERE a.is_correct=0 ORDER BY a.answered_at DESC,a.id DESC LIMIT 1",[],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).optional().map_err(err)?{let p=parse_part(&part);let v:Value=serde_json::from_str(&json).unwrap_or(Value::Null);return Ok(vec![(p,humanize(&find_skill(&v,&item).unwrap_or_else(||default_skill(p))))])}
    }
    Ok(w.iter()
        .filter(|x| {
            x.sufficient_sample && matches!(x.label.as_str(), "Priority" | "Needs Practice")
        })
        .take(3)
        .map(|x| (x.part_number, x.skill.clone()))
        .collect())
}
fn build_priorities(w: &[WeaknessDto]) -> Vec<PriorityDto> {
    w.iter()
        .filter(|x| {
            x.sufficient_sample && matches!(x.label.as_str(), "Priority" | "Needs Practice")
        })
        .take(3)
        .enumerate()
        .map(|(i, x)| PriorityDto {
            rank: i as u32 + 1,
            part_number: x.part_number,
            skill: x.skill.clone(),
            reason: format!(
                "{}% recent-weighted accuracy across {} first attempts.",
                x.accuracy, x.total
            ),
            route: "/toeic/personalized".into(),
        })
        .collect()
}
fn build_recommendations(latest: Option<u32>, p: &[PriorityDto]) -> Vec<RecommendationDto> {
    let mut out = Vec::new();
    if !p.is_empty() {
        out.push(RecommendationDto {
            title: "Smart Practice · 15 questions".into(),
            description: format!("Focus first on Part {} — {}.", p[0].part_number, p[0].skill),
            route: "/toeic/personalized".into(),
        })
    } else {
        out.push(RecommendationDto {
            title: "TOEIC Daily Practice · 12 questions".into(),
            description:
                "Build a reliable skill sample with a short Listening and Reading session.".into(),
            route: "/toeic/personalized".into(),
        })
    }
    if latest.is_none() {
        out.push(RecommendationDto {
            title: "Full TOEIC L&R simulation".into(),
            description:
                "Complete a full simulation to establish a current unofficial practice estimate."
                    .into(),
            route: "/toeic/full".into(),
        })
    }
    out
}
fn part_route(part: u32, sid: &str, pid: &str, quota: u32) -> String {
    let base = match part {
        1 => "/toeic/session/",
        2 => "/toeic/part2/session/",
        3 => "/toeic/part3/session/",
        4 => "/toeic/part4/session/",
        5 => "/toeic/part5/session/",
        6 => "/toeic/part6/session/",
        _ => "/toeic/part7/session/",
    };
    format!("{base}{sid}?toeicPractice={pid}&limit={quota}")
}
fn err(e: rusqlite::Error) -> String {
    format!("TOEIC personalization database error: {e}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        toeic_item_bank::ToeicItemBank, toeic_part2::Part2Bank, toeic_part3::Part3Bank,
        toeic_part4::Part4Bank, toeic_part5::Part5Bank, toeic_part6::Part6Bank,
        toeic_part7::Part7Bank,
    };

    fn setup() -> (ToeicPersonalizationRepository, PathBuf) {
        let dir =
            std::env::temp_dir().join(format!("toeic-personalization-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = dir.join("test.sqlite3");
        database::migrate(&db).unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/toeic/item-bank-v1");
        let repo = ToeicPersonalizationRepository::new(
            db.clone(),
            ToeicRepository::new(db.clone(), ToeicItemBank::load(root.clone()).unwrap()).unwrap(),
            Part2Repository::new(
                db.clone(),
                Part2Bank::load(root.join("part2.json")).unwrap(),
            ),
            Part3Repository::new(
                db.clone(),
                Part3Bank::load(root.join("part3.json")).unwrap(),
            ),
            Part4Repository::new(
                db.clone(),
                Part4Bank::load(root.join("part4.json")).unwrap(),
            ),
            Part5Repository::new(
                db.clone(),
                Part5Bank::load(root.join("part5.json")).unwrap(),
            ),
            Part6Repository::new(
                db.clone(),
                Part6Bank::load(root.join("part6.json")).unwrap(),
            ),
            Part7Repository::new(db, Part7Bank::load(root.join("part7.json")).unwrap()),
        );
        (repo, dir)
    }
    #[test]
    fn target_bounds_and_labels_are_deterministic() {
        assert!(validate_target(10).is_ok());
        assert!(validate_target(990).is_ok());
        assert!(validate_target(9).is_err());
        assert!(validate_target(991).is_err());
        assert_eq!(humanize("cross_document"), "Cross Document");
    }
    #[test]
    fn insufficient_samples_never_become_priorities() {
        let w = vec![WeaknessDto {
            part_number: 7,
            skill: "Inference".into(),
            correct: 0,
            total: 2,
            accuracy: 0,
            label: "Insufficient Data".into(),
            sufficient_sample: false,
            last_seen_at: "now".into(),
        }];
        assert!(build_priorities(&w).is_empty());
    }

    #[test]
    fn target_persists_without_changing_any_score_record() {
        let (repo, dir) = setup();
        assert_eq!(repo.target().unwrap(), 750);
        let dashboard = repo.set_target(850).unwrap();
        assert_eq!(dashboard.target_score, 850);
        let c = database::open(&repo.database).unwrap();
        let scores: u32 = c
            .query_row("SELECT COUNT(*) FROM toeic_full_lr_session", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(scores, 0);
        drop(c);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn daily_and_smart_practice_freeze_validated_question_snapshots() {
        let (repo, dir) = setup();
        let daily = repo
            .start_practice(StartPracticeRequest {
                kind: "daily".into(),
                question_count: None,
            })
            .unwrap();
        assert_eq!(daily.requested_count, 12);
        assert_eq!(daily.steps.iter().map(|x| x.quota).sum::<u32>(), 12);
        assert!(daily.steps.iter().any(|x| x.part_number <= 4));
        assert!(daily.steps.iter().any(|x| x.part_number >= 5));
        let c = database::open(&repo.database).unwrap();
        let frozen: Vec<String> = c.prepare("SELECT frozen_item_ids_json FROM toeic_personalized_practice_step WHERE practice_session_id=?1 ORDER BY step_number").unwrap().query_map([&daily.session_id], |r| r.get(0)).unwrap().collect::<Result<_,_>>().unwrap();
        assert_eq!(
            frozen
                .iter()
                .map(|x| serde_json::from_str::<Vec<String>>(x).unwrap().len())
                .sum::<usize>(),
            12
        );
        c.execute("UPDATE toeic_personalized_practice_session SET status='abandoned',abandoned_at='now' WHERE id=?1", [&daily.session_id]).unwrap();
        c.execute("UPDATE toeic_session SET status='abandoned',abandoned_at='now' WHERE status='in_progress'", []).unwrap();
        drop(c);
        let smart = repo
            .start_practice(StartPracticeRequest {
                kind: "smart".into(),
                question_count: Some(20),
            })
            .unwrap();
        assert_eq!((smart.requested_count, smart.steps[0].quota), (20, 20));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn exposure_counts_only_real_scored_answers() {
        let (repo, dir) = setup();
        let smart = repo
            .start_practice(StartPracticeRequest {
                kind: "smart".into(),
                question_count: Some(10),
            })
            .unwrap();
        let c = database::open(&repo.database).unwrap();
        let step = &smart.steps[0];
        let item: String = c.query_row("SELECT json_extract(frozen_item_ids_json,'$[0]') FROM toeic_personalized_practice_step WHERE practice_session_id=?1", [&smart.session_id], |r| r.get(0)).unwrap();
        c.execute("INSERT INTO toeic_answer(id,session_id,item_id,item_version,selected_choice,is_correct,first_attempt,answered_at) VALUES('scored',?1,?2,1,'A',0,1,'2026-09-02T00:00:00Z')", params![step.session_id,item]).unwrap();
        drop(c);
        let exposure = repo.exposure().unwrap();
        assert_eq!(
            (
                exposure.total_answers,
                exposure.unique_seen,
                exposure.unseen
            ),
            (1, 1, 599)
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
