use crate::placement::{
    CefrBand, PlacementConfidence, PlacementSpeakingEvidenceDto, PLACEMENT_SPEAKING_SCHEMA_VERSION,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{future::Future, time::Duration};

pub const SPEAKING_EVALUATOR_PROMPT: &str =
    include_str!("../prompts/placement_speaking_evaluator_v1.txt");
const OLLAMA_CHAT_URL: &str = "http://127.0.0.1:11434/api/chat";
const MODEL: &str = "qwen3.5:4b";
const TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Clone, Debug)]
pub struct SpeakingSample {
    pub prompt: String,
    pub transcript: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeakingEvidence {
    pub criterion: String,
    pub observation: String,
    pub example: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeakingEvaluationPayload {
    pub schema_version: u32,
    pub estimated_band: CefrBand,
    pub confidence: PlacementConfidence,
    pub evidence: Vec<SpeakingEvidence>,
    pub summary: String,
    pub pronunciation_available: bool,
}

#[derive(Clone, Debug)]
pub struct ValidatedSpeakingEvaluation {
    pub payload: SpeakingEvaluationPayload,
    pub canonical_json: String,
}

impl ValidatedSpeakingEvaluation {
    pub fn evidence_dtos(&self) -> Vec<PlacementSpeakingEvidenceDto> {
        self.payload
            .evidence
            .iter()
            .map(|e| PlacementSpeakingEvidenceDto {
                criterion: e.criterion.clone(),
                observation: e.observation.clone(),
                example: e.example.clone(),
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct PlacementSpeakingEvaluator {
    client: Client,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ChatMessage,
}
#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

impl PlacementSpeakingEvaluator {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            client: Client::builder()
                .connect_timeout(Duration::from_secs(2))
                .timeout(TIMEOUT)
                .no_proxy()
                .build()
                .map_err(|e| format!("Could not create local placement evaluator: {e}"))?,
        })
    }
    pub async fn evaluate(
        &self,
        samples: &[SpeakingSample],
    ) -> Result<ValidatedSpeakingEvaluation, String> {
        let input=serde_json::to_string(&samples.iter().enumerate().map(|(i,s)|json!({"sequenceIndex":i,"prompt":s.prompt,"studentTranscript":s.transcript})).collect::<Vec<_>>()).map_err(|e|e.to_string())?;
        let initial = self.request(SPEAKING_EVALUATOR_PROMPT, &input, 0.1).await?;
        parse_with_one_repair(initial, samples, |repair| async move {
            self.request(
                "You repair an existing placement speaking evaluation JSON. Do not re-evaluate or add facts.",
                &repair,
                0.0,
            ).await
        })
        .await
    }
    async fn request(&self, system: &str, user: &str, temperature: f32) -> Result<String, String> {
        let body = json!({"model":MODEL,"stream":false,"think":false,"format":"json","keep_alive":"10m","options":{"temperature":temperature,"top_p":0.9,"num_predict":700,"num_ctx":4096},"messages":[{"role":"system","content":system},{"role":"user","content":user}]});
        let response = self
            .client
            .post(OLLAMA_CHAT_URL)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Local placement evaluator request failed: {e}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "Local placement evaluator returned {}.",
                response.status()
            ));
        }
        let payload: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Invalid local placement evaluator response: {e}"))?;
        if payload.message.content.trim().is_empty() {
            Err("Local placement evaluator returned an empty response.".into())
        } else {
            Ok(payload.message.content)
        }
    }
}

async fn parse_with_one_repair<F, Fut>(
    initial: String,
    samples: &[SpeakingSample],
    repair: F,
) -> Result<ValidatedSpeakingEvaluation, String>
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Result<String, String>>,
{
    match parse_and_validate(&initial, samples) {
        Ok(value) => Ok(value),
        Err(first) => {
            let request = format!("Repair only the JSON structure below. Preserve substantive values and evidence. Return JSON only. Validation error: {first}\n\n{initial}");
            let repaired = repair(request).await?;
            parse_and_validate(&repaired, samples).map_err(|second| {
                format!("Speaking evaluator output invalid ({first}); repair invalid ({second})")
            })
        }
    }
}

