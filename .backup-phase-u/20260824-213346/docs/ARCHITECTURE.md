# Architecture

The webview owns presentation, microphone capture, the lightweight energy gate, and audio playback. The Rust boundary owns filesystem paths, process invocation, loopback HTTP, temporary file cleanup, and SQLite. The frontend receives five narrow Tauri commands and no shell capability.

The conversation store enforces `IDLE → PREPARING → LISTENING → STUDENT_SPEAKING → TRANSCRIBING → TEACHER_THINKING → TEACHER_SPEAKING`. Pause, ending, completion, and error are explicit states.

Runtime data is under Tauri's application-local-data path. Downloaded models and generated data are never committed. The app remains navigable when components are absent. The current milestone is the vertical slice; analyzer, full memory, progress, and setup UI are subsequent modules using the same boundaries.
