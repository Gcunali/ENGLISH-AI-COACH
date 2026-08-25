use crate::{
    lesson_modes::ValidatedLessonConfiguration,
    lesson_repository::{
        CorrectionCandidate, Lesson, LessonRepository, LessonSummary, NewLesson, TranscriptMessage,
    },
    voice_engine::VoiceEngineEvent,
};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone)]
pub struct LessonSessionManager {
    repository: LessonRepository,
    active: Arc<Mutex<Option<ActiveLesson>>>,
}

#[derive(Clone, Debug)]
struct ActiveLesson {
    lesson_id: String,
    pending_student_message_id: Option<String>,
}

impl LessonSessionManager {
    pub fn new(repository: LessonRepository) -> Self {
        Self {
            repository,
            active: Arc::new(Mutex::new(None)),
        }
    }

    pub fn begin_lesson(&self, metadata: &NewLesson) -> Result<Lesson, String> {
        let mut active = self.lock()?;
        if let Some(current) = active.as_ref() {
            return Err(format!(
                "Lesson {} is already active. End it before starting another lesson.",
                current.lesson_id
            ));
        }
        let lesson = self.repository.create_lesson(metadata)?;
        *active = Some(ActiveLesson {
            lesson_id: lesson.id.clone(),
            pending_student_message_id: None,
        });
        Ok(lesson)
    }

    pub fn begin_configured_lesson(
        &self,
        metadata: &NewLesson,
        configuration: &ValidatedLessonConfiguration,
    ) -> Result<Lesson, String> {
        let mut active = self.lock()?;
        if let Some(current) = active.as_ref() {
            return Err(format!(
                "Lesson {} is already active. End it before starting another lesson.",
                current.lesson_id
            ));
        }
        let lesson = self
            .repository
            .create_configured_lesson(metadata, configuration)?;
        *active = Some(ActiveLesson {
            lesson_id: lesson.id.clone(),
            pending_student_message_id: None,
        });
        Ok(lesson)
    }

