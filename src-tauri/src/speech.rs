use crate::{
    models::{SpeechAudio, TimedText},
    paths::{LocalAiPaths, LocalPaths},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::{fs, process::Command, time::Instant};

pub async fn transcribe(paths: LocalPaths, audio_base64: String) -> Result<TimedText, String> {
    let started = Instant::now();
    let audio = STANDARD
        .decode(audio_base64)
        .map_err(|_| "Microphone audio payload is invalid.".to_owned())?;
    if audio.len() < 48 || audio.len() > 4_000_000 {
        return Err("Recorded utterance is empty or too long.".to_owned());
    }
    let executable = paths.whisper_executable().ok_or_else(|| {
        "Local AI component unavailable. Install whisper.cpp in the app tools folder.".to_owned()
    })?;
    let model = paths.whisper_model();
    if !model.is_file() {
        return Err(format!(
            "Whisper model missing. Place ggml-base.en.bin in {}",
            model.parent().unwrap_or(&paths.models).display()
        ));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let input = paths.temporary_audio.join(format!("{id}-input.wav"));
    let output_base = paths.temporary_audio.join(format!("{id}-transcript"));
    let output_text = output_base.with_extension("txt");
    fs::write(&input, audio)
        .map_err(|error| format!("Could not create temporary audio: {error}"))?;
    let vad = paths.vad_model();
    let mut command = Command::new(executable);
    command
        .arg("-m")
        .arg(model)
        .arg("-f")
        .arg(&input)
        .args(["-l", "en", "-otxt", "-np", "-nt", "-of"])
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
        .map_err(|error| format!("Whisper task failed: {error}"))?
        .map_err(|error| format!("Could not start whisper.cpp: {error}"));
    let _ = fs::remove_file(&input);
    let output = result?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        let _ = fs::remove_file(&output_text);
        return Err(format!(
            "Local transcription failed: {}",
            detail.chars().take(300).collect::<String>()
        ));
    }
    let text = fs::read_to_string(&output_text)
        .map_err(|error| format!("Whisper did not create a transcript: {error}"))?;
    let _ = fs::remove_file(&output_text);
    Ok(TimedText {
        text: text.trim().to_owned(),
        elapsed_ms: started.elapsed().as_millis(),
    })
}

pub async fn transcribe_official_local_ai(
    paths: LocalPaths,
    local_ai: LocalAiPaths,
    audio_base64: String,
) -> Result<TimedText, String> {
    let started = Instant::now();
    let audio = STANDARD
        .decode(audio_base64)
        .map_err(|_| "Microphone audio payload is invalid.".to_owned())?;
    if audio.len() < 48 || audio.len() > 4_000_000 {
        return Err("Recorded utterance is empty or too long.".into());
    }
    let executable = local_ai.whisper_cli();
    let model = local_ai.whisper_model("ggml-small.en-q5_1.bin");
    if !executable.is_file() || !model.is_file() {
        return Err("The validated local Whisper engine is unavailable.".into());
    }
    let id = uuid::Uuid::new_v4().to_string();
    let input = paths
        .temporary_audio
        .join(format!("{id}-practice-input.wav"));
    let output_base = paths
        .temporary_audio
        .join(format!("{id}-practice-transcript"));
    let output_text = output_base.with_extension("txt");
    fs::write(&input, audio)
        .map_err(|error| format!("Could not create temporary audio: {error}"))?;
    let command_input = input.clone();
    let output = tokio::task::spawn_blocking(move || {
        Command::new(executable)
            .arg("-m")
            .arg(model)
            .arg("-f")
            .arg(&command_input)
            .args(["-l", "en", "-t", "12", "--output-txt", "--output-file"])
            .arg(&output_base)
            .arg("-np")
            .output()
    })
    .await
    .map_err(|error| format!("Whisper task failed: {error}"))?
    .map_err(|error| format!("Could not start Whisper: {error}"));
    let _ = fs::remove_file(&input);
    let output = output?;
    if !output.status.success() {
        let _ = fs::remove_file(&output_text);
        return Err(format!(
            "Local transcription failed: {}",
            String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(300)
                .collect::<String>()
        ));
    }
    let text = fs::read_to_string(&output_text)
        .map_err(|error| format!("Whisper did not create a transcript: {error}"))?;
    let _ = fs::remove_file(&output_text);
    Ok(TimedText {
        text: text.trim().into(),
        elapsed_ms: started.elapsed().as_millis(),
    })
}

pub async fn synthesize(paths: LocalPaths, text: String) -> Result<SpeechAudio, String> {
    let started = Instant::now();
    let clean_text = sanitize_for_speech(&text);
    if clean_text.is_empty() || clean_text.len() > 4_000 {
        return Err("Teacher speech is empty or too long.".to_owned());
    }
    let executable = paths.piper_executable().ok_or_else(|| {
        "Local AI component unavailable. Install Piper in the app tools folder.".to_owned()
    })?;
    let voice = paths.piper_voice();
    if !voice.is_file() || !voice.with_extension("onnx.json").is_file() {
        return Err(format!(
            "Piper voice missing. Add the ONNX model and JSON config to {}",
            paths.voices.display()
        ));
    }
    let output_path = paths
        .temporary_audio
        .join(format!("{}-teacher.wav", uuid::Uuid::new_v4()));
    let mut command = Command::new(executable);
    command
        .arg("-m")
        .arg(voice)
        .arg("-f")
        .arg(&output_path)
        .arg("--")
        .arg(clean_text);
    let result = tokio::task::spawn_blocking(move || command.output())
        .await
        .map_err(|error| format!("Piper task failed: {error}"))?
        .map_err(|error| format!("Could not start Piper: {error}"))?;
    if !result.status.success() {
        return Err(format!(
            "Local voice synthesis failed: {}",
            String::from_utf8_lossy(&result.stderr)
                .chars()
                .take(300)
                .collect::<String>()
        ));
    }
    let audio =
        fs::read(&output_path).map_err(|error| format!("Piper did not create audio: {error}"))?;
    let _ = fs::remove_file(&output_path);
    Ok(SpeechAudio {
        audio_base64: STANDARD.encode(audio),
        mime_type: "audio/wav",
        elapsed_ms: started.elapsed().as_millis(),
    })
}

pub fn sanitize_for_speech(input: &str) -> String {
    input
        .replace("```", " ")
        .replace(['*', '#', '`', '_', '>'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::sanitize_for_speech;
    #[test]
    fn removes_markdown_for_tts() {
        assert_eq!(
            sanitize_for_speech("## Small **correction**"),
            "Small correction"
        );
    }
}
