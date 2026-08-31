# SPEAKING RECALL V1

## Flow

The learner sees a familiar situation/instruction derived from a completed Guided Lesson and must record before the model expression can be revealed. Local Whisper then displays the learner transcript. After that, the learner may reveal and hear the persisted model expression, save the recall, or retry.

An exact normalized substring check may say that the model expression appeared in the transcript. Failure to match is explicitly described as not pass/fail. There is no semantic score, completion gate, CEFR change, or Qwen feedback in V1.

## Sources and privacy

Speaking Check instructions/targets and Guided Conversation scenario, goal, and target expressions are read from existing completed lesson snapshots. Audio is temporary; only the local transcript/self-assessment result may be persisted in the practice ledger. No new curriculum catalog is created.