    pub fn enrich_event(
        &self,
        lesson_id: &str,
        event: &mut VoiceEngineEvent,
    ) -> Result<(), String> {
        let mut active = self.lock()?;
        let session = active
            .as_mut()
            .filter(|session| session.lesson_id == lesson_id)
            .ok_or_else(|| "Voice event does not belong to the active lesson.".to_owned())?;

        match event {
            VoiceEngineEvent::EngineStarted => {
                self.repository.mark_lesson_active(lesson_id)?;
            }
            VoiceEngineEvent::Transcript { text, message } => {
                let persisted = self.repository.insert_student_message(lesson_id, text)?;
                session.pending_student_message_id =
                    persisted.as_ref().map(|message| message.id.clone());
                *message = persisted;
            }
            VoiceEngineEvent::TeacherResponse {
                text,
                message,
                correction_candidate,
            } => {
                let Some(student_message_id) = session.pending_student_message_id.take() else {
                    return Ok(());
                };
                let persisted = self.repository.insert_teacher_response(
                    lesson_id,
                    &student_message_id,
                    text,
                )?;
                *message = Some(persisted.message);
                *correction_candidate = persisted.correction_candidate;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn complete_lesson(&self, lesson_id: &str) -> Result<LessonSummary, String> {
        self.finish_active(lesson_id, |repository| {
            repository.complete_lesson(lesson_id)
        })
    }

    pub fn interrupt_lesson(&self, lesson_id: &str) -> Result<LessonSummary, String> {
        self.finish_active(lesson_id, |repository| {
            repository.interrupt_lesson(lesson_id)
        })
    }

    pub fn fail_lesson(&self, lesson_id: &str, error: &str) -> Result<LessonSummary, String> {
        self.finish_active(lesson_id, |repository| {
            repository.fail_lesson(lesson_id, error)
        })
    }

    pub fn interrupt_active(&self) -> Result<Option<LessonSummary>, String> {
        let lesson_id = self
            .lock()?
            .as_ref()
            .map(|session| session.lesson_id.clone());
        lesson_id
            .map(|lesson_id| self.interrupt_lesson(&lesson_id))
            .transpose()
    }

    pub fn get_active_lesson(&self) -> Result<Option<Lesson>, String> {
        let lesson_id = self
            .lock()?
            .as_ref()
            .map(|session| session.lesson_id.clone());
        lesson_id
            .map(|lesson_id| self.repository.get_lesson(&lesson_id))
            .transpose()
            .map(Option::flatten)
    }

    pub fn get_lesson(&self, lesson_id: &str) -> Result<Option<Lesson>, String> {
        self.repository.get_lesson(lesson_id)
    }

    pub fn get_latest_completed_lesson(&self) -> Result<Option<Lesson>, String> {
        self.repository.get_latest_completed_lesson()
    }

    pub fn get_messages(&self, lesson_id: &str) -> Result<Vec<TranscriptMessage>, String> {
        self.repository.get_lesson_messages(lesson_id)
    }

    pub fn get_corrections(&self, lesson_id: &str) -> Result<Vec<CorrectionCandidate>, String> {
        self.repository.get_correction_candidates(lesson_id)
    }

    fn finish_active<F>(&self, lesson_id: &str, finish: F) -> Result<LessonSummary, String>
    where
        F: FnOnce(&LessonRepository) -> Result<LessonSummary, String>,
    {
        let mut active = self.lock()?;
        if active.as_ref().map(|session| session.lesson_id.as_str()) != Some(lesson_id) {
            return self
                .repository
                .get_lesson_summary(lesson_id)?
                .ok_or_else(|| "Lesson was not found.".to_owned());
        }
        let summary = finish(&self.repository)?;
        *active = None;
        Ok(summary)
    }

    fn lock(&self) -> Result<MutexGuard<'_, Option<ActiveLesson>>, String> {
        self.active
            .lock()
            .map_err(|_| "Active lesson lock is unavailable.".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{database, lesson_repository::LessonStatus};

    fn manager() -> (std::path::PathBuf, LessonSessionManager) {
        let directory =
            std::env::temp_dir().join(format!("english-ai-coach-session-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let database_path = directory.join("session.sqlite3");
        database::migrate(&database_path).unwrap();
        let repository = LessonRepository::new(database_path);
        (directory, LessonSessionManager::new(repository))
    }

    fn metadata() -> NewLesson {
        NewLesson {
            topic: None,
            mode: "free_conversation".to_owned(),
            whisper_model: "whisper.bin".to_owned(),
            whisper_threads: 12,
            ollama_model: "qwen".to_owned(),
            piper_voice: "lessac".to_owned(),
            voice_engine_version: "voice_v2_bridge_v1".to_owned(),
        }
    }

    #[test]
    fn event_pipeline_persists_before_exposing_messages() {
        let (directory, manager) = manager();
        let lesson = manager.begin_lesson(&metadata()).unwrap();
        let mut started = VoiceEngineEvent::EngineStarted;
        manager.enrich_event(&lesson.id, &mut started).unwrap();
        let mut transcript = VoiceEngineEvent::Transcript {
            text: "I like cook.".to_owned(),
            message: None,
        };
        manager.enrich_event(&lesson.id, &mut transcript).unwrap();
        assert!(matches!(
            transcript,
            VoiceEngineEvent::Transcript {
                message: Some(_),
                ..
            }
        ));
        let mut response = VoiceEngineEvent::TeacherResponse {
            text: "A more natural way to say that is 'I like cooking.'".to_owned(),
            message: None,
            correction_candidate: None,
        };
        manager.enrich_event(&lesson.id, &mut response).unwrap();
        assert!(matches!(
            response,
            VoiceEngineEvent::TeacherResponse {
                message: Some(_),
                correction_candidate: Some(_),
                ..
            }
        ));
        let summary = manager.complete_lesson(&lesson.id).unwrap();
        assert_eq!(summary.status, LessonStatus::Completed);
        assert_eq!(summary.student_turns, 1);
        assert_eq!(summary.teacher_turns, 1);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn technical_transcript_cannot_attach_teacher_to_a_previous_turn() {
        let (directory, manager) = manager();
        let lesson = manager.begin_lesson(&metadata()).unwrap();
        let mut started = VoiceEngineEvent::EngineStarted;
        manager.enrich_event(&lesson.id, &mut started).unwrap();
        let mut technical = VoiceEngineEvent::Transcript {
            text: "[INAUDIBLE]".to_owned(),
            message: None,
        };
        manager.enrich_event(&lesson.id, &mut technical).unwrap();
        let mut response = VoiceEngineEvent::TeacherResponse {
            text: "Could you repeat that?".to_owned(),
            message: None,
            correction_candidate: None,
        };
        manager.enrich_event(&lesson.id, &mut response).unwrap();
        assert!(manager.get_messages(&lesson.id).unwrap().is_empty());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
