use crate::{
    interactive_lesson::*,
    paths::{LocalAiPaths, LocalPaths},
    sha256,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    process::Command,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const BLUETOOTH_WAKE_MS: u64 = 500;
const PIPER_VOICE: &str = "en_US-lessac-medium";
const STATIC_TTS_CACHE_FORMAT_VERSION: u32 = 1;
const STATIC_TTS_ENGINE_VERSION: u32 = 1;
const STATIC_TTS_CACHE_MAX_BYTES: u64 = 250 * 1024 * 1024;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuidedAudioDto {
    pub playback_id: String,
    pub audio_base64: String,
    pub mime_type: String,
    pub source: String,
    pub duration_ms: u64,
    pub runtime_version: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticTtsCacheStatusDto {
    pub format_version: u32,
    pub entry_count: u64,
    pub size_bytes: u64,
    pub max_size_bytes: u64,
}

#[derive(Clone)]
pub struct GuidedLessonAudioRuntime {
    inner: Arc<Mutex<RuntimeState>>,
}

#[derive(Default)]
struct RuntimeState {
    active: Option<ActivePlayback>,
    cache: BTreeMap<String, std::path::PathBuf>,
}

struct ActivePlayback {
    id: String,
    request: GuidedPlaybackRequest,
    not_before: Instant,
}

impl Default for GuidedLessonAudioRuntime {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RuntimeState::default())),
        }
    }
}

