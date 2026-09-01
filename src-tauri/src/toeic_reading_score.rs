use serde::Serialize;

pub const PROFILE_ID: &str = "toeic-reading-unofficial-banded";
pub const PROFILE_VERSION: u32 = 1;

const ANCHORS: &[(u32, u32)] = &[
    (0, 5),
    (1, 10),
    (10, 40),
    (20, 85),
    (30, 135),
    (40, 190),
    (50, 250),
    (60, 310),
    (70, 365),
    (80, 415),
    (90, 460),
    (95, 480),
    (100, 495),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingEstimate {
    pub raw_correct: u32,
    pub estimated_score: u32,
    pub range_low: u32,
    pub range_high: u32,
    pub profile_id: &'static str,
    pub profile_version: u32,
    pub label: &'static str,
}

pub fn estimate(raw_correct: u32) -> Result<ReadingEstimate, String> {
    if raw_correct > 100 {
        return Err("Reading raw score must be between 0 and 100.".into());
    }
    let central = interpolate(raw_correct);
    let uncertainty = if raw_correct < 20 {
        30
    } else if raw_correct < 80 {
        25
    } else {
        20
    };
    Ok(ReadingEstimate {
        raw_correct,
        estimated_score: central,
        range_low: round_five(central.saturating_sub(uncertainty)).max(5),
        range_high: round_five((central + uncertainty).min(495)),
        profile_id: PROFILE_ID,
        profile_version: PROFILE_VERSION,
        label: "Unofficial estimated TOEIC Reading score",
    })
}

fn interpolate(raw: u32) -> u32 {
    if raw == 100 {
        return 495;
    }
    let (left, right) = ANCHORS
        .windows(2)
        .find(|pair| raw >= pair[0].0 && raw <= pair[1].0)
        .map(|pair| (pair[0], pair[1]))
        .unwrap_or(((0, 5), (1, 10)));
    let span = right.0 - left.0;
    let value = left.1 + ((raw - left.0) * (right.1 - left.1) + span / 2) / span;
    round_five(value).clamp(5, 495)
}

fn round_five(value: u32) -> u32 {
    ((value + 2) / 5) * 5
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn anchors_are_stable_and_in_five_point_steps() {
        for (raw, expected) in [
            (0, 5),
            (1, 10),
            (10, 40),
            (20, 85),
            (50, 250),
            (75, 405),
            (90, 460),
            (100, 495),
        ] {
            let value = estimate(raw).unwrap();
            assert_eq!(value.estimated_score, expected);
            assert_eq!(value.estimated_score % 5, 0);
            assert!(value.range_low >= 5 && value.range_high <= 495);
        }
    }
    #[test]
    fn all_raw_scores_are_monotonic() {
        let values = (0..=100)
            .map(|raw| estimate(raw).unwrap().estimated_score)
            .collect::<Vec<_>>();
        assert!(values.windows(2).all(|pair| pair[0] <= pair[1]));
    }
    #[test]
    fn rejects_incomplete_or_impossible_raw_domain() {
        assert!(estimate(101).is_err());
    }
}


