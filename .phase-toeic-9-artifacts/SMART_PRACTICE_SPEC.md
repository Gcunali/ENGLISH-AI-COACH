# Smart Practice Specification

`Practice My Weak Areas` supports 10, 15 or 20 questions. Selection is deterministic from sufficiently supported Priority/Needs Practice skills, with a validated-bank fallback while evidence is insufficient.

Each parent stores focus, requested size and one or more child steps. Each step stores Part, form/version, child session, quota and frozen item IDs. At the quota boundary the UI returns to the parent plan, which finalizes the step and preserves resume state.

Completion reports raw `correct/requested` and accuracy only. It never produces a scaled estimate. Grouped Parts retain their existing all-questions-before-feedback semantics.
