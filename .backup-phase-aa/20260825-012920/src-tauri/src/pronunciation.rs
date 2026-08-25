use serde::{Deserialize, Serialize};

pub const PRONUNCIATION_ENGINE_VERSION: u32 = 1;
pub const PRONUNCIATION_SCORE_VERSION: u32 = 1;
pub const PRONUNCIATION_RESULT_SCHEMA_VERSION: u32 = 1;
pub const PRONUNCIATION_MODEL_ID: &str = "facebook/wav2vec2-lv-60-espeak-cv-ft";
pub const PRONUNCIATION_MODEL_REVISION: &str = "ae45363bf3413b374fecd9dc8bc1df0e24c3b7f4";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzePronunciationRequest {
    pub target_text: String,
    pub audio_base64: String,
    pub source_type: String,
    pub source_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PronunciationPhoneResult {
    pub phone: String,
    pub score: f64,
    pub start_ms: u32,
    pub end_ms: u32,
    pub frame_count: u32,
    pub closest_alternative: Option<String>,
    pub hint: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PronunciationWordResult {
    pub index: u32,
    pub word: String,
    pub score: f64,
    pub start_ms: u32,
    pub end_ms: u32,
    pub expected_phones: Vec<String>,
    pub phone_results: Vec<PronunciationPhoneResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PronunciationResult {
    pub schema_version: u32,
    pub engine_version: u32,
    pub score_version: u32,
    pub status: String,
    pub target_text: String,
    pub locale: String,
    pub model_id: String,
    pub model_revision: String,
    pub model_manifest_hash: String,
    pub overall_score: Option<f64>,
    pub confidence: Option<String>,
    pub content_match_score: Option<f64>,
    pub alignment_coverage: Option<f64>,
    pub duration_ms: Option<u32>,
    pub words: Vec<PronunciationWordResult>,
    pub issues: Vec<PronunciationPhoneResult>,
    #[serde(default)]
    pub quality_warnings: Vec<String>,
    pub heard_text: Option<String>,
    pub analysis_ms: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PronunciationEngineStatus {
    pub installed: bool,
    pub available: bool,
    pub ready: bool,
    pub engine_version: u32,
    pub score_version: u32,
    pub result_schema_version: u32,
    pub model_id: String,
    pub model_revision: String,
    pub phonemizer_ready: bool,
    pub load_ms: Option<u32>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PronunciationAttemptDto {
    pub id: String,
    pub status: String,
    pub source_type: String,
    pub source_id: Option<String>,
    pub target_text: String,
    pub locale: String,
    pub overall_score: Option<f64>,
    pub confidence: Option<String>,
    pub content_match_score: Option<f64>,
    pub alignment_coverage: Option<f64>,
    pub audio_duration_ms: Option<u32>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub words: Vec<PronunciationWordResult>,
}

pub fn validate_target(target: &str) -> Result<Vec<String>, String> {
    if target.trim().is_empty() || target.chars().count() > 160 {
        return Err("Target must contain 1–160 characters.".into());
    }
    let words = target
        .split_whitespace()
        .filter(|value| value.chars().any(char::is_alphabetic))
        .map(|value| value.to_owned())
        .collect::<Vec<_>>();
    if words.is_empty() || words.len() > 12 {
        return Err("Target must contain between 1 and 12 words.".into());
    }
    Ok(words)
}

pub fn validate_result(result: &PronunciationResult, expected_target: &str) -> Result<(), String> {
    if !matches!(
        result.status.as_str(),
        "completed"
            | "content_mismatch"
            | "insufficient_audio"
            | "alignment_failed"
            | "engine_unavailable"
            | "cancelled"
            | "failed"
    ) {
        return Err("Pronunciation worker returned an invalid status.".into());
    }
    if result.schema_version != PRONUNCIATION_RESULT_SCHEMA_VERSION
        || result.engine_version != PRONUNCIATION_ENGINE_VERSION
        || result.score_version != PRONUNCIATION_SCORE_VERSION
        || result.model_id != PRONUNCIATION_MODEL_ID
        || result.model_revision != PRONUNCIATION_MODEL_REVISION
    {
        return Err("Pronunciation worker returned an unsupported version.".into());
    }
    if result.target_text != expected_target || result.locale != "en-US" {
        return Err("Pronunciation worker returned mismatched target metadata.".into());
    }
    if result.model_manifest_hash.len() != 64 {
        return Err("Pronunciation model manifest hash is invalid.".into());
    }
    for value in [result.overall_score, result.content_match_score] {
        if value.is_some_and(|value| !value.is_finite()) {
            return Err("Pronunciation result contains a non-finite score.".into());
        }
    }
    if result
        .overall_score
        .is_some_and(|value| !(0.0..=100.0).contains(&value))
        || result
            .content_match_score
            .is_some_and(|value| !(0.0..=1.0).contains(&value))
        || result
            .alignment_coverage
            .is_some_and(|value| !(0.0..=1.0).contains(&value))
    {
        return Err("Pronunciation result score is outside its valid range.".into());
    }
    let target_words = validate_target(expected_target)?;
    if result.status == "completed" && result.words.len() != target_words.len() {
        return Err("Pronunciation worker returned an invalid word mapping.".into());
    }
    if result.status == "completed" && result.overall_score.is_none() {
        return Err("Completed pronunciation result is missing its acoustic score.".into());
    }
    if result.status != "completed" && result.overall_score.is_some() {
        return Err("Non-completed pronunciation result cannot contain an overall score.".into());
    }
    for (index, word) in result.words.iter().enumerate() {
        if word.index as usize != index || word.phone_results.len() != word.expected_phones.len() {
            return Err("Pronunciation worker returned an invalid phone mapping.".into());
        }
        if word.end_ms < word.start_ms || result.duration_ms.is_some_and(|d| word.end_ms > d) {
            return Err("Pronunciation worker returned invalid word timing.".into());
        }
        for (phone, expected) in word.phone_results.iter().zip(&word.expected_phones) {
            if &phone.phone != expected
                || phone.end_ms < phone.start_ms
                || result.duration_ms.is_some_and(|d| phone.end_ms > d)
                || !phone.score.is_finite()
                || !(0.0..=100.0).contains(&phone.score)
            {
                return Err("Pronunciation worker returned invalid phoneme evidence.".into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn result() -> PronunciationResult {
        PronunciationResult {
            schema_version: 1,
            engine_version: 1,
            score_version: 1,
            status: "completed".into(),
            target_text: "think".into(),
            locale: "en-US".into(),
            model_id: PRONUNCIATION_MODEL_ID.into(),
            model_revision: PRONUNCIATION_MODEL_REVISION.into(),
            model_manifest_hash: "a".repeat(64),
            overall_score: Some(80.0),
            confidence: Some("high".into()),
            content_match_score: Some(1.0),
            alignment_coverage: Some(1.0),
            duration_ms: Some(500),
            words: vec![PronunciationWordResult {
                index: 0,
                word: "think".into(),
                score: 80.0,
                start_ms: 10,
                end_ms: 400,
                expected_phones: vec!["θ".into()],
                phone_results: vec![PronunciationPhoneResult {
                    phone: "θ".into(),
                    score: 80.0,
                    start_ms: 10,
                    end_ms: 400,
                    frame_count: 2,
                    closest_alternative: None,
                    hint: None,
                }],
            }],
            issues: vec![],
            quality_warnings: vec![],
            heard_text: Some("think".into()),
            analysis_ms: Some(10),
        }
    }
    #[test]
    fn validates_target_limits() {
        assert!(validate_target("think").is_ok());
        assert!(validate_target("").is_err());
        assert!(validate_target(&"x ".repeat(13)).is_err())
    }
    #[test]
    fn rejects_fake_score() {
        let mut value = result();
        value.overall_score = Some(140.0);
        assert!(validate_result(&value, "think").is_err())
    }
    #[test]
    fn rejects_invalid_timing() {
        let mut value = result();
        value.words[0].phone_results[0].end_ms = 600;
        assert!(validate_result(&value, "think").is_err())
    }
    #[test]
    fn accepts_valid_acoustic_result() {
        assert!(validate_result(&result(), "think").is_ok())
    }
}
