use crate::{
    database,
    placement::*,
    placement_bank::PlacementBank,
    placement_evaluator::{parse_persisted, SpeakingSample, ValidatedSpeakingEvaluation},
    placement_scoring::{
        domain_state, lower_median, overall_confidence, DomainState, ScoredAnswer,
    },
};
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::path::{Path, PathBuf};

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";

#[derive(Clone)]
pub struct PlacementRepository {
    database: PathBuf,
    bank: PlacementBank,
}

impl PlacementRepository {
    pub fn new(database: PathBuf) -> Result<Self, String> {
        Ok(Self {
            database,
            bank: PlacementBank::load()?,
        })
    }
    pub fn database_path(&self) -> &Path {
        &self.database
    }
    pub fn overview(&self) -> Result<PlacementOverviewDto, String> {
        let c = database::open(&self.database)?;
        let active = self.active_with(&c)?;
        let current = self.current_result_with(&c)?;
        let count = c
            .query_row("SELECT COUNT(*) FROM placement_attempt", [], |r| {
                r.get::<_, u32>(0)
            })
            .map_err(db)?;
        Ok(PlacementOverviewDto {
            active_attempt: active,
            current_result: current,
            attempt_count: count,
        })
    }
    pub fn start(&self, start_over: bool) -> Result<PlacementSessionDto, String> {
        let mut c = database::open(&self.database)?;
        let tx = c.transaction().map_err(db)?;
        if let Some(active) = self.active_with(&tx)? {
            if !start_over {
                return Err(format!(
                    "Placement attempt {} is already in progress.",
                    active.id
                ));
            }
            tx.execute(&format!("UPDATE placement_attempt SET status='abandoned',updated_at={NOW} WHERE id=?1 AND status='in_progress'"),[active.id]).map_err(db)?;
        }
        let id = uuid::Uuid::new_v4().to_string();
        tx.execute(&format!("INSERT INTO placement_attempt(id,status,test_version,question_bank_version,scoring_version,speaking_prompt_version,started_at,created_at,updated_at) VALUES(?1,'in_progress',?2,?3,?4,?5,{NOW},{NOW},{NOW})"),params![id,PLACEMENT_TEST_VERSION,PLACEMENT_QUESTION_BANK_VERSION,PLACEMENT_SCORING_VERSION,PLACEMENT_SPEAKING_PROMPT_VERSION]).map_err(db)?;
        tx.commit().map_err(db)?;
        self.session(&id)
    }
    pub fn abandon(&self, id: &str) -> Result<PlacementAttemptDto, String> {
        let c = database::open(&self.database)?;
        let changed=c.execute(&format!("UPDATE placement_attempt SET status='abandoned',updated_at={NOW} WHERE id=?1 AND status='in_progress'"),[id]).map_err(db)?;
        if changed != 1 {
            return Err("Only an in-progress placement can be abandoned.".into());
        }
        self.get_attempt(id)?
            .ok_or_else(|| "Placement attempt not found.".into())
    }
    pub fn session(&self, id: &str) -> Result<PlacementSessionDto, String> {
        let c = database::open(&self.database)?;
        let attempt = self
            .attempt_with(&c, id)?
            .ok_or_else(|| "Placement attempt not found.".to_owned())?;
        if attempt.status != PlacementAttemptStatus::InProgress {
            return Err("Only an in-progress placement can be resumed.".into());
        }
        self.session_with(&c, attempt)
    }
    pub fn submit_answer(
        &self,
        request: SubmitPlacementAnswerRequest,
    ) -> Result<PlacementSessionDto, String> {
        let mut c = database::open(&self.database)?;
        let tx = c.transaction().map_err(db)?;
        let attempt = self
            .attempt_with(&tx, &request.attempt_id)?
            .ok_or_else(|| "Placement attempt not found.".to_owned())?;
        if attempt.status != PlacementAttemptStatus::InProgress {
            return Err("Placement result is immutable after completion.".into());
        }
        let answers = self.answers_with(&tx, &attempt.id)?;
        let (skill, band, index) = self
            .current_objective(&attempt, &answers)?
            .ok_or_else(|| "Objective sections are already complete.".to_owned())?;
        let question = self
            .bank
            .question_at(skill, band, index)
            .ok_or_else(|| "Question bank state is invalid.".to_owned())?;
        if question.id != request.question_id {
            return Err("Submitted question is not the current placement question.".into());
        }
        if !question
            .options
            .iter()
            .any(|o| o.id == request.selected_option_id)
        {
            return Err("Selected option does not exist.".into());
        }
        tx.execute(&format!("INSERT INTO placement_answer(id,attempt_id,question_id,skill,cefr_band,selected_option_id,is_correct,answered_at) VALUES(?1,?2,?3,?4,?5,?6,?7,{NOW})"),params![uuid::Uuid::new_v4().to_string(),attempt.id,question.id,skill.as_str(),band.as_str(),request.selected_option_id,question.correct_option_id==request.selected_option_id]).map_err(|e|if e.to_string().contains("UNIQUE"){"This question has already been answered.".into()}else{db(e)})?;
        let updated = self.answers_with(&tx, &attempt.id)?;
        if let DomainState::Complete(level, _) = domain_state(skill, &updated) {
            let column = match skill {
                PlacementSkill::Grammar => "grammar_level",
                PlacementSkill::Vocabulary => "vocabulary_level",
                PlacementSkill::Reading => "reading_level",
            };
            tx.execute(
                &format!("UPDATE placement_attempt SET {column}=?1,updated_at={NOW} WHERE id=?2"),
                params![level.as_str(), attempt.id],
            )
            .map_err(db)?;
        }
        tx.commit().map_err(db)?;
        self.session(&request.attempt_id)
    }
    pub fn confirm_speaking(
        &self,
        request: ConfirmSpeakingResponseRequest,
    ) -> Result<PlacementSessionDto, String> {
        let transcript = normalize_transcript(&request.transcript)?;
        let mut c = database::open(&self.database)?;
        let tx = c.transaction().map_err(db)?;
        let attempt = self
            .attempt_with(&tx, &request.attempt_id)?
            .ok_or_else(|| "Placement attempt not found.".to_owned())?;
        if attempt.status != PlacementAttemptStatus::InProgress
            || attempt.grammar_level.is_none()
            || attempt.vocabulary_level.is_none()
            || attempt.reading_level.is_none()
        {
            return Err("Speaking responses are not available yet.".into());
        }
        let responses = self.responses_with(&tx, &attempt.id)?;
        let prompt = self
            .bank
            .speaking_prompt(responses.len())
            .ok_or_else(|| "All speaking prompts are already confirmed.".to_owned())?;
        if prompt.id != request.prompt_id {
            return Err("Speaking prompt is not current.".into());
        }
        let words = word_count(&transcript);
        tx.execute(&format!("INSERT INTO placement_speaking_response(id,attempt_id,prompt_id,prompt_version,prompt_text_snapshot,sequence_index,transcript,word_count,status,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,'confirmed',{NOW})"),params![uuid::Uuid::new_v4().to_string(),attempt.id,prompt.id,PLACEMENT_SPEAKING_PROMPT_VERSION,prompt.text,prompt.sequence_index,transcript,words]).map_err(db)?;
        tx.commit().map_err(db)?;
        self.session(&request.attempt_id)
    }
    pub fn skip_speaking(&self, id: &str) -> Result<PlacementSessionDto, String> {
        let c = database::open(&self.database)?;
        let attempt = self
            .get_attempt(id)?
            .ok_or_else(|| "Placement attempt not found.".to_owned())?;
        if attempt.status != PlacementAttemptStatus::InProgress
            || attempt.grammar_level.is_none()
            || attempt.vocabulary_level.is_none()
            || attempt.reading_level.is_none()
        {
            return Err("Speaking cannot be skipped before objective sections complete.".into());
        }
        c.execute(&format!("UPDATE placement_attempt SET speaking_status='skipped',updated_at={NOW} WHERE id=?1"),[id]).map_err(db)?;
        self.session(id)
    }
    pub fn speaking_samples(&self, id: &str) -> Result<Vec<SpeakingSample>, String> {
        let c = database::open(&self.database)?;
        Ok(self
            .responses_with(&c, id)?
            .into_iter()
            .map(|r| SpeakingSample {
                prompt: r.1,
                transcript: r.2.transcript,
            })
            .collect())
    }
    pub fn has_minimum_speaking_data(&self, id: &str) -> Result<bool, String> {
        let c = database::open(&self.database)?;
        let r = self.responses_with(&c, id)?;
        Ok(r.len() >= MINIMUM_SPEAKING_RESPONSES
            && r.iter().map(|x| x.2.word_count).sum::<u32>() >= MINIMUM_SPEAKING_WORDS)
    }
    pub fn finalize(
        &self,
        id: &str,
        evaluation: Option<ValidatedSpeakingEvaluation>,
    ) -> Result<PlacementResultDto, String> {
        let mut c = database::open(&self.database)?;
        let tx = c.transaction().map_err(db)?;
        let attempt = self
            .attempt_with(&tx, id)?
            .ok_or_else(|| "Placement attempt not found.".to_owned())?;
        if attempt.status != PlacementAttemptStatus::InProgress {
            return Err("Only an in-progress placement can be finalized.".into());
        }
        let objective = [
            attempt.grammar_level,
            attempt.vocabulary_level,
            attempt.reading_level,
        ];
        if objective.iter().any(Option::is_none) {
            return Err("Objective placement domains are incomplete.".into());
        }
        let speaking_status = if evaluation.is_some() {
            PlacementSpeakingStatus::Completed
        } else if attempt.speaking_status == PlacementSpeakingStatus::Skipped {
            PlacementSpeakingStatus::Skipped
        } else {
            PlacementSpeakingStatus::Unavailable
        };
        let spoken = evaluation.as_ref().map(|e| e.payload.estimated_band);
        let mut levels = objective.into_iter().flatten().collect::<Vec<_>>();
        if let Some(v) = spoken {
            levels.push(v);
        }
        let overall =
            lower_median(&levels).ok_or_else(|| "No placement domains available.".to_owned())?;
        let weak = self.objective_weak_boundary_with(&tx, id)?;
        let confidence = overall_confidence(&levels, spoken.is_some(), weak);
        let raw = evaluation.as_ref().map(|e| e.canonical_json.as_str());
        tx.execute(&format!("UPDATE placement_attempt SET status='completed',completed_at={NOW},spoken_production_level=?1,overall_estimated_level=?2,confidence=?3,speaking_status=?4,speaking_evaluator_version=?5,speaking_schema_version=?6,speaking_evaluator_json=?7,updated_at={NOW} WHERE id=?8"),params![spoken.map(CefrBand::as_str),overall.as_str(),confidence.as_str(),speaking_status.as_str(),evaluation.as_ref().map(|_|PLACEMENT_SPEAKING_EVALUATOR_VERSION),evaluation.as_ref().map(|_|PLACEMENT_SPEAKING_SCHEMA_VERSION),raw,id]).map_err(db)?;
        tx.commit().map_err(db)?;
        self.result(id)?
            .ok_or_else(|| "Placement result could not be read back.".into())
    }
    pub fn get_attempt(&self, id: &str) -> Result<Option<PlacementAttemptDto>, String> {
        let c = database::open(&self.database)?;
        self.attempt_with(&c, id)
    }
    pub fn result(&self, id: &str) -> Result<Option<PlacementResultDto>, String> {
        let c = database::open(&self.database)?;
        self.result_with(&c, id)
    }
    pub fn current_result(&self) -> Result<Option<PlacementResultDto>, String> {
        let c = database::open(&self.database)?;
        self.current_result_with(&c)
    }
    pub fn list_attempts(&self) -> Result<Vec<PlacementAttemptDto>, String> {
        let c = database::open(&self.database)?;
        let mut s=c.prepare("SELECT id,status,test_version,question_bank_version,scoring_version,speaking_prompt_version,speaking_evaluator_version,speaking_schema_version,started_at,completed_at,grammar_level,vocabulary_level,reading_level,spoken_production_level,overall_estimated_level,confidence,speaking_status,error_message FROM placement_attempt ORDER BY started_at DESC").map_err(db)?;
        let result = s
            .query_map([], map_attempt)
            .map_err(db)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db);
        result
    }

    fn active_with(&self, c: &Connection) -> Result<Option<PlacementAttemptDto>, String> {
        c.query_row("SELECT id,status,test_version,question_bank_version,scoring_version,speaking_prompt_version,speaking_evaluator_version,speaking_schema_version,started_at,completed_at,grammar_level,vocabulary_level,reading_level,spoken_production_level,overall_estimated_level,confidence,speaking_status,error_message FROM placement_attempt WHERE status='in_progress' ORDER BY started_at DESC LIMIT 1",[],map_attempt).optional().map_err(db)
    }
    fn attempt_with(
        &self,
        c: &Connection,
        id: &str,
    ) -> Result<Option<PlacementAttemptDto>, String> {
        c.query_row("SELECT id,status,test_version,question_bank_version,scoring_version,speaking_prompt_version,speaking_evaluator_version,speaking_schema_version,started_at,completed_at,grammar_level,vocabulary_level,reading_level,spoken_production_level,overall_estimated_level,confidence,speaking_status,error_message FROM placement_attempt WHERE id=?1",[id],map_attempt).optional().map_err(db)
    }
    fn answers_with(&self, c: &Connection, id: &str) -> Result<Vec<ScoredAnswer>, String> {
        let mut s=c.prepare("SELECT skill,cefr_band,is_correct FROM placement_answer WHERE attempt_id=?1 ORDER BY answered_at,id").map_err(db)?;
        let result = s
            .query_map([id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, bool>(2)?,
                ))
            })
            .map_err(db)?
            .map(|v| {
                let (s, b, c) = v.map_err(db)?;
                Ok(ScoredAnswer {
                    skill: PlacementSkill::parse(&s)?,
                    band: CefrBand::parse(&b)?,
                    correct: c,
                })
            })
            .collect();
        result
    }
    fn responses_with(
        &self,
        c: &Connection,
        id: &str,
    ) -> Result<Vec<(String, String, PlacementSpeakingResponseDto)>, String> {
        let mut s=c.prepare("SELECT id,prompt_id,prompt_version,prompt_text_snapshot,sequence_index,transcript,word_count,status,created_at FROM placement_speaking_response WHERE attempt_id=?1 ORDER BY sequence_index").map_err(db)?;
        let result = s
            .query_map([id], |r| {
                Ok((
                    r.get::<_, String>(3)?,
                    PlacementSpeakingResponseDto {
                        id: r.get(0)?,
                        prompt_id: r.get(1)?,
                        prompt_version: r.get(2)?,
                        sequence_index: r.get(4)?,
                        transcript: r.get(5)?,
                        word_count: r.get(6)?,
                        status: r.get(7)?,
                        created_at: r.get(8)?,
                    },
                ))
            })
            .map_err(db)?
            .map(|v| {
                let (p, d) = v.map_err(db)?;
                Ok((d.prompt_id.clone(), p, d))
            })
            .collect();
        result
    }
    fn current_objective(
        &self,
        attempt: &PlacementAttemptDto,
        answers: &[ScoredAnswer],
    ) -> Result<Option<(PlacementSkill, CefrBand, usize)>, String> {
        for skill in PlacementSkill::ALL {
            let stored = match skill {
                PlacementSkill::Grammar => attempt.grammar_level,
                PlacementSkill::Vocabulary => attempt.vocabulary_level,
                PlacementSkill::Reading => attempt.reading_level,
            };
            if stored.is_none() {
                return match domain_state(skill, answers) {
                    DomainState::Question(b, i) => Ok(Some((skill, b, i))),
                    DomainState::Complete(_, _) => {
                        Err("Completed domain was not persisted.".into())
                    }
                };
            }
        }
        Ok(None)
    }
    fn session_with(
        &self,
        c: &Connection,
        attempt: PlacementAttemptDto,
    ) -> Result<PlacementSessionDto, String> {
        let answers = self.answers_with(c, &attempt.id)?;
        let responses = self.responses_with(c, &attempt.id)?;
        let current = self.current_objective(&attempt, &answers)?;
        let question = current
            .and_then(|(s, b, i)| self.bank.question_at(s, b, i))
            .map(|q| self.bank.public_question(q));
        let speaking_prompt =
            if current.is_none() && attempt.speaking_status == PlacementSpeakingStatus::Pending {
                self.bank
                    .speaking_prompt(responses.len())
                    .map(|p| self.bank.public_speaking_prompt(p))
            } else {
                None
            };
        let phase = if current.is_some() {
            "objective"
        } else if attempt.speaking_status == PlacementSpeakingStatus::Pending {
            "speaking"
        } else {
            "ready_to_finalize"
        };
        let domains = PlacementSkill::ALL
            .into_iter()
            .map(|skill| {
                let level = match skill {
                    PlacementSkill::Grammar => attempt.grammar_level,
                    PlacementSkill::Vocabulary => attempt.vocabulary_level,
                    PlacementSkill::Reading => attempt.reading_level,
                };
                let count = answers.iter().filter(|a| a.skill == skill).count() as u32;
                let status = if level.is_some() {
                    "complete"
                } else if current.is_some_and(|c| c.0 == skill) {
                    "in_progress"
                } else {
                    "pending"
                };
                PlacementDomainProgressDto {
                    skill: skill.as_str().into(),
                    status: status.into(),
                    estimated_level: level,
                    answered_questions: count,
                }
            })
            .collect();
        Ok(PlacementSessionDto {
            attempt,
            progress: PlacementProgressDto {
                domains,
                phase: phase.into(),
                speaking_responses: responses.len() as u32,
                speaking_word_count: responses.iter().map(|r| r.2.word_count).sum(),
            },
            question,
            speaking_prompt,
        })
    }
    fn result_with(&self, c: &Connection, id: &str) -> Result<Option<PlacementResultDto>, String> {
        let Some(a) = self.attempt_with(c, id)? else {
            return Ok(None);
        };
        if a.status != PlacementAttemptStatus::Completed {
            return Ok(None);
        };
        let overall = a
            .overall_estimated_level
            .ok_or_else(|| "Completed placement has no overall estimate.".to_owned())?;
        let confidence = a
            .confidence
            .ok_or_else(|| "Completed placement has no confidence.".to_owned())?;
        let raw: OptionalText = c
            .query_row(
                "SELECT speaking_evaluator_json FROM placement_attempt WHERE id=?1",
                [id],
                |r| r.get(0),
            )
            .map_err(db)?;
        let parsed = raw.as_deref().map(parse_persisted).transpose()?;
        let evidence = parsed
            .as_ref()
            .map_or_else(Vec::new, ValidatedSpeakingEvaluation::evidence_dtos);
        let summary = parsed.map(|v| v.payload.summary);
        let domains = vec![
            PlacementDomainResultDto {
                skill: "grammar".into(),
                level: a.grammar_level,
                assessed: true,
            },
            PlacementDomainResultDto {
                skill: "vocabulary".into(),
                level: a.vocabulary_level,
                assessed: true,
            },
            PlacementDomainResultDto {
                skill: "reading".into(),
                level: a.reading_level,
                assessed: true,
            },
            PlacementDomainResultDto {
                skill: "spoken_production".into(),
                level: a.spoken_production_level,
                assessed: a.spoken_production_level.is_some(),
            },
        ];
        Ok(Some(PlacementResultDto {
            attempt: a,
            estimated_cefr_level: overall,
            confidence,
            domains,
            speaking_evidence: evidence,
            speaking_summary: summary,
            listening_assessed: false,
            pronunciation_assessed: false,
            writing_assessed: false,
            disclaimer: PLACEMENT_DISCLAIMER.into(),
        }))
    }
    fn current_result_with(&self, c: &Connection) -> Result<Option<PlacementResultDto>, String> {
        let id: OptionalText = c.query_row("SELECT id FROM placement_attempt WHERE status='completed' ORDER BY completed_at DESC LIMIT 1",[],|r|r.get(0)).optional().map_err(db)?.flatten();
        match id {
            Some(v) => self.result_with(c, &v),
            None => Ok(None),
        }
    }
    fn objective_weak_boundary_with(&self, c: &Connection, id: &str) -> Result<bool, String> {
        let a = self.answers_with(c, id)?;
        Ok(PlacementSkill::ALL
            .into_iter()
            .any(|s| matches!(domain_state(s, &a), DomainState::Complete(_, false))))
    }
}

