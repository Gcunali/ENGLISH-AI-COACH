use crate::placement::{CefrBand, PlacementConfidence, PlacementSkill};

#[derive(Clone, Debug)]
pub struct ScoredAnswer {
    pub skill: PlacementSkill,
    pub band: CefrBand,
    pub correct: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomainState {
    Question(CefrBand, usize),
    Complete(CefrBand, bool),
}

pub fn domain_state(skill: PlacementSkill, answers: &[ScoredAnswer]) -> DomainState {
    let block = |band| {
        answers
            .iter()
            .filter(|a| a.skill == skill && a.band == band)
            .collect::<Vec<_>>()
    };
    let outcome = |band| {
        let values = block(band);
        (values.len() == 3).then(|| values.iter().filter(|a| a.correct).count() >= 2)
    };
    let pending = |band| DomainState::Question(band, block(band).len());
    match outcome(CefrBand::B1) {
        None => pending(CefrBand::B1),
        Some(true) => {
            for (band, previous) in [
                (CefrBand::B2, CefrBand::B1),
                (CefrBand::C1, CefrBand::B2),
                (CefrBand::C2, CefrBand::C1),
            ] {
                match outcome(band) {
                    None => return pending(band),
                    Some(false) => return DomainState::Complete(previous, true),
                    Some(true) => {}
                }
            }
            DomainState::Complete(CefrBand::C2, true)
        }
        Some(false) => match outcome(CefrBand::A2) {
            None => pending(CefrBand::A2),
            Some(true) => DomainState::Complete(CefrBand::A2, true),
            Some(false) => match outcome(CefrBand::A1) {
                None => pending(CefrBand::A1),
                Some(_) => DomainState::Complete(CefrBand::A1, false),
            },
        },
    }
}

pub fn lower_median(levels: &[CefrBand]) -> Option<CefrBand> {
    if levels.is_empty() {
        return None;
    }
    let mut values = levels.to_vec();
    values.sort();
    Some(values[(values.len() - 1) / 2])
}

pub fn overall_confidence(
    levels: &[CefrBand],
    speaking_available: bool,
    weak_boundary: bool,
) -> PlacementConfidence {
    if levels.is_empty() {
        return PlacementConfidence::Low;
    }
    let spread = levels.iter().map(|v| v.ordinal()).max().unwrap()
        - levels.iter().map(|v| v.ordinal()).min().unwrap();
    if weak_boundary || !speaking_available || spread >= 3 {
        PlacementConfidence::Low
    } else if levels.len() == 4 && spread <= 1 {
        PlacementConfidence::High
    } else {
        PlacementConfidence::Medium
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn block(skill: PlacementSkill, band: CefrBand, correct: usize) -> Vec<ScoredAnswer> {
        (0..3)
            .map(|i| ScoredAnswer {
                skill,
                band,
                correct: i < correct,
            })
            .collect()
    }
    #[test]
    fn adaptive_passes_up_to_b2_boundary() {
        let mut a = block(PlacementSkill::Grammar, CefrBand::B1, 2);
        a.extend(block(PlacementSkill::Grammar, CefrBand::B2, 3));
        a.extend(block(PlacementSkill::Grammar, CefrBand::C1, 1));
        assert_eq!(
            domain_state(PlacementSkill::Grammar, &a),
            DomainState::Complete(CefrBand::B2, true)
        );
    }
    #[test]
    fn adaptive_fails_down_to_a2() {
        let mut a = block(PlacementSkill::Grammar, CefrBand::B1, 1);
        a.extend(block(PlacementSkill::Grammar, CefrBand::A2, 2));
        assert_eq!(
            domain_state(PlacementSkill::Grammar, &a),
            DomainState::Complete(CefrBand::A2, true)
        );
    }
    #[test]
    fn low_and_high_boundaries() {
        let mut low = block(PlacementSkill::Grammar, CefrBand::B1, 0);
        low.extend(block(PlacementSkill::Grammar, CefrBand::A2, 0));
        low.extend(block(PlacementSkill::Grammar, CefrBand::A1, 0));
        assert_eq!(
            domain_state(PlacementSkill::Grammar, &low),
            DomainState::Complete(CefrBand::A1, false)
        );
        let mut high = block(PlacementSkill::Grammar, CefrBand::B1, 3);
        for b in [CefrBand::B2, CefrBand::C1, CefrBand::C2] {
            high.extend(block(PlacementSkill::Grammar, b, 3));
        }
        assert_eq!(
            domain_state(PlacementSkill::Grammar, &high),
            DomainState::Complete(CefrBand::C2, true)
        );
    }
    #[test]
    fn skills_are_independent() {
        let a = block(PlacementSkill::Grammar, CefrBand::B1, 3);
        assert_eq!(
            domain_state(PlacementSkill::Vocabulary, &a),
            DomainState::Question(CefrBand::B1, 0)
        );
    }
    #[test]
    fn lower_median_is_conservative() {
        assert_eq!(
            lower_median(&[CefrBand::A2, CefrBand::B1, CefrBand::B1, CefrBand::B2]),
            Some(CefrBand::B1)
        );
        assert_eq!(
            lower_median(&[CefrBand::B1, CefrBand::B2, CefrBand::B2, CefrBand::C1]),
            Some(CefrBand::B2)
        );
        assert_eq!(
            lower_median(&[CefrBand::A1, CefrBand::A2, CefrBand::B2, CefrBand::C1]),
            Some(CefrBand::A2)
        );
        assert_eq!(
            lower_median(&[CefrBand::A2, CefrBand::B1, CefrBand::B2]),
            Some(CefrBand::B1)
        );
    }
    #[test]
    fn confidence_is_deterministic() {
        assert_eq!(
            overall_confidence(
                &[CefrBand::B1, CefrBand::B1, CefrBand::B2, CefrBand::B2],
                true,
                false
            ),
            PlacementConfidence::High
        );
        assert_eq!(
            overall_confidence(
                &[CefrBand::A2, CefrBand::B1, CefrBand::B2, CefrBand::B2],
                true,
                false
            ),
            PlacementConfidence::Medium
        );
        assert_eq!(
            overall_confidence(&[CefrBand::B1, CefrBand::B1, CefrBand::B2], false, false),
            PlacementConfidence::Low
        );
    }
}
