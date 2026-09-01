# Part 1 Content Report

Status: production content assembled and validator-approved; final user listening/visual acceptance remains a human gate.

- Forms: 3 (A, B, C)
- Items: 18, all unique and publication state `published`
- Per form: exactly 6 (2 easy, 2 medium, 2 hard)
- Statements: 72 original authored statements
- Feedback: one correct rationale plus three choice-specific distractor explanations per item
- Images: 18 original local PNGs, all present and SHA-256 verified
- External images/downloads: none
- ETS/copied content: none

Images were created with the built-in image generation tool in `photorealistic-natural` mode, one distinct asset call per scene. Prompts specified realistic horizontal business/public scenes, an unambiguous primary action or state, natural lighting, no logos, no readable essential text, and no overlays. Project copies are under `src-tauri/resources/toeic/item-bank-v1/images`; generation originals remain in the Codex generated-images directory.

The Rust validator loaded all content and verified 18 assets/3 forms. Visual inspection found each generated scene suitable for its authored answer set. The content does not claim to reproduce ETS questions or official exam calibration.