impl GuidedLessonAudioRuntime {
    pub async fn prepare(
        &self,
        paths: LocalPaths,
        local_ai: LocalAiPaths,
        request: GuidedPlaybackRequest,
        source: GuidedPlaybackSource,
    ) -> Result<GuidedAudioDto, String> {
        self.prepare_with_voice(paths, local_ai, request, source, PIPER_VOICE)
            .await
    }
    pub async fn prepare_with_voice(
        &self,
        paths: LocalPaths,
        local_ai: LocalAiPaths,
        request: GuidedPlaybackRequest,
        source: GuidedPlaybackSource,
        voice: &str,
    ) -> Result<GuidedAudioDto, String> {
        self.cancel_active();
        let model = local_ai.piper_voice(voice);
        let model_identity = sha256::file(&model).unwrap_or_else(|_| "unavailable".into());
        let config_identity = sha256::file(&model.with_extension("onnx.json"))
            .unwrap_or_else(|_| "unavailable".into());
        let normalized_text = source.text.split_whitespace().collect::<Vec<_>>().join(" ");
        let cache_key = static_cache_key(
            &normalized_text,
            voice,
            &model_identity,
            &config_identity,
            STATIC_TTS_ENGINE_VERSION,
            BLUETOOTH_WAKE_MS,
            STATIC_TTS_CACHE_FORMAT_VERSION,
        );
        let cached = if source.asset_id.is_none() {
            self.inner
                .lock()
                .map_err(|_| "Guided audio runtime lock failed.")?
                .cache
                .get(&cache_key)
                .cloned()
        } else {
            None
        };
        let persistent_path = paths.static_tts_cache.join(format!("{cache_key}.wav"));
        let disk_cached = source.asset_id.is_none()
            && persistent_path.is_file()
            && fs::read(&persistent_path)
                .ok()
                .is_some_and(|bytes| wav_duration_ms(&bytes).is_ok());
        if persistent_path.is_file() && !disk_cached {
            let _ = fs::remove_file(&persistent_path);
        }
        let (audio_path, kind) = if let Some(path) = cached.filter(|path| path.is_file()) {
            (path, "piper_cache".to_owned())
        } else if disk_cached {
            self.inner
                .lock()
                .map_err(|_| "Guided audio runtime lock failed.")?
                .cache
                .insert(cache_key.clone(), persistent_path.clone());
            touch(&persistent_path);
            (persistent_path, "piper_cache".to_owned())
        } else if let Some(asset_id) = &source.asset_id {
            let path = paths
                .interactive_lesson_assets
                .join(&request.session_id)
                .join(format!("{asset_id}.wav"));
            if !path.is_file() {
                return Err("The declared Guided Lesson audio asset is unavailable; Piper fallback is intentionally disabled.".into());
            }
            (path, "bundled".to_owned())
        } else {
            let python = local_ai.piper_python();
            if !python.is_file() || !model.is_file() || !model.with_extension("onnx.json").is_file()
            {
                return Err("Piper is unavailable for this Guided Lesson reference.".into());
            }
            let final_path = persistent_path;
            let raw_path = paths
                .temporary_audio
                .join(format!("guided-lesson-{}-raw.wav", uuid::Uuid::new_v4()));
            let atomic_path = paths
                .static_tts_cache
                .join(format!(".{cache_key}-{}.tmp", uuid::Uuid::new_v4()));
            let text = source.text.clone();
            let cwd = local_ai.piper_root();
            let raw = raw_path.clone();
            let output = tokio::task::spawn_blocking(move || {
                Command::new(python)
                    .args(["-m", "piper", "-m"])
                    .arg(model)
                    .args(["-f"])
                    .arg(&raw)
                    .arg("--")
                    .arg(text)
                    .current_dir(cwd)
                    .output()
            })
            .await
            .map_err(|error| format!("Piper task failed: {error}"))?
            .map_err(|error| format!("Could not start Piper: {error}"))?;
            if !output.status.success() {
                let _ = fs::remove_file(&raw_path);
                return Err(format!(
                    "Local Guided Lesson synthesis failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                        .chars()
                        .take(300)
                        .collect::<String>()
                ));
            }
            let raw_bytes = fs::read(&raw_path)
                .map_err(|error| format!("Piper did not create Guided Lesson audio: {error}"))?;
            let _ = fs::remove_file(&raw_path);
            let (woken, _) = prepend_start_silence(&raw_bytes, BLUETOOTH_WAKE_MS)?;
            fs::write(&atomic_path, woken)
                .map_err(|error| format!("Could not cache Guided Lesson audio: {error}"))?;
            fs::rename(&atomic_path, &final_path).map_err(|error| {
                let _ = fs::remove_file(&atomic_path);
                format!("Could not publish Guided Lesson audio cache: {error}")
            })?;
            prune_cache(&paths.static_tts_cache, STATIC_TTS_CACHE_MAX_BYTES);
            let mut state = self
                .inner
                .lock()
                .map_err(|_| "Guided audio runtime lock failed.")?;
            state.cache.insert(cache_key, final_path.clone());
            (final_path, "piper".to_owned())
        };
        let bytes = fs::read(&audio_path)
            .map_err(|error| format!("Could not read Guided Lesson audio: {error}"))?;
        let (playable, duration_ms) = if kind == "bundled" {
            prepend_start_silence(&bytes, BLUETOOTH_WAKE_MS)?
        } else {
            let duration = wav_duration_ms(&bytes)?;
            (bytes, duration)
        };
        let playback_id = uuid::Uuid::new_v4().to_string();
        let not_before = Instant::now() + Duration::from_millis(duration_ms);
        self.inner
            .lock()
            .map_err(|_| "Guided audio runtime lock failed.")?
            .active = Some(ActivePlayback {
            id: playback_id.clone(),
            request,
            not_before,
        });
        Ok(GuidedAudioDto {
            playback_id,
            audio_base64: STANDARD.encode(playable),
            mime_type: "audio/wav".into(),
            source: kind,
            duration_ms,
            runtime_version: GUIDED_LESSON_AUDIO_RUNTIME_VERSION,
        })
    }
    pub fn confirm_completed(
        &self,
        playback_id: &str,
        request: &GuidedPlaybackRequest,
    ) -> Result<(), String> {
        let mut state = self
            .inner
            .lock()
            .map_err(|_| "Guided audio runtime lock failed.")?;
        let active = state
            .active
            .as_ref()
            .ok_or("Guided playback is no longer active.")?;
        if active.id != playback_id
            || active.request.session_id != request.session_id
            || active.request.stage_id != request.stage_id
            || active.request.item_id != request.item_id
        {
            return Err("Stale Guided playback completion was ignored.".into());
        }
        if Instant::now() < active.not_before {
            return Err("Partial Guided playback does not count as completed.".into());
        }
        state.active = None;
        Ok(())
    }
    pub fn cancel(&self, playback_id: &str) -> bool {
        let Ok(mut state) = self.inner.lock() else {
            return false;
        };
        if state
            .active
            .as_ref()
            .is_some_and(|active| active.id == playback_id)
        {
            state.active = None;
            true
        } else {
            false
        }
    }
    pub fn cancel_active(&self) {
        if let Ok(mut state) = self.inner.lock() {
            state.active = None
        }
    }
    pub fn cleanup_session(&self, _session_id: &str) {
        self.cancel_active();
    }
    pub fn shutdown(&self) {
        self.cancel_active();
    }
    pub fn cache_status(&self, paths: &LocalPaths) -> StaticTtsCacheStatusDto {
        let entries = cache_entries(&paths.static_tts_cache);
        StaticTtsCacheStatusDto {
            format_version: STATIC_TTS_CACHE_FORMAT_VERSION,
            entry_count: entries.len() as u64,
            size_bytes: entries.iter().map(|(_, size, _)| *size).sum(),
            max_size_bytes: STATIC_TTS_CACHE_MAX_BYTES,
        }
    }
    pub fn clear_cache(&self, paths: &LocalPaths) -> Result<StaticTtsCacheStatusDto, String> {
        self.cancel_active();
        if let Ok(mut state) = self.inner.lock() {
            state.cache.clear();
        }
        for (path, _, _) in cache_entries(&paths.static_tts_cache) {
            fs::remove_file(&path)
                .map_err(|error| format!("Could not clear static TTS cache: {error}"))?;
        }
        Ok(self.cache_status(paths))
    }
}

