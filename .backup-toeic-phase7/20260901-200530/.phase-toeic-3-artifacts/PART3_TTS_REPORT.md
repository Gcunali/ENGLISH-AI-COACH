# Part 3 TTS Report

Runtime uses the existing local Piper static cache and synthesizes each turn with its declared voice, preserving short natural transition gaps between speakers.

Installed/tested voices: `en_US-amy-medium` and `en_US-lessac-medium`. Representative real synthesis passed:

- Amy WAV: 205,868 bytes; SHA-256 `0263F5E33E90A13B0A7D7C349C7C1412E9876D44E5B3E70B2124C053ADA5A3B5`.
- Lessac WAV: 139,820 bytes; SHA-256 `C37C0C3E3546DACA4968B1A47AB0241366643FEAB6F814944DA724E062B4AF2F`.

The QA WAV files remain under `src-tauri/target/toeic-p3-tts-qa` and are not bundled. Automated synthesis establishes operational compatibility, not human naturalness. A complete listening review remains pending.
