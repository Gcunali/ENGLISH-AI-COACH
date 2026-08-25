use crate::models::{OllamaModel, TimedText};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::{
    path::Path,
    time::{Duration, Instant},
};

const OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";
const TEACHER_PROMPT: &str = include_str!("../prompts/conversation_teacher.txt");
#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<TagModel>,
}
#[derive(Deserialize)]
struct TagModel {
    name: String,
    size: u64,
    modified_at: String,
}
#[derive(Deserialize)]
struct ChatResponse {
    message: ChatMessage,
}
#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

pub fn client() -> Result<Client, String> {
    Client::builder()
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(120))
        .no_proxy()
        .build()
        .map_err(|error| format!("Could not create local AI client: {error}"))
}
pub async fn health(client: &Client) -> bool {
    client
        .get(format!("{OLLAMA_BASE_URL}/api/tags"))
        .send()
        .await
        .map(|response| response.status().is_success())
        .unwrap_or(false)
}
pub async fn list_models(client: &Client) -> Result<Vec<OllamaModel>, String> {
    let response = client
        .get(format!("{OLLAMA_BASE_URL}/api/tags"))
        .send()
        .await
        .map_err(|_| "Local AI component unavailable. Start Ollama and try again.".to_owned())?;
    if !response.status().is_success() {
        return Err(format!("Ollama returned {}", response.status()));
    }
    let tags: TagsResponse = response
        .json()
        .await
        .map_err(|error| format!("Invalid response from local Ollama: {error}"))?;
    Ok(tags
        .models
        .into_iter()
        .map(|model| OllamaModel {
            name: model.name,
            size: model.size,
            modified_at: model.modified_at,
        })
        .collect())
}
pub async fn chat(
    client: &Client,
    database: &Path,
    model: &str,
    student_text: &str,
) -> Result<TimedText, String> {
    validate_model_name(model)?;
    if student_text.trim().is_empty() || student_text.len() > 8_000 {
        return Err("Student message is empty or too long.".to_owned());
    }
    let started = Instant::now();
    let body = json!({
        "model": model, "stream": false, "think": false, "keep_alive": "10m",
        "options": { "temperature": 0.65, "top_p": 0.9, "num_predict": 180 },
        "messages": [{ "role": "system", "content": TEACHER_PROMPT }, { "role": "user", "content": student_text }]
    });
    let response = client
        .post(format!("{OLLAMA_BASE_URL}/api/chat"))
        .json(&body)
        .send()
        .await
        .map_err(|_| "Local AI component unavailable. Make sure Ollama is running.".to_owned())?;
    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        return Err(format!(
            "Local model request failed ({status}): {}",
            compact_error(&detail)
        ));
    }
    let payload: ChatResponse = response
        .json()
        .await
        .map_err(|error| format!("Invalid Ollama response: {error}"))?;
    let text = payload.message.content.trim().to_owned();
    if text.is_empty() {
        return Err("The local model returned an empty response.".to_owned());
    }
    crate::database::save_exchange(database, student_text, &text)?;
    Ok(TimedText {
        text,
        elapsed_ms: started.elapsed().as_millis(),
    })
}
fn validate_model_name(model: &str) -> Result<(), String> {
    if model.is_empty()
        || model.len() > 120
        || !model
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | ':' | '/'))
    {
        return Err("Invalid local model name.".to_owned());
    }
    Ok(())
}
fn compact_error(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(240)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{compact_error, validate_model_name};

    #[test]
    fn accepts_local_registry_model_names() {
        assert!(validate_model_name("qwen3.5:4b").is_ok());
        assert!(validate_model_name("library/my-model:latest").is_ok());
    }

    #[test]
    fn rejects_model_name_injection() {
        assert!(validate_model_name("qwen; curl example.com").is_err());
        assert!(validate_model_name("").is_err());
    }

    #[test]
    fn removes_control_characters_from_local_errors() {
        assert_eq!(compact_error("bad\r\nrequest"), "badrequest");
    }
}
