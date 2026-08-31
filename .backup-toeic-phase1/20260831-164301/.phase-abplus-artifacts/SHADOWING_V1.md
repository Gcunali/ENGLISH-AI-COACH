# SHADOWING V1

## Flow

V1 deliberately avoids simultaneous playback and recording:

1. listen to a real Repeat target from a completed Guided Lesson;
2. wait for playback completion;
3. record the target;
4. run the existing local pronunciation engine;
5. display phrase acoustic match and reliable word-level focus;
6. save or retry.

The Record control stays disabled until reference playback completes. Reference audio is Piper/cache-first. Captured audio remains temporary.

## Feedback safety

The mode reuses Pronunciation Score v1 without changing its algorithm. Word-specific claims use Word Pronunciation Feedback v1 and disappear when alignment confidence/coverage is inadequate. The UI does not claim to measure native accent, prosody, rhythm, or intonation.

## Sources

Repeat targets are consumed from completed Guided Lesson snapshots; lesson packages are not rewritten and no second content catalog is created.