pub fn parse_and_validate(
    raw: &str,
    samples: &[SpeakingSample],
) -> Result<ValidatedSpeakingEvaluation, String> {
    let trimmed = raw.trim();
    let json_text = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|v| v.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or(trimmed);
    let payload: SpeakingEvaluationPayload =
        serde_json::from_str(json_text).map_err(|e| format!("Invalid speaking JSON: {e}"))?;
    if payload.schema_version != PLACEMENT_SPEAKING_SCHEMA_VERSION {
        return Err("Unsupported speaking schema version.".into());
    }
    if payload.pronunciation_available {
        return Err("Pronunciation must not be evaluated.".into());
    }
    if payload.summary.trim().is_empty() || payload.summary.chars().count() > 600 {
        return Err("Speaking summary is empty or too long.".into());
    }
    if payload.evidence.is_empty() || payload.evidence.len() > 5 {
        return Err("Speaking evidence must contain 1 to 5 items.".into());
    }
    let combined = samples
        .iter()
        .map(|s| s.transcript.as_str())
        .collect::<Vec<_>>()
        .join("\n")
        .to_lowercase();
    let allowed = [
        "grammatical_control",
        "lexical_range",
        "coherence_development",
        "task_response",
        "spoken_language_complexity",
    ];
    for evidence in &payload.evidence {
        if !allowed.contains(&evidence.criterion.as_str())
            || evidence.observation.trim().is_empty()
            || evidence.example.trim().is_empty()
        {
            return Err("Invalid speaking evidence.".into());
        }
        if evidence.observation.chars().count() > 400 || evidence.example.chars().count() > 180 {
            return Err("Speaking evidence is too long.".into());
        }
        if !combined.contains(&evidence.example.trim().to_lowercase()) {
            return Err("Speaking evidence example is not present in the transcript.".into());
        }
    }
    let canonical_json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    Ok(ValidatedSpeakingEvaluation {
        payload,
        canonical_json,
    })
}

pub fn parse_persisted(raw: &str) -> Result<ValidatedSpeakingEvaluation, String> {
    let payload: SpeakingEvaluationPayload = serde_json::from_str(raw)
        .map_err(|e| format!("Invalid persisted speaking evaluation: {e}"))?;
    let canonical_json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    Ok(ValidatedSpeakingEvaluation {
        payload,
        canonical_json,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn samples() -> Vec<SpeakingSample> {
        vec![
            SpeakingSample {
                prompt: "Describe a place".into(),
                transcript: "I enjoy the quiet park because I can read there every weekend.".into(),
            },
            SpeakingSample {
                prompt: "Tell a story".into(),
                transcript:
                    "My plan changed when the bus stopped, so I called a friend and arrived later."
                        .into(),
            },
        ]
    }
    fn valid() -> String {
        r#"{"schemaVersion":1,"estimatedBand":"B1","confidence":"medium","evidence":[{"criterion":"grammatical_control","observation":"Uses connected clauses.","example":"because I can read there"}],"summary":"The student develops familiar ideas clearly.","pronunciationAvailable":false}"#.into()
    }
    #[test]
    fn validates_real_transcript_evidence() {
        assert!(parse_and_validate(&valid(), &samples()).is_ok());
    }
    #[test]
    fn rejects_invalid_band_confidence_pronunciation_schema_and_invented_evidence() {
        for bad in [
            valid().replace("\"B1\"", "\"Z9\""),
            valid().replace("\"medium\"", "\"certain\""),
            valid().replace("false", "true"),
            valid().replace("\"schemaVersion\":1", "\"schemaVersion\":2"),
            valid().replace("because I can read there", "invented words"),
        ] {
            assert!(parse_and_validate(&bad, &samples()).is_err());
        }
    }
    #[test]
    fn known_markdown_wrapper_is_accepted() {
        assert!(parse_and_validate(&format!("```json\n{}\n```", valid()), &samples()).is_ok());
    }
    #[test]
    fn one_repair_can_recover_but_never_retries_twice() {
        tauri::async_runtime::block_on(async {
            let recovered =
                parse_with_one_repair("not json".into(), &samples(), |_| async { Ok(valid()) })
                    .await;
            assert!(recovered.is_ok());
            let failed = parse_with_one_repair("not json".into(), &samples(), |_| async {
                Ok("still invalid".into())
            })
            .await;
            assert!(failed.is_err());
        });
    }
}
