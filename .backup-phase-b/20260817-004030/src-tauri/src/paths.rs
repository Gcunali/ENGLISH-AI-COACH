use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct LocalPaths {
    pub root: PathBuf,
    pub models: PathBuf,
    pub voices: PathBuf,
    pub temporary_audio: PathBuf,
    pub database: PathBuf,
    pub logs: PathBuf,
    pub tools: PathBuf,
}

impl LocalPaths {
    pub fn create(root: PathBuf) -> Result<Self, String> {
        let paths = Self {
            models: root.join("models"),
            voices: root.join("voices"),
            temporary_audio: root.join("temporary_audio"),
            database: root.join("database"),
            logs: root.join("logs"),
            tools: root.join("tools"),
            root,
        };
        for directory in [
            &paths.root,
            &paths.models,
            &paths.voices,
            &paths.temporary_audio,
            &paths.database,
            &paths.logs,
            &paths.tools,
        ] {
            fs::create_dir_all(directory)
                .map_err(|error| format!("Could not create {}: {error}", directory.display()))?;
        }
        paths.cleanup_temporary_audio();
        Ok(paths)
    }
    pub fn whisper_executable(&self) -> Option<PathBuf> {
        first_existing(&[
            self.tools.join("whisper/whisper-cli.exe"),
            self.tools.join("whisper-cli.exe"),
        ])
        .or_else(|| executable_on_path("whisper-cli.exe"))
    }
    pub fn piper_executable(&self) -> Option<PathBuf> {
        first_existing(&[
            self.tools.join("piper/piper.exe"),
            self.tools.join("piper.exe"),
        ])
        .or_else(|| executable_on_path("piper.exe"))
    }
    pub fn whisper_model(&self) -> PathBuf {
        self.models.join("whisper/ggml-base.en.bin")
    }
    pub fn vad_model(&self) -> PathBuf {
        self.models.join("whisper/ggml-silero-v6.2.0.bin")
    }
    pub fn piper_voice(&self) -> PathBuf {
        self.voices.join("en_US-lessac-medium.onnx")
    }
    pub fn db_file(&self) -> PathBuf {
        self.database.join("english-ai-coach.sqlite3")
    }
    fn cleanup_temporary_audio(&self) {
        let Ok(entries) = fs::read_dir(&self.temporary_audio) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) == Some("wav") {
                let _ = fs::remove_file(path);
            }
        }
    }
}
fn first_existing(paths: &[PathBuf]) -> Option<PathBuf> {
    paths.iter().find(|path| path.is_file()).cloned()
}
fn executable_on_path(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|directory| directory.join(name))
            .find(|path| path.is_file())
    })
}
pub fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown")
        .to_owned()
}
