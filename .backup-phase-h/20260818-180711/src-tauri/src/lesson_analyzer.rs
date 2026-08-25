use crate::{
    learning_memory_repository::LearningMemoryRepository,
    lesson_analysis::{
        parse_and_validate, LessonAnalysis, LessonAnalysisPayload, LessonAnalysisStatus,
        PedagogicalAnalysisInput, ANALYZER_SYSTEM_PROMPT, MAX_ANALYSIS_INPUT_BYTES,
        MINIMUM_STUDENT_TURNS,
    },
    lesson_analysis_repository::LessonAnalysisRepository,
    lesson_repository::{LessonRepository, LessonStatus},
};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::{future::Future, time::Duration};

const OLLAMA_CHAT_URL: &str = "http://127.0.0.1:11434/api/chat";
const ANALYZER_TIMEOUT: Duration = Duration::from_secs(180);
const ANALYZER_TEMPERATURE: f32 = 0.1;

#[derive(Clone)]
pub struct LessonAnalyzer {
    client: Client,
    lessons: LessonRepository,
    analyses: LessonAnalysisRepository,
    memory: LearningMemoryRepository,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

impl LessonAnalyzer {
    pub fn new(
        lessons: LessonRepository,
        analyses: LessonAnalysisRepository,
        memory: LearningMemoryRepository,
    ) -> Result<Self, String> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(ANALYZER_TIMEOUT)
            .no_proxy()
            .build()
            .map_err(|error| format!("Could not create local analyzer client: {error}"))?;
        Ok(Self {
            client,
            lessons,
            analyses,
            memory,
        })
    }

    pub fn get(&self, lesson_id: &str) -> Result<Option<LessonAnalysis>, String> {
        self.analyses.get_by_lesson(lesson_id)
    }

    pub async fn analyze(&self, lesson_id: &str) -> Result<LessonAnalysis, String> {
        self.run(lesson_id, false).await
    }

    pub async fn retry(&self, lesson_id: &str) -> Result<LessonAnalysis, String> {
        self.run(lesson_id, true).await
    }

    async fn run(&self, lesson_id: &str, retry: bool) -> Result<LessonAnalysis, String> {
        let source = self.lessons.get_analysis_input(lesson_id)?;
        validate_lesson_status(source.lesson.status)?;

        if let Some(existing) = self.analyses.get_by_lesson(lesson_id)? {
            match (existing.status, retry) {
                (LessonAnalysisStatus::Completed, _) => {
                    self.sync_memory_without_blocking(&existing);
                    return Ok(existing);
                }
                (LessonAnalysisStatus::InsufficientData, _) => return Ok(existing),
                (LessonAnalysisStatus::Failed, true) => {
                    self.analyses.reset_failed_for_retry(lesson_id)?;
                }
                (LessonAnalysisStatus::Failed, false)
                | (LessonAnalysisStatus::Pending, _)
                | (LessonAnalysisStatus::Running, _) => return Ok(existing),
            }
        } else {
            if retry {
                return Err("There is no failed analysis to retry.".to_owned());
            }
            self.analyses
                .create_pending(lesson_id, &source.lesson.ollama_model)?;
        }

        if source.lesson.student_turn_count < MINIMUM_STUDENT_TURNS {
            return self
                .analyses
                .mark_insufficient_data(lesson_id, source.lesson.student_turn_count);
        }

        self.analyses.mark_running(lesson_id)?;
        let model = source.lesson.ollama_model.clone();
        let input = PedagogicalAnalysisInput::from(source);
        let input_json = match serde_json::to_string(&input) {
            Ok(value) if value.len() <= MAX_ANALYSIS_INPUT_BYTES => value,
            Ok(value) => {
                return self.fail(
                    lesson_id,
                    &format!(
                        "Lesson analysis input is too large ({} bytes; maximum {}).",
                        value.len(),
                        MAX_ANALYSIS_INPUT_BYTES
                    ),
                )
            }
            Err(error) => {
                return self.fail(
                    lesson_id,
                    &format!("Could not serialize lesson analysis input: {error}"),
                )
            }
        };

        let initial = match self.request_analysis(&model, &input_json).await {
            Ok(value) => value,
            Err(error) => return self.fail(lesson_id, &error),
        };
        let result = parse_with_one_repair(initial, &input, |invalid| {
            self.request_repair(&model, invalid)
        })
        .await;
        match result {
            Ok((payload, canonical)) => {
                let analysis = self
                    .analyses
                    .save_completed(lesson_id, &payload, &canonical)?;
                self.sync_memory_without_blocking(&analysis);
                Ok(analysis)
            }
            Err(error) => self.fail(lesson_id, &error),
        }
    }

