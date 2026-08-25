use crate::{
    models::{
        DEFAULT_OLLAMA_MODEL, DEFAULT_PIPER_VOICE, DEFAULT_WHISPER_MODEL, DEFAULT_WHISPER_THREADS,
        OPTIONAL_WHISPER_MODELS,
    },
    ollama,
    paths::LocalAiPaths,
};
use reqwest::Client;
use serde::Serialize;
use std::{fs, path::Path};
use tokio::process::Command;

const OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalVoiceEngineProbe {
    pub project_root: String,
    pub local_ai_root: String,
    pub whisper: WhisperProbe,
    pub ollama: OllamaProbe,
    pub piper: PiperProbe,
    pub voice_defaults: VoiceDefaults,
    pub optional_components: OptionalComponents,
    pub offline_ready: bool,
    pub problems: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WhisperProbe {
    pub cli_found: bool,
    pub cli_path: String,
    pub stream_found: bool,
    pub stream_path: String,
    pub model_found: bool,
    pub model_path: String,
    pub model_name: &'static str,
    pub threads: u16,
    pub additional_models: Vec<OptionalModelProbe>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalModelProbe {
    pub name: &'static str,
    pub found: bool,
    pub path: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaProbe {
    pub reachable: bool,
    pub base_url: &'static str,
    pub model_found: bool,
    pub model_name: &'static str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PiperProbe {
    pub python_found: bool,
    pub python_path: String,
    pub installed: bool,
    pub version: Option<String>,
    pub voice_found: bool,
    pub voice_config_found: bool,
    pub voice_model_path: String,
    pub voice_config_path: String,
    pub voice_name: &'static str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceDefaults {
    pub whisper_model: &'static str,
    pub whisper_threads: u16,
    pub silence_to_stop_seconds: f32,
    pub pre_roll_seconds: f32,
    pub start_voice_blocks: u16,
    pub minimum_voice_threshold: u16,
    pub noise_multiplier: f32,
    pub piper_voice: &'static str,
    pub tts_start_silence_seconds: f32,
    pub ollama_model: &'static str,
    pub ollama_thinking: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalComponents {
    pub silero_found: bool,
    pub silero_path: String,
}

#[derive(Clone, Copy)]
struct RequiredComponents {
    whisper_cli: bool,
    whisper_model: bool,
    piper_python: bool,
    piper_installed: bool,
    piper_voice: bool,
    piper_voice_config: bool,
    ollama: bool,
    ollama_model: bool,
}

pub async fn run(paths: &LocalAiPaths, client: &Client) -> LocalVoiceEngineProbe {
    let whisper_cli = paths.whisper_cli();
    let whisper_stream = paths.whisper_stream();
    let whisper_model = paths.whisper_model(DEFAULT_WHISPER_MODEL);
    let piper_python = paths.piper_python();
    let (piper_voice, piper_voice_config) = find_piper_voice(paths);
    let silero = paths.silero_model();

    let python_found = piper_python.is_file();
    let piper_version = if python_found {
        detect_piper_version(&piper_python).await
    } else {
        None
    };
    let piper_installed = piper_version.is_some();

    let ollama_models = ollama::list_models(client).await;
    let ollama_reachable = ollama_models.is_ok();
    let ollama_model_found = ollama_models
        .as_ref()
        .map(|models| {
            models
                .iter()
                .any(|model| model.name == DEFAULT_OLLAMA_MODEL)
        })
        .unwrap_or(false);

    let required = RequiredComponents {
        whisper_cli: whisper_cli.is_file(),
        whisper_model: whisper_model.is_file(),
        piper_python: python_found,
        piper_installed,
        piper_voice: piper_voice.is_file(),
        piper_voice_config: piper_voice_config.is_file(),
        ollama: ollama_reachable,
        ollama_model: ollama_model_found,
    };
    let offline_ready = required.offline_ready();
    let problems = required.problems();

    LocalVoiceEngineProbe {
        project_root: display(&paths.project_root),
        local_ai_root: display(&paths.local_ai_root),
        whisper: WhisperProbe {
            cli_found: required.whisper_cli,
            cli_path: display(&whisper_cli),
            stream_found: whisper_stream.is_file(),
            stream_path: display(&whisper_stream),
            model_found: required.whisper_model,
            model_path: display(&whisper_model),
            model_name: DEFAULT_WHISPER_MODEL,
            threads: DEFAULT_WHISPER_THREADS,
            additional_models: OPTIONAL_WHISPER_MODELS
                .iter()
                .map(|name| {
                    let path = paths.whisper_model(name);
                    OptionalModelProbe {
                        name,
                        found: path.is_file(),
                        path: display(&path),
                    }
                })
                .collect(),
        },
        ollama: OllamaProbe {
            reachable: ollama_reachable,
            base_url: OLLAMA_BASE_URL,
            model_found: ollama_model_found,
            model_name: DEFAULT_OLLAMA_MODEL,
        },
        piper: PiperProbe {
            python_found,
            python_path: display(&piper_python),
            installed: piper_installed,
            version: piper_version,
            voice_found: required.piper_voice,
            voice_config_found: required.piper_voice_config,
            voice_model_path: display(&piper_voice),
            voice_config_path: display(&piper_voice_config),
            voice_name: DEFAULT_PIPER_VOICE,
        },
        voice_defaults: VoiceDefaults {
            whisper_model: DEFAULT_WHISPER_MODEL,
            whisper_threads: DEFAULT_WHISPER_THREADS,
            silence_to_stop_seconds: 3.5,
            pre_roll_seconds: 0.4,
            start_voice_blocks: 3,
            minimum_voice_threshold: 350,
            noise_multiplier: 3.0,
            piper_voice: DEFAULT_PIPER_VOICE,
            tts_start_silence_seconds: 0.5,
            ollama_model: DEFAULT_OLLAMA_MODEL,
            ollama_thinking: false,
        },
        optional_components: OptionalComponents {
            silero_found: silero.is_file(),
            silero_path: display(&silero),
        },
        offline_ready,
        problems,
    }
}

impl RequiredComponents {
    fn offline_ready(self) -> bool {
        self.whisper_cli
            && self.whisper_model
            && self.piper_python
            && self.piper_installed
            && self.piper_voice
            && self.piper_voice_config
            && self.ollama
            && self.ollama_model
    }

    fn problems(self) -> Vec<String> {
        let mut problems = Vec::new();
        if !self.whisper_cli {
            problems.push("Whisper CLI was not found at the resolved local path.".to_owned());
        }
        if !self.whisper_model {
            problems.push(format!(
                "Required Whisper model {DEFAULT_WHISPER_MODEL} was not found."
            ));
        }
        if !self.piper_python {
            problems.push("Piper virtual-environment Python was not found.".to_owned());
        }
        if self.piper_python && !self.piper_installed {
            problems.push(
                "The piper-tts package was not detected in the local environment.".to_owned(),
            );
        }
        if !self.piper_voice {
            problems.push(format!("Piper voice {DEFAULT_PIPER_VOICE} was not found."));
        }
        if !self.piper_voice_config {
            problems.push(format!(
                "Piper voice configuration for {DEFAULT_PIPER_VOICE} was not found."
            ));
        }
        if !self.ollama {
            problems.push("Ollama is not reachable on the local API.".to_owned());
        } else if !self.ollama_model {
            problems.push(format!(
                "Ollama model {DEFAULT_OLLAMA_MODEL} was not found."
            ));
        }
        problems
    }
}

async fn detect_piper_version(python: &Path) -> Option<String> {
    let output = Command::new(python)
        .args(["-m", "pip", "show", "piper-tts"])
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("Version:").map(str::trim))
        .filter(|version| !version.is_empty())
        .map(str::to_owned)
}

fn find_piper_voice(paths: &LocalAiPaths) -> (std::path::PathBuf, std::path::PathBuf) {
    let exact_model = paths.piper_voice(DEFAULT_PIPER_VOICE);
    let exact_config = exact_model.with_extension("onnx.json");
    if exact_model.is_file() || exact_config.is_file() {
        return (exact_model, exact_config);
    }

    let discovered = fs::read_dir(paths.piper_root())
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.extension().and_then(|extension| extension.to_str()) == Some("onnx")
                && path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(|stem| stem.starts_with(DEFAULT_PIPER_VOICE))
                    .unwrap_or(false)
        });

    discovered
        .map(|model| {
            let config = model.with_extension("onnx.json");
            (model, config)
        })
        .unwrap_or((exact_model, exact_config))
}

fn display(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::RequiredComponents;

    fn ready_components() -> RequiredComponents {
        RequiredComponents {
            whisper_cli: true,
            whisper_model: true,
            piper_python: true,
            piper_installed: true,
            piper_voice: true,
            piper_voice_config: true,
            ollama: true,
            ollama_model: true,
        }
    }

    #[test]
    fn silero_is_not_required_for_offline_readiness() {
        assert!(ready_components().offline_ready());
    }

    #[test]
    fn a_missing_required_component_makes_the_engine_not_ready() {
        let mut components = ready_components();
        components.whisper_model = false;

        assert!(!components.offline_ready());
        assert!(components
            .problems()
            .iter()
            .any(|problem| problem.contains("ggml-small.en-q5_1.bin")));
    }

    #[test]
    #[ignore = "manual physical probe of the current development computer"]
    fn physical_probe_reports_the_current_environment() {
        tauri::async_runtime::block_on(async {
            let paths = crate::paths::LocalAiPaths::resolve();
            let client = crate::ollama::client().expect("local HTTP client");
            let probe = super::run(&paths, &client).await;
            println!(
                "{}",
                serde_json::to_string_pretty(&probe).expect("serializable probe")
            );
            assert!(probe.offline_ready, "probe problems: {:?}", probe.problems);
        });
    }
}
