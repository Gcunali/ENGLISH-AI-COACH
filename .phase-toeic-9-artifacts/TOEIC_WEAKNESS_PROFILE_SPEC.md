# TOEIC Weakness Profile Specification

The profile is deterministic and TOEIC-only. It reads the first scored attempt for each `itemId:itemVersion`, joins the frozen form metadata, groups by Part and authored skill, and gives the five most recent observations double weight.

Classification requires at least five observations:

- under 5: Insufficient Data
- 85–100%: Strong
- 70–84%: Stable
- 50–69%: Needs Practice
- 0–49%: Priority

Insufficient samples never become ranked priorities. No Qwen or generated content participates.
