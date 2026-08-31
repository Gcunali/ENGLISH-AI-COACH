use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const INTERACTIVE_EXERCISE_ENGINE_VERSION: u32 = 1;
pub const EXERCISE_STAGE_SCHEMA_VERSION: u32 = 1;
pub const EXERCISE_ATTEMPT_RESULT_VERSION: u32 = 1;
pub const EXERCISE_NORMALIZATION_VERSION: u32 = 1;
pub const EXERCISE_RESPONSE_SCHEMA_VERSION: u32 = 1;

const MAX_PROMPT: usize = 500;
const MAX_INSTRUCTIONS: usize = 300;
const MAX_HINT: usize = 300;
const MAX_OPTION_TEXT: usize = 240;
const MAX_FEEDBACK: usize = 600;
const MAX_TEXT_ANSWER: usize = 200;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExerciseType {
    SingleChoice,
    MultipleSelect,
    FillBlank,
    WordOrder,
    Matching,
    ShortAnswerExact,
}

impl ExerciseType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SingleChoice => "single_choice",
            Self::MultipleSelect => "multiple_select",
            Self::FillBlank => "fill_blank",
            Self::WordOrder => "word_order",
            Self::Matching => "matching",
            Self::ShortAnswerExact => "short_answer_exact",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExerciseOption {
    pub option_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExerciseToken {
    pub token_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MatchingItem {
    pub item_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MatchingPair {
    pub left_id: String,
    pub right_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExerciseFeedback {
    pub correct: String,
    pub incorrect: String,
    pub explanation: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ExercisePayload {
    SingleChoice {
        options: Vec<ExerciseOption>,
        #[serde(rename = "correctOptionId")]
        correct_option_id: String,
    },
    MultipleSelect {
        options: Vec<ExerciseOption>,
        #[serde(rename = "correctOptionIds")]
        correct_option_ids: Vec<String>,
    },
    FillBlank {
        prefix: String,
        suffix: String,
        #[serde(rename = "acceptedAnswers")]
        accepted_answers: Vec<String>,
        #[serde(rename = "normalizationProfile")]
        normalization_profile: String,
    },
    WordOrder {
        tokens: Vec<ExerciseToken>,
        #[serde(rename = "correctOrder")]
        correct_order: Vec<String>,
    },
    Matching {
        #[serde(rename = "leftItems")]
        left_items: Vec<MatchingItem>,
        #[serde(rename = "rightItems")]
        right_items: Vec<MatchingItem>,
        #[serde(rename = "correctPairs")]
        correct_pairs: Vec<MatchingPair>,
    },
    ShortAnswerExact {
        #[serde(rename = "acceptedAnswers")]
        accepted_answers: Vec<String>,
        #[serde(rename = "normalizationProfile")]
        normalization_profile: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseItem {
    pub exercise_id: String,
    pub exercise_type: ExerciseType,
    pub prompt: String,
    pub instructions: Option<String>,
    pub hint: Option<String>,
    pub payload: ExercisePayload,
    pub feedback: ExerciseFeedback,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RawExerciseItem {
    pub exercise_id: String,
    pub exercise_type: ExerciseType,
    pub prompt: String,
    pub instructions: Option<String>,
    pub hint: Option<String>,
    pub payload: Value,
    pub feedback: ExerciseFeedback,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SingleChoicePayload {
    options: Vec<ExerciseOption>,
    correct_option_id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MultipleSelectPayload {
    options: Vec<ExerciseOption>,
    correct_option_ids: Vec<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FillBlankPayload {
    prefix: String,
    suffix: String,
    accepted_answers: Vec<String>,
    normalization_profile: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WordOrderPayload {
    tokens: Vec<ExerciseToken>,
    correct_order: Vec<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MatchingPayload {
    left_items: Vec<MatchingItem>,
    right_items: Vec<MatchingItem>,
    correct_pairs: Vec<MatchingPair>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ShortAnswerPayload {
    accepted_answers: Vec<String>,
    normalization_profile: String,
}

pub fn parse_item(raw: RawExerciseItem) -> Result<ExerciseItem, String> {
    let invalid = |error: serde_json::Error| {
        format!(
            "invalid {} exercise payload: {error}",
            raw.exercise_type.as_str()
        )
    };
    let payload = match raw.exercise_type {
        ExerciseType::SingleChoice => {
            let value: SingleChoicePayload =
                serde_json::from_value(raw.payload).map_err(invalid)?;
            ExercisePayload::SingleChoice {
                options: value.options,
                correct_option_id: value.correct_option_id,
            }
        }
        ExerciseType::MultipleSelect => {
            let value: MultipleSelectPayload =
                serde_json::from_value(raw.payload).map_err(invalid)?;
            ExercisePayload::MultipleSelect {
                options: value.options,
                correct_option_ids: value.correct_option_ids,
            }
        }
        ExerciseType::FillBlank => {
            let value: FillBlankPayload = serde_json::from_value(raw.payload).map_err(invalid)?;
            ExercisePayload::FillBlank {
                prefix: value.prefix,
                suffix: value.suffix,
                accepted_answers: value.accepted_answers,
                normalization_profile: value.normalization_profile,
            }
        }
        ExerciseType::WordOrder => {
            let value: WordOrderPayload = serde_json::from_value(raw.payload).map_err(invalid)?;
            ExercisePayload::WordOrder {
                tokens: value.tokens,
                correct_order: value.correct_order,
            }
        }
        ExerciseType::Matching => {
            let value: MatchingPayload = serde_json::from_value(raw.payload).map_err(invalid)?;
            ExercisePayload::Matching {
                left_items: value.left_items,
                right_items: value.right_items,
                correct_pairs: value.correct_pairs,
            }
        }
        ExerciseType::ShortAnswerExact => {
            let value: ShortAnswerPayload = serde_json::from_value(raw.payload).map_err(invalid)?;
            ExercisePayload::ShortAnswerExact {
                accepted_answers: value.accepted_answers,
                normalization_profile: value.normalization_profile,
            }
        }
    };
    let item = ExerciseItem {
        exercise_id: raw.exercise_id,
        exercise_type: raw.exercise_type,
        prompt: raw.prompt,
        instructions: raw.instructions,
        hint: raw.hint,
        payload,
        feedback: raw.feedback,
    };
    validate_item(&item)?;
    Ok(item)
}

pub fn validate_items(items: &[ExerciseItem]) -> Result<(), String> {
    if items.is_empty() || items.len() > 20 {
        return Err("exercise requires 1..20 items".into());
    }
    let mut ids = BTreeSet::new();
    for item in items {
        crate::interactive_lesson_content::validate_slug(&item.exercise_id, "exerciseId")?;
        if !ids.insert(&item.exercise_id) {
            return Err("duplicate exerciseId".into());
        }
        validate_item(item)?;
    }
    Ok(())
}

fn validate_item(item: &ExerciseItem) -> Result<(), String> {
    plain(&item.prompt, 1, MAX_PROMPT, "exercise prompt")?;
    optional_plain(
        item.instructions.as_deref(),
        MAX_INSTRUCTIONS,
        "exercise instructions",
    )?;
    optional_plain(item.hint.as_deref(), MAX_HINT, "exercise hint")?;
    plain(&item.feedback.correct, 1, MAX_FEEDBACK, "correct feedback")?;
    plain(
        &item.feedback.incorrect,
        1,
        MAX_FEEDBACK,
        "incorrect feedback",
    )?;
    optional_plain(
        item.feedback.explanation.as_deref(),
        MAX_FEEDBACK,
        "exercise explanation",
    )?;
    match (&item.exercise_type, &item.payload) {
        (
            ExerciseType::SingleChoice,
            ExercisePayload::SingleChoice {
                options,
                correct_option_id,
            },
        ) => {
            validate_options(options, 2, 8)?;
            if !options
                .iter()
                .any(|option| option.option_id == *correct_option_id)
            {
                return Err("single_choice correctOptionId does not exist".into());
            }
        }
        (
            ExerciseType::MultipleSelect,
            ExercisePayload::MultipleSelect {
                options,
                correct_option_ids,
            },
        ) => {
            validate_options(options, 2, 10)?;
            if correct_option_ids.is_empty() {
                return Err("multiple_select requires at least one correct option".into());
            }
            let ids = correct_option_ids.iter().collect::<BTreeSet<_>>();
            if ids.len() != correct_option_ids.len()
                || ids
                    .iter()
                    .any(|id| !options.iter().any(|option| option.option_id == ***id))
            {
                return Err("multiple_select correctOptionIds are invalid".into());
            }
        }
        (
            ExerciseType::FillBlank,
            ExercisePayload::FillBlank {
                prefix,
                suffix,
                accepted_answers,
                normalization_profile,
            },
        ) => {
            plain(prefix, 0, MAX_PROMPT, "fill_blank prefix")?;
            plain(suffix, 0, MAX_PROMPT, "fill_blank suffix")?;
            validate_answers(accepted_answers, normalization_profile)?;
        }
        (
            ExerciseType::WordOrder,
            ExercisePayload::WordOrder {
                tokens,
                correct_order,
            },
        ) => {
            if tokens.len() < 2 || tokens.len() > 20 {
                return Err("word_order requires 2..20 tokens".into());
            }
            let mut ids = BTreeSet::new();
            for token in tokens {
                crate::interactive_lesson_content::validate_slug(&token.token_id, "tokenId")?;
                if !ids.insert(&token.token_id) {
                    return Err("duplicate word_order tokenId".into());
                };
                plain(&token.text, 1, MAX_OPTION_TEXT, "token text")?;
            }
            if correct_order.len() != tokens.len()
                || correct_order.iter().collect::<BTreeSet<_>>().len() != tokens.len()
                || correct_order
                    .iter()
                    .any(|id| !tokens.iter().any(|token| token.token_id == *id))
            {
                return Err(
                    "word_order correctOrder must contain every tokenId exactly once".into(),
                );
            }
        }
        (
            ExerciseType::Matching,
            ExercisePayload::Matching {
                left_items,
                right_items,
                correct_pairs,
            },
        ) => validate_matching(left_items, right_items, correct_pairs)?,
        (
            ExerciseType::ShortAnswerExact,
            ExercisePayload::ShortAnswerExact {
                accepted_answers,
                normalization_profile,
            },
        ) => validate_answers(accepted_answers, normalization_profile)?,
        _ => return Err("exerciseType and payload do not match".into()),
    }
    Ok(())
}

fn validate_options(options: &[ExerciseOption], min: usize, max: usize) -> Result<(), String> {
    if options.len() < min || options.len() > max {
        return Err(format!("exercise options must contain {min}..{max} items"));
    }
    let mut ids = BTreeSet::new();
    for option in options {
        crate::interactive_lesson_content::validate_slug(&option.option_id, "optionId")?;
        if !ids.insert(&option.option_id) {
            return Err("duplicate optionId".into());
        };
        plain(&option.text, 1, MAX_OPTION_TEXT, "option text")?;
    }
    Ok(())
}
fn validate_answers(answers: &[String], profile: &str) -> Result<(), String> {
    if profile != "english_basic_v1" {
        return Err("unsupported exercise normalizationProfile".into());
    }
    if answers.is_empty() || answers.len() > 12 {
        return Err("acceptedAnswers must contain 1..12 values".into());
    }
    let mut normalized = BTreeSet::new();
    for answer in answers {
        plain(answer, 1, MAX_TEXT_ANSWER, "accepted answer")?;
        if !normalized.insert(normalize_english_basic_v1(answer)) {
            return Err("acceptedAnswers contain deterministic duplicates".into());
        }
    }
    Ok(())
}
fn validate_matching(
    left: &[MatchingItem],
    right: &[MatchingItem],
    pairs: &[MatchingPair],
) -> Result<(), String> {
    if left.len() < 2 || left.len() > 10 || right.len() != left.len() || pairs.len() != left.len() {
        return Err("matching requires 2..10 one-to-one pairs".into());
    }
    let mut left_ids = BTreeSet::new();
    let mut right_ids = BTreeSet::new();
    for value in left {
        crate::interactive_lesson_content::validate_slug(&value.item_id, "matching itemId")?;
        if !left_ids.insert(&value.item_id) {
            return Err("duplicate matching left itemId".into());
        };
        plain(&value.text, 1, MAX_OPTION_TEXT, "matching text")?
    }
    for value in right {
        crate::interactive_lesson_content::validate_slug(&value.item_id, "matching itemId")?;
        if !right_ids.insert(&value.item_id) {
            return Err("duplicate matching right itemId".into());
        };
        plain(&value.text, 1, MAX_OPTION_TEXT, "matching text")?
    }
    let mut used_left = BTreeSet::new();
    let mut used_right = BTreeSet::new();
    for pair in pairs {
        if !left_ids.contains(&pair.left_id)
            || !right_ids.contains(&pair.right_id)
            || !used_left.insert(&pair.left_id)
            || !used_right.insert(&pair.right_id)
        {
            return Err("matching correctPairs must be one-to-one and complete".into());
        }
    }
    Ok(())
}
fn optional_plain(value: Option<&str>, max: usize, label: &str) -> Result<(), String> {
    match value {
        Some(value) => plain(value, 1, max, label),
        None => Ok(()),
    }
}
fn plain(value: &str, min: usize, max: usize, label: &str) -> Result<(), String> {
    let size = value.chars().count();
    if size < min || size > max {
        return Err(format!("{label} length is outside {min}..{max}"));
    }
    if value
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
        || value.contains('<')
        || value.contains('>')
    {
        return Err(format!("{label} must be plain text"));
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "exerciseType", content = "value", rename_all = "snake_case")]
pub enum ExerciseResponse {
    SingleChoice(SingleChoiceResponse),
    MultipleSelect(MultipleSelectResponse),
    FillBlank(TextResponse),
    WordOrder(WordOrderResponse),
    Matching(MatchingResponse),
    ShortAnswerExact(TextResponse),
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SingleChoiceResponse {
    pub option_id: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MultipleSelectResponse {
    pub option_ids: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextResponse {
    pub text: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WordOrderResponse {
    pub token_ids: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MatchingResponse {
    pub pairs: Vec<MatchingPair>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum PublicExercisePayload {
    Options {
        options: Vec<ExerciseOption>,
    },
    FillBlank {
        prefix: String,
        suffix: String,
    },
    WordOrder {
        tokens: Vec<ExerciseToken>,
    },
    Matching {
        #[serde(rename = "leftItems")]
        left_items: Vec<MatchingItem>,
        #[serde(rename = "rightItems")]
        right_items: Vec<MatchingItem>,
    },
    ShortAnswerExact {
        #[serde(rename = "normalizationProfile")]
        normalization_profile: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExpectedAnswerDto {
    SingleChoice { option: ExerciseOption },
    MultipleSelect { options: Vec<ExerciseOption> },
    FillBlank { answer: String },
    WordOrder { tokens: Vec<ExerciseToken> },
    Matching { pairs: Vec<ExpectedMatchingPairDto> },
    ShortAnswerExact { answer: String },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpectedMatchingPairDto {
    pub left: MatchingItem,
    pub right: MatchingItem,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedExerciseResult {
    pub schema_version: u32,
    pub correct: bool,
    pub feedback: String,
    pub explanation: Option<String>,
    pub expected_answer: ExpectedAnswerDto,
    pub normalization_version: u32,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseAttemptDto {
    pub attempt_id: String,
    pub attempt_index: u32,
    pub correct: bool,
    pub selected: bool,
    pub feedback: String,
    pub explanation: Option<String>,
    pub expected_answer: ExpectedAnswerDto,
    pub normalization_version: u32,
    pub submitted_at: String,
    pub selected_at: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicExerciseItemDto {
    pub exercise_id: String,
    pub exercise_type: ExerciseType,
    pub prompt: String,
    pub instructions: Option<String>,
    pub hint: Option<String>,
    pub payload: PublicExercisePayload,
    pub attempts: Vec<ExerciseAttemptDto>,
    pub selected_attempt_id: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseStageSummaryDto {
    pub exercise_count: u32,
    pub selected_correct_count: u32,
    pub selected_incorrect_count: u32,
    pub total_attempt_count: u32,
    pub accuracy_percent: u32,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuidedExerciseStageDto {
    pub engine_version: u32,
    pub stage_schema_version: u32,
    pub attempt_result_version: u32,
    pub normalization_version: u32,
    pub current_exercise_index: u32,
    pub items: Vec<PublicExerciseItemDto>,
    pub summary: Option<ExerciseStageSummaryDto>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubmitExerciseAttemptRequest {
    pub session_id: String,
    pub stage_id: String,
    pub exercise_id: String,
    pub submission_id: String,
    pub response: ExerciseResponse,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectExerciseAttemptRequest {
    pub session_id: String,
    pub stage_id: String,
    pub exercise_id: String,
    pub attempt_id: String,
}

pub fn public_payload(item: &ExerciseItem) -> PublicExercisePayload {
    match &item.payload {
        ExercisePayload::SingleChoice { options, .. }
        | ExercisePayload::MultipleSelect { options, .. } => PublicExercisePayload::Options {
            options: options.clone(),
        },
        ExercisePayload::FillBlank { prefix, suffix, .. } => PublicExercisePayload::FillBlank {
            prefix: prefix.clone(),
            suffix: suffix.clone(),
        },
        ExercisePayload::WordOrder { tokens, .. } => PublicExercisePayload::WordOrder {
            tokens: tokens.clone(),
        },
        ExercisePayload::Matching {
            left_items,
            right_items,
            ..
        } => PublicExercisePayload::Matching {
            left_items: left_items.clone(),
            right_items: right_items.clone(),
        },
        ExercisePayload::ShortAnswerExact {
            normalization_profile,
            ..
        } => PublicExercisePayload::ShortAnswerExact {
            normalization_profile: normalization_profile.clone(),
        },
    }
}

pub fn grade(
    item: &ExerciseItem,
    response: &ExerciseResponse,
) -> Result<PersistedExerciseResult, String> {
    let (correct, expected) = match (&item.payload, response) {
        (
            ExercisePayload::SingleChoice {
                options,
                correct_option_id,
            },
            ExerciseResponse::SingleChoice(value),
        ) => {
            let chosen = options.iter().any(|o| o.option_id == value.option_id);
            if !chosen {
                return Err("Unknown single_choice optionId.".into());
            }
            (
                value.option_id == *correct_option_id,
                ExpectedAnswerDto::SingleChoice {
                    option: options
                        .iter()
                        .find(|o| o.option_id == *correct_option_id)
                        .unwrap()
                        .clone(),
                },
            )
        }
        (
            ExercisePayload::MultipleSelect {
                options,
                correct_option_ids,
            },
            ExerciseResponse::MultipleSelect(value),
        ) => {
            let selected = value.option_ids.iter().collect::<BTreeSet<_>>();
            if selected.len() != value.option_ids.len()
                || selected.is_empty()
                || selected
                    .iter()
                    .any(|id| !options.iter().any(|o| o.option_id == ***id))
            {
                return Err("multiple_select response contains invalid option IDs.".into());
            }
            let answer = correct_option_ids.iter().collect::<BTreeSet<_>>();
            (
                selected == answer,
                ExpectedAnswerDto::MultipleSelect {
                    options: correct_option_ids
                        .iter()
                        .map(|id| options.iter().find(|o| o.option_id == *id).unwrap().clone())
                        .collect(),
                },
            )
        }
        (
            ExercisePayload::FillBlank {
                accepted_answers, ..
            },
            ExerciseResponse::FillBlank(value),
        ) => grade_text(accepted_answers, &value.text, true)?,
        (
            ExercisePayload::ShortAnswerExact {
                accepted_answers, ..
            },
            ExerciseResponse::ShortAnswerExact(value),
        ) => grade_text(accepted_answers, &value.text, false)?,
        (
            ExercisePayload::WordOrder {
                tokens,
                correct_order,
            },
            ExerciseResponse::WordOrder(value),
        ) => {
            let ids = value.token_ids.iter().collect::<BTreeSet<_>>();
            if value.token_ids.len() != tokens.len()
                || ids.len() != tokens.len()
                || ids
                    .iter()
                    .any(|id| !tokens.iter().any(|t| t.token_id == ***id))
            {
                return Err("word_order response must use every tokenId exactly once.".into());
            }
            (
                value.token_ids == *correct_order,
                ExpectedAnswerDto::WordOrder {
                    tokens: correct_order
                        .iter()
                        .map(|id| tokens.iter().find(|t| t.token_id == *id).unwrap().clone())
                        .collect(),
                },
            )
        }
        (
            ExercisePayload::Matching {
                left_items,
                right_items,
                correct_pairs,
            },
            ExerciseResponse::Matching(value),
        ) => {
            validate_response_pairs(&value.pairs, left_items, right_items)?;
            let supplied = value
                .pairs
                .iter()
                .map(|p| (&p.left_id, &p.right_id))
                .collect::<BTreeMap<_, _>>();
            let answer = correct_pairs
                .iter()
                .map(|p| (&p.left_id, &p.right_id))
                .collect::<BTreeMap<_, _>>();
            let pairs = correct_pairs
                .iter()
                .map(|p| ExpectedMatchingPairDto {
                    left: left_items
                        .iter()
                        .find(|x| x.item_id == p.left_id)
                        .unwrap()
                        .clone(),
                    right: right_items
                        .iter()
                        .find(|x| x.item_id == p.right_id)
                        .unwrap()
                        .clone(),
                })
                .collect();
            (supplied == answer, ExpectedAnswerDto::Matching { pairs })
        }
        _ => return Err("Exercise response type does not match the current exercise.".into()),
    };
    Ok(PersistedExerciseResult {
        schema_version: EXERCISE_ATTEMPT_RESULT_VERSION,
        correct,
        feedback: if correct {
            item.feedback.correct.clone()
        } else {
            item.feedback.incorrect.clone()
        },
        explanation: item.feedback.explanation.clone(),
        expected_answer: expected,
        normalization_version: EXERCISE_NORMALIZATION_VERSION,
    })
}
fn grade_text(
    answers: &[String],
    text: &str,
    fill: bool,
) -> Result<(bool, ExpectedAnswerDto), String> {
    if text.chars().count() > MAX_TEXT_ANSWER || text.trim().is_empty() {
        return Err("Text exercise response must contain 1..200 characters.".into());
    }
    let actual = normalize_english_basic_v1(text);
    let correct = answers
        .iter()
        .any(|a| normalize_english_basic_v1(a) == actual);
    let expected = if fill {
        ExpectedAnswerDto::FillBlank {
            answer: answers[0].clone(),
        }
    } else {
        ExpectedAnswerDto::ShortAnswerExact {
            answer: answers[0].clone(),
        }
    };
    Ok((correct, expected))
}
fn validate_response_pairs(
    pairs: &[MatchingPair],
    left: &[MatchingItem],
    right: &[MatchingItem],
) -> Result<(), String> {
    if pairs.len() != left.len() {
        return Err("matching response must contain every pair.".into());
    }
    let mut l = BTreeSet::new();
    let mut r = BTreeSet::new();
    for p in pairs {
        if !left.iter().any(|x| x.item_id == p.left_id)
            || !right.iter().any(|x| x.item_id == p.right_id)
            || !l.insert(&p.left_id)
            || !r.insert(&p.right_id)
        {
            return Err("matching response must be a valid one-to-one mapping.".into());
        }
    }
    Ok(())
}

pub fn normalize_english_basic_v1(value: &str) -> String {
    let normalized = nfkc(value);
    let quotes = normalized
        .replace(['\u{2018}', '\u{2019}', '\u{02bc}'], "'")
        .replace(['\u{201c}', '\u{201d}'], "\"");
    let mut value = quotes
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    if matches!(value.chars().last(), Some('.' | '?' | '!')) {
        value.pop();
    }
    value
}
#[cfg(windows)]
fn nfkc(value: &str) -> String {
    #[link(name = "Normaliz")]
    extern "system" {
        fn NormalizeString(
            form: i32,
            source: *const u16,
            source_len: i32,
            destination: *mut u16,
            destination_len: i32,
        ) -> i32;
    }
    let source = value.encode_utf16().collect::<Vec<_>>();
    if source.is_empty() {
        return String::new();
    }
    let required = unsafe {
        NormalizeString(
            5,
            source.as_ptr(),
            source.len() as i32,
            std::ptr::null_mut(),
            0,
        )
    };
    if required <= 0 {
        return value.to_owned();
    }
    let mut target = vec![0u16; required as usize];
    let written = unsafe {
        NormalizeString(
            5,
            source.as_ptr(),
            source.len() as i32,
            target.as_mut_ptr(),
            required,
        )
    };
    if written <= 0 {
        return value.to_owned();
    }
    target.truncate(written as usize);
    String::from_utf16(&target).unwrap_or_else(|_| value.to_owned())
}
#[cfg(not(windows))]
fn nfkc(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            '\u{00a0}' => ' ',
            '\u{ff01}'..='\u{ff5e}' => char::from_u32(c as u32 - 0xfee0).unwrap(),
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item(payload: ExercisePayload, kind: ExerciseType) -> ExerciseItem {
        ExerciseItem {
            exercise_id: "item-one".into(),
            exercise_type: kind,
            prompt: "Prompt".into(),
            instructions: None,
            hint: None,
            payload,
            feedback: ExerciseFeedback {
                correct: "Good".into(),
                incorrect: "Try again".into(),
                explanation: Some("Explanation".into()),
            },
        }
    }
    #[test]
    fn normalization_is_exact_and_does_not_spellcheck_or_stem() {
        assert_eq!(
            normalize_english_basic_v1("  I\u{2019}m   HERE!  "),
            "i'm here"
        );
        assert_eq!(normalize_english_basic_v1("ＣＯＦＦＥＥ."), "coffee");
        assert_ne!(
            normalize_english_basic_v1("cofee"),
            normalize_english_basic_v1("coffee")
        );
        assert_ne!(
            normalize_english_basic_v1("walked"),
            normalize_english_basic_v1("walk")
        );
        assert_ne!(
            normalize_english_basic_v1("don't"),
            normalize_english_basic_v1("dont")
        );
        assert_ne!(
            normalize_english_basic_v1("hello, world"),
            normalize_english_basic_v1("hello world")
        );
    }
    #[test]
    fn single_and_multiple_are_exact() {
        let single = item(
            ExercisePayload::SingleChoice {
                options: vec![
                    ExerciseOption {
                        option_id: "a".into(),
                        text: "A".into(),
                    },
                    ExerciseOption {
                        option_id: "b".into(),
                        text: "B".into(),
                    },
                ],
                correct_option_id: "b".into(),
            },
            ExerciseType::SingleChoice,
        );
        assert!(
            grade(
                &single,
                &ExerciseResponse::SingleChoice(SingleChoiceResponse {
                    option_id: "b".into()
                })
            )
            .unwrap()
            .correct
        );
        assert!(
            !grade(
                &single,
                &ExerciseResponse::SingleChoice(SingleChoiceResponse {
                    option_id: "a".into()
                })
            )
            .unwrap()
            .correct
        );
        assert!(grade(
            &single,
            &ExerciseResponse::SingleChoice(SingleChoiceResponse {
                option_id: "z".into()
            })
        )
        .is_err());
        let multi = item(
            ExercisePayload::MultipleSelect {
                options: vec![
                    ExerciseOption {
                        option_id: "a".into(),
                        text: "A".into(),
                    },
                    ExerciseOption {
                        option_id: "b".into(),
                        text: "B".into(),
                    },
                    ExerciseOption {
                        option_id: "c".into(),
                        text: "C".into(),
                    },
                ],
                correct_option_ids: vec!["a".into(), "c".into()],
            },
            ExerciseType::MultipleSelect,
        );
        assert!(
            grade(
                &multi,
                &ExerciseResponse::MultipleSelect(MultipleSelectResponse {
                    option_ids: vec!["c".into(), "a".into()]
                })
            )
            .unwrap()
            .correct
        );
        assert!(
            !grade(
                &multi,
                &ExerciseResponse::MultipleSelect(MultipleSelectResponse {
                    option_ids: vec!["a".into()]
                })
            )
            .unwrap()
            .correct
        )
    }
    #[test]
    fn text_word_order_and_matching_are_deterministic() {
        let fill = item(
            ExercisePayload::FillBlank {
                prefix: "A ".into(),
                suffix: ".".into(),
                accepted_answers: vec!["coffee".into()],
                normalization_profile: "english_basic_v1".into(),
            },
            ExerciseType::FillBlank,
        );
        assert!(
            grade(
                &fill,
                &ExerciseResponse::FillBlank(TextResponse {
                    text: " COFFEE. ".into()
                })
            )
            .unwrap()
            .correct
        );
        assert!(
            !grade(
                &fill,
                &ExerciseResponse::FillBlank(TextResponse {
                    text: "cofee".into()
                })
            )
            .unwrap()
            .correct
        );
        let tokens = vec![
            ExerciseToken {
                token_id: "one".into(),
                text: "is".into(),
            },
            ExerciseToken {
                token_id: "two".into(),
                text: "is".into(),
            },
        ];
        let order = item(
            ExercisePayload::WordOrder {
                tokens: tokens.clone(),
                correct_order: vec!["two".into(), "one".into()],
            },
            ExerciseType::WordOrder,
        );
        assert!(
            grade(
                &order,
                &ExerciseResponse::WordOrder(WordOrderResponse {
                    token_ids: vec!["two".into(), "one".into()]
                })
            )
            .unwrap()
            .correct
        );
        assert!(grade(
            &order,
            &ExerciseResponse::WordOrder(WordOrderResponse {
                token_ids: vec!["one".into(), "one".into()]
            })
        )
        .is_err());
        let left = vec![
            MatchingItem {
                item_id: "l1".into(),
                text: "one".into(),
            },
            MatchingItem {
                item_id: "l2".into(),
                text: "two".into(),
            },
        ];
        let right = vec![
            MatchingItem {
                item_id: "r1".into(),
                text: "um".into(),
            },
            MatchingItem {
                item_id: "r2".into(),
                text: "dois".into(),
            },
        ];
        let matching = item(
            ExercisePayload::Matching {
                left_items: left.clone(),
                right_items: right.clone(),
                correct_pairs: vec![
                    MatchingPair {
                        left_id: "l1".into(),
                        right_id: "r1".into(),
                    },
                    MatchingPair {
                        left_id: "l2".into(),
                        right_id: "r2".into(),
                    },
                ],
            },
            ExerciseType::Matching,
        );
        assert!(
            grade(
                &matching,
                &ExerciseResponse::Matching(MatchingResponse {
                    pairs: vec![
                        MatchingPair {
                            left_id: "l2".into(),
                            right_id: "r2".into()
                        },
                        MatchingPair {
                            left_id: "l1".into(),
                            right_id: "r1".into()
                        }
                    ]
                })
            )
            .unwrap()
            .correct
        );
        assert!(grade(
            &matching,
            &ExerciseResponse::Matching(MatchingResponse {
                pairs: vec![
                    MatchingPair {
                        left_id: "l1".into(),
                        right_id: "r1".into()
                    },
                    MatchingPair {
                        left_id: "l2".into(),
                        right_id: "r1".into()
                    }
                ]
            })
        )
        .is_err())
    }
}
