use crate::placement::{CefrBand, PlacementConfidence};
use crate::pronunciation::PronunciationAttemptDto;
use serde::{Deserialize, Serialize};

pub const INTERACTIVE_LESSON_ENGINE_VERSION: u32 = 1;
pub const INTERACTIVE_LESSON_PACKAGE_SCHEMA_VERSION: u32 = 1;
pub const INTERACTIVE_LESSON_FLOW_VERSION: u32 = 1;
pub const INTERACTIVE_LESSON_SESSION_SNAPSHOT_VERSION: u32 = 1;
pub const INTERACTIVE_LESSON_STAGE_RESULT_VERSION: u32 = 1;
pub const GUIDED_LESSON_AUDIO_RUNTIME_VERSION: u32 = 1;
pub const GUIDED_LESSON_RUNTIME_STATE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractiveStageType {
    Theory,
    VisualVocabulary,
    Listening,
    Repeat,
    SpeakingCheck,
    Exercise,
    GuidedConversation,
    Analysis,
}

impl InteractiveStageType {
    pub const ORDER: [Self; 8] = [
        Self::Theory,
        Self::VisualVocabulary,
        Self::Listening,
        Self::Repeat,
        Self::SpeakingCheck,
        Self::Exercise,
        Self::GuidedConversation,
        Self::Analysis,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Theory => "theory",
            Self::VisualVocabulary => "visual_vocabulary",
            Self::Listening => "listening",
            Self::Repeat => "repeat",
            Self::SpeakingCheck => "speaking_check",
            Self::Exercise => "exercise",
            Self::GuidedConversation => "guided_conversation",
            Self::Analysis => "analysis",
        }
    }
    pub fn runtime_available(self, schema_version: u32) -> bool {
        schema_version == 1
            && matches!(
                self,
                Self::Theory
                    | Self::VisualVocabulary
                    | Self::Listening
                    | Self::Repeat
                    | Self::SpeakingCheck
            )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationState {
    Draft,
    Published,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Image,
    Audio,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LessonAsset {
    pub asset_id: String,
    pub r#type: AssetType,
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TheoryBlockType {
    Paragraph,
    BulletList,
    Example,
    Callout,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TheoryBlock {
    pub r#type: TheoryBlockType,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub items: Option<Vec<String>>,
    #[serde(default)]
    pub english: Option<String>,
    #[serde(default)]
    pub explanation: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisualVocabularyItem {
    pub item_id: String,
    pub term: String,
    pub meaning: String,
    pub example: String,
    pub image_asset_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListeningSegment {
    pub segment_id: String,
    pub text: String,
    pub audio_asset_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepeatTarget {
    pub target_id: String,
    pub text: String,
    pub reference_audio_asset_id: Option<String>,
    pub hint: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeakingCheckTarget {
    pub target_id: String,
    pub instruction: String,
    pub target_text: String,
    pub hint: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StagePayload {
    Theory {
        blocks: Vec<TheoryBlock>,
    },
    VisualVocabulary {
        items: Vec<VisualVocabularyItem>,
    },
    Listening {
        segments: Vec<ListeningSegment>,
        reveal_text_after_first_play: bool,
    },
    Repeat {
        targets: Vec<RepeatTarget>,
    },
    SpeakingCheck {
        targets: Vec<SpeakingCheckTarget>,
    },
    Exercise {},
    GuidedConversation {},
    Analysis {},
}

impl StagePayload {
    pub fn stage_type(&self) -> InteractiveStageType {
        match self {
            Self::Theory { .. } => InteractiveStageType::Theory,
            Self::VisualVocabulary { .. } => InteractiveStageType::VisualVocabulary,
            Self::Listening { .. } => InteractiveStageType::Listening,
            Self::Repeat { .. } => InteractiveStageType::Repeat,
            Self::SpeakingCheck { .. } => InteractiveStageType::SpeakingCheck,
            Self::Exercise {} => InteractiveStageType::Exercise,
            Self::GuidedConversation {} => InteractiveStageType::GuidedConversation,
            Self::Analysis {} => InteractiveStageType::Analysis,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InteractiveStage {
    pub stage_id: String,
    pub stage_type: InteractiveStageType,
    pub stage_schema_version: u32,
    pub title: String,
    pub instructions: String,
    pub required: bool,
    pub payload: StagePayload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InteractiveLessonPackage {
    pub package_schema_version: u32,
    pub lesson_flow_version: u32,
    pub lesson_id: String,
    pub content_version: u32,
    pub publication_state: PublicationState,
    pub title: String,
    pub description: String,
    pub language: String,
    pub reference_locale: String,
    pub cefr_band: CefrBand,
    pub estimated_minutes: u32,
    pub objectives: Vec<String>,
    pub tags: Vec<String>,
    pub stages: Vec<InteractiveStage>,
    pub assets: Vec<LessonAsset>,
}

#[derive(Clone, Debug)]
pub struct RegisteredLesson {
    pub package: InteractiveLessonPackage,
    pub package_hash: String,
    pub asset_files: std::collections::BTreeMap<String, std::path::PathBuf>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StageCapabilityDto {
    pub stage_type: InteractiveStageType,
    pub stage_schema_version: u32,
    pub runtime_available: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuidedLessonOverviewDto {
    pub published_lesson_count: usize,
    pub active_session: Option<InteractiveLessonSessionDto>,
    pub capabilities: Vec<StageCapabilityDto>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractiveLessonSummaryDto {
    pub lesson_id: String,
    pub content_version: u32,
    pub title: String,
    pub description: String,
    pub cefr_band: CefrBand,
    pub estimated_minutes: u32,
    pub objectives: Vec<String>,
    pub tags: Vec<String>,
    pub stage_count: usize,
    pub startable: bool,
    pub unavailable_reasons: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StageOverviewDto {
    pub stage_id: String,
    pub stage_type: InteractiveStageType,
    pub title: String,
    pub required: bool,
    pub available: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractiveLessonDetailDto {
    #[serde(flatten)]
    pub summary: InteractiveLessonSummaryDto,
    pub stage_overview: Vec<StageOverviewDto>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractiveSessionStatus {
    InProgress,
    Completed,
    Abandoned,
    Failed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractiveStageStatus {
    Pending,
    Active,
    Completed,
    Skipped,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStageDto {
    pub stage_id: String,
    pub sequence_index: u32,
    pub stage_type: InteractiveStageType,
    pub title: String,
    pub required: bool,
    pub status: InteractiveStageStatus,
    pub attempt_count: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractiveLessonSessionDto {
    pub id: String,
    pub lesson_id: String,
    pub content_version: u32,
    pub title: String,
    pub cefr_band: CefrBand,
    pub status: InteractiveSessionStatus,
    pub current_stage_index: u32,
    pub stage_count: u32,
    pub progress_percent: u32,
    pub stages: Vec<SessionStageDto>,
    pub active_stage: Option<ActiveStageDto>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub abandoned_at: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveStageDto {
    pub stage_id: String,
    pub sequence_index: u32,
    pub stage_type: InteractiveStageType,
    pub title: String,
    pub instructions: String,
    pub required: bool,
    pub content: ActiveStageContentDto,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActiveStageContentDto {
    Theory {
        blocks: Vec<TheoryBlock>,
    },
    VisualVocabulary {
        items: Vec<VisualVocabularyItem>,
    },
    Listening {
        segments: Vec<GuidedListeningSegmentDto>,
        reveal_text_after_first_play: bool,
    },
    Repeat {
        targets: Vec<GuidedRepeatTargetDto>,
    },
    SpeakingCheck {
        targets: Vec<GuidedSpeakingTargetDto>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListeningItemState {
    pub segment_id: String,
    pub completed_playback_count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PronunciationItemState {
    pub item_id: String,
    pub completed_reference_playback_count: u32,
    pub selected_attempt_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GuidedStageRuntimeState {
    Listening {
        segments: Vec<ListeningItemState>,
    },
    Repeat {
        targets: Vec<PronunciationItemState>,
    },
    SpeakingCheck {
        targets: Vec<PronunciationItemState>,
    },
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuidedPronunciationAttemptDto {
    pub id: String,
    pub attempt_index: u32,
    pub status: String,
    pub selected: bool,
    pub result: Option<PronunciationAttemptDto>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuidedListeningSegmentDto {
    pub segment_id: String,
    pub text: String,
    pub has_bundled_audio: bool,
    pub completed_playback_count: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuidedRepeatTargetDto {
    pub target_id: String,
    pub text: String,
    pub hint: Option<String>,
    pub has_bundled_audio: bool,
    pub completed_reference_playback_count: u32,
    pub selected_attempt_id: Option<String>,
    pub attempts: Vec<GuidedPronunciationAttemptDto>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuidedSpeakingTargetDto {
    pub target_id: String,
    pub instruction: String,
    pub target_text: String,
    pub hint: Option<String>,
    pub selected_attempt_id: Option<String>,
    pub attempts: Vec<GuidedPronunciationAttemptDto>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GuidedPlaybackRequest {
    pub session_id: String,
    pub stage_id: String,
    pub item_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GuidedPronunciationRequest {
    pub session_id: String,
    pub stage_id: String,
    pub item_id: String,
    pub audio_base64: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectGuidedAttemptRequest {
    pub session_id: String,
    pub stage_id: String,
    pub item_id: String,
    pub attempt_id: String,
}

#[derive(Clone, Debug)]
pub struct GuidedAttemptContext {
    pub attempt_id: String,
    pub target_text: String,
}

#[derive(Clone, Debug)]
pub struct GuidedPlaybackSource {
    pub text: String,
    pub asset_id: Option<String>,
    pub package_hash: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartInteractiveLessonRequest {
    pub lesson_id: String,
    #[serde(default)]
    pub start_over: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StageActionRequest {
    pub session_id: String,
    pub stage_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudentContextSnapshot {
    pub profile_schema_version: u32,
    pub placement_attempt_id: Option<String>,
    pub estimated_cefr: Option<CefrBand>,
    pub placement_confidence: Option<PlacementConfidence>,
    pub target_cefr: Option<CefrBand>,
    pub learning_goals: Vec<String>,
}

pub fn summary(lesson: &RegisteredLesson) -> InteractiveLessonSummaryDto {
    let unavailable_reasons: Vec<String> = lesson
        .package
        .stages
        .iter()
        .filter(|stage| {
            !stage
                .stage_type
                .runtime_available(stage.stage_schema_version)
        })
        .map(|stage| {
            format!(
                "{} v{} is not available in this engine.",
                stage.stage_type.as_str(),
                stage.stage_schema_version
            )
        })
        .collect();
    InteractiveLessonSummaryDto {
        lesson_id: lesson.package.lesson_id.clone(),
        content_version: lesson.package.content_version,
        title: lesson.package.title.clone(),
        description: lesson.package.description.clone(),
        cefr_band: lesson.package.cefr_band,
        estimated_minutes: lesson.package.estimated_minutes,
        objectives: lesson.package.objectives.clone(),
        tags: lesson.package.tags.clone(),
        stage_count: lesson.package.stages.len(),
        startable: unavailable_reasons.is_empty(),
        unavailable_reasons,
    }
}
