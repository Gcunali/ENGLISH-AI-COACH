# DICTATION V1

## Flow

The learner plays locally synthesized audio while the target remains hidden, types what was heard, submits, sees the expected phrase and a word-position diff, and may replay it. The source is a Listening segment from a completed Guided Lesson snapshot.

## Deterministic grading

Qwen and semantic similarity are not used. Normalization performs lowercase conversion, trimming, whitespace collapse, punctuation removal, common smart-apostrophe normalization, and full-width ASCII compatibility conversion. Apostrophes inside contractions are preserved. Expected and submitted word arrays are compared at the same positions.

The UI labels the result `Exact`, `Almost there` (at least 80% word-position match), or `Needs review`. This is practice feedback, not a language score. Extra, missing, or repeated words remain visible through the deterministic percentage/diff behavior.

## Audio and privacy

Audio uses the local Piper static cache. The typed response is stored only as a local practice item result. No microphone audio, cloud service, or external request is involved.
