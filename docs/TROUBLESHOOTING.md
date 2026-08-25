# Troubleshooting

If Rust reports `link.exe not found`, install Visual Studio 2022 Build Tools with Desktop development with C++ and restart the terminal. WebView2 is included with normal Windows 11 installations.

- Ollama: start it and check `ollama list`.
- LLM: run `ollama pull qwen3.5:4b` only when ready for a multi-gigabyte download.
- Whisper: verify `whisper-cli.exe` and `models/whisper/ggml-base.en.bin` in app data.
- VAD: verify `ggml-silero-v6.2.0.bin` next to the Whisper model.
- Piper: verify the executable, voice `.onnx`, and matching `.onnx.json`.
- Microphone: allow desktop microphone access in Windows Settings.

Run `scripts/setup-windows.ps1` without switches for diagnostics. The app never redirects failed audio or transcripts online.
