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
}

impl InteractiveLessonEngine {
    pub fn new(
        content: InteractiveLessonContentRegistry,
        repository: InteractiveLessonRepository,
        profiles: StudentProfileRepository,
    ) -> Self {
        Self {
            content,
            repository,
            profiles,
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
        let lesson = self
            .content
            .get(&request.lesson_id)
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
        self.repository.start(&lesson, &context, request.start_over)
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
}

#[cfg(test)]
mod tests {
    use super::*;
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
        );
        (root, content, engine)
    }

    #[test]
    fn physical_session_resumes_from_snapshot_and_has_no_downstream_side_effects() {
        let (root, content, engine) = harness();
        let request = || StartInteractiveLessonRequest {
            lesson_id: "everyday-greetings-a1".into(),
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
                start_over: false,
            })
            .unwrap();
        let second = engine
            .start(StartInteractiveLessonRequest {
                lesson_id: "everyday-greetings-a1".into(),
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
}