type OptionalText = Option<String>;
fn map_attempt(r: &Row<'_>) -> rusqlite::Result<PlacementAttemptDto> {
    let parse_band = |v: Option<String>| {
        v.map(|x| CefrBand::parse(&x).map_err(|_| rusqlite::Error::InvalidQuery))
            .transpose()
    };
    Ok(PlacementAttemptDto {
        id: r.get(0)?,
        status: PlacementAttemptStatus::parse(&r.get::<_, String>(1)?)
            .map_err(|_| rusqlite::Error::InvalidQuery)?,
        test_version: r.get(2)?,
        question_bank_version: r.get(3)?,
        scoring_version: r.get(4)?,
        speaking_prompt_version: r.get(5)?,
        speaking_evaluator_version: r.get(6)?,
        speaking_schema_version: r.get(7)?,
        started_at: r.get(8)?,
        completed_at: r.get(9)?,
        grammar_level: parse_band(r.get(10)?)?,
        vocabulary_level: parse_band(r.get(11)?)?,
        reading_level: parse_band(r.get(12)?)?,
        spoken_production_level: parse_band(r.get(13)?)?,
        overall_estimated_level: parse_band(r.get(14)?)?,
        confidence: r
            .get::<_, Option<String>>(15)?
            .map(|v| PlacementConfidence::parse(&v).map_err(|_| rusqlite::Error::InvalidQuery))
            .transpose()?,
        speaking_status: PlacementSpeakingStatus::parse(&r.get::<_, String>(16)?)
            .map_err(|_| rusqlite::Error::InvalidQuery)?,
        error_message: r.get(17)?,
    })
}
fn db(e: rusqlite::Error) -> String {
    format!("Placement database error: {e}")
}
pub fn word_count(value: &str) -> u32 {
    value
        .split_whitespace()
        .filter(|w| w.chars().any(char::is_alphabetic))
        .count() as u32
}
pub fn normalize_transcript(value: &str) -> Result<String, String> {
    let clean = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.is_empty() || clean.chars().count() > 4000 {
        return Err("Speaking transcript is empty or too long.".into());
    }
    let upper = clean.to_ascii_uppercase();
    let technical = [
        "[INAUDIBLE]",
        "[SILENCE]",
        "[BLANK_AUDIO]",
        "[BLANK AUDIO]",
        "[MUSIC]",
        "[NO SPEECH]",
    ];
    if technical.iter().any(|t| upper == *t) || word_count(&clean) == 0 {
        return Err("No valid speech was recognized. Please record again.".into());
    }
    Ok(clean)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn repo() -> (std::path::PathBuf, PlacementRepository) {
        let d = std::env::temp_dir().join(format!("placement-repo-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("db.sqlite3");
        database::migrate(&p).unwrap();
        let r = PlacementRepository::new(p).unwrap();
        (d, r)
    }
    fn answer_current(r: &PlacementRepository, id: &str, correct: bool) {
        let s = r.session(id).unwrap();
        let q = s.question.unwrap();
        let internal = r.bank.question(&q.question_id).unwrap();
        let choice = if correct {
            internal.correct_option_id.clone()
        } else {
            internal
                .options
                .iter()
                .find(|o| o.id != internal.correct_option_id)
                .unwrap()
                .id
                .clone()
        };
        r.submit_answer(SubmitPlacementAnswerRequest {
            attempt_id: id.into(),
            question_id: q.question_id,
            selected_option_id: choice,
        })
        .unwrap();
    }
    fn complete_objectives(r: &PlacementRepository, id: &str) {
        while r.session(id).unwrap().question.is_some() {
            answer_current(r, id, true);
        }
    }
    #[test]
    fn resume_reconstructs_exact_next_question() {
        let (d, r) = repo();
        let s = r.start(false).unwrap();
        answer_current(&r, &s.attempt.id, true);
        let reopened = PlacementRepository::new(r.database.clone()).unwrap();
        let resumed = reopened.session(&s.attempt.id).unwrap();
        assert_eq!(resumed.progress.domains[0].answered_questions, 1);
        assert_eq!(resumed.question.unwrap().question_id, "grammar-b1-2");
        std::fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn start_over_abandons_and_retake_preserves_old() {
        let (d, r) = repo();
        let first = r.start(false).unwrap().attempt.id;
        let second = r.start(true).unwrap().attempt.id;
        assert_eq!(
            r.get_attempt(&first).unwrap().unwrap().status,
            PlacementAttemptStatus::Abandoned
        );
        assert_ne!(first, second);
        assert_eq!(r.list_attempts().unwrap().len(), 2);
        std::fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn duplicate_or_wrong_question_is_rejected() {
        let (d, r) = repo();
        let s = r.start(false).unwrap();
        let q = s.question.unwrap();
        assert!(r
            .submit_answer(SubmitPlacementAnswerRequest {
                attempt_id: s.attempt.id.clone(),
                question_id: "wrong".into(),
                selected_option_id: "a".into()
            })
            .is_err());
        let key = r
            .bank
            .question(&q.question_id)
            .unwrap()
            .correct_option_id
            .clone();
        r.submit_answer(SubmitPlacementAnswerRequest {
            attempt_id: s.attempt.id.clone(),
            question_id: q.question_id.clone(),
            selected_option_id: key.clone(),
        })
        .unwrap();
        assert!(r
            .submit_answer(SubmitPlacementAnswerRequest {
                attempt_id: s.attempt.id,
                question_id: q.question_id,
                selected_option_id: key
            })
            .is_err());
        std::fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn technical_transcripts_do_not_count() {
        for v in ["[INAUDIBLE]", " [silence] ", "[BLANK_AUDIO]"] {
            assert!(normalize_transcript(v).is_err());
        }
        assert_eq!(word_count("Hello, world! 123"), 2);
    }

    #[test]
    fn completed_result_is_immutable_and_remains_current_during_retake() {
        let (d, r) = repo();
        let first = r.start(false).unwrap().attempt.id;
        complete_objectives(&r, &first);
        r.skip_speaking(&first).unwrap();
        let result = r.finalize(&first, None).unwrap();
        assert_eq!(result.estimated_cefr_level, CefrBand::C2);
        assert_eq!(result.confidence, PlacementConfidence::Low);
        let second = r.start(false).unwrap().attempt.id;
        assert_ne!(first, second);
        assert_eq!(r.current_result().unwrap().unwrap().attempt.id, first);
        assert!(r.finalize(&first, None).is_err());
        std::fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn speaking_minimum_requires_two_responses_and_forty_words() {
        let (d, r) = repo();
        let id = r.start(false).unwrap().attempt.id;
        complete_objectives(&r, &id);
        let prompt = r.session(&id).unwrap().speaking_prompt.unwrap();
        r.confirm_speaking(ConfirmSpeakingResponseRequest { attempt_id:id.clone(), prompt_id:prompt.prompt_id, transcript:"This is a valid but deliberately short response about a familiar activity and why it is useful.".into() }).unwrap();
        assert!(!r.has_minimum_speaking_data(&id).unwrap());
        std::fs::remove_dir_all(d).unwrap();
    }

    #[test]
    #[ignore = "manual resume/abandon audit against the user's physical SQLite database"]
    fn physical_phase_j_resume_without_creating_a_cefr_result() {
        let database = PathBuf::from(std::env::var_os("LOCALAPPDATA").expect("LOCALAPPDATA"))
            .join("com.englishaicoach.desktop")
            .join("database")
            .join("english-ai-coach.sqlite3");
        database::migrate(&database).expect("migrate physical database");
        let repository = PlacementRepository::new(database.clone()).expect("repository");
        if let Some(active) = repository.overview().unwrap().active_attempt {
            repository
                .abandon(&active.id)
                .expect("abandon prior test-only attempt");
        }
        let current_before = repository.current_result().unwrap().map(|r| r.attempt.id);
        let started = repository.start(false).expect("start physical attempt");
        assert_eq!(
            started.question.as_ref().unwrap().question_id,
            "grammar-b1-1"
        );
        drop(repository);
        let reopened = PlacementRepository::new(database).expect("reopen repository");
        let resumed = reopened
            .session(&started.attempt.id)
            .expect("resume physical attempt");
        assert_eq!(
            resumed.question.as_ref().unwrap().question_id,
            "grammar-b1-1"
        );
        reopened
            .abandon(&started.attempt.id)
            .expect("abandon test-only attempt");
        assert_eq!(
            reopened.current_result().unwrap().map(|r| r.attempt.id),
            current_before
        );
    }
}
