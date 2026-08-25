# Local AI

Ollama is contacted only at `http://127.0.0.1:11434`; proxy environment variables are ignored. Conversation requests disable thinking, cap output at 180 tokens, use a ten-minute keep-alive, and use a dedicated spoken-language prompt. The recommended model is `qwen3.5:4b`; `llama3.2:3b` is the configurable fallback.

Whisper runs as `whisper-cli` against `ggml-base.en.bin`. If `ggml-silero-v6.2.0.bin` exists, Whisper's supported `--vad` path is enabled. Piper 1.4.x receives sanitized text and writes a short WAV response.

Downloads only happen through the explicit `-DownloadModels` setup switch. There is no remote inference fallback and no API-key configuration.
