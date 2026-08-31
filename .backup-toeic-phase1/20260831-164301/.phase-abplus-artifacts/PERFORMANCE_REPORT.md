# PERFORMANCE REPORT

## Whisper

| Metric | Legacy CLI | Persistent worker |
| --- | ---: | ---: |
| Cold/model-load behavior | model loaded every request | 503 ms once in controlled run |
| First controlled end-to-end indicator | 3135 ms | 3009 ms (load + first inference) |
| Median comparable requests | 2971 ms | 2443 ms, all 5 |
| Warm median | not applicable; every run reloads | 2429 ms |
| p95 indicator (n=5) | 6249 ms maximum | 2506 ms maximum |
| Transcript consistency | 5/5 | 5/5 |
| Approximate memory | 480.4 MB peak working set observed | 362.0 MB working set; 854.6 MB private commit in separate probe |

Warm median latency improved 18.24% against the legacy median. A separate persistent probe under different system load produced a 7193 ms inference outlier; no number was hidden or replaced. No orphan worker remained after shutdown.

## Piper static cache

| Metric | Result |
| --- | ---: |
| Uncached synthesis | 6222.53 ms |
| First cached read | 19.1672 ms |
| Warm cached read median | 0.0855 ms |
| Cached WAV size | 81,452 bytes |
| Cache limit | 250 MiB |

The first cached read reduced measured availability time by 99.69%; the warm median reduced it by 99.9986%. Playback-device latency was not included.

## Combined runtime memory

The app deliberately does not load all heavy engines at startup. Voice keeps Qwen/Piper and lazy Whisper for the session; pronunciation loads Wav2Vec2 only for acoustic work; static Course browsing loads none. A trustworthy combined Qwen + Whisper + Piper + Wav2Vec2 peak requires a physical voice/pronunciation session and was not fabricated in automation. It remains in the human smoke gate.