    fn fail(&self, lesson_id: &str, error: &str) -> Result<LessonAnalysis, String> {
        self.analyses.mark_failed(lesson_id, error)
    }

    fn sync_memory_without_blocking(&self, analysis: &LessonAnalysis) {
        if let Err(error) = self.memory.sync_analysis(&analysis.id) {
            log::error!(
                "Lesson analysis {} completed, but learning memory sync failed: {}",
                analysis.id,
                error
            );
        }
    }

    async fn request_analysis(&self, model: &str, input_json: &str) -> Result<String, String> {
        let body = json!({
            "model": model,
            "stream": false,
            "think": false,
            "format": "json",
            "keep_alive": "10m",
            "options": {
                "temperature": ANALYZER_TEMPERATURE,
                "top_p": 0.9,
                "num_predict": 1100,
                "num_ctx": 8192
            },
            "messages": [
                { "role": "system", "content": ANALYZER_SYSTEM_PROMPT },
                { "role": "user", "content": input_json }
            ]
        });
        self.request(body).await
    }

    async fn request_repair(&self, model: &str, invalid: String) -> Result<String, String> {
        let repair_prompt = format!(
            "Retorne a mesma análise como JSON válido no schema obrigatório. Não adicione informação, não reavalie a aula, não altere scores deliberadamente e não use Markdown. Corrija somente a estrutura. Preserve literalmente todos os valores substantivos, inclusive scores, textos e evidências. Complete apenas propriedades obrigatórias ausentes com o valor determinado pelo contrato, como pronunciationAvailable=false e scores.pronunciation=null.\n\nSaída inválida e erro de validação:\n{invalid}\n\nSchema obrigatório:\n{}",
            required_schema(),
        );
        let body = json!({
            "model": model,
            "stream": false,
            "think": false,
            "format": "json",
            "keep_alive": "10m",
            "options": {
                "temperature": 0.0,
                "top_p": 0.9,
                "num_predict": 1100,
                "num_ctx": 8192
            },
            "messages": [
                { "role": "system", "content": "Você repara somente a estrutura de um JSON existente. Retorne apenas JSON, não adicione fatos, não reavalie o conteúdo e preserve todos os valores substantivos." },
                { "role": "user", "content": repair_prompt }
            ]
        });
        self.request(body).await
    }

    async fn request(&self, body: serde_json::Value) -> Result<String, String> {
        let request = async {
            let response = self
                .client
                .post(OLLAMA_CHAT_URL)
                .json(&body)
                .send()
                .await
                .map_err(|error| format!("Local analyzer request failed: {error}"))?;
            if !response.status().is_success() {
                let status = response.status();
                let detail = response.text().await.unwrap_or_default();
                return Err(format!(
                    "Local analyzer returned {status}: {}",
                    compact_error(&detail)
                ));
            }
            let payload: ChatResponse = response
                .json()
                .await
                .map_err(|error| format!("Invalid local analyzer response: {error}"))?;
            let content = payload.message.content.trim().to_owned();
            if content.is_empty() {
                return Err("Local analyzer returned an empty response.".to_owned());
            }
            Ok(content)
        };
        tokio::time::timeout(ANALYZER_TIMEOUT, request)
            .await
            .map_err(|_| "Local lesson analysis timed out after 180 seconds.".to_owned())?
    }
}

async fn parse_with_one_repair<F, Fut>(
    initial: String,
    input: &PedagogicalAnalysisInput,
    repair: F,
) -> Result<(LessonAnalysisPayload, String), String>
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Result<String, String>>,
{
    match parse_and_validate(&initial, input) {
        Ok(valid) => Ok(valid),
        Err(initial_error) => {
            let repair_input =
                format!("Erro de validação: {initial_error}\n\nSaída original:\n{initial}");
            let repaired = repair(repair_input).await.map_err(|repair_error| {
                format!("Initial analyzer output was invalid ({initial_error}); repair failed: {repair_error}")
            })?;
            parse_and_validate(&repaired, input).map_err(|repair_error| {
                format!(
                    "Initial analyzer output was invalid ({initial_error}); repaired output was invalid ({repair_error})"
                )
            })
        }
    }
}

