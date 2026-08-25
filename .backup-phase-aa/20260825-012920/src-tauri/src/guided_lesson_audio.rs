use crate::{
    interactive_lesson::*,
    paths::{LocalAiPaths, LocalPaths},
    sha256,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    process::Command,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const BLUETOOTH_WAKE_MS: u64 = 500;
const PIPER_VOICE: &str = "en_US-lessac-medium";

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

#[derive(Clone)]
pub struct GuidedLessonAudioRuntime {
    inner: Arc<Mutex<RuntimeState>>,
}

#[derive(Default)]
struct RuntimeState {
    active: Option<ActivePlayback>,
    cache: BTreeMap<String, std::path::PathBuf>,
    owned: BTreeSet<std::path::PathBuf>,
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
        self.cancel_active();
        let cache_key = sha256::bytes(
            format!(
                "{}|{}|{}|{}|{}|{}",
                source.package_hash,
                request.stage_id,
                request.item_id,
                source.text,
                PIPER_VOICE,
                GUIDED_LESSON_AUDIO_RUNTIME_VERSION
            )
            .as_bytes(),
        );
        let cached = {
            self.inner
                .lock()
                .map_err(|_| "Guided audio runtime lock failed.")?
                .cache
                .get(&cache_key)
                .cloned()
        };
        let (audio_path, kind) = if let Some(path) = cached.filter(|path| path.is_file()) {
            (path, "piper_cache".to_owned())
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
            let model = local_ai.piper_voice(PIPER_VOICE);
            if !python.is_file() || !model.is_file() || !model.with_extension("onnx.json").is_file()
            {
                return Err("Piper is unavailable for this Guided Lesson reference.".into());
            }
            let final_path = paths
                .temporary_audio
                .join(format!("guided-lesson-{cache_key}.wav"));
            let raw_path = paths
                .temporary_audio
                .join(format!("guided-lesson-{cache_key}-raw.wav"));
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
            fs::write(&final_path, woken)
                .map_err(|error| format!("Could not cache Guided Lesson audio: {error}"))?;
            let mut state = self
                .inner
                .lock()
                .map_err(|_| "Guided audio runtime lock failed.")?;
            state.cache.insert(cache_key, final_path.clone());
            state.owned.insert(final_path.clone());
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
        self.cleanup_owned()
    }
    pub fn shutdown(&self) {
        self.cancel_active();
        self.cleanup_owned()
    }
    fn cleanup_owned(&self) {
        if let Ok(mut state) = self.inner.lock() {
            for path in std::mem::take(&mut state.owned) {
                let _ = fs::remove_file(path);
            }
            state.cache.clear();
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
}
