use crate::{database, interactive_lesson::*};
use rusqlite::{params, OptionalExtension, Transaction};
use serde_json::json;
use std::path::PathBuf;

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";

#[derive(Clone)]
pub struct InteractiveLessonRepository {
    database: PathBuf,
}

impl InteractiveLessonRepository {
    pub fn new(database: PathBuf) -> Self {
        Self { database }
    }

    pub fn start(
        &self,
        lesson: &RegisteredLesson,
        context: &StudentContextSnapshot,
        start_over: bool,
    ) -> Result<InteractiveLessonSessionDto, String> {
        let mut connection = database::open(&self.database)?;
        let tx = connection.transaction().map_err(db)?;
        let active: Option<String> = tx
            .query_row(
                "SELECT id FROM interactive_lesson_session WHERE status='in_progress' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(db)?;
        if let Some(id) = active {
            if !start_over {
                return Err(format!(
                    "An interactive lesson is already in progress: {id}"
                ));
            }
            abandon_tx(&tx, &id)?;
        }
        let id = uuid::Uuid::new_v4().to_string();
        let package_json = serde_json::to_string(&lesson.package).map_err(json_error)?;
        let context_json = serde_json::to_string(context).map_err(json_error)?;
        tx.execute(&format!("INSERT INTO interactive_lesson_session(id,lesson_id,lesson_content_version,package_schema_version,lesson_flow_version,package_hash,engine_version,snapshot_version,status,stage_count,current_stage_index,package_snapshot_json,student_context_snapshot_json,started_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,'in_progress',?9,0,?10,?11,{NOW},{NOW})"),params![id,lesson.package.lesson_id,lesson.package.content_version,lesson.package.package_schema_version,lesson.package.lesson_flow_version,lesson.package_hash,INTERACTIVE_LESSON_ENGINE_VERSION,INTERACTIVE_LESSON_SESSION_SNAPSHOT_VERSION,lesson.package.stages.len() as u32,package_json,context_json]).map_err(db)?;
        for (index, stage) in lesson.package.stages.iter().enumerate() {
            let status = if index == 0 { "active" } else { "pending" };
            tx.execute(&format!("INSERT INTO interactive_lesson_stage_state(id,session_id,stage_id,sequence_index,stage_type,stage_schema_version,required,status,attempt_count,started_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,0,CASE WHEN ?8='active' THEN {NOW} END,{NOW})"),params![uuid::Uuid::new_v4().to_string(),id,stage.stage_id,index as u32,stage.stage_type.as_str(),stage.stage_schema_version,stage.required,status]).map_err(db)?;
        }
        tx.commit().map_err(db)?;
        self.get(&id)?
            .ok_or_else(|| "Created interactive lesson session is unavailable.".into())
    }

    pub fn active(&self) -> Result<Option<InteractiveLessonSessionDto>, String> {
        let connection = database::open(&self.database)?;
        let id: Option<String> = connection
            .query_row(
                "SELECT id FROM interactive_lesson_session WHERE status='in_progress' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(db)?;
        id.map(|value| self.get(&value))
            .transpose()
            .map(|value| value.flatten())
    }
    pub fn get(&self, id: &str) -> Result<Option<InteractiveLessonSessionDto>, String> {
        let connection = database::open(&self.database)?;
        read_session(&connection, id)
    }
    pub fn recent(&self, limit: u32) -> Result<Vec<InteractiveLessonSessionDto>, String> {
        let connection = database::open(&self.database)?;
        let mut statement = connection
            .prepare("SELECT id FROM interactive_lesson_session ORDER BY started_at DESC LIMIT ?1")
            .map_err(db)?;
        let ids = statement
            .query_map([limit.min(50)], |row| row.get::<_, String>(0))
            .map_err(db)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db)?;
        ids.iter()
            .map(|id| {
                read_session(&connection, id)?
                    .ok_or_else(|| "Interactive session disappeared.".into())
            })
            .collect()
    }
    pub fn abandon(&self, id: &str) -> Result<InteractiveLessonSessionDto, String> {
        let mut connection = database::open(&self.database)?;
        let tx = connection.transaction().map_err(db)?;
        abandon_tx(&tx, id)?;
        tx.commit().map_err(db)?;
        self.get(id)?
            .ok_or_else(|| "Interactive lesson session not found.".into())
    }

    pub fn complete_current(
        &self,
        session_id: &str,
        stage_id: &str,
    ) -> Result<InteractiveLessonSessionDto, String> {
        let mut connection = database::open(&self.database)?;
        let tx = connection.transaction().map_err(db)?;
        let row:Option<(String,i64,i64,String)>=tx.query_row("SELECT status,current_stage_index,stage_count,package_snapshot_json FROM interactive_lesson_session WHERE id=?1",[session_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(db)?;
        let (status, current, count, snapshot) =
            row.ok_or("Interactive lesson session not found.")?;
        if status != "in_progress" {
            let already:i64=tx.query_row("SELECT COUNT(*) FROM interactive_lesson_stage_state WHERE session_id=?1 AND stage_id=?2 AND status='completed'",params![session_id,stage_id],|r|r.get(0)).map_err(db)?;
            if already == 1 {
                drop(tx);
                return self
                    .get(session_id)?
                    .ok_or_else(|| "Interactive lesson session not found.".into());
            }
            return Err("This interactive lesson session is immutable.".into());
        }
        let requested_status: Option<String> = tx
            .query_row(
                "SELECT status FROM interactive_lesson_stage_state WHERE session_id=?1 AND stage_id=?2",
                params![session_id, stage_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(db)?;
        if requested_status.as_deref() == Some("completed") {
            drop(tx);
            return self
                .get(session_id)?
                .ok_or_else(|| "Interactive lesson session not found.".into());
        }
        let (actual_id,kind,stage_status):(String,String,String)=tx.query_row("SELECT stage_id,stage_type,status FROM interactive_lesson_stage_state WHERE session_id=?1 AND sequence_index=?2",params![session_id,current],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(db)?;
        if actual_id != stage_id {
            return Err("Only the current stage can be completed.".into());
        }
        if stage_status == "completed" {
            drop(tx);
            return self
                .get(session_id)?
                .ok_or_else(|| "Interactive lesson session not found.".into());
        }
        if !matches!(kind.as_str(), "theory" | "visual_vocabulary") {
            return Err("This stage has no runtime executor.".into());
        }
        let package: InteractiveLessonPackage =
            serde_json::from_str(&snapshot).map_err(json_error)?;
        let stage = package
            .stages
            .get(current as usize)
            .ok_or("Session snapshot does not contain its current stage.")?;
        let result = match &stage.payload {
            StagePayload::Theory { .. } => {
                json!({"schemaVersion":INTERACTIVE_LESSON_STAGE_RESULT_VERSION,"kind":"acknowledged"})
            }
            StagePayload::VisualVocabulary { items } => {
                json!({"schemaVersion":INTERACTIVE_LESSON_STAGE_RESULT_VERSION,"kind":"acknowledged","itemCount":items.len()})
            }
            _ => return Err("This stage has no runtime executor.".into()),
        };
        tx.execute(&format!("UPDATE interactive_lesson_stage_state SET status='completed',attempt_count=1,completion_result_version=?1,completion_json=?2,completed_at={NOW},updated_at={NOW} WHERE session_id=?3 AND stage_id=?4 AND status='active'"),params![INTERACTIVE_LESSON_STAGE_RESULT_VERSION,result.to_string(),session_id,stage_id]).map_err(db)?;
        if current + 1 >= count {
            tx.execute(&format!("UPDATE interactive_lesson_session SET status='completed',current_stage_index=?1,completed_at={NOW},updated_at={NOW} WHERE id=?2 AND status='in_progress'"),params![current,session_id]).map_err(db)?;
        } else {
            let next = current + 1;
            tx.execute(&format!("UPDATE interactive_lesson_stage_state SET status='active',started_at={NOW},updated_at={NOW} WHERE session_id=?1 AND sequence_index=?2 AND status='pending'"),params![session_id,next]).map_err(db)?;
            tx.execute(&format!("UPDATE interactive_lesson_session SET current_stage_index=?1,updated_at={NOW} WHERE id=?2"),params![next,session_id]).map_err(db)?;
        }
        tx.commit().map_err(db)?;
        self.get(session_id)?
            .ok_or_else(|| "Interactive lesson session not found.".into())
    }

    pub fn skip_current(
        &self,
        session_id: &str,
        stage_id: &str,
    ) -> Result<InteractiveLessonSessionDto, String> {
        let mut connection = database::open(&self.database)?;
        let tx = connection.transaction().map_err(db)?;
        let (status,current,count):(String,i64,i64)=tx.query_row("SELECT status,current_stage_index,stage_count FROM interactive_lesson_session WHERE id=?1",[session_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(db)?;
        if status != "in_progress" {
            return Err("This interactive lesson session is immutable.".into());
        }
        let (actual,required):(String,bool)=tx.query_row("SELECT stage_id,required FROM interactive_lesson_stage_state WHERE session_id=?1 AND sequence_index=?2 AND status='active'",params![session_id,current],|r|Ok((r.get(0)?,r.get(1)?))).map_err(db)?;
        if actual != stage_id {
            return Err("Only the current stage can be skipped.".into());
        }
        if required {
            return Err("A required stage cannot be skipped.".into());
        }
        tx.execute(&format!("UPDATE interactive_lesson_stage_state SET status='skipped',skipped_at={NOW},updated_at={NOW} WHERE session_id=?1 AND stage_id=?2"),params![session_id,stage_id]).map_err(db)?;
        if current + 1 >= count {
            tx.execute(&format!("UPDATE interactive_lesson_session SET status='completed',completed_at={NOW},updated_at={NOW} WHERE id=?1"),[session_id]).map_err(db)?;
        } else {
            let next = current + 1;
            tx.execute(&format!("UPDATE interactive_lesson_stage_state SET status='active',started_at={NOW},updated_at={NOW} WHERE session_id=?1 AND sequence_index=?2"),params![session_id,next]).map_err(db)?;
            tx.execute(&format!("UPDATE interactive_lesson_session SET current_stage_index=?1,updated_at={NOW} WHERE id=?2"),params![next,session_id]).map_err(db)?;
        }
        tx.commit().map_err(db)?;
        self.get(session_id)?
            .ok_or_else(|| "Interactive lesson session not found.".into())
    }
}

fn abandon_tx(tx: &Transaction<'_>, id: &str) -> Result<(), String> {
    let changed=tx.execute(&format!("UPDATE interactive_lesson_session SET status='abandoned',abandoned_at={NOW},updated_at={NOW} WHERE id=?1 AND status='in_progress'"),[id]).map_err(db)?;
    if changed != 1 {
        return Err("Only an in-progress interactive lesson can be abandoned.".into());
    }
    Ok(())
}

fn read_session(
    connection: &rusqlite::Connection,
    id: &str,
) -> Result<Option<InteractiveLessonSessionDto>, String> {
    let raw:Option<(String,String,u32,String,u32,u32,String,Option<String>,Option<String>)>=connection.query_row("SELECT lesson_id,package_snapshot_json,lesson_content_version,status,current_stage_index,stage_count,started_at,completed_at,abandoned_at FROM interactive_lesson_session WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?))).optional().map_err(db)?;
    let Some((
        lesson_id,
        snapshot,
        content_version,
        status,
        current,
        stage_count,
        started_at,
        completed_at,
        abandoned_at,
    )) = raw
    else {
        return Ok(None);
    };
    let package: InteractiveLessonPackage = serde_json::from_str(&snapshot).map_err(json_error)?;
    let mut statement=connection.prepare("SELECT stage_id,sequence_index,stage_type,required,status,attempt_count FROM interactive_lesson_stage_state WHERE session_id=?1 ORDER BY sequence_index").map_err(db)?;
    let rows = statement
        .query_map([id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, u32>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, bool>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, u32>(5)?,
            ))
        })
        .map_err(db)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db)?;
    let mut stages = Vec::new();
    for (stage_id, index, kind, required, state, attempt_count) in rows {
        let source = package
            .stages
            .get(index as usize)
            .ok_or("Session stage snapshot mismatch.")?;
        stages.push(SessionStageDto {
            stage_id,
            sequence_index: index,
            stage_type: parse_stage_type(&kind)?,
            title: source.title.clone(),
            required,
            status: parse_stage_status(&state)?,
            attempt_count,
        });
    }
    let session_status = parse_session_status(&status)?;
    let completed = stages
        .iter()
        .filter(|stage| {
            matches!(
                stage.status,
                InteractiveStageStatus::Completed | InteractiveStageStatus::Skipped
            )
        })
        .count() as u32;
    let progress_percent = if stage_count == 0 {
        0
    } else {
        completed * 100 / stage_count
    };
    let active_stage = if session_status == InteractiveSessionStatus::InProgress {
        let stage = package
            .stages
            .get(current as usize)
            .ok_or("Current stage snapshot mismatch.")?;
        let content = match &stage.payload {
            StagePayload::Theory { blocks } => Some(ActiveStageContentDto::Theory {
                blocks: blocks.clone(),
            }),
            StagePayload::VisualVocabulary { items } => {
                Some(ActiveStageContentDto::VisualVocabulary {
                    items: items.clone(),
                })
            }
            _ => None,
        };
        content.map(|content| ActiveStageDto {
            stage_id: stage.stage_id.clone(),
            sequence_index: current,
            stage_type: stage.stage_type,
            title: stage.title.clone(),
            instructions: stage.instructions.clone(),
            required: stage.required,
            content,
        })
    } else {
        None
    };
    Ok(Some(InteractiveLessonSessionDto {
        id: id.into(),
        lesson_id,
        content_version,
        title: package.title,
        cefr_band: package.cefr_band,
        status: session_status,
        current_stage_index: current,
        stage_count,
        progress_percent,
        stages,
        active_stage,
        started_at,
        completed_at,
        abandoned_at,
    }))
}
fn parse_stage_type(v: &str) -> Result<InteractiveStageType, String> {
    InteractiveStageType::ORDER
        .into_iter()
        .find(|x| x.as_str() == v)
        .ok_or_else(|| format!("Invalid interactive stage type: {v}"))
}
fn parse_session_status(v: &str) -> Result<InteractiveSessionStatus, String> {
    match v {
        "in_progress" => Ok(InteractiveSessionStatus::InProgress),
        "completed" => Ok(InteractiveSessionStatus::Completed),
        "abandoned" => Ok(InteractiveSessionStatus::Abandoned),
        "failed" => Ok(InteractiveSessionStatus::Failed),
        _ => Err("Invalid interactive session status.".into()),
    }
}
fn parse_stage_status(v: &str) -> Result<InteractiveStageStatus, String> {
    match v {
        "pending" => Ok(InteractiveStageStatus::Pending),
        "active" => Ok(InteractiveStageStatus::Active),
        "completed" => Ok(InteractiveStageStatus::Completed),
        "skipped" => Ok(InteractiveStageStatus::Skipped),
        _ => Err("Invalid interactive stage status.".into()),
    }
}
fn db(error: rusqlite::Error) -> String {
    format!("Interactive lesson database error: {error}")
}
fn json_error(error: serde_json::Error) -> String {
    format!("Interactive lesson snapshot error: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_parsers_reject_unknown_values() {
        assert!(parse_session_status("legacy").is_err());
        assert!(parse_stage_status("done").is_err());
    }
}
