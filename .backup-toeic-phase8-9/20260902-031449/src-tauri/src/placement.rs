use serde::{Deserialize, Serialize};

pub const PLACEMENT_TEST_VERSION: u32 = 1;
pub const PLACEMENT_QUESTION_BANK_VERSION: u32 = 1;
pub const PLACEMENT_SCORING_VERSION: u32 = 1;
pub const PLACEMENT_SPEAKING_PROMPT_VERSION: u32 = 1;
pub const PLACEMENT_SPEAKING_EVALUATOR_VERSION: u32 = 1;
pub const PLACEMENT_SPEAKING_SCHEMA_VERSION: u32 = 1;
pub const MINIMUM_SPEAKING_RESPONSES: usize = 2;
pub const MINIMUM_SPEAKING_WORDS: u32 = 40;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum CefrBand {
    A1,
    A2,
    B1,
    B2,
    C1,
    C2,
}

impl CefrBand {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::A1 => "A1",
            Self::A2 => "A2",
            Self::B1 => "B1",
            Self::B2 => "B2",
            Self::C1 => "C1",
            Self::C2 => "C2",
        }
    }
    pub fn ordinal(self) -> u8 {
        match self {
            Self::A1 => 1,
            Self::A2 => 2,
            Self::B1 => 3,
            Self::B2 => 4,
            Self::C1 => 5,
            Self::C2 => 6,
        }
    }
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "A1" => Ok(Self::A1),
            "A2" => Ok(Self::A2),
            "B1" => Ok(Self::B1),
            "B2" => Ok(Self::B2),
            "C1" => Ok(Self::C1),
            "C2" => Ok(Self::C2),
            _ => Err(format!("Invalid CEFR band: {value}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementSkill {
    Grammar,
    Vocabulary,
    Reading,
}
impl PlacementSkill {
    pub const ALL: [Self; 3] = [Self::Grammar, Self::Vocabulary, Self::Reading];
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Grammar => "grammar",
            Self::Vocabulary => "vocabulary",
            Self::Reading => "reading",
        }
    }
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "grammar" => Ok(Self::Grammar),
            "vocabulary" => Ok(Self::Vocabulary),
            "reading" => Ok(Self::Reading),
            _ => Err(format!("Invalid placement skill: {value}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementAttemptStatus {
    InProgress,
    Completed,
    Abandoned,
    Failed,
}
impl PlacementAttemptStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Abandoned => "abandoned",
            Self::Failed => "failed",
        }
    }
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "in_progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            "abandoned" => Ok(Self::Abandoned),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("Invalid placement status: {value}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementSpeakingStatus {
    Pending,
    Completed,
    Skipped,
    Unavailable,
}
impl PlacementSpeakingStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Skipped => "skipped",
            Self::Unavailable => "unavailable",
        }
    }
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "pending" => Ok(Self::Pending),
            "completed" => Ok(Self::Completed),
            "skipped" => Ok(Self::Skipped),
            "unavailable" => Ok(Self::Unavailable),
            _ => Err(format!("Invalid speaking status: {value}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementConfidence {
    Low,
    Medium,
    High,
}
impl PlacementConfidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            _ => Err(format!("Invalid confidence: {value}")),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementAttemptDto {
    pub id: String,
    pub status: PlacementAttemptStatus,
    pub test_version: u32,
    pub question_bank_version: u32,
    pub scoring_version: u32,
    pub speaking_prompt_version: u32,
    pub speaking_evaluator_version: Option<u32>,
    pub speaking_schema_version: Option<u32>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub grammar_level: Option<CefrBand>,
    pub vocabulary_level: Option<CefrBand>,
    pub reading_level: Option<CefrBand>,
    pub spoken_production_level: Option<CefrBand>,
    pub overall_estimated_level: Option<CefrBand>,
    pub confidence: Option<PlacementConfidence>,
    pub speaking_status: PlacementSpeakingStatus,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementOptionDto {
    pub id: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementQuestionDto {
    pub question_id: String,
    pub skill: PlacementSkill,
    pub prompt: String,
    pub options: Vec<PlacementOptionDto>,
    pub passage: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementSpeakingPromptDto {
    pub prompt_id: String,
    pub prompt_version: u32,
    pub sequence_index: u32,
    pub prompt: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementSpeakingResponseDto {
    pub id: String,
    pub prompt_id: String,
    pub prompt_version: u32,
    pub sequence_index: u32,
    pub transcript: String,
    pub word_count: u32,
    pub status: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementDomainProgressDto {
    pub skill: String,
    pub status: String,
    pub estimated_level: Option<CefrBand>,
    pub answered_questions: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementProgressDto {
    pub domains: Vec<PlacementDomainProgressDto>,
    pub phase: String,
    pub speaking_responses: u32,
    pub speaking_word_count: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementSessionDto {
    pub attempt: PlacementAttemptDto,
    pub progress: PlacementProgressDto,
    pub question: Option<PlacementQuestionDto>,
    pub speaking_prompt: Option<PlacementSpeakingPromptDto>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementDomainResultDto {
    pub skill: String,
    pub level: Option<CefrBand>,
    pub assessed: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementSpeakingEvidenceDto {
    pub criterion: String,
    pub observation: String,
    pub example: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementResultDto {
    pub attempt: PlacementAttemptDto,
    pub estimated_cefr_level: CefrBand,
    pub confidence: PlacementConfidence,
    pub domains: Vec<PlacementDomainResultDto>,
    pub speaking_evidence: Vec<PlacementSpeakingEvidenceDto>,
    pub speaking_summary: Option<String>,
    pub listening_assessed: bool,
    pub pronunciation_assessed: bool,
    pub writing_assessed: bool,
    pub disclaimer: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementOverviewDto {
    pub active_attempt: Option<PlacementAttemptDto>,
    pub current_result: Option<PlacementResultDto>,
    pub attempt_count: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubmitPlacementAnswerRequest {
    pub attempt_id: String,
    pub question_id: String,
    pub selected_option_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfirmSpeakingResponseRequest {
    pub attempt_id: String,
    pub prompt_id: String,
    pub transcript: String,
}

pub const PLACEMENT_DISCLAIMER: &str = "This is an internal CEFR-informed estimate based on the skills assessed in this placement test. It is not an official CEFR certification.";
