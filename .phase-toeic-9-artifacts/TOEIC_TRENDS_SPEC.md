# TOEIC Trends Specification

Trend points come only from `toeic_full_lr_session` rows with `status='completed'` and a completion timestamp. Each point preserves family, Listening/Reading/total raw values and the versioned unofficial estimates.

Partial, abandoned, standalone Part, Smart Practice and Daily Practice sessions never enter the Full L&R trend. The latest uncertainty range is read from the same completed parent.
