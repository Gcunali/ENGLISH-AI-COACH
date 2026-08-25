use crate::lesson_repository::LessonAnalysisInput;
use serde::{Deserialize, Serialize};

pub const ANALYZER_PROMPT_VERSION: u32 = 1;
pub const ANALYSIS_SCHEMA_VERSION: u32 = 1;
pub const MINIMUM_STUDENT_TURNS: u32 = 3;
pub const MAX_ANALYSIS_INPUT_BYTES: usize = 48_000;
pub const ANALYZER_SYSTEM_PROMPT: &str = include_str!("../prompts/lesson_analyzer_v1.txt");

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonAnalysisStatus {
    Pending,
    Running,
    Completed,
    Failed,
    InsufficientData,
}

impl LessonAnalysisStatus {
    pub fn parse(value: &str) -> rusqlite::Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "insufficient_data" => Ok(Self::InsufficientData),
            _ => Err(rusqlite::Error::InvalidColumnType(
                2,
                "status".to_owned(),
                rusqlite::types::Type::Text,
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonAnalysisScores {
    pub fluency: i32,
    pub grammar: i32,
    pub vocabulary: i32,
    pub comprehension: i32,
    pub interaction: i32,
    pub pronunciation: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonAnalysisStrength {
    pub title: String,
    pub evidence: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonAnalysisImprovement {
    pub area: String,
    pub title: String,
    pub explanation: String,
    pub example_from_lesson: String,
    pub better_alternative: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonAnalysisCorrectionCategory {
    Grammar,
    Vocabulary,
    WordChoice,
    VerbTense,
    Preposition,
    Article,
    WordOrder,
    Naturalness,
    Other,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonAnalysisCorrection {
    pub original: String,
    pub corrected: String,
    pub explanation: String,
    pub category: LessonAnalysisCorrectionCategory,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonAnalysisNaturalAlternative {
    pub original: String,
    pub alternative: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonAnalysisVocabulary {
    pub word_or_phrase: String,
    pub meaning: String,
    pub example: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonAnalysisRecurringPattern {
    pub pattern: String,
    pub count: u32,
    pub explanation: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonAnalysisPayload {
    pub schema_version: u32,
    pub scores: LessonAnalysisScores,
    pub strengths: Vec<LessonAnalysisStrength>,
    pub priority_improvements: Vec<LessonAnalysisImprovement>,
    pub corrections: Vec<LessonAnalysisCorrection>,
    pub natural_alternatives: Vec<LessonAnalysisNaturalAlternative>,
    pub vocabulary: Vec<LessonAnalysisVocabulary>,
    pub recurring_patterns: Vec<LessonAnalysisRecurringPattern>,
    pub next_lesson_recommendations: Vec<String>,
    pub summary: String,
    pub pronunciation_available: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonAnalysis {
    pub id: String,
    pub lesson_id: String,
    pub status: LessonAnalysisStatus,
    pub schema_version: u32,
    pub prompt_version: u32,
    pub analyzer_model: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub overall_score: Option<i32>,
    pub scores: Option<LessonAnalysisScores>,
    pub strengths: Vec<LessonAnalysisStrength>,
    pub priority_improvements: Vec<LessonAnalysisImprovement>,
    pub corrections: Vec<LessonAnalysisCorrection>,
    pub natural_alternatives: Vec<LessonAnalysisNaturalAlternative>,
    pub vocabulary: Vec<LessonAnalysisVocabulary>,
    pub recurring_patterns: Vec<LessonAnalysisRecurringPattern>,
    pub next_lesson_recommendations: Vec<String>,
    pub summary: Option<String>,
    pub pronunciation_available: bool,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PedagogicalAnalysisInput {
    pub lesson: PedagogicalLesson,
    pub transcript: Vec<PedagogicalMessage>,
    pub correction_candidates: Vec<PedagogicalCorrectionCandidate>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PedagogicalLesson {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_seconds: Option<i64>,
    pub student_turn_count: u32,
    pub teacher_turn_count: u32,
    pub correction_count: u32,
    pub whisper_model: String,
    pub ollama_model: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PedagogicalMessage {
    pub sequence_index: u32,
    pub role: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PedagogicalCorrectionCandidate {
    pub student_text: String,
    pub teacher_response_text: String,
}

impl From<LessonAnalysisInput> for PedagogicalAnalysisInput {
    fn from(input: LessonAnalysisInput) -> Self {
        Self {
            lesson: PedagogicalLesson {
                id: input.lesson.id,
                started_at: input.lesson.started_at,
                ended_at: input.lesson.ended_at,
                duration_seconds: input.lesson.duration_seconds,
                student_turn_count: input.lesson.student_turn_count,
                teacher_turn_count: input.lesson.teacher_turn_count,
                correction_count: input.lesson.correction_count,
                whisper_model: input.lesson.whisper_model,
                ollama_model: input.lesson.ollama_model,
            },
            transcript: input
                .transcript
                .into_iter()
                .map(|message| PedagogicalMessage {
                    sequence_index: message.sequence_index,
                    role: message.role,
                    text: message.text,
                })
                .collect(),
            correction_candidates: input
                .correction_candidates
                .into_iter()
                .map(|candidate| PedagogicalCorrectionCandidate {
                    student_text: candidate.student_text,
                    teacher_response_text: candidate.teacher_response_text,
                })
                .collect(),
        }
    }
}

pub fn overall_score(scores: &LessonAnalysisScores) -> i32 {
    (scores.fluency
        + scores.grammar
        + scores.vocabulary
        + scores.comprehension
        + scores.interaction
        + 2)
        / 5
}

pub fn parse_and_validate(
    raw: &str,
    input: &PedagogicalAnalysisInput,
) -> Result<(LessonAnalysisPayload, String), String> {
    let json = strip_known_json_fence(raw)?;
    let payload: LessonAnalysisPayload = serde_json::from_str(json)
        .map_err(|error| format!("Analyzer returned invalid JSON: {error}"))?;
    validate_payload(&payload, input)?;
    let canonical = serde_json::to_string(&payload)
        .map_err(|error| format!("Could not serialize validated analysis: {error}"))?;
    Ok((payload, canonical))
}

fn strip_known_json_fence(raw: &str) -> Result<&str, String> {
    let trimmed = raw.trim();
    if let Some(content) = trimmed.strip_prefix("```json") {
        return content
            .strip_suffix("```")
            .map(str::trim)
            .ok_or_else(|| "Analyzer returned an incomplete JSON fence.".to_owned());
    }
    if let Some(content) = trimmed.strip_prefix("```") {
        return content
            .strip_suffix("```")
            .map(str::trim)
            .ok_or_else(|| "Analyzer returned an incomplete JSON fence.".to_owned());
    }
    Ok(trimmed)
}

pub fn validate_payload(
    payload: &LessonAnalysisPayload,
    input: &PedagogicalAnalysisInput,
) -> Result<(), String> {
    if payload.schema_version != ANALYSIS_SCHEMA_VERSION {
        return Err(format!(
            "Unsupported analysis schema version {}.",
            payload.schema_version
        ));
    }
    for (name, score) in [
        ("fluency", payload.scores.fluency),
        ("grammar", payload.scores.grammar),
        ("vocabulary", payload.scores.vocabulary),
        ("comprehension", payload.scores.comprehension),
        ("interaction", payload.scores.interaction),
    ] {
        if !(0..=100).contains(&score) {
            return Err(format!("{name} score must be between 0 and 100."));
        }
    }
    if payload.scores.pronunciation.is_some() || payload.pronunciation_available {
        return Err("Pronunciation must remain unavailable and null.".to_owned());
    }
    if [
        payload.scores.fluency,
        payload.scores.grammar,
        payload.scores.vocabulary,
        payload.scores.comprehension,
        payload.scores.interaction,
    ]
    .iter()
    .all(|score| *score == 0)
    {
        return Err("A full analysis cannot use placeholder zeroes for every score.".to_owned());
    }
    check_limit("strengths", payload.strengths.len(), 3)?;
    check_limit(
        "priorityImprovements",
        payload.priority_improvements.len(),
        3,
    )?;
    check_limit("corrections", payload.corrections.len(), 8)?;
    check_limit("naturalAlternatives", payload.natural_alternatives.len(), 5)?;
    check_limit("vocabulary", payload.vocabulary.len(), 8)?;
    check_limit("recurringPatterns", payload.recurring_patterns.len(), 5)?;
    check_limit(
        "nextLessonRecommendations",
        payload.next_lesson_recommendations.len(),
        3,
    )?;
    if payload.strengths.is_empty()
        || payload.priority_improvements.is_empty()
        || payload.next_lesson_recommendations.is_empty()
    {
        return Err(
            "A full analysis requires at least one strength, improvement, and recommendation."
                .to_owned(),
        );
    }
    if !input.correction_candidates.is_empty() && payload.corrections.is_empty() {
        return Err(
            "The lesson contains correction candidates but the analysis contains no correction."
                .to_owned(),
        );
    }

    for strength in &payload.strengths {
        required("strength.title", &strength.title, 200)?;
        required("strength.evidence", &strength.evidence, 600)?;
        if !student_evidence_contains(input, &strength.evidence) {
            return Err(format!(
                "Strength is not supported by a student message: {:?}.",
                strength.evidence
            ));
        }
    }
    for improvement in &payload.priority_improvements {
        required("improvement.area", &improvement.area, 100)?;
        required("improvement.title", &improvement.title, 200)?;
        required("improvement.explanation", &improvement.explanation, 1_000)?;
        required(
            "improvement.exampleFromLesson",
            &improvement.example_from_lesson,
            800,
        )?;
        required(
            "improvement.betterAlternative",
            &improvement.better_alternative,
            800,
        )?;
        if !student_evidence_contains(input, &improvement.example_from_lesson) {
            return Err(format!(
                "Priority improvement is not supported by a student message: {:?}.",
                improvement.example_from_lesson
            ));
        }
    }
    for correction in &payload.corrections {
        required("correction.original", &correction.original, 800)?;
        required("correction.corrected", &correction.corrected, 800)?;
        required("correction.explanation", &correction.explanation, 1_000)?;
        if !student_evidence_contains(input, &correction.original) {
            return Err(format!(
                "Correction is not supported by a student message: {:?}.",
                correction.original
            ));
        }
    }
    for alternative in &payload.natural_alternatives {
        required("naturalAlternative.original", &alternative.original, 800)?;
        required(
            "naturalAlternative.alternative",
            &alternative.alternative,
            800,
        )?;
        if !student_evidence_contains(input, &alternative.original) {
            return Err(format!(
                "Natural alternative is not supported by a student message: {:?}.",
                alternative.original
            ));
        }
    }
    for vocabulary in &payload.vocabulary {
        required("vocabulary.wordOrPhrase", &vocabulary.word_or_phrase, 300)?;
        required("vocabulary.meaning", &vocabulary.meaning, 800)?;
        required("vocabulary.example", &vocabulary.example, 800)?;
    }
    for pattern in &payload.recurring_patterns {
        required("recurringPattern.pattern", &pattern.pattern, 300)?;
        required("recurringPattern.explanation", &pattern.explanation, 1_000)?;
        if pattern.count < 2 {
            return Err("Recurring pattern count must be at least 2.".to_owned());
        }
    }
    for recommendation in &payload.next_lesson_recommendations {
        required("nextLessonRecommendation", recommendation, 400)?;
    }
    required("summary", &payload.summary, 1_200)?;
    if payload.summary.trim().chars().count() < 40
        || payload
            .summary
            .trim()
            .eq_ignore_ascii_case("Portuguese summary")
    {
        return Err("Analysis summary is too short or is a schema placeholder.".to_owned());
    }
    Ok(())
}

fn check_limit(name: &str, actual: usize, maximum: usize) -> Result<(), String> {
    if actual > maximum {
        Err(format!(
            "{name} contains {actual} items; maximum is {maximum}."
        ))
    } else {
        Ok(())
    }
}

fn required(name: &str, value: &str, maximum: usize) -> Result<(), String> {
    let length = value.trim().chars().count();
    if length == 0 {
        return Err(format!("{name} must not be empty."));
    }
    if length > maximum {
        return Err(format!("{name} exceeds {maximum} characters."));
    }
    Ok(())
}

fn student_evidence_contains(input: &PedagogicalAnalysisInput, evidence: &str) -> bool {
    let evidence = normalize_evidence(evidence);
    evidence.len() >= 3
        && input.transcript.iter().any(|message| {
            message.role == "student" && normalize_evidence(&message.text).contains(&evidence)
        })
}

fn normalize_evidence(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '\'' {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

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
            transcript: vec![
                PedagogicalMessage {
                    sequence_index: 1,
                    role: "student".to_owned(),
                    text: "Today I play tennis.".to_owned(),
                },
                PedagogicalMessage {
                    sequence_index: 2,
                    role: "teacher".to_owned(),
                    text:
                        "If you mean earlier today, say 'Today I played tennis.' Did you enjoy it?"
                            .to_owned(),
                },
                PedagogicalMessage {
                    sequence_index: 3,
                    role: "student".to_owned(),
                    text: "Yes, I play with my friend yesterday.".to_owned(),
                },
            ],
            correction_candidates: vec![PedagogicalCorrectionCandidate {
                student_text: "Today I play tennis.".to_owned(),
                teacher_response_text: "If you mean earlier today, say 'Today I played tennis.'"
                    .to_owned(),
            }],
        }
    }

    fn valid_payload() -> LessonAnalysisPayload {
        LessonAnalysisPayload {
            schema_version: 1,
            scores: LessonAnalysisScores {
                fluency: 70,
                grammar: 60,
                vocabulary: 65,
                comprehension: 80,
                interaction: 75,
                pronunciation: None,
            },
            strengths: vec![LessonAnalysisStrength {
                title: "Boa compreensão".to_owned(),
                evidence: "Yes, I play with my friend yesterday.".to_owned(),
            }],
            priority_improvements: vec![LessonAnalysisImprovement {
                area: "grammar".to_owned(),
                title: "Passado simples".to_owned(),
                explanation: "Use o passado para ações concluídas.".to_owned(),
                example_from_lesson: "Today I play tennis.".to_owned(),
                better_alternative: "Today I played tennis.".to_owned(),
            }],
            corrections: vec![LessonAnalysisCorrection {
                original: "Today I play tennis.".to_owned(),
                corrected: "Today I played tennis.".to_owned(),
                explanation: "Use o passado para uma ação concluída.".to_owned(),
                category: LessonAnalysisCorrectionCategory::VerbTense,
            }],
            natural_alternatives: vec![],
            vocabulary: vec![],
            recurring_patterns: vec![],
            next_lesson_recommendations: vec!["Praticar passado simples.".to_owned()],
            summary: "Você manteve a conversa e pode melhorar o passado simples.".to_owned(),
            pronunciation_available: false,
        }
    }

    #[test]
    fn calculates_deterministic_rounded_overall_score() {
        assert_eq!(overall_score(&valid_payload().scores), 70);
    }

    #[test]
    fn accepts_valid_json_plain_fenced_and_with_whitespace() {
        let json = serde_json::to_string(&valid_payload()).unwrap();
        for raw in [
            json.clone(),
            format!("  {json}\n"),
            format!("```json\n{json}\n```"),
        ] {
            assert!(parse_and_validate(&raw, &input()).is_ok());
        }
    }

    #[test]
    fn rejects_incomplete_invalid_and_surrounded_json() {
        for raw in [
            "{",
            "not json",
            "before {\"schemaVersion\":1} after",
            "```json\n{",
        ] {
            assert!(parse_and_validate(raw, &input()).is_err(), "{raw}");
        }
    }

    #[test]
    fn rejects_scores_outside_range() {
        for score in [-1, 101] {
            let mut payload = valid_payload();
            payload.scores.grammar = score;
            assert!(validate_payload(&payload, &input()).is_err());
        }
    }

    #[test]
    fn rejects_pronunciation_value_and_availability() {
        let mut payload = valid_payload();
        payload.scores.pronunciation = Some(80);
        assert!(validate_payload(&payload, &input()).is_err());
        payload.scores.pronunciation = None;
        payload.pronunciation_available = true;
        assert!(validate_payload(&payload, &input()).is_err());
    }

    #[test]
    fn rejects_limits_recurring_count_empty_strings_and_schema() {
        let mut too_many = valid_payload();
        too_many.strengths = (0..4)
            .map(|index| LessonAnalysisStrength {
                title: format!("Strength {index}"),
                evidence: "Evidence".to_owned(),
            })
            .collect();
        assert!(validate_payload(&too_many, &input()).is_err());

        let mut too_many_corrections = valid_payload();
        too_many_corrections.corrections = vec![too_many_corrections.corrections[0].clone(); 9];
        assert!(validate_payload(&too_many_corrections, &input()).is_err());

        let mut recurring = valid_payload();
        recurring.recurring_patterns = vec![LessonAnalysisRecurringPattern {
            pattern: "Past tense".to_owned(),
            count: 1,
            explanation: "Repeated".to_owned(),
        }];
        assert!(validate_payload(&recurring, &input()).is_err());

        let mut empty = valid_payload();
        empty.summary = "  ".to_owned();
        assert!(validate_payload(&empty, &input()).is_err());

        let mut schema = valid_payload();
        schema.schema_version = 2;
        assert!(validate_payload(&schema, &input()).is_err());
    }

    #[test]
    fn rejects_correction_without_student_evidence() {
        let mut payload = valid_payload();
        payload.corrections[0].original = "A sentence the student never said.".to_owned();
        assert!(validate_payload(&payload, &input()).is_err());
    }

    #[test]
    fn unknown_category_fails_during_deserialization() {
        let json = serde_json::to_string(&valid_payload())
            .unwrap()
            .replace("verb_tense", "unknown_category");
        assert!(parse_and_validate(&json, &input()).is_err());
    }

    #[test]
    fn prompt_contains_critical_safety_contracts() {
        let prompt = ANALYZER_SYSTEM_PROMPT;
        for required in [
            "RETORNE SOMENTE JSON",
            "Não invente erros",
            "reconhecimento de fala",
            "Pronúncia não está disponível",
            "0 a 100",
            "pelo menos 2 ocorrências",
            "strengths <= 3",
            "DEVEM ser escritos em português brasileiro",
            "strengths.evidence",
            "Nunca corrija, parafraseie, complete nem acrescente Markdown",
        ] {
            assert!(prompt.contains(required), "missing {required}");
        }
    }
}
