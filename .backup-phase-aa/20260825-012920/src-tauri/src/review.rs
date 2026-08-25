use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

pub const REVIEW_SYSTEM_SCHEMA_VERSION: u32 = 1;
pub const REVIEW_QUEUE_VERSION: u32 = 1;
pub const REVIEW_ITEM_SNAPSHOT_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewMode {
    Mixed,
    Vocabulary,
    Mistakes,
}
impl ReviewMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mixed => "mixed",
            Self::Vocabulary => "vocabulary",
            Self::Mistakes => "mistakes",
        }
    }
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "mixed" => Ok(Self::Mixed),
            "vocabulary" => Ok(Self::Vocabulary),
            "mistakes" => Ok(Self::Mistakes),
            _ => Err(format!("Unsupported review mode: {value}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewItemType {
    Vocabulary,
    RecurringMistake,
}
impl ReviewItemType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vocabulary => "vocabulary",
            Self::RecurringMistake => "recurring_mistake",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewOutcome {
    KeepPracticing,
    MarkLearning,
    MarkKnown,
    ReviewAgain,
    Reviewed,
}
impl ReviewOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::KeepPracticing => "keep_practicing",
            Self::MarkLearning => "mark_learning",
            Self::MarkKnown => "mark_known",
            Self::ReviewAgain => "review_again",
            Self::Reviewed => "reviewed",
        }
    }
    pub fn valid_for(self, kind: ReviewItemType) -> bool {
        matches!(
            (kind, self),
            (
                ReviewItemType::Vocabulary,
                Self::KeepPracticing | Self::MarkLearning | Self::MarkKnown
            ) | (
                ReviewItemType::RecurringMistake,
                Self::ReviewAgain | Self::Reviewed
            )
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueCandidate {
    pub source_id: String,
    pub item_type: ReviewItemType,
    pub vocabulary_status: Option<String>,
    pub review_count: u32,
    pub last_reviewed_at: Option<String>,
    pub occurrence_count: u32,
    pub lesson_count: u32,
    pub last_seen_at: String,
}

pub fn validate_session_size(size: u32) -> Result<(), String> {
    if [5, 10, 15].contains(&size) {
        Ok(())
    } else {
        Err("Review item count must be 5, 10, or 15.".to_owned())
    }
}

pub fn rank_vocabulary(items: &mut [QueueCandidate]) {
    items.sort_by(compare_common_vocabulary);
}
pub fn rank_mistakes(items: &mut [QueueCandidate]) {
    items.sort_by(compare_common_mistake);
}

fn review_order(left: &QueueCandidate, right: &QueueCandidate) -> Ordering {
    (left.review_count > 0)
        .cmp(&(right.review_count > 0))
        .then_with(|| left.review_count.cmp(&right.review_count))
        .then_with(|| left.last_reviewed_at.cmp(&right.last_reviewed_at))
}
fn compare_common_vocabulary(left: &QueueCandidate, right: &QueueCandidate) -> Ordering {
    review_order(left, right)
        .then_with(|| right.occurrence_count.cmp(&left.occurrence_count))
        .then_with(|| right.lesson_count.cmp(&left.lesson_count))
        .then_with(|| right.last_seen_at.cmp(&left.last_seen_at))
        .then_with(|| left.source_id.cmp(&right.source_id))
}
fn compare_common_mistake(left: &QueueCandidate, right: &QueueCandidate) -> Ordering {
    review_order(left, right)
        .then_with(|| right.lesson_count.cmp(&left.lesson_count))
        .then_with(|| right.occurrence_count.cmp(&left.occurrence_count))
        .then_with(|| right.last_seen_at.cmp(&left.last_seen_at))
        .then_with(|| left.source_id.cmp(&right.source_id))
}

pub fn build_queue(
    mode: ReviewMode,
    size: u32,
    mut vocabulary: Vec<QueueCandidate>,
    mut mistakes: Vec<QueueCandidate>,
) -> Result<Vec<QueueCandidate>, String> {
    validate_session_size(size)?;
    vocabulary.retain(|item| {
        item.item_type == ReviewItemType::Vocabulary
            && matches!(item.vocabulary_status.as_deref(), Some("new" | "learning"))
    });
    mistakes.retain(|item| item.item_type == ReviewItemType::RecurringMistake);
    rank_mistakes(&mut mistakes);
    match mode {
        ReviewMode::Vocabulary => Ok(select_vocabulary(size as usize, vocabulary)),
        ReviewMode::Mistakes => Ok(mistakes.into_iter().take(size as usize).collect()),
        ReviewMode::Mixed => {
            let requested = size as usize;
            let target = if mistakes.is_empty() {
                0
            } else {
                ((requested as f64 * 0.30).floor() as usize).max(1)
            };
            let mistake_take = target.min(mistakes.len());
            let mut selected_mistakes = mistakes.drain(..mistake_take).collect::<Vec<_>>();
            let mut selected_vocabulary = select_vocabulary(requested - mistake_take, vocabulary);
            let remaining = requested - selected_mistakes.len() - selected_vocabulary.len();
            selected_mistakes.extend(mistakes.into_iter().take(remaining));
            interleave(selected_vocabulary.drain(..).collect(), selected_mistakes)
        }
    }
}

fn select_vocabulary(limit: usize, items: Vec<QueueCandidate>) -> Vec<QueueCandidate> {
    let mut learning = items
        .iter()
        .filter(|item| item.vocabulary_status.as_deref() == Some("learning"))
        .cloned()
        .collect::<Vec<_>>();
    let mut new = items
        .into_iter()
        .filter(|item| item.vocabulary_status.as_deref() == Some("new"))
        .collect::<Vec<_>>();
    rank_vocabulary(&mut learning);
    rank_vocabulary(&mut new);
    let learning_target = (limit * 6 + 9) / 10;
    let mut selected = learning
        .drain(..learning_target.min(learning.len()))
        .collect::<Vec<_>>();
    let new_target = limit - selected.len();
    selected.extend(new.drain(..new_target.min(new.len())));
    let remaining = limit - selected.len();
    selected.extend(learning.into_iter().take(remaining));
    let remaining = limit - selected.len();
    selected.extend(new.into_iter().take(remaining));
    selected
}

fn interleave(
    vocabulary: Vec<QueueCandidate>,
    mistakes: Vec<QueueCandidate>,
) -> Result<Vec<QueueCandidate>, String> {
    let total = vocabulary.len() + mistakes.len();
    let mut result = Vec::with_capacity(total);
    let mut vi = 0;
    let mut mi = 0;
    for position in 0..total {
        let should_mistake = mi < mistakes.len() && ((position + 1) * mistakes.len() / total) > mi;
        if should_mistake {
            result.push(mistakes[mi].clone());
            mi += 1
        } else if vi < vocabulary.len() {
            result.push(vocabulary[vi].clone());
            vi += 1
        } else {
            result.push(mistakes[mi].clone());
            mi += 1
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn v(id: &str, status: &str, reviews: u32) -> QueueCandidate {
        QueueCandidate {
            source_id: id.into(),
            item_type: ReviewItemType::Vocabulary,
            vocabulary_status: Some(status.into()),
            review_count: reviews,
            last_reviewed_at: if reviews > 0 {
                Some(format!("2026-08-{reviews:02}"))
            } else {
                None
            },
            occurrence_count: 1,
            lesson_count: 1,
            last_seen_at: "2026-08-20".into(),
        }
    }
    fn m(id: &str, reviews: u32) -> QueueCandidate {
        QueueCandidate {
            source_id: id.into(),
            item_type: ReviewItemType::RecurringMistake,
            vocabulary_status: None,
            review_count: reviews,
            last_reviewed_at: if reviews > 0 {
                Some(format!("2026-08-{reviews:02}"))
            } else {
                None
            },
            occurrence_count: 2,
            lesson_count: 2,
            last_seen_at: "2026-08-20".into(),
        }
    }
    #[test]
    fn sizes_are_controlled() {
        for size in [5, 10, 15] {
            assert!(validate_session_size(size).is_ok())
        }
        for size in [0, 4, 6, 20] {
            assert!(validate_session_size(size).is_err())
        }
    }
    #[test]
    fn known_is_excluded_and_never_reviewed_ranks_first() {
        let q = build_queue(
            ReviewMode::Vocabulary,
            5,
            vec![
                v("reviewed", "learning", 2),
                v("known", "known", 0),
                v("fresh", "learning", 0),
            ],
            vec![],
        )
        .unwrap();
        assert_eq!(
            q.iter().map(|x| x.source_id.as_str()).collect::<Vec<_>>(),
            vec!["fresh", "reviewed"]
        )
    }
    #[test]
    fn mixed_ratios_are_exact() {
        for (size, expected) in [(5, 1), (10, 3), (15, 4)] {
            let q = build_queue(
                ReviewMode::Mixed,
                size,
                (0..20)
                    .map(|i| {
                        v(
                            &format!("v{i}"),
                            if i % 2 == 0 { "learning" } else { "new" },
                            0,
                        )
                    })
                    .collect(),
                (0..10).map(|i| m(&format!("m{i}"), 0)).collect(),
            )
            .unwrap();
            assert_eq!(
                q.iter()
                    .filter(|x| x.item_type == ReviewItemType::RecurringMistake)
                    .count(),
                expected
            )
        }
    }
    #[test]
    fn fallbacks_fill_without_duplicates() {
        let q = build_queue(
            ReviewMode::Mixed,
            10,
            (0..4).map(|i| v(&format!("v{i}"), "learning", 0)).collect(),
            (0..8).map(|i| m(&format!("m{i}"), 0)).collect(),
        )
        .unwrap();
        assert_eq!(q.len(), 10);
        let ids = q
            .iter()
            .map(|x| &x.source_id)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), 10)
    }
    #[test]
    fn vocabulary_balance_is_sixty_forty() {
        let q = build_queue(
            ReviewMode::Vocabulary,
            10,
            (0..20)
                .map(|i| v(&format!("v{i}"), if i < 10 { "learning" } else { "new" }, 0))
                .collect(),
            vec![],
        )
        .unwrap();
        assert_eq!(
            q.iter()
                .filter(|x| x.vocabulary_status.as_deref() == Some("learning"))
                .count(),
            6
        )
    }
    #[test]
    fn queue_is_deterministic() {
        let values = vec![v("b", "new", 1), v("a", "new", 0), v("c", "learning", 0)];
        assert_eq!(
            build_queue(ReviewMode::Vocabulary, 5, values.clone(), vec![]).unwrap(),
            build_queue(ReviewMode::Vocabulary, 5, values, vec![]).unwrap()
        )
    }
    #[test]
    fn outcomes_are_type_safe() {
        assert!(ReviewOutcome::MarkKnown.valid_for(ReviewItemType::Vocabulary));
        assert!(!ReviewOutcome::MarkKnown.valid_for(ReviewItemType::RecurringMistake));
        assert!(ReviewOutcome::Reviewed.valid_for(ReviewItemType::RecurringMistake))
    }
}
