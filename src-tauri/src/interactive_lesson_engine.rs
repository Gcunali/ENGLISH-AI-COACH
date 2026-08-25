use crate::{
    interactive_lesson::*, interactive_lesson_content::InteractiveLessonContentRegistry,
    interactive_lesson_repository::InteractiveLessonRepository,
    student_profile_repository::StudentProfileRepository,
};

#[derive(Clone)]
pub struct InteractiveLessonEngine {
    content: InteractiveLessonContentRegistry,
    repository: InteractiveLessonRepository,
    profiles: StudentProfileRepository,
    session_assets: std::path::PathBuf,
}

impl InteractiveLessonEngine {
    pub fn new(
        content: InteractiveLessonContentRegistry,
        repository: InteractiveLessonRepository,
        profiles: StudentProfileRepository,
        session_assets: std::path::PathBuf,
    ) -> Self {
        Self {
            content,
            repository,
            profiles,
            session_assets,
        }
    }
    pub fn overview(&self) -> Result<GuidedLessonOverviewDto, String> {
        Ok(GuidedLessonOverviewDto {
            published_lesson_count: self.content.published_count(),
            active_session: self.repository.active()?,
            capabilities: InteractiveStageType::ORDER
                .into_iter()
                .map(|stage_type| StageCapabilityDto {
                    stage_type,
                    runtime_available: stage_type.runtime_available(1),
                    stage_schema_version: 1,
                })
                .collect(),
        })
    }
    pub fn list(&self) -> Vec<InteractiveLessonSummaryDto> {
        self.content.list().iter().map(summary).collect()
    }
    pub fn detail(&self, lesson_id: &str) -> Option<InteractiveLessonDetailDto> {
        self.content.get(lesson_id).map(|lesson| {
            let summary = summary(&lesson);
            let stage_overview = lesson
                .package
                .stages
                .iter()
                .map(|stage| StageOverviewDto {
                    stage_id: stage.stage_id.clone(),
                    stage_type: stage.stage_type,
                    title: stage.title.clone(),
                    required: stage.required,
                    available: stage
                        .stage_type
                        .runtime_available(stage.stage_schema_version),
                })
                .collect();
            InteractiveLessonDetailDto {
                summary,
                stage_overview,
            }
        })
    }
    pub fn start(
        &self,
        request: StartInteractiveLessonRequest,
    ) -> Result<InteractiveLessonSessionDto, String> {
        let lesson = request
            .content_version
            .and_then(|version| self.content.get_exact(&request.lesson_id, version))
            .or_else(|| {
                request
                    .content_version
                    .is_none()
                    .then(|| self.content.get(&request.lesson_id))
                    .flatten()
            })
            .ok_or("Guided lesson not found.")?;
        let descriptor = summary(&lesson);
        if !descriptor.startable {
            return Err(format!(
                "This guided lesson cannot start: {}",
                descriptor.unavailable_reasons.join(" ")
            ));
        }
        let profile = self.profiles.get()?;
        let current = profile.current_placement;
        let context = StudentContextSnapshot {
            profile_schema_version: profile.schema_version,
            placement_attempt_id: current.as_ref().map(|value| value.attempt_id.clone()),
            estimated_cefr: current.as_ref().map(|value| value.estimated_level),
            placement_confidence: current.as_ref().map(|value| value.confidence),
            target_cefr: profile.target_level,
            learning_goals: profile.learning_goals,
        };
        let session_id = uuid::Uuid::new_v4().to_string();
        let asset_directory = self.session_assets.join(&session_id);
        if !lesson.asset_files.is_empty() {
            std::fs::create_dir_all(&asset_directory).map_err(|error| {
                format!("Could not create the Guided Lesson asset snapshot: {error}")
            })?;
            for asset in &lesson.package.assets {
                if !matches!(asset.r#type, AssetType::Audio) {
                    continue;
                }
                let source = lesson
                    .asset_files
                    .get(&asset.asset_id)
                    .ok_or("Validated Guided Lesson audio asset is unavailable.")?;
                let metadata = std::fs::metadata(source).map_err(|_| {
                    "Validated Guided Lesson audio asset is unavailable.".to_owned()
                })?;
                if metadata.len() > 20 * 1024 * 1024 {
                    let _ = std::fs::remove_dir_all(&asset_directory);
                    return Err("Guided Lesson audio asset exceeds 20 MB.".into());
                }
                if let Err(error) = std::fs::copy(
                    source,
                    asset_directory.join(format!("{}.wav", asset.asset_id)),
                ) {
                    let _ = std::fs::remove_dir_all(&asset_directory);
                    return Err(format!("Could not snapshot Guided Lesson audio: {error}"));
                }
            }
        }
        match self
            .repository
            .start(&session_id, &lesson, &context, request.start_over)
        {
            Ok(session) => Ok(session),
            Err(error) => {
                let _ = std::fs::remove_dir_all(&asset_directory);
                Err(error)
            }
        }
    }
    pub fn active(&self) -> Result<Option<InteractiveLessonSessionDto>, String> {
        self.repository.active()
    }
    pub fn resume(&self, id: &str) -> Result<InteractiveLessonSessionDto, String> {
        let session = self
            .repository
            .get(id)?
            .ok_or("Guided lesson session not found.")?;
        if session.status != InteractiveSessionStatus::InProgress {
            return Err("Only an in-progress guided lesson can be resumed.".into());
        }
        Ok(session)
    }
    pub fn get_session(&self, id: &str) -> Result<Option<InteractiveLessonSessionDto>, String> {
        self.repository.get(id)
    }
    pub fn complete(
        &self,
        request: StageActionRequest,
    ) -> Result<InteractiveLessonSessionDto, String> {
        self.repository
            .complete_current(&request.session_id, &request.stage_id)
    }
    pub fn skip(&self, request: StageActionRequest) -> Result<InteractiveLessonSessionDto, String> {
        self.repository
            .skip_current(&request.session_id, &request.stage_id)
    }
    pub fn abandon(&self, id: &str) -> Result<InteractiveLessonSessionDto, String> {
        self.repository.abandon(id)
    }
    pub fn recent(&self, limit: u32) -> Result<Vec<InteractiveLessonSessionDto>, String> {
        self.repository.recent(limit)
    }
    pub fn playback_source(
        &self,
        request: &GuidedPlaybackRequest,
    ) -> Result<GuidedPlaybackSource, String> {
        self.repository.playback_source(request)
    }
    pub fn reference_completed(
        &self,
        request: &GuidedPlaybackRequest,
    ) -> Result<InteractiveLessonSessionDto, String> {
        self.repository.mark_reference_playback_completed(request)
    }
    pub fn begin_pronunciation(
        &self,
        request: &GuidedPronunciationRequest,
    ) -> Result<GuidedAttemptContext, String> {
        self.repository.begin_pronunciation_attempt(request)
    }
    pub fn finish_pronunciation(
        &self,
        id: &str,
        status: &str,
        pronunciation_id: Option<&str>,
        error: Option<&str>,
    ) -> Result<InteractiveLessonSessionDto, String> {
        self.repository
            .finish_pronunciation_attempt(id, status, pronunciation_id, error)
    }
    pub fn select_pronunciation(
        &self,
        request: &SelectGuidedAttemptRequest,
    ) -> Result<InteractiveLessonSessionDto, String> {
        self.repository.select_pronunciation_attempt(request)
    }
    pub fn submit_exercise(
        &self,
        request: &crate::interactive_exercise::SubmitExerciseAttemptRequest,
    ) -> Result<InteractiveLessonSessionDto, String> {
        self.repository.submit_exercise_attempt(request)
    }
    pub fn select_exercise(
        &self,
        request: &crate::interactive_exercise::SelectExerciseAttemptRequest,
    ) -> Result<InteractiveLessonSessionDto, String> {
        self.repository.select_exercise_attempt(request)
    }
    pub fn recover_interrupted_attempts(&self) -> Result<u32, String> {
        self.repository.recover_interrupted_attempts()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interactive_exercise::*;
    use crate::{database, placement_repository::PlacementRepository};
    use rusqlite::Connection;
    use std::{fs, path::PathBuf};

    fn harness() -> (PathBuf, PathBuf, InteractiveLessonEngine) {
        let root = std::env::temp_dir().join(format!("guided-engine-{}", uuid::Uuid::new_v4()));
        let content = root.join("content");
        let package = content.join("foundation-v1");
        fs::create_dir_all(&package).unwrap();
        fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("test-fixtures/interactive-lessons/foundation-v1/lesson.json"),
            package.join("lesson.json"),
        )
        .unwrap();
        let database_path = root.join("test.sqlite3");
        database::migrate(&database_path).unwrap();
        let registry = InteractiveLessonContentRegistry::load(content.clone());
        let placement = PlacementRepository::new(database_path.clone()).unwrap();
        let profiles = StudentProfileRepository::new(database_path.clone(), placement);
        let engine = InteractiveLessonEngine::new(
            registry,
            InteractiveLessonRepository::new(database_path.clone()),
            profiles,
            root.join("session-assets"),
        );
        (root, content, engine)
    }