fn static_cache_key(
    normalized_text: &str,
    voice: &str,
    model_identity: &str,
    config_identity: &str,
    engine_version: u32,
    wake_ms: u64,
    format_version: u32,
) -> String {
    sha256::bytes(
        format!(
            "{normalized_text}|{voice}|{model_identity}|{config_identity}|engine={engine_version}|wake={wake_ms}|format={format_version}"
        )
        .as_bytes(),
    )
}

fn touch(path: &std::path::Path) {
    if let Ok(bytes) = fs::read(path) {
        let _ = fs::write(path, bytes);
    }
}

fn cache_entries(
    directory: &std::path::Path,
) -> Vec<(std::path::PathBuf, u64, std::time::SystemTime)> {
    let mut result = fs::read_dir(directory)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            (metadata.is_file() && entry.path().extension().and_then(|v| v.to_str()) == Some("wav"))
                .then(|| {
                    (
                        entry.path(),
                        metadata.len(),
                        metadata
                            .modified()
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                    )
                })
        })
        .collect::<Vec<_>>();
    result.sort_by_key(|(_, _, modified)| *modified);
    result
}

fn prune_cache(directory: &std::path::Path, maximum: u64) {
    let entries = cache_entries(directory);
    let mut total = entries.iter().map(|(_, size, _)| *size).sum::<u64>();
    for (path, size, _) in entries {
        if total <= maximum {
            break;
        }
        if fs::remove_file(path).is_ok() {
            total = total.saturating_sub(size);
        }
    }
}

fn wav_duration_ms(bytes: &[u8]) -> Result<u64, String> {
    let (_, byte_rate, data_size) = wav_parts(bytes)?;
    Ok((data_size as u64 * 1000) / (byte_rate as u64))
}

fn prepend_start_silence(bytes: &[u8], milliseconds: u64) -> Result<(Vec<u8>, u64), String> {
    let (data_offset, byte_rate, data_size) = wav_parts(bytes)?;
    let data_start = data_offset + 8;
    let block_align = u16::from_le_bytes(
        bytes[32..34]
            .try_into()
            .map_err(|_| "Invalid WAV block alignment.")?,
    ) as usize;
    if block_align == 0 {
        return Err("Invalid WAV block alignment.".into());
    }
    let requested = (byte_rate as u64 * milliseconds / 1000) as usize;
    let silence = requested - (requested % block_align);
    let mut out = Vec::with_capacity(bytes.len() + silence);
    out.extend_from_slice(&bytes[..data_start]);
    out.extend(std::iter::repeat(0u8).take(silence));
    out.extend_from_slice(&bytes[data_start..]);
    let new_data = data_size
        .checked_add(silence)
        .ok_or("Guided WAV is too large.")?;
    let riff_size = (out.len() - 8) as u32;
    out[data_offset + 4..data_offset + 8].copy_from_slice(&(new_data as u32).to_le_bytes());
    out[4..8].copy_from_slice(&riff_size.to_le_bytes());
    Ok((out, (new_data as u64 * 1000) / (byte_rate as u64)))
}

