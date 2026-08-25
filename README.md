# English AI Coach

Desktop conversation coach that keeps microphone audio, transcripts, AI inference, speech synthesis, and learning data on the user's computer. There are no API keys, paid APIs, cloud authentication, analytics, or automatic remote fallbacks.

## Current vertical slice

The implemented path is:

`Microphone → Web Audio VAD → 16 kHz PCM WAV → whisper.cpp → Ollama → Piper → speaker`

It includes automatic end-of-speech detection, 320 ms pre-roll, push-to-talk, barge-in, a typed conversation state machine, live transcript, technical latency metrics, component diagnostics, temporary-audio cleanup, and SQLite persistence. The UI opens without the AI stack and reports missing components.

## Stack

- Tauri 2 and Rust for a small native desktop boundary.
- React 19, strict TypeScript, Vite 8, and Tailwind CSS 4.
- Ollama on fixed loopback endpoint `127.0.0.1:11434`.
- whisper.cpp 1.9.x with English Whisper and optional Silero VAD.
- Piper 1.4.x with an explicitly installed voice.
- SQLite in WAL mode through bundled `rusqlite`.

No arbitrary shell permission is exposed to the webview. Rust launches only resolved Whisper and Piper executables and validates Ollama model names.

## System requirements

- Windows 11 x64 (initial target), 16 GB RAM recommended.
- Microsoft Edge WebView2 (included with Windows 11).
- Node.js 22 LTS or newer.
- Rust stable MSVC.
- Visual Studio 2022 Build Tools with **Desktop development with C++**.
- Around 6 GB free for recommended models and build artifacts.

CPU-only operation is supported. CUDA is not required.

## Development and build

```powershell
npm install
npm run typecheck
npm test
npm run lint
npm run tauri:dev
```

Production builds:

```powershell
npm run build
npm run tauri:build
```

## Local AI setup

Inspect the machine and create only the empty local data folders:

```powershell
.\scripts\setup-windows.ps1
```

Install compiler prerequisites (may request administrator approval):

```powershell
.\scripts\setup-windows.ps1 -InstallToolchain
```

Build/install local engines, then explicitly download models:

```powershell
.\scripts\setup-windows.ps1 -InstallLocalAi
.\scripts\setup-windows.ps1 -DownloadModels
```

`-DownloadModels` acknowledges about 4 GB of downloads. It installs `qwen3.5:4b` (about 3.4 GB), Whisper `base.en`, Silero VAD, and `en_US-lessac-medium`. Use `llama3.2:3b` manually if the primary model is unsuitable.

Runtime data lives under `%LOCALAPPDATA%\com.englishaicoach.desktop\` in `database`, `models`, `voices`, `logs`, `temporary_audio`, and `tools`. Lesson audio is not retained: temporary WAVs are removed after use and stale WAVs are cleared on startup.

## Privacy and offline mode

Ollama requests use hard-coded loopback and ignore system proxies. Whisper and Piper are local subprocesses. SQLite is a local file. Once engines and models are installed, the vertical slice has no network dependency. Model downloads are never automatic.

After setup, disconnect networking, refresh System Status, and complete at least two spoken turns. A failed local component produces `Local AI component unavailable` with repair guidance and never falls back to cloud inference.

## Tests

- `npm test`: speech sanitization, sentence streaming boundaries, and state transitions.
- `npm run typecheck`: strict frontend contracts.
- `npm run lint`: frontend static analysis.
- `cargo test` in `src-tauri`: Rust tests and native compile/link validation.

## Troubleshooting

- **`link.exe not found`**: install Visual Studio Build Tools with Desktop development with C++, then open a new terminal.
- **Ollama unavailable**: start Ollama and inspect `ollama list`.
- **Model missing**: run `ollama pull qwen3.5:4b`; downloads are intentionally visible.
- **Whisper/Piper missing**: run the setup script and refresh diagnostics.
- **Microphone denied**: enable microphone access for desktop apps in Windows Privacy settings.

Detailed documents: [architecture](docs/ARCHITECTURE.md), [local AI](docs/LOCAL_AI.md), [audio pipeline](docs/AUDIO_PIPELINE.md), [database](docs/DATABASE.md), [pedagogy](docs/PEDAGOGY.md), and [troubleshooting](docs/TROUBLESHOOTING.md).

## Licensing

The application source is MIT. Engines, models, and voices have independent terms. Piper's current engine is GPL-3.0, while every voice must be checked separately. Review [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) before distributing binaries or model files.
