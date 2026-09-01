use crate::{
    models::{TimedText, DEFAULT_WHISPER_MODEL, DEFAULT_WHISPER_THREADS},
    paths::{LocalAiPaths, LocalPaths},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::{fs, process::Command, time::Instant};

pub async fn transcribe(
    paths: LocalPaths,
    local_ai: LocalAiPaths,
    audio_base64: String,
) -> Result<TimedText, String> {
    let started = Instant::now();
    let audio = STANDARD
        .decode(audio_base64)
        .map_err(|_| "Placement microphone audio payload is invalid.".to_owned())?;
    if audio.len() < 48 || audio.len() > 8_000_000 {
        return Err("Placement recording is empty or too long.".to_owned());
    }
    let executable = local_ai.whisper_cli();
    let model = local_ai.whisper_model(DEFAULT_WHISPER_MODEL);
    if !executable.is_file() || !model.is_file() {
        return Err(format!(
            "Placement Whisper is unavailable. Expected {} and {}.",
            executable.display(),
            model.display()
        ));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let input = paths
        .temporary_audio
        .join(format!("placement-{id}-input.wav"));
    let output_base = paths
        .temporary_audio
        .join(format!("placement-{id}-transcript"));
    let output_text = output_base.with_extension("txt");
    fs::write(&input, audio)
        .map_err(|error| format!("Could not create temporary placement audio: {error}"))?;
    let vad = local_ai.silero_model();
    let threads = DEFAULT_WHISPER_THREADS.to_string();
    let mut command = Command::new(executable);
    command
        .arg("-m")
        .arg(model)
        .arg("-f")
        .arg(&input)
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
    let result = tokio::task::spawn_blocking(move || command.output())
        .await
        .map_err(|error| format!("Placement Whisper task failed: {error}"))?
        .map_err(|error| format!("Could not start placement Whisper: {error}"));
    let _ = fs::remove_file(&input);
    let output = result?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        let _ = fs::remove_file(&output_text);
        return Err(format!(
            "Local placement transcription failed: {}",
            detail.chars().take(300).collect::<String>()
        ));
    }
    let text = fs::read_to_string(&output_text)
        .map_err(|error| format!("Placement Whisper did not create a transcript: {error}"))?;
    let _ = fs::remove_file(&output_text);
    Ok(TimedText {
        text: text.trim().to_owned(),
        elapsed_ms: started.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn placement_uses_validated_voice_v2_whisper_defaults() {
        assert_eq!(DEFAULT_WHISPER_MODEL, "ggml-small.en-q5_1.bin");
        assert_eq!(DEFAULT_WHISPER_THREADS, 12);
    }
}
