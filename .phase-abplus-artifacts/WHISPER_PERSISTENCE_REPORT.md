# WHISPER PERSISTENCE REPORT

## Before AB+

The persistent conversation bridge launched `whisper-cli.exe` for every student turn. That process loaded `ggml-small.en-q5_1.bin`, used 12 threads, transcribed one WAV, and exited. Five controlled runs on `local-ai\mic-test.wav` were 3135, 2968, 2947, 2971, and 6249 ms (median 2971 ms; mean 3654 ms). All five returned the same transcript. Peak working set observed during the legacy probe was approximately 480.4 MB.

## After AB+

The existing local `whisper-server.exe` is managed lazily by `whisper_server_worker.py`. The voice bridge remains JSONL-based; its managed worker talks only to a random `127.0.0.1` port owned by the child server. The same model and 12-thread configuration are protected. There is no external listener, download, new package, Python Whisper, cloud service, or OpenAI API.

Lifecycle controls include request UUID, worker generation, a 45-second request timeout, stale-generation rejection, one bounded restart, legacy CLI fallback, explicit session shutdown, and a Windows Job Object with `KILL_ON_JOB_CLOSE`. After both physical probes, no `whisper-server` process remained.

## Measured result

One five-request persistent run loaded the model once in 503 ms. Inference times were 2506, 2484, 2400, 2443, and 2415 ms. Median of all five was 2443 ms; warm median after excluding the first request was 2429 ms; the maximum/p95 indicator for this small sample was 2506 ms. Compared with the 2971 ms legacy median, the warm median improved by 18.24%. Transcript equivalence was 5/5.

A separate memory probe reported 362.0 MB working set and 854.6 MB private commit after a real transcription. That probe was slower (1293 ms load plus 7193 ms inference) and is retained as an honest system-load outlier, not substituted into the controlled latency series.

The persistent path is scoped to a voice session and is shut down at session end; it is not loaded at app startup or kept forever while idle. Practice-only transcription retains the protected local CLI fallback path.