fn validate_lesson_status(status: LessonStatus) -> Result<(), String> {
    if status == LessonStatus::Completed {
        Ok(())
    } else {
        Err(format!(
            "Only a completed lesson can be analyzed; current status is {}.",
            match status {
                LessonStatus::Starting => "starting",
                LessonStatus::Active => "active",
                LessonStatus::Completed => "completed",
                LessonStatus::Interrupted => "interrupted",
                LessonStatus::Failed => "failed",
            }
        ))
    }
}

fn required_schema() -> &'static str {
    r#"{"schemaVersion":1,"scores":{"fluency":<int 0-100>,"grammar":<int 0-100>,"vocabulary":<int 0-100>,"comprehension":<int 0-100>,"interaction":<int 0-100>,"pronunciation":null},"strengths":[{"title":<string>,"evidence":<string>}],"priorityImprovements":[{"area":<string>,"title":<string>,"explanation":<string>,"exampleFromLesson":<string>,"betterAlternative":<string>}],"corrections":[{"original":<string>,"corrected":<string>,"explanation":<string>,"category":<enum>}],"naturalAlternatives":[{"original":<string>,"alternative":<string>}],"vocabulary":[{"wordOrPhrase":<string>,"meaning":<string>,"example":<string>}],"recurringPatterns":[{"pattern":<string>,"count":<int >=2>,"explanation":<string>}],"nextLessonRecommendations":[<string>],"summary":<string>,"pronunciationAvailable":false}"#
}