    fn phase_s_harness() -> (PathBuf, InteractiveLessonEngine) {
        let root =
            std::env::temp_dir().join(format!("guided-audio-engine-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let database_path = root.join("test.sqlite3");
        database::migrate(&database_path).unwrap();
        let registry = InteractiveLessonContentRegistry::load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("test-fixtures/interactive-lessons-phase-s"),
        );
        let placement = PlacementRepository::new(database_path.clone()).unwrap();
        let profiles = StudentProfileRepository::new(database_path.clone(), placement);
        let engine = InteractiveLessonEngine::new(
            registry,
            InteractiveLessonRepository::new(database_path),
            profiles,
            root.join("session-assets"),
        );
        (root, engine)
    }
    fn phase_t_harness() -> (PathBuf, PathBuf, InteractiveLessonEngine) {
        let root =
            std::env::temp_dir().join(format!("guided-exercise-engine-{}", uuid::Uuid::new_v4()));
        let content = root.join("content");
        let package = content.join("exercise");
        fs::create_dir_all(&package).unwrap();
        fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("test-fixtures/interactive-lessons-phase-t/deterministic-v1/lesson.json"),
            package.join("lesson.json"),
        )
        .unwrap();
        let database_path = root.join("test.sqlite3");
        database::migrate(&database_path).unwrap();
        let registry = InteractiveLessonContentRegistry::load(content.clone());
        let placement = PlacementRepository::new(database_path.clone()).unwrap();
        let profiles = StudentProfileRepository::new(database_path.clone(), placement);
        let engine = InteractiveLessonEngine::new(
            registry,
            InteractiveLessonRepository::new(database_path),
            profiles,
            root.join("session-assets"),
        );
        (root, content, engine)
    }
    fn latest_exercise_attempt(session: &InteractiveLessonSessionDto) -> String {
        let ActiveStageContentDto::Exercise { stage } =
            &session.active_stage.as_ref().unwrap().content
        else {
            panic!()
        };
        stage.items[stage.current_exercise_index as usize]
            .attempts
            .last()
            .unwrap()
            .attempt_id
            .clone()
    }
    fn insert_guided_pronunciation(root: &std::path::Path, id: &str, source_id: &str, score: f64) {
        let connection = Connection::open(root.join("test.sqlite3")).unwrap();
        connection.execute("INSERT INTO pronunciation_attempt(id,status,source_type,source_id,target_text,normalized_target,locale,engine_version,score_version,result_schema_version,model_id,model_revision,model_manifest_hash,overall_score,confidence,content_match_score,alignment_coverage,audio_duration_ms,word_count,created_at,completed_at) VALUES(?1,'completed','interactive_lesson',?2,'hello','hello','en-US',1,1,1,'model','revision',?3,?4,'low',1,1,500,1,'now','now')",rusqlite::params![id,source_id,"a".repeat(64),score]).unwrap();
    }

