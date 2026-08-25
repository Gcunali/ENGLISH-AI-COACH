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
        id: &str,
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
        let package_json = serde_json::to_string(&lesson.package).map_err(json_error)?;
        let context_json = serde_json::to_string(context).map_err(json_error)?;
        tx.execute(&format!("INSERT INTO interactive_lesson_session(id,lesson_id,lesson_content_version,package_schema_version,lesson_flow_version,package_hash,engine_version,snapshot_version,status,stage_count,current_stage_index,package_snapshot_json,student_context_snapshot_json,started_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,'in_progress',?9,0,?10,?11,{NOW},{NOW})"),params![id,lesson.package.lesson_id,lesson.package.content_version,lesson.package.package_schema_version,lesson.package.lesson_flow_version,lesson.package_hash,INTERACTIVE_LESSON_ENGINE_VERSION,INTERACTIVE_LESSON_SESSION_SNAPSHOT_VERSION,lesson.package.stages.len() as u32,package_json,context_json]).map_err(db)?;
        for (index, stage) in lesson.package.stages.iter().enumerate() {
            let status = if index == 0 { "active" } else { "pending" };
            tx.execute(&format!("INSERT INTO interactive_lesson_stage_state(id,session_id,stage_id,sequence_index,stage_type,stage_schema_version,required,status,attempt_count,started_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,0,CASE WHEN ?8='active' THEN {NOW} END,{NOW})"),params![uuid::Uuid::new_v4().to_string(),id,stage.stage_id,index as u32,stage.stage_type.as_str(),stage.stage_schema_version,stage.required,status]).map_err(db)?;
            if let Some(runtime) = initial_runtime_state(&stage.payload) {
                tx.execute(&format!("INSERT INTO interactive_lesson_stage_runtime_state(session_id,stage_id,runtime_state_schema_version,state_json,updated_at) VALUES(?1,?2,?3,?4,{NOW})"),params![id,stage.stage_id,GUIDED_LESSON_RUNTIME_STATE_SCHEMA_VERSION,serde_json::to_string(&runtime).map_err(json_error)?]).map_err(db)?;
            }
        }
        tx.commit().map_err(db)?;
        self.get(id)?
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

    pub fn mark_reference_playback_completed(
        &self,
        request: &GuidedPlaybackRequest,
    ) -> Result<InteractiveLessonSessionDto, String> {
        let mut connection = database::open(&self.database)?;
        let tx = connection.transaction().map_err(db)?;
        let (stage, mut runtime) =
            active_stage_and_runtime(&tx, &request.session_id, &request.stage_id)?;
        match (&stage.payload, &mut runtime) {
            (
                StagePayload::Listening { segments, .. },
                GuidedStageRuntimeState::Listening { segments: state },
            ) => {
                if !segments
                    .iter()
                    .any(|item| item.segment_id == request.item_id)
                {
                    return Err("Listening segment not found.".into());
                }
                let item = state
                    .iter_mut()
                    .find(|item| item.segment_id == request.item_id)
                    .ok_or("Listening runtime state is invalid.")?;
                item.completed_playback_count = item.completed_playback_count.saturating_add(1);
            }
            (
                StagePayload::Repeat { targets },
                GuidedStageRuntimeState::Repeat { targets: state },
            ) => {
                if !targets.iter().any(|item| item.target_id == request.item_id) {
                    return Err("Repeat target not found.".into());
                }
                let item = state
                    .iter_mut()
                    .find(|item| item.item_id == request.item_id)
                    .ok_or("Repeat runtime state is invalid.")?;
                item.completed_reference_playback_count =
                    item.completed_reference_playback_count.saturating_add(1);
            }
            _ => return Err("This stage item has no reference playback.".into()),
        }
        save_runtime(&tx, &request.session_id, &request.stage_id, &runtime)?;
        tx.commit().map_err(db)?;
        self.get(&request.session_id)?
            .ok_or_else(|| "Interactive lesson session not found.".into())
    }

    pub fn begin_pronunciation_attempt(
        &self,
        request: &GuidedPronunciationRequest,
    ) -> Result<GuidedAttemptContext, String> {
        let mut connection = database::open(&self.database)?;
        let tx = connection.transaction().map_err(db)?;
        let (stage, mut runtime) =
            active_stage_and_runtime(&tx, &request.session_id, &request.stage_id)?;
        let analyzing:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM interactive_lesson_pronunciation_attempt WHERE session_id=?1 AND status='analyzing')",[&request.session_id],|row|row.get(0)).map_err(db)?;
        if analyzing {
            return Err("A Guided Lesson pronunciation analysis is already active.".into());
        }
        let target_text = match (&stage.payload, &mut runtime) {
            (
                StagePayload::Repeat { targets },
                GuidedStageRuntimeState::Repeat { targets: state },
            ) => {
                let progress = state
                    .iter_mut()
                    .find(|item| item.item_id == request.item_id)
                    .ok_or("Repeat target not found.")?;
                if progress.completed_reference_playback_count == 0 {
                    return Err("Listen to the complete reference before recording.".into());
                }
                progress.selected_attempt_id = None;
                targets
                    .iter()
                    .find(|item| item.target_id == request.item_id)
                    .map(|item| item.text.clone())
                    .ok_or("Repeat target not found.")?
            }
            (
                StagePayload::SpeakingCheck { targets },
                GuidedStageRuntimeState::SpeakingCheck { targets: state },
            ) => {
                let progress = state
                    .iter_mut()
                    .find(|item| item.item_id == request.item_id)
                    .ok_or("Speaking Check target not found.")?;
                progress.selected_attempt_id = None;
                targets
                    .iter()
                    .find(|item| item.target_id == request.item_id)
                    .map(|item| item.target_text.clone())
                    .ok_or("Speaking Check target not found.")?
            }
            _ => return Err("This stage does not accept pronunciation attempts.".into()),
        };
        save_runtime(&tx, &request.session_id, &request.stage_id, &runtime)?;
        let next: u32 = tx.query_row("SELECT COALESCE(MAX(attempt_index),0)+1 FROM interactive_lesson_pronunciation_attempt WHERE session_id=?1 AND stage_id=?2 AND item_id=?3",params![request.session_id,request.stage_id,request.item_id],|row|row.get(0)).map_err(db)?;
        let id = uuid::Uuid::new_v4().to_string();
        tx.execute(&format!("INSERT INTO interactive_lesson_pronunciation_attempt(id,session_id,stage_id,item_id,stage_type,attempt_index,status,result_schema_version,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,'analyzing',1,{NOW},{NOW})"),params![id,request.session_id,request.stage_id,request.item_id,stage.stage_type.as_str(),next]).map_err(db)?;
        tx.commit().map_err(db)?;
        Ok(GuidedAttemptContext {
            attempt_id: id,
            target_text,
        })
    }

    pub fn finish_pronunciation_attempt(
        &self,
        attempt_id: &str,
        status: &str,
        pronunciation_attempt_id: Option<&str>,
        error_code: Option<&str>,
    ) -> Result<InteractiveLessonSessionDto, String> {
        if !matches!(
            status,
            "completed"
                | "content_mismatch"
                | "insufficient_audio"
                | "alignment_failed"
                | "engine_unavailable"
                | "cancelled"
                | "failed"
        ) {
            return Err("Invalid guided pronunciation status.".into());
        }
        if status == "completed" && pronunciation_attempt_id.is_none() {
            return Err(
                "A completed Guided pronunciation attempt requires its acoustic result.".into(),
            );
        }
        let connection = database::open(&self.database)?;
        let session_id: String = connection
            .query_row(
                "SELECT session_id FROM interactive_lesson_pronunciation_attempt WHERE id=?1",
                [attempt_id],
                |row| row.get(0),
            )
            .map_err(db)?;
        let result = json!({"schemaVersion":1,"status":status});
        let changed=connection.execute(&format!("UPDATE interactive_lesson_pronunciation_attempt SET status=?1,pronunciation_attempt_id=?2,result_json=?3,error_code=?4,completed_at={NOW},updated_at={NOW} WHERE id=?5 AND status='analyzing'"),params![status,pronunciation_attempt_id,result.to_string(),error_code,attempt_id]).map_err(db)?;
        if changed != 1 {
            return Err("Guided pronunciation attempt is stale.".into());
        }
        self.get(&session_id)?
            .ok_or_else(|| "Interactive lesson session not found.".into())
    }

    pub fn select_pronunciation_attempt(
        &self,
        request: &SelectGuidedAttemptRequest,
    ) -> Result<InteractiveLessonSessionDto, String> {
        let mut connection = database::open(&self.database)?;
        let tx = connection.transaction().map_err(db)?;
        let (_stage, mut runtime) =
            active_stage_and_runtime(&tx, &request.session_id, &request.stage_id)?;
        let valid: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM interactive_lesson_pronunciation_attempt WHERE id=?1 AND session_id=?2 AND stage_id=?3 AND item_id=?4 AND status='completed')",params![request.attempt_id,request.session_id,request.stage_id,request.item_id],|row|row.get(0)).map_err(db)?;
        if !valid {
            return Err("Only a completed attempt for this target can be selected.".into());
        }
        let items = match &mut runtime {
            GuidedStageRuntimeState::Repeat { targets }
            | GuidedStageRuntimeState::SpeakingCheck { targets } => targets,
            _ => return Err("This stage does not select pronunciation attempts.".into()),
        };
        let item = items
            .iter_mut()
            .find(|item| item.item_id == request.item_id)
            .ok_or("Guided target not found.")?;
        item.selected_attempt_id = Some(request.attempt_id.clone());
        save_runtime(&tx, &request.session_id, &request.stage_id, &runtime)?;
        tx.commit().map_err(db)?;
        self.get(&request.session_id)?
            .ok_or_else(|| "Interactive lesson session not found.".into())
    }

    pub fn recover_interrupted_attempts(&self) -> Result<u32, String> {
        let connection = database::open(&self.database)?;
        connection.execute(&format!("UPDATE interactive_lesson_pronunciation_attempt SET status='failed',error_code='interrupted',result_json='{{\"schemaVersion\":1,\"status\":\"failed\"}}',completed_at={NOW},updated_at={NOW} WHERE status='analyzing'"),[]).map(|count| count as u32).map_err(db)
    }

    pub fn playback_source(
        &self,
        request: &GuidedPlaybackRequest,
    ) -> Result<GuidedPlaybackSource, String> {
        let connection = database::open(&self.database)?;
        let row:Option<(String,String,u32)>=connection.query_row("SELECT status,package_snapshot_json,current_stage_index FROM interactive_lesson_session WHERE id=?1",[&request.session_id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).optional().map_err(db)?;
        let (status, snapshot, current) = row.ok_or("Interactive lesson session not found.")?;
        if status != "in_progress" {
            return Err("This interactive lesson session is immutable.".into());
        }
        let package_hash: String = connection
            .query_row(
                "SELECT package_hash FROM interactive_lesson_session WHERE id=?1",
                [&request.session_id],
                |row| row.get(0),
            )
            .map_err(db)?;
        let package: InteractiveLessonPackage =
            serde_json::from_str(&snapshot).map_err(json_error)?;
        let stage = package
            .stages
            .get(current as usize)
            .ok_or("Current stage snapshot mismatch.")?;
        if stage.stage_id != request.stage_id {
            return Err("Only the current stage can play audio.".into());
        }
        let (text, asset_id) = match &stage.payload {
            StagePayload::Listening { segments, .. } => segments
                .iter()
                .find(|item| item.segment_id == request.item_id)
                .map(|item| (item.text.clone(), item.audio_asset_id.clone()))
                .ok_or("Listening segment not found.")?,
            StagePayload::Repeat { targets } => targets
                .iter()
                .find(|item| item.target_id == request.item_id)
                .map(|item| (item.text.clone(), item.reference_audio_asset_id.clone()))
                .ok_or("Repeat target not found.")?,
            _ => return Err("This stage has no reference audio.".into()),
        };
        Ok(GuidedPlaybackSource {
            text,
            asset_id,
            package_hash,
        })
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
        let (actual_id,_kind,stage_status):(String,String,String)=tx.query_row("SELECT stage_id,stage_type,status FROM interactive_lesson_stage_state WHERE session_id=?1 AND sequence_index=?2",params![session_id,current],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(db)?;
        if actual_id != stage_id {
            return Err("Only the current stage can be completed.".into());
        }
        if stage_status == "completed" {
            drop(tx);
            return self
                .get(session_id)?
                .ok_or_else(|| "Interactive lesson session not found.".into());
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
            StagePayload::Listening { segments, .. } => {
                let state = load_runtime(&tx, session_id, stage_id)?;
                let GuidedStageRuntimeState::Listening { segments: progress } = state else {
                    return Err("Listening runtime state is invalid.".into());
                };
                if progress
                    .iter()
                    .any(|item| item.completed_playback_count == 0)
                {
                    return Err("Listen to every complete segment before continuing.".into());
                }
                json!({"schemaVersion":INTERACTIVE_LESSON_STAGE_RESULT_VERSION,"kind":"listening_completed","segmentCount":segments.len(),"completedPlaybackCounts":progress.iter().map(|item|json!({"segmentId":item.segment_id,"playCount":item.completed_playback_count})).collect::<Vec<_>>()})
            }
            StagePayload::Repeat { targets } => {
                let state = load_runtime(&tx, session_id, stage_id)?;
                let GuidedStageRuntimeState::Repeat { targets: progress } = state else {
                    return Err("Repeat runtime state is invalid.".into());
                };
                if progress
                    .iter()
                    .any(|item| item.selected_attempt_id.is_none())
                {
                    return Err("Select one completed attempt for every Repeat target.".into());
                }
                json!({"schemaVersion":INTERACTIVE_LESSON_STAGE_RESULT_VERSION,"kind":"repeat_completed","targetCount":targets.len(),"attemptCount":guided_attempt_count(&tx,session_id,stage_id)?,"selectedAttemptIds":progress.iter().map(|item|item.selected_attempt_id.clone().unwrap()).collect::<Vec<_>>()})
            }
            StagePayload::SpeakingCheck { targets } => {
                let state = load_runtime(&tx, session_id, stage_id)?;
                let GuidedStageRuntimeState::SpeakingCheck { targets: progress } = state else {
                    return Err("Speaking Check runtime state is invalid.".into());
                };
                if progress
                    .iter()
                    .any(|item| item.selected_attempt_id.is_none())
                {
                    return Err(
                        "Select one completed attempt for every Speaking Check target.".into(),
                    );
                }
                json!({"schemaVersion":INTERACTIVE_LESSON_STAGE_RESULT_VERSION,"kind":"speaking_check_completed","targetCount":targets.len(),"attemptCount":guided_attempt_count(&tx,session_id,stage_id)?,"selectedAttemptIds":progress.iter().map(|item|item.selected_attempt_id.clone().unwrap()).collect::<Vec<_>>()})
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

fn initial_runtime_state(payload: &StagePayload) -> Option<GuidedStageRuntimeState> {
    match payload {
        StagePayload::Listening { segments, .. } => Some(GuidedStageRuntimeState::Listening {
            segments: segments
                .iter()
                .map(|item| ListeningItemState {
                    segment_id: item.segment_id.clone(),
                    completed_playback_count: 0,
                })
                .collect(),
        }),
        StagePayload::Repeat { targets } => Some(GuidedStageRuntimeState::Repeat {
            targets: targets
                .iter()
                .map(|item| PronunciationItemState {
                    item_id: item.target_id.clone(),
                    completed_reference_playback_count: 0,
                    selected_attempt_id: None,
                })
                .collect(),
        }),
        StagePayload::SpeakingCheck { targets } => Some(GuidedStageRuntimeState::SpeakingCheck {
            targets: targets
                .iter()
                .map(|item| PronunciationItemState {
                    item_id: item.target_id.clone(),
                    completed_reference_playback_count: 0,
                    selected_attempt_id: None,
                })
                .collect(),
        }),
        _ => None,
    }
}

fn load_runtime(
    connection: &rusqlite::Connection,
    session_id: &str,
    stage_id: &str,
) -> Result<GuidedStageRuntimeState, String> {
    let raw: String = connection.query_row("SELECT state_json FROM interactive_lesson_stage_runtime_state WHERE session_id=?1 AND stage_id=?2", params![session_id,stage_id], |row| row.get(0)).map_err(db)?;
    serde_json::from_str(&raw).map_err(json_error)
}

fn save_runtime(
    tx: &Transaction<'_>,
    session_id: &str,
    stage_id: &str,
    state: &GuidedStageRuntimeState,
) -> Result<(), String> {
    tx.execute(&format!("UPDATE interactive_lesson_stage_runtime_state SET state_json=?1,updated_at={NOW} WHERE session_id=?2 AND stage_id=?3"),params![serde_json::to_string(state).map_err(json_error)?,session_id,stage_id]).map_err(db)?;
    Ok(())
}

fn active_stage_and_runtime(
    tx: &Transaction<'_>,
    session_id: &str,
    stage_id: &str,
) -> Result<(InteractiveStage, GuidedStageRuntimeState), String> {
    let row: Option<(String, String, u32)> = tx.query_row("SELECT status,package_snapshot_json,current_stage_index FROM interactive_lesson_session WHERE id=?1",[session_id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).optional().map_err(db)?;
    let (status, snapshot, current) = row.ok_or("Interactive lesson session not found.")?;
    if status != "in_progress" {
        return Err("This interactive lesson session is immutable.".into());
    }
    let package: InteractiveLessonPackage = serde_json::from_str(&snapshot).map_err(json_error)?;
    let stage = package
        .stages
        .get(current as usize)
        .cloned()
        .ok_or("Current stage snapshot mismatch.")?;
    if stage.stage_id != stage_id {
        return Err("Only the current stage can be changed.".into());
    }
    let runtime = load_runtime(tx, session_id, stage_id)?;
    Ok((stage, runtime))
}

fn guided_attempt_count(
    connection: &rusqlite::Connection,
    session_id: &str,
    stage_id: &str,
) -> Result<u32, String> {
    connection.query_row("SELECT COUNT(*) FROM interactive_lesson_pronunciation_attempt WHERE session_id=?1 AND stage_id=?2",params![session_id,stage_id],|row|row.get(0)).map_err(db)
}

fn guided_attempts(
    connection: &rusqlite::Connection,
    database_path: &std::path::Path,
    session_id: &str,
    stage_id: &str,
    item_id: &str,
    selected_id: Option<&str>,
) -> Result<Vec<GuidedPronunciationAttemptDto>, String> {
    let mut statement=connection.prepare("SELECT id,attempt_index,status,pronunciation_attempt_id,created_at,completed_at FROM interactive_lesson_pronunciation_attempt WHERE session_id=?1 AND stage_id=?2 AND item_id=?3 ORDER BY attempt_index").map_err(db)?;
    let rows = statement
        .query_map(params![session_id, stage_id, item_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u32>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })
        .map_err(db)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db)?;
    let pronunciation =
        crate::pronunciation_repository::PronunciationRepository::new(database_path.to_path_buf());
    rows.into_iter()
        .map(
            |(id, attempt_index, status, pronunciation_id, created_at, completed_at)| {
                let result = pronunciation_id
                    .as_deref()
                    .map(|value| pronunciation.get(value))
                    .transpose()?
                    .flatten();
                Ok(GuidedPronunciationAttemptDto {
                    selected: selected_id == Some(id.as_str()),
                    id,
                    attempt_index,
                    status,
                    result,
                    created_at,
                    completed_at,
                })
            },
        )
        .collect()
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
            StagePayload::Listening {
                segments,
                reveal_text_after_first_play,
            } => {
                let runtime = load_runtime(connection, id, &stage.stage_id)?;
                let GuidedStageRuntimeState::Listening { segments: progress } = runtime else {
                    return Err("Listening runtime state is invalid.".into());
                };
                Some(ActiveStageContentDto::Listening {
                    reveal_text_after_first_play: *reveal_text_after_first_play,
                    segments: segments
                        .iter()
                        .map(|item| GuidedListeningSegmentDto {
                            segment_id: item.segment_id.clone(),
                            text: item.text.clone(),
                            has_bundled_audio: item.audio_asset_id.is_some(),
                            completed_playback_count: progress
                                .iter()
                                .find(|state| state.segment_id == item.segment_id)
                                .map(|state| state.completed_playback_count)
                                .unwrap_or(0),
                        })
                        .collect(),
                })
            }
            StagePayload::Repeat { targets } => {
                let runtime = load_runtime(connection, id, &stage.stage_id)?;
                let GuidedStageRuntimeState::Repeat { targets: progress } = runtime else {
                    return Err("Repeat runtime state is invalid.".into());
                };
                let mut values = Vec::new();
                for item in targets {
                    let state = progress
                        .iter()
                        .find(|value| value.item_id == item.target_id)
                        .ok_or("Repeat runtime target is missing.")?;
                    values.push(GuidedRepeatTargetDto {
                        target_id: item.target_id.clone(),
                        text: item.text.clone(),
                        hint: item.hint.clone(),
                        has_bundled_audio: item.reference_audio_asset_id.is_some(),
                        completed_reference_playback_count: state
                            .completed_reference_playback_count,
                        selected_attempt_id: state.selected_attempt_id.clone(),
                        attempts: guided_attempts(
                            connection,
                            &connection
                                .path()
                                .map(std::path::PathBuf::from)
                                .unwrap_or_default(),
                            id,
                            &stage.stage_id,
                            &item.target_id,
                            state.selected_attempt_id.as_deref(),
                        )?,
                    });
                }
                Some(ActiveStageContentDto::Repeat { targets: values })
            }
            StagePayload::SpeakingCheck { targets } => {
                let runtime = load_runtime(connection, id, &stage.stage_id)?;
                let GuidedStageRuntimeState::SpeakingCheck { targets: progress } = runtime else {
                    return Err("Speaking Check runtime state is invalid.".into());
                };
                let mut values = Vec::new();
                for item in targets {
                    let state = progress
                        .iter()
                        .find(|value| value.item_id == item.target_id)
                        .ok_or("Speaking Check runtime target is missing.")?;
                    values.push(GuidedSpeakingTargetDto {
                        target_id: item.target_id.clone(),
                        instruction: item.instruction.clone(),
                        target_text: item.target_text.clone(),
                        hint: item.hint.clone(),
                        selected_attempt_id: state.selected_attempt_id.clone(),
                        attempts: guided_attempts(
                            connection,
                            &connection
                                .path()
                                .map(std::path::PathBuf::from)
                                .unwrap_or_default(),
                            id,
                            &stage.stage_id,
                            &item.target_id,
                            state.selected_attempt_id.as_deref(),
                        )?,
                    });
                }
                Some(ActiveStageContentDto::SpeakingCheck { targets: values })
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