fn compact_error(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(500)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lesson_analysis::{
        LessonAnalysisCorrection, LessonAnalysisCorrectionCategory, LessonAnalysisScores,
        LessonAnalysisVocabulary, PedagogicalLesson, PedagogicalMessage,
    };
    use crate::lesson_repository::NewLesson;
    use std::sync::{Arc, Mutex};

    fn input() -> PedagogicalAnalysisInput {
        PedagogicalAnalysisInput {
            lesson: PedagogicalLesson {
                id: "lesson".to_owned(),
                started_at: "start".to_owned(),
                ended_at: Some("end".to_owned()),
                duration_seconds: Some(60),
                student_turn_count: 3,
                teacher_turn_count: 3,
                correction_count: 1,
                whisper_model: "whisper".to_owned(),
                ollama_model: "qwen".to_owned(),
            },
            transcript: vec![PedagogicalMessage {
                sequence_index: 1,
                role: "student".to_owned(),
                text: "Today I play tennis.".to_owned(),
            }],
            correction_candidates: vec![],
        }
    }

    fn valid_json() -> String {
        serde_json::to_string(&LessonAnalysisPayload {
            schema_version: 1,
            scores: LessonAnalysisScores {
                fluency: 70,
                grammar: 60,
                vocabulary: 65,
                comprehension: 80,
                interaction: 75,
                pronunciation: None,
            },
            strengths: vec![crate::lesson_analysis::LessonAnalysisStrength {
                title: "Boa interação".to_owned(),
                evidence: "Today I play tennis.".to_owned(),
            }],
            priority_improvements: vec![
                crate::lesson_analysis::LessonAnalysisImprovement {
                    area: "grammar".to_owned(),
                    title: "Passado simples".to_owned(),
                    explanation: "Use o passado em ações concluídas.".to_owned(),
                    example_from_lesson: "Today I play tennis.".to_owned(),
                    better_alternative: "Today I played tennis.".to_owned(),
                },
            ],
            corrections: vec![LessonAnalysisCorrection {
                original: "Today I play tennis.".to_owned(),
                corrected: "Today I played tennis.".to_owned(),
                explanation: "Use o passado.".to_owned(),
                category: LessonAnalysisCorrectionCategory::VerbTense,
            }],
            natural_alternatives: vec![],
            vocabulary: vec![],
            recurring_patterns: vec![],
            next_lesson_recommendations: vec!["Praticar o passado simples.".to_owned()],
            summary: "Você manteve a interação e deve praticar o uso do passado simples em ações concluídas.".to_owned(),
            pronunciation_available: false,
        })
        .unwrap()
    }

    #[test]
    fn repair_flow_accepts_one_valid_repair() {
        let calls = Arc::new(Mutex::new(0));
        let calls_for_repair = calls.clone();
        let result = tauri::async_runtime::block_on(parse_with_one_repair(
            "invalid".to_owned(),
            &input(),
            move |_| async move {
                *calls_for_repair.lock().unwrap() += 1;
                Ok(valid_json())
            },
        ));
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(*calls.lock().unwrap(), 1);
    }

    #[test]
    fn repair_flow_fails_after_one_invalid_repair() {
        let calls = Arc::new(Mutex::new(0));
        let calls_for_repair = calls.clone();
        let result = tauri::async_runtime::block_on(parse_with_one_repair(
            "invalid".to_owned(),
            &input(),
            move |_| async move {
                *calls_for_repair.lock().unwrap() += 1;
                Ok("still invalid".to_owned())
            },
        ));
        assert!(result.is_err());
        assert_eq!(*calls.lock().unwrap(), 1);
    }

    #[test]
    fn only_completed_lessons_are_eligible() {
        assert!(validate_lesson_status(LessonStatus::Completed).is_ok());
        for status in [
            LessonStatus::Starting,
            LessonStatus::Active,
            LessonStatus::Interrupted,
            LessonStatus::Failed,
        ] {
            assert!(validate_lesson_status(status).is_err());
        }
    }

    #[test]
    fn analyzer_configuration_is_local_low_temperature_and_bounded() {
        assert_eq!(OLLAMA_CHAT_URL, "http://127.0.0.1:11434/api/chat");
        assert_eq!(ANALYZER_TEMPERATURE, 0.1);
        assert_eq!(ANALYZER_TIMEOUT, Duration::from_secs(180));
    }

    #[test]
    fn completed_analysis_syncs_memory_without_another_model_request() {
        let directory = std::env::temp_dir().join(format!(
            "english-ai-coach-analyzer-memory-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("memory.sqlite3");
        crate::database::migrate(&path).unwrap();
        let lessons = LessonRepository::new(path.clone());
        let lesson = lessons
            .create_lesson(&NewLesson {
                topic: None,
                mode: "free_conversation".to_owned(),
                whisper_model: "whisper".to_owned(),
                whisper_threads: 12,
                ollama_model: "qwen3.5:4b".to_owned(),
                piper_voice: "lessac".to_owned(),
                voice_engine_version: "voice-v2".to_owned(),
            })
            .unwrap();
        lessons.mark_lesson_active(&lesson.id).unwrap();
        lessons.complete_lesson(&lesson.id).unwrap();
        let analyses = LessonAnalysisRepository::new(path.clone());
        analyses.create_pending(&lesson.id, "qwen3.5:4b").unwrap();
        analyses.mark_running(&lesson.id).unwrap();
        let mut payload: LessonAnalysisPayload = serde_json::from_str(&valid_json()).unwrap();
        payload.vocabulary = vec![LessonAnalysisVocabulary {
            word_or_phrase: "look forward to".to_owned(),
            meaning: "aguardar com expectativa".to_owned(),
            example: "I look forward to the weekend.".to_owned(),
        }];
        let canonical = serde_json::to_string(&payload).unwrap();
        let analysis = analyses
            .save_completed(&lesson.id, &payload, &canonical)
            .unwrap();
        let memory = LearningMemoryRepository::new(path.clone());
        let analyzer = LessonAnalyzer::new(lessons, analyses, memory.clone()).unwrap();
        analyzer.sync_memory_without_blocking(&analysis);
        assert_eq!(memory.vocabulary_summary().unwrap().total, 1);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    #[ignore = "manual physical analysis using the real local Ollama and completed lesson database"]
    fn physical_analyzes_latest_completed_lesson() {
        tauri::async_runtime::block_on(async {
            let local_app_data = std::env::var_os("LOCALAPPDATA")
                .map(std::path::PathBuf::from)
                .expect("LOCALAPPDATA");
            let database_path = local_app_data
                .join("com.englishaicoach.desktop")
                .join("database")
                .join("english-ai-coach.sqlite3");
            crate::database::migrate(&database_path).expect("migrate physical database");
            let lessons = LessonRepository::new(database_path.clone());
            let lesson = lessons
                .get_latest_completed_lesson()
                .expect("read latest completed lesson")
                .expect("a completed physical lesson");
            assert!(lesson.student_turn_count >= MINIMUM_STUDENT_TURNS);
            let analyses = LessonAnalysisRepository::new(database_path.clone());
            let memory = LearningMemoryRepository::new(database_path);
            let analyzer =
                LessonAnalyzer::new(lessons, analyses, memory).expect("local analyzer client");
            let analysis = match analyzer.get(&lesson.id).expect("read physical analysis") {
                Some(existing) if existing.status == LessonAnalysisStatus::Failed => analyzer
                    .retry(&lesson.id)
                    .await
                    .expect("retry physical analysis"),
                _ => analyzer
                    .analyze(&lesson.id)
                    .await
                    .expect("physical analysis"),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&analysis).expect("serializable physical analysis")
            );
            assert_eq!(analysis.lesson_id, lesson.id);
            assert_eq!(analysis.status, LessonAnalysisStatus::Completed);
            assert_eq!(
                analysis
                    .scores
                    .as_ref()
                    .and_then(|scores| scores.pronunciation),
                None
            );
        });
    }

    #[test]
    #[ignore = "manual read-only audit of the completed physical lesson analysis"]
    fn physical_database_analysis_integrity() {
        let local_app_data = std::env::var_os("LOCALAPPDATA")
            .map(std::path::PathBuf::from)
            .expect("LOCALAPPDATA");
        let database_path = local_app_data
            .join("com.englishaicoach.desktop")
            .join("database")
            .join("english-ai-coach.sqlite3");
        let lessons = LessonRepository::new(database_path.clone());
        let lesson = lessons
            .get_latest_completed_lesson()
            .expect("read latest completed lesson")
            .expect("a completed physical lesson");
        let input = PedagogicalAnalysisInput::from(
            lessons
                .get_analysis_input(&lesson.id)
                .expect("read physical analysis input"),
        );
        let connection = crate::database::open(&database_path).expect("open physical database");
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM lesson_analysis WHERE lesson_id = ?1",
                [&lesson.id],
                |row| row.get(0),
            )
            .expect("count physical analyses");
        assert_eq!(count, 1);

        let (status, schema, prompt, overall, fluency, grammar, vocabulary, comprehension, interaction, pronunciation, raw): (
            String,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            Option<i32>,
            String,
        ) = connection
            .query_row(
                "SELECT status, schema_version, prompt_version, overall_score, fluency_score, grammar_score, vocabulary_score, comprehension_score, interaction_score, pronunciation_score, raw_json FROM lesson_analysis WHERE lesson_id = ?1",
                [&lesson.id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?, row.get(10)?)),
            )
            .expect("read physical analysis row");
        assert_eq!(status, "completed");
        assert_eq!(schema, 1);
        assert_eq!(prompt, 1);
        assert_eq!(pronunciation, None);
        for score in [fluency, grammar, vocabulary, comprehension, interaction] {
            assert!((0..=100).contains(&score));
        }
        let expected_overall =
            ((fluency + grammar + vocabulary + comprehension + interaction) as f64 / 5.0).round()
                as i32;
        assert_eq!(overall, expected_overall);
        parse_and_validate(&raw, &input).expect("validated physical raw_json");

        let foreign_key_violations: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .expect("foreign key check");
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .expect("integrity check");
        assert_eq!(foreign_key_violations, 0);
        assert_eq!(integrity, "ok");
        println!(
            "lesson_id={} count={} status={} scores={}/{}/{}/{}/{} overall={} pronunciation={:?} schema={} prompt={} raw_json_bytes={} foreign_key_violations={} integrity={}",
            lesson.id,
            count,
            status,
            fluency,
            grammar,
            vocabulary,
            comprehension,
            interaction,
            overall,
            pronunciation,
            schema,
            prompt,
            raw.len(),
            foreign_key_violations,
            integrity,
        );
    }
}
