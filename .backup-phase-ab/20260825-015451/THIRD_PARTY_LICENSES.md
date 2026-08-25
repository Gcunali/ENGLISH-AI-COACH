# Third-party components and models

Engineering inventory, not legal advice. Re-check exact artifacts before distribution.

| Component | Version/artifact | Source | License | Distribution note |
|---|---|---|---|---|
| Tauri | 2.11.x | https://github.com/tauri-apps/tauri | Apache-2.0 / MIT | Preserve applicable notices. |
| React | 19.2.x | https://github.com/facebook/react | MIT | Preserve notice. |
| Tailwind CSS | 4.3.x | https://github.com/tailwindlabs/tailwindcss | MIT | Build-time/UI dependency. |
| SQLite | bundled via rusqlite 0.38 | https://sqlite.org | Public domain | `rusqlite` is MIT. |
| Ollama engine | current local release | https://github.com/ollama/ollama | MIT for open-source engine | Application packaging may have separate terms; verify distributed artifact. |
| Qwen 3.5 4B | `qwen3.5:4b`, Q4_K_M | https://ollama.com/library/qwen3.5:4b | Apache-2.0 | Include Apache notice if redistributed. |
| whisper.cpp | 1.9.1 target | https://github.com/ggml-org/whisper.cpp | MIT | Preserve license. Review optional components separately. |
| Whisper `base.en` | `ggml-base.en.bin` | https://huggingface.co/ggerganov/whisper.cpp | MIT upstream | Verify exact weight metadata before redistribution. |
| Silero VAD | `ggml-silero-v6.2.0.bin` | https://github.com/snakers4/silero-vad | MIT | Verify converted artifact provenance. |
| Piper engine | 1.4.2 | https://github.com/OHF-Voice/piper1-gpl | GPL-3.0 | Bundling can impose GPL obligations; legal review is required. Currently installed separately. |
| Piper lessac voice | `en_US-lessac-medium` | https://huggingface.co/rhasspy/piper-voices | Repository marked MIT | Model card references Lessac Blizzard 2013 dataset terms; review both before redistribution. |
| Wav2Vec2 phoneme model | `facebook/wav2vec2-lv-60-espeak-cv-ft` revision `ae45363bf3413b374fecd9dc8bc1df0e24c3b7f4` | https://huggingface.co/facebook/wav2vec2-lv-60-espeak-cv-ft | Apache-2.0 | Phase O local acoustic model; exact file hashes are recorded in `pronunciation_model_manifest.json`. |
| PyTorch CPU | 2.7.1 | https://pytorch.org | BSD-style | Isolated in `local-ai/pronunciation/.venv`. |
| Transformers | 4.53.2 | https://github.com/huggingface/transformers | Apache-2.0 | Isolated Phase O runtime. |
| Piper TTS Python / eSpeak-NG | 1.7.0 / bundled | https://github.com/OHF-Voice/piper1-gpl | GPL-3.0 | Separate Phase O copy used for compatible target phonemization and technical calibration; existing voice venv remains untouched. |

Downloaded voices are never assumed to share Piper's engine license. Preserve each voice's model card and license metadata.
