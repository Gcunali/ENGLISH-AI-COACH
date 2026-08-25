# Architecture

The webview owns presentation, microphone capture, the lightweight energy gate, and audio playback. The Rust boundary owns filesystem paths, process invocation, loopback HTTP, temporary file cleanup, and SQLite. The frontend receives five narrow Tauri commands and no shell capability.

The conversation store enforces `IDLE → PREPARING → LISTENING → STUDENT_SPEAKING → TRANSCRIBING → TEACHER_THINKING → TEACHER_SPEAKING`. Pause, ending, completion, and error are explicit states.

Runtime data is under Tauri's application-local-data path. Downloaded models and generated data are never committed. The app remains navigable when components are absent. The current milestone is the vertical slice; analyzer, full memory, progress, and setup UI are subsequent modules using the same boundaries.
## Guided Conversation v1

Guided Conversation reuses the existing Voice V2 bridge, Whisper, Qwen streaming runtime, sentence chunker, Piper queue, cancellation and process ownership. Rust/SQLite own the immutable lesson snapshot, bounded safe context, committed transcript and deterministic completion. Python receives serialized Guided configuration and initial committed history; it does not read package files or own lesson state. The capability registry exposes Guided Conversation v1 while Analysis remains unavailable.
