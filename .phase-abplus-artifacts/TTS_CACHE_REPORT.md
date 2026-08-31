# TTS CACHE REPORT

## Architecture

Static Guided Lesson and Practice reference speech is cache-first under the OS app-local data directory:

`%LOCALAPPDATA%\com.englishaicoach.desktop\cache\static_tts`

The production cache is not stored inside the source tree and no audio is stored in SQLite. Bundled lesson audio has precedence. On a static Piper miss, synthesis writes a unique temporary file, prepends the protected 500 ms Bluetooth wake silence, atomically renames the result to its final SHA-256 key, validates the WAV on future hits, and regenerates invalid entries.

The key contains whitespace-normalized exact text, voice `en_US-lessac-medium`, SHA-256 identities of the ONNX model and JSON config, static engine version, 500 ms wake parameter, and cache format version. Model hash: `5EFE09E69902187827AF646E1A6E9D269DEE769F9877D17B16B1B46EEAAF019F`. Config hash: `EFE19C417BED055F2D69908248C6BA650FA135BC868B0E6ABB3DA181DAB690A0`.

The cap is 250 MiB. Modified-time recency provides a simple LRU-equivalent pruning order. Settings reports entry count/bytes and can clear only this cache. History, lessons, vocabulary, progress, learning memory, and microphone audio are untouched.

Dynamic Qwen replies, guided personalized text, learner speech, personal corrections, and microphone audio do not use this persistent static cache.

## Physical benchmark

For one real Piper phrase, uncached synthesis took 6222.53 ms and produced an 81,452-byte WAV. Consecutive cached file reads took 19.1672, 0.1694, 0.0855, 0.0703, and 0.0796 ms. The warm read median was 0.0855 ms (99.9986% less than synthesis); even the first observed cached read was 99.69% less. These figures measure synthesis/file-read availability, not sound-device startup latency.

Automated tests cover deterministic key inputs, invalid WAV rejection, 500 ms silence without source mutation, and bounded size pruning.
