use std::{
    fs,
    path::{Path, PathBuf},
};

const OFFICIAL_PROJECT_ROOT: &str = r"C:\ENGLISH AI COACH";

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

#[derive(Clone, Debug)]
pub struct LocalAiPaths {
    pub project_root: PathBuf,
    pub local_ai_root: PathBuf,
}

impl LocalAiPaths {
    pub fn resolve() -> Self {
        if let Some(root) = std::env::var_os("ENGLISH_AI_COACH_ROOT")
            .map(PathBuf::from)
            .filter(|root| root.join("local-ai").is_dir())
        {
            return Self::from_project_root(root);
        }

        let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent();
        if let Some(root) = manifest_root.filter(|root| root.join("local-ai").is_dir()) {
            return Self::from_project_root(root.to_path_buf());
        }

        if let Some(root) = std::env::current_exe()
            .ok()
            .as_deref()
            .and_then(find_project_root)
        {
            return Self::from_project_root(root);
        }

        if let Some(root) = std::env::current_dir()
            .ok()
            .as_deref()
            .and_then(find_project_root)
        {
            return Self::from_project_root(root);
        }

        Self::from_project_root(PathBuf::from(OFFICIAL_PROJECT_ROOT))
    }

    pub fn from_project_root(project_root: PathBuf) -> Self {
        let local_ai_root = project_root.join("local-ai");
        Self {
            project_root,
            local_ai_root,
        }
    }

    pub fn whisper_root(&self) -> PathBuf {
        self.local_ai_root.join("whisper")
    }

    pub fn whisper_cli(&self) -> PathBuf {
        self.whisper_root()
            .join("build")
            .join("bin")
            .join("Release")
            .join("whisper-cli.exe")
    }

    pub fn whisper_stream(&self) -> PathBuf {
        self.whisper_root()
            .join("build-sdl")
            .join("bin")
            .join("Release")
            .join("whisper-stream.exe")
    }

    pub fn whisper_model(&self, name: &str) -> PathBuf {
        self.whisper_root().join(name)
    }

    pub fn piper_root(&self) -> PathBuf {
        self.local_ai_root.join("piper")
    }

    pub fn piper_python(&self) -> PathBuf {
        self.piper_root()
            .join(".venv")
            .join("Scripts")
            .join("python.exe")
    }

    pub fn voice_engine_bridge(&self) -> PathBuf {
        self.local_ai_root.join("voice_engine_bridge.py")
    }

    pub fn piper_voice(&self, name: &str) -> PathBuf {
        self.piper_root().join(format!("{name}.onnx"))
    }

    pub fn silero_model(&self) -> PathBuf {
        self.whisper_root().join("ggml-silero-v6.2.0.bin")
    }
}

fn find_project_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|candidate| candidate.join("local-ai").is_dir())
        .map(Path::to_path_buf)
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

#[cfg(test)]
mod tests {
    use super::LocalAiPaths;
    use std::path::PathBuf;

    #[test]
    fn constructs_local_ai_paths_from_an_injected_project_root() {
        let root = PathBuf::from(r"D:\portable\English Coach");
        let paths = LocalAiPaths::from_project_root(root.clone());

        assert_eq!(paths.project_root, root);
        assert_eq!(paths.local_ai_root, root.join("local-ai"));
        assert_eq!(
            paths.whisper_cli(),
            root.join("local-ai/whisper/build/bin/Release/whisper-cli.exe")
        );
        assert_eq!(
            paths.whisper_model("ggml-small.en-q5_1.bin"),
            root.join("local-ai/whisper/ggml-small.en-q5_1.bin")
        );
        assert_eq!(
            paths.piper_python(),
            root.join("local-ai/piper/.venv/Scripts/python.exe")
        );
        assert_eq!(
            paths.voice_engine_bridge(),
            root.join("local-ai/voice_engine_bridge.py")
        );
    }
}
