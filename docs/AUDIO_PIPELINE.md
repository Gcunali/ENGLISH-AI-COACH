# Audio pipeline

An `AudioWorklet` receives mono microphone PCM without blocking the UI. The frontend keeps 320 ms of pre-roll, requires 250 ms of speech, closes an utterance after 800 ms of silence, and caps it at 30 seconds. These values are centralized in `useVoicePipeline.ts`.

Audio is downsampled to 16 kHz and encoded as PCM16 WAV in memory. Rust writes one randomized temporary file for Whisper and removes it immediately. Piper output follows the same lifecycle. Stale WAV files are cleared on startup.

When microphone energy crosses the threshold during teacher playback, playback stops immediately and the state returns to student speech. Push-to-talk bypasses automatic detection for noisy rooms.
