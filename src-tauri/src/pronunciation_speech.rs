use crate::{
    models::{DEFAULT_WHISPER_MODEL, DEFAULT_WHISPER_THREADS},
    paths::LocalAiPaths,
};
use std::{fs, path::Path, process::Command, time::Instant};

pub async fn transcribe_path(
    local_ai: LocalAiPaths,
    input: &Path,
    output_directory: &Path,
) -> Result<(String, u32), String> {
    let started = Instant::now();
    let executable = local_ai.whisper_cli();
    let model = local_ai.whisper_model(DEFAULT_WHISPER_MODEL);
    if !executable.is_file() || !model.is_file() {
        return Err("Pronunciation content-check Whisper is unavailable.".into());
    }
    let output_base =
        output_directory.join(format!("pronunciation-{}-heard", uuid::Uuid::new_v4()));
    let output_text = output_base.with_extension("txt");
    let vad = local_ai.silero_model();
    let threads = DEFAULT_WHISPER_THREADS.to_string();
    let mut command = Command::new(executable);
    command
        .arg("-m")
        .arg(model)
        .arg("-f")
        .arg(input)
        .args(["-l", "en", "-t", &threads, "-otxt", "-np", "-nt", "-of"])
        .arg(&output_base);
    if vad.is_file() {
        command.arg("--vad").arg("--vad-model").arg(vad).args([
            "--vad-min-speech-duration-ms",
            "250",
            "--vad-min-silence-duration-ms",
            "100",
        ]);
    }
    let output = tokio::task::spawn_blocking(move || command.output())
        .await
        .map_err(|e| format!("Pronunciation Whisper task failed: {e}"))?
        .map_err(|e| format!("Could not start pronunciation Whisper: {e}"))?;
    if !output.status.success() {
        let _ = fs::remove_file(&output_text);
        return Err(format!(
            "Pronunciation content check failed: {}",
            String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(300)
                .collect::<String>()
        ));
    }
    let text = fs::read_to_string(&output_text)
        .map_err(|e| format!("Pronunciation Whisper did not create a transcript: {e}"))?;
    let _ = fs::remove_file(output_text);
    Ok((
        text.trim().to_owned(),
        started.elapsed().as_millis().min(u32::MAX as u128) as u32,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_content_check_defaults() {
        assert_eq!(DEFAULT_WHISPER_MODEL, "ggml-small.en-q5_1.bin");
        assert_eq!(DEFAULT_WHISPER_THREADS, 12)
    }
}