    #[test]
    fn physical_session_resumes_from_snapshot_and_has_no_downstream_side_effects() {
        let (root, content, engine) = harness();
        let request = || StartInteractiveLessonRequest {
            lesson_id: "everyday-greetings-a1".into(),
            content_version: None,
            start_over: false,
        };
        let started = engine.start(request()).unwrap();
        assert_eq!(started.status, InteractiveSessionStatus::InProgress);
        assert_eq!(started.current_stage_index, 0);
        assert!(engine.start(request()).is_err());
        fs::remove_dir_all(content).unwrap();
        let resumed = engine.resume(&started.id).unwrap();
        assert_eq!(resumed.title, "Everyday Greetings");
        let first = engine
            .complete(StageActionRequest {
                session_id: started.id.clone(),
                stage_id: "greeting-theory".into(),
            })
            .unwrap();
        assert_eq!(first.current_stage_index, 1);
        let repeated = engine
            .complete(StageActionRequest {
                session_id: started.id.clone(),
                stage_id: "greeting-theory".into(),
            })
            .unwrap();
        assert_eq!(repeated.current_stage_index, 1);
        assert!(engine
            .skip(StageActionRequest {
                session_id: started.id.clone(),
                stage_id: "greeting-words".into()
            })
            .is_err());
        let completed = engine
            .complete(StageActionRequest {
                session_id: started.id.clone(),
                stage_id: "greeting-words".into(),
            })
            .unwrap();
        assert_eq!(completed.status, InteractiveSessionStatus::Completed);
        assert_eq!(completed.progress_percent, 100);
        assert!(engine.abandon(&started.id).is_err());
        let connection = Connection::open(root.join("test.sqlite3")).unwrap();
        let result: String = connection.query_row("SELECT completion_json FROM interactive_lesson_stage_state WHERE stage_id='greeting-words'", [], |row| row.get(0)).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&result).unwrap()["itemCount"],
            2
        );
        for table in [
            "lesson",
            "vocabulary_item",
            "gamification_xp_event",
            "achievement_unlock",
            "review_session",
            "pronunciation_attempt",
            "voice_turn_performance",
        ] {
            let count: i64 = connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(count, 0, "unexpected side effect in {table}");
        }
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn start_over_abandons_the_only_active_session_and_snapshots_minimal_profile_metadata() {
        let (root, _content, engine) = harness();
        let first = engine
            .start(StartInteractiveLessonRequest {
                lesson_id: "everyday-greetings-a1".into(),
                content_version: None,
                start_over: false,
            })
            .unwrap();
        let second = engine
            .start(StartInteractiveLessonRequest {
                lesson_id: "everyday-greetings-a1".into(),
                content_version: None,
                start_over: true,
            })
            .unwrap();
        assert_ne!(first.id, second.id);
        assert_eq!(
            engine.get_session(&first.id).unwrap().unwrap().status,
            InteractiveSessionStatus::Abandoned
        );
        assert_eq!(engine.active().unwrap().unwrap().id, second.id);
        let connection = Connection::open(root.join("test.sqlite3")).unwrap();
        let snapshot: String = connection
            .query_row(
                "SELECT student_context_snapshot_json FROM interactive_lesson_session WHERE id=?1",
                [&second.id],
                |row| row.get(0),
            )
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&snapshot).unwrap();
        assert_eq!(value.as_object().unwrap().len(), 6);
        assert!(value.get("placementAttemptId").unwrap().is_null());
        assert!(value
            .get("learningGoals")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty());
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn phase_s_progress_requires_completed_playback_and_explicit_completed_attempt_selection() {
        let (root, engine) = phase_s_harness();
        let mut session = engine
            .start(StartInteractiveLessonRequest {
                lesson_id: "audio-foundation-a1".into(),
                content_version: None,
                start_over: false,
            })
            .unwrap();
        for stage in ["audio-theory", "audio-words"] {
            session = engine
                .complete(StageActionRequest {
                    session_id: session.id.clone(),
                    stage_id: stage.into(),
                })
                .unwrap();
        }
        assert_eq!(session.current_stage_index, 2);
        assert!(engine
            .complete(StageActionRequest {
                session_id: session.id.clone(),
                stage_id: "audio-listening".into()
            })
            .is_err());
        session = engine
            .reference_completed(&GuidedPlaybackRequest {
                session_id: session.id.clone(),
                stage_id: "audio-listening".into(),
                item_id: "hello-one".into(),
            })
            .unwrap();
        session = engine
            .reference_completed(&GuidedPlaybackRequest {
                session_id: session.id.clone(),
                stage_id: "audio-listening".into(),
                item_id: "hello-one".into(),
            })
            .unwrap();
        let ActiveStageContentDto::Listening { segments, .. } =
            &session.active_stage.as_ref().unwrap().content
        else {
            panic!()
        };
        assert_eq!(segments[0].completed_playback_count, 2);
        session = engine
            .complete(StageActionRequest {
                session_id: session.id.clone(),
                stage_id: "audio-listening".into(),
            })
            .unwrap();
        let repeat = GuidedPronunciationRequest {
            session_id: session.id.clone(),
            stage_id: "audio-repeat".into(),
            item_id: "repeat-one".into(),
            audio_base64: "unused".into(),
        };
        assert!(engine.begin_pronunciation(&repeat).is_err());
        session = engine
            .reference_completed(&GuidedPlaybackRequest {
                session_id: session.id.clone(),
                stage_id: "audio-repeat".into(),
                item_id: "repeat-one".into(),
            })
            .unwrap();
        let first = engine.begin_pronunciation(&repeat).unwrap();
        insert_guided_pronunciation(&root, "pron-repeat", &first.attempt_id, 12.0);
        session = engine
            .finish_pronunciation(&first.attempt_id, "completed", Some("pron-repeat"), None)
            .unwrap();
        assert!(engine
            .complete(StageActionRequest {
                session_id: session.id.clone(),
                stage_id: "audio-repeat".into()
            })
            .is_err());
        session = engine
            .select_pronunciation(&SelectGuidedAttemptRequest {
                session_id: session.id.clone(),
                stage_id: "audio-repeat".into(),
                item_id: "repeat-one".into(),
                attempt_id: first.attempt_id,
            })
            .unwrap();
        let retry = engine.begin_pronunciation(&repeat).unwrap();
        insert_guided_pronunciation(&root, "pron-repeat-retry", &retry.attempt_id, 99.0);
        session = engine
            .finish_pronunciation(
                &retry.attempt_id,
                "completed",
                Some("pron-repeat-retry"),
                None,
            )
            .unwrap();
        assert!(engine
            .complete(StageActionRequest {
                session_id: session.id.clone(),
                stage_id: "audio-repeat".into()
            })
            .is_err());
        session = engine
            .select_pronunciation(&SelectGuidedAttemptRequest {
                session_id: session.id.clone(),
                stage_id: "audio-repeat".into(),
                item_id: "repeat-one".into(),
                attempt_id: retry.attempt_id,
            })
            .unwrap();
        session = engine
            .complete(StageActionRequest {
                session_id: session.id.clone(),
                stage_id: "audio-repeat".into(),
            })
            .unwrap();
        let speaking = GuidedPronunciationRequest {
            session_id: session.id.clone(),
            stage_id: "audio-speaking".into(),
            item_id: "speaking-one".into(),
            audio_base64: "unused".into(),
        };
        let failed = engine.begin_pronunciation(&speaking).unwrap();
        session = engine
            .finish_pronunciation(&failed.attempt_id, "content_mismatch", None, None)
            .unwrap();
        assert!(engine
            .select_pronunciation(&SelectGuidedAttemptRequest {
                session_id: session.id.clone(),
                stage_id: "audio-speaking".into(),
                item_id: "speaking-one".into(),
                attempt_id: failed.attempt_id
            })
            .is_err());
        let completed = engine.begin_pronunciation(&speaking).unwrap();
        insert_guided_pronunciation(&root, "pron-speaking", &completed.attempt_id, 0.0);
        session = engine
            .finish_pronunciation(
                &completed.attempt_id,
                "completed",
                Some("pron-speaking"),
                None,
            )
            .unwrap();
        session = engine
            .select_pronunciation(&SelectGuidedAttemptRequest {
                session_id: session.id.clone(),
                stage_id: "audio-speaking".into(),
                item_id: "speaking-one".into(),
                attempt_id: completed.attempt_id,
            })
            .unwrap();
        session = engine
            .complete(StageActionRequest {
                session_id: session.id.clone(),
                stage_id: "audio-speaking".into(),
            })
            .unwrap();
        assert_eq!(session.status, InteractiveSessionStatus::Completed);
        let connection = Connection::open(root.join("test.sqlite3")).unwrap();
        let attempts: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM interactive_lesson_pronunciation_attempt",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(attempts, 4);
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn audio_asset_snapshot_survives_source_package_deletion() {
        let root =
            std::env::temp_dir().join(format!("guided-audio-snapshot-{}", uuid::Uuid::new_v4()));
        let content = root.join("content");
        let package = content.join("audio");
        fs::create_dir_all(package.join("assets")).unwrap();
        let bytes = b"immutable-test-wav";
        fs::write(package.join("assets/listen.wav"), bytes).unwrap();
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-fixtures/interactive-lessons-phase-s/audio-foundation-v1/lesson.json");
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(source).unwrap()).unwrap();
        value["assets"] = serde_json::json!([{"assetId":"listen-wav","type":"audio","path":"assets/listen.wav","sha256":crate::sha256::bytes(bytes)}]);
        value["stages"][2]["payload"]["segments"][0]["audioAssetId"] =
            serde_json::json!("listen-wav");
        fs::write(
            package.join("lesson.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        let database_path = root.join("test.sqlite3");
        database::migrate(&database_path).unwrap();
        let registry = InteractiveLessonContentRegistry::load(content.clone());
        let placement = PlacementRepository::new(database_path.clone()).unwrap();
        let profiles = StudentProfileRepository::new(database_path.clone(), placement);
        let assets = root.join("session-assets");
        let engine = InteractiveLessonEngine::new(
            registry,
            InteractiveLessonRepository::new(database_path),
            profiles,
            assets.clone(),
        );
        let session = engine
            .start(StartInteractiveLessonRequest {
                lesson_id: "audio-foundation-a1".into(),
                content_version: None,
                start_over: false,
            })
            .unwrap();
        fs::remove_dir_all(content).unwrap();
        assert_eq!(
            fs::read(assets.join(&session.id).join("listen-wav.wav")).unwrap(),
            bytes
        );
        assert_eq!(engine.resume(&session.id).unwrap().id, session.id);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn deterministic_exercise_is_private_resumable_retryable_and_has_no_score_gate() {
        let (root, content, engine) = phase_t_harness();
        let mut session = engine
            .start(StartInteractiveLessonRequest {
                lesson_id: "deterministic-exercises-a1".into(),
                content_version: None,
                start_over: false,
            })
            .unwrap();
        for stage in ["exercise-theory", "exercise-words"] {
            session = engine
                .complete(StageActionRequest {
                    session_id: session.id.clone(),
                    stage_id: stage.into(),
                })
                .unwrap();
        }
        let public = serde_json::to_string(&session.active_stage).unwrap();
        for secret in [
            "correctOptionId",
            "correctOptionIds",
            "acceptedAnswers",
            "correctOrder",
            "correctPairs",
            "That is a polite request.",
        ] {
            assert!(!public.contains(secret), "answer leak: {secret}");
        }
        fs::remove_dir_all(content).unwrap();
        let session_id = session.id.clone();
        let submit = |submission: &str, response: ExerciseResponse| SubmitExerciseAttemptRequest {
            session_id: session_id.clone(),
            stage_id: "exercise-practice".into(),
            exercise_id: "polite-choice".into(),
            submission_id: submission.into(),
            response,
        };
        session = engine
            .submit_exercise(&submit(
                "one",
                ExerciseResponse::SingleChoice(SingleChoiceResponse {
                    option_id: "a".into(),
                }),
            ))
            .unwrap();
        session = engine
            .submit_exercise(&submit(
                "one",
                ExerciseResponse::SingleChoice(SingleChoiceResponse {
                    option_id: "a".into(),
                }),
            ))
            .unwrap();
        session = engine
            .submit_exercise(&submit(
                "two",
                ExerciseResponse::SingleChoice(SingleChoiceResponse {
                    option_id: "b".into(),
                }),
            ))
            .unwrap();
        session = engine
            .submit_exercise(&submit(
                "three",
                ExerciseResponse::SingleChoice(SingleChoiceResponse {
                    option_id: "c".into(),
                }),
            ))
            .unwrap();
        let selected = latest_exercise_attempt(&session);
        session = engine
            .select_exercise(&SelectExerciseAttemptRequest {
                session_id: session.id.clone(),
                stage_id: "exercise-practice".into(),
                exercise_id: "polite-choice".into(),
                attempt_id: selected.clone(),
            })
            .unwrap();
        session = engine
            .select_exercise(&SelectExerciseAttemptRequest {
                session_id: session.id.clone(),
                stage_id: "exercise-practice".into(),
                exercise_id: "polite-choice".into(),
                attempt_id: selected,
            })
            .unwrap();
        let cases = [
            (
                "polite-multiple",
                ExerciseResponse::MultipleSelect(MultipleSelectResponse {
                    option_ids: vec!["c".into()],
                }),
            ),
            (
                "coffee-blank",
                ExerciseResponse::FillBlank(TextResponse { text: "tea".into() }),
            ),
            (
                "water-order",
                ExerciseResponse::WordOrder(WordOrderResponse {
                    token_ids: vec![
                        "t6".into(),
                        "t5".into(),
                        "t4".into(),
                        "t3".into(),
                        "t2".into(),
                        "t1".into(),
                    ],
                }),
            ),
            (
                "menu-matching",
                ExerciseResponse::Matching(MatchingResponse {
                    pairs: vec![
                        MatchingPair {
                            left_id: "l1".into(),
                            right_id: "r2".into(),
                        },
                        MatchingPair {
                            left_id: "l2".into(),
                            right_id: "r3".into(),
                        },
                        MatchingPair {
                            left_id: "l3".into(),
                            right_id: "r1".into(),
                        },
                    ],
                }),
            ),
            (
                "cold-exact",
                ExerciseResponse::ShortAnswerExact(TextResponse {
                    text: "warm".into(),
                }),
            ),
        ];
        for (index, (exercise_id, response)) in cases.into_iter().enumerate() {
            session = engine
                .submit_exercise(&SubmitExerciseAttemptRequest {
                    session_id: session.id.clone(),
                    stage_id: "exercise-practice".into(),
                    exercise_id: exercise_id.into(),
                    submission_id: format!("wrong-{index}"),
                    response,
                })
                .unwrap();
            let attempt = latest_exercise_attempt(&session);
            session = engine
                .select_exercise(&SelectExerciseAttemptRequest {
                    session_id: session.id.clone(),
                    stage_id: "exercise-practice".into(),
                    exercise_id: exercise_id.into(),
                    attempt_id: attempt,
                })
                .unwrap();
        }
        let ActiveStageContentDto::Exercise { stage } =
            &session.active_stage.as_ref().unwrap().content
        else {
            panic!()
        };
        let summary = stage.summary.as_ref().unwrap();
        assert_eq!(summary.selected_correct_count, 0);
        assert_eq!(summary.accuracy_percent, 0);
        assert_eq!(summary.total_attempt_count, 8);
        let resumed = engine.resume(&session.id).unwrap();
        assert_eq!(resumed.id, session.id);
        session = engine
            .complete(StageActionRequest {
                session_id: session.id.clone(),
                stage_id: "exercise-practice".into(),
            })
            .unwrap();
        assert_eq!(session.status, InteractiveSessionStatus::Completed);
        let connection = Connection::open(root.join("test.sqlite3")).unwrap();
        let selected_correct:i64=connection.query_row("SELECT COUNT(*) FROM interactive_lesson_exercise_attempt WHERE selected=1 AND correct=1",[],|row|row.get(0)).unwrap();
        assert_eq!(selected_correct, 0);
        for table in [
            "lesson",
            "lesson_analysis",
            "vocabulary_item",
            "recurring_mistake",
            "gamification_xp_event",
            "achievement_unlock",
            "review_session",
            "pronunciation_attempt",
            "voice_turn_performance",
        ] {
            let count: i64 = connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(count, 0, "unexpected Exercise side effect in {table}");
        }
        let completion:String=connection.query_row("SELECT completion_json FROM interactive_lesson_stage_state WHERE session_id=?1 AND stage_id='exercise-practice'",[&session.id],|row|row.get(0)).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&completion).unwrap()["accuracyPercent"],
            0
        );
        assert!(!completion.contains("passed"));
        assert!(!completion.contains("failed"));
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }
}