fn wav_parts(bytes: &[u8]) -> Result<(usize, u32, usize), String> {
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("Guided audio is not a valid WAV file.".into());
    }
    let mut offset = 12usize;
    let mut byte_rate = None;
    let mut data = None;
    while offset + 8 <= bytes.len() {
        let size = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        if offset + 8 + size > bytes.len() {
            return Err("Guided WAV chunk is truncated.".into());
        }
        match &bytes[offset..offset + 4] {
            b"fmt " if size >= 16 => {
                byte_rate = Some(u32::from_le_bytes(
                    bytes[offset + 16..offset + 20].try_into().unwrap(),
                ))
            }
            b"data" => {
                data = Some((offset, size));
                break;
            }
            _ => {}
        }
        offset += 8 + size + (size % 2);
    }
    let (data_offset, data_size) = data.ok_or("Guided WAV has no data chunk.")?;
    let rate = byte_rate
        .filter(|value| *value > 0)
        .ok_or("Guided WAV has no valid byte rate.")?;
    Ok((data_offset, rate, data_size))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wav() -> Vec<u8> {
        let mut v = b"RIFF".to_vec();
        v.extend_from_slice(&40u32.to_le_bytes());
        v.extend_from_slice(b"WAVEfmt ");
        v.extend_from_slice(&16u32.to_le_bytes());
        v.extend_from_slice(&1u16.to_le_bytes());
        v.extend_from_slice(&1u16.to_le_bytes());
        v.extend_from_slice(&16000u32.to_le_bytes());
        v.extend_from_slice(&32000u32.to_le_bytes());
        v.extend_from_slice(&2u16.to_le_bytes());
        v.extend_from_slice(&16u16.to_le_bytes());
        v.extend_from_slice(b"data");
        v.extend_from_slice(&4u32.to_le_bytes());
        v.extend_from_slice(&[1, 2, 3, 4]);
        v
    }
    #[test]
    fn prepends_half_second_without_mutating_source() {
        let source = wav();
        let (out, duration) = prepend_start_silence(&source, 500).unwrap();
        assert_eq!(source.len(), 48);
        assert_eq!(out.len(), 16048);
        assert_eq!(duration, 500);
        assert_eq!(&out[out.len() - 4..], &[1, 2, 3, 4]);
    }
    #[test]
    fn rejects_stale_and_partial_completion_events() {
        let runtime = GuidedLessonAudioRuntime::default();
        let request = GuidedPlaybackRequest {
            session_id: "s".into(),
            stage_id: "stage".into(),
            item_id: "item".into(),
        };
        runtime.inner.lock().unwrap().active = Some(ActivePlayback {
            id: "active".into(),
            request: request.clone(),
            not_before: Instant::now() + Duration::from_secs(1),
        });
        assert!(runtime.confirm_completed("stale", &request).is_err());
        assert!(runtime
            .confirm_completed("active", &request)
            .unwrap_err()
            .contains("Partial"));
        runtime
            .inner
            .lock()
            .unwrap()
            .active
            .as_mut()
            .unwrap()
            .not_before = Instant::now() - Duration::from_millis(1);
        runtime.confirm_completed("active", &request).unwrap();
        assert!(runtime.inner.lock().unwrap().active.is_none());
    }

    #[test]
    fn static_cache_key_is_deterministic_and_tracks_every_identity_input() {
        let baseline = static_cache_key("hello world", "voice-a", "model-a", "config-a", 1, 500, 1);
        assert_eq!(
            baseline,
            static_cache_key("hello world", "voice-a", "model-a", "config-a", 1, 500, 1)
        );
        assert_ne!(
            baseline,
            static_cache_key("hello!", "voice-a", "model-a", "config-a", 1, 500, 1)
        );
        assert_ne!(
            baseline,
            static_cache_key("hello world", "voice-b", "model-a", "config-a", 1, 500, 1)
        );
        assert_ne!(
            baseline,
            static_cache_key("hello world", "voice-a", "model-b", "config-a", 1, 500, 1)
        );
        assert_ne!(
            baseline,
            static_cache_key("hello world", "voice-a", "model-a", "config-b", 1, 500, 1)
        );
        assert_ne!(
            baseline,
            static_cache_key("hello world", "voice-a", "model-a", "config-a", 2, 500, 1)
        );
        assert_ne!(
            baseline,
            static_cache_key("hello world", "voice-a", "model-a", "config-a", 1, 0, 1)
        );
        assert_ne!(
            baseline,
            static_cache_key("hello world", "voice-a", "model-a", "config-a", 1, 500, 2)
        );
    }

    #[test]
    fn invalid_wav_is_rejected_and_size_pruning_is_bounded() {
        assert!(wav_duration_ms(b"not a wav").is_err());
        let directory =
            std::env::temp_dir().join(format!("tts-cache-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("one.wav"), wav()).unwrap();
        std::fs::write(directory.join("two.wav"), wav()).unwrap();
        prune_cache(&directory, 60);
        assert!(
            cache_entries(&directory)
                .iter()
                .map(|(_, size, _)| size)
                .sum::<u64>()
                <= 60
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
}
