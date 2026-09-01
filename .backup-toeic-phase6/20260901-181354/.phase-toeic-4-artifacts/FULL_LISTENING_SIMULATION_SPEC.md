# Full Listening Simulation Specification

Composition is fixed and versioned: Part 1 Form A (6), Part 2 Form A (25), Part 3 Form A (39), Part 4 Form A (30), total 100. A parent record durably maps the four child sessions and stores mode, current part, composition snapshot, timestamps, and final score-profile snapshot.

Simulation mode is default and suppresses per-question/set feedback while automatically advancing. Learning mode retains normal feedback. Parts unlock in order. Sessions are untimed and resumable. No estimate is exposed before every child is complete; completed simulations link to each part's review.
