use serde::Serialize;
use std::collections::BTreeSet;

pub const GAMIFICATION_SCHEMA_VERSION: u32 = 1;
pub const GAMIFICATION_XP_RULE_VERSION: u32 = 1;
pub const GUIDED_XP_RULE_VERSION: u32 = 1;
pub const GUIDED_SESSION_XP: u32 = 60;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LocalDate {
    days: i64,
}

impl LocalDate {
    pub fn parse(value: &str) -> Result<Self, String> {
        let parts = value.split('-').collect::<Vec<_>>();
        if parts.len() != 3 {
            return Err(format!("Invalid local date: {value}"));
        }
        let year = parts[0]
            .parse::<i64>()
            .map_err(|_| format!("Invalid local date: {value}"))?;
        let month = parts[1]
            .parse::<u32>()
            .map_err(|_| format!("Invalid local date: {value}"))?;
        let day = parts[2]
            .parse::<u32>()
            .map_err(|_| format!("Invalid local date: {value}"))?;
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
            return Err(format!("Invalid local date: {value}"));
        }
        Ok(Self {
            days: days_from_civil(year, month, day),
        })
    }
    pub fn add_days(self, days: i64) -> Self {
        Self {
            days: self.days + days,
        }
    }
    pub fn days_since(self, other: Self) -> i64 {
        self.days - other.days
    }
    pub fn iso_week_start(self) -> Self {
        let weekday_from_monday = (self.days + 3).rem_euclid(7);
        self.add_days(-weekday_from_monday)
    }
    #[cfg(test)]
    pub fn as_string(self) -> String {
        let (year, month, day) = civil_from_days(self.days);
        format!("{year:04}-{month:02}-{day:02}")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelProgress {
    pub practice_level: u64,
    pub current_level_threshold: u64,
    pub next_level_threshold: u64,
    pub xp_into_current_level: u64,
    pub xp_needed_for_next_level: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyGoalProgress {
    pub goal_minutes: u32,
    pub practiced_minutes: u64,
    pub progress_percent: u32,
    pub reached: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AchievementCriterion {
    Lessons(u64),
    PracticeMinutes(u64),
    LongestStreak(u64),
    PlacementCompleted,
    GuidedLessons(u64),
    FirstGuidedUnitComplete,
    GuidedLevelComplete(&'static str),
    GuidedCourseComplete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AchievementDefinition {
    pub id: &'static str,
    pub version: u32,
    pub title: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub criterion: AchievementCriterion,
    pub hidden: bool,
}

pub const ACHIEVEMENTS: &[AchievementDefinition] = &[
    AchievementDefinition {
        id: "first_conversation",
        version: 1,
        title: "First Conversation",
        description: "Complete your first real conversation lesson.",
        category: "practice",
        criterion: AchievementCriterion::Lessons(1),
        hidden: false,
    },
    AchievementDefinition {
        id: "five_lessons",
        version: 1,
        title: "Five Lessons",
        description: "Complete 5 real conversation lessons.",
        category: "milestone",
        criterion: AchievementCriterion::Lessons(5),
        hidden: false,
    },
    AchievementDefinition {
        id: "ten_lessons",
        version: 1,
        title: "Ten Lessons",
        description: "Complete 10 real conversation lessons.",
        category: "milestone",
        criterion: AchievementCriterion::Lessons(10),
        hidden: false,
    },
    AchievementDefinition {
        id: "twenty_five_lessons",
        version: 1,
        title: "Twenty-Five Lessons",
        description: "Complete 25 real conversation lessons.",
        category: "milestone",
        criterion: AchievementCriterion::Lessons(25),
        hidden: false,
    },
    AchievementDefinition {
        id: "one_hour_practice",
        version: 1,
        title: "One Hour of Practice",
        description: "Complete 60 minutes of real conversation practice.",
        category: "practice",
        criterion: AchievementCriterion::PracticeMinutes(60),
        hidden: false,
    },
    AchievementDefinition {
        id: "five_hours_practice",
        version: 1,
        title: "Five Hours of Practice",
        description: "Complete 300 minutes of real conversation practice.",
        category: "practice",
        criterion: AchievementCriterion::PracticeMinutes(300),
        hidden: false,
    },
    AchievementDefinition {
        id: "ten_hours_practice",
        version: 1,
        title: "Ten Hours of Practice",
        description: "Complete 600 minutes of real conversation practice.",
        category: "practice",
        criterion: AchievementCriterion::PracticeMinutes(600),
        hidden: false,
    },
    AchievementDefinition {
        id: "three_day_streak",
        version: 1,
        title: "Three-Day Streak",
        description: "Practice on 3 consecutive days.",
        category: "consistency",
        criterion: AchievementCriterion::LongestStreak(3),
        hidden: false,
    },
    AchievementDefinition {
        id: "seven_day_streak",
        version: 1,
        title: "Seven-Day Streak",
        description: "Practice on 7 consecutive days.",
        category: "consistency",
        criterion: AchievementCriterion::LongestStreak(7),
        hidden: false,
    },
    AchievementDefinition {
        id: "fourteen_day_streak",
        version: 1,
        title: "Fourteen-Day Streak",
        description: "Practice on 14 consecutive days.",
        category: "consistency",
        criterion: AchievementCriterion::LongestStreak(14),
        hidden: false,
    },
    AchievementDefinition {
        id: "placement_complete",
        version: 1,
        title: "Placement Complete",
        description: "Complete a Placement Test assessment.",
        category: "assessment",
        criterion: AchievementCriterion::PlacementCompleted,
        hidden: false,
    },
    AchievementDefinition {
        id: "first_guided_lesson",
        version: 1,
        title: "First Guided Lesson",
        description: "Complete your first Guided Lesson.",
        category: "course",
        criterion: AchievementCriterion::GuidedLessons(1),
        hidden: false,
    },
    AchievementDefinition {
        id: "ten_guided_lessons",
        version: 1,
        title: "Ten Guided Lessons",
        description: "Complete 10 Guided Lesson sessions.",
        category: "course",
        criterion: AchievementCriterion::GuidedLessons(10),
        hidden: false,
    },
    AchievementDefinition {
        id: "fifty_guided_lessons",
        version: 1,
        title: "Fifty Guided Lessons",
        description: "Complete 50 Guided Lesson sessions.",
        category: "course",
        criterion: AchievementCriterion::GuidedLessons(50),
        hidden: false,
    },
    AchievementDefinition {
        id: "first_unit_complete",
        version: 1,
        title: "First Unit Complete",
        description: "Complete every Lesson in one Course unit.",
        category: "course",
        criterion: AchievementCriterion::FirstGuidedUnitComplete,
        hidden: false,
    },
    AchievementDefinition {
        id: "a1_complete",
        version: 1,
        title: "A1 Course Level Complete",
        description: "Complete all Guided Lessons in the A1 Course level.",
        category: "course",
        criterion: AchievementCriterion::GuidedLevelComplete("a1"),
        hidden: false,
    },
    AchievementDefinition {
        id: "a2_complete",
        version: 1,
        title: "A2 Course Level Complete",
        description: "Complete all Guided Lessons in the A2 Course level.",
        category: "course",
        criterion: AchievementCriterion::GuidedLevelComplete("a2"),
        hidden: false,
    },
    AchievementDefinition {
        id: "b1_complete",
        version: 1,
        title: "B1 Course Level Complete",
        description: "Complete all Guided Lessons in the B1 Course level.",
        category: "course",
        criterion: AchievementCriterion::GuidedLevelComplete("b1"),
        hidden: false,
    },
    AchievementDefinition {
        id: "b2_complete",
        version: 1,
        title: "B2 Course Level Complete",
        description: "Complete all Guided Lessons in the B2 Course level.",
        category: "course",
        criterion: AchievementCriterion::GuidedLevelComplete("b2"),
        hidden: false,
    },
    AchievementDefinition {
        id: "c1_complete",
        version: 1,
        title: "C1 Course Level Complete",
        description: "Complete all Guided Lessons in the C1 Course level.",
        category: "course",
        criterion: AchievementCriterion::GuidedLevelComplete("c1"),
        hidden: false,
    },
    AchievementDefinition {
        id: "c2_complete",
        version: 1,
        title: "C2 Course Level Complete",
        description: "Complete all Guided Lessons in the C2 Course level.",
        category: "course",
        criterion: AchievementCriterion::GuidedLevelComplete("c2"),
        hidden: false,
    },
    AchievementDefinition {
        id: "english_course_complete",
        version: 1,
        title: "English Course Complete",
        description: "Complete all 288 Guided Lesson IDs in the English Course.",
        category: "course",
        criterion: AchievementCriterion::GuidedCourseComplete,
        hidden: false,
    },
];

pub fn is_qualifying_lesson(valid_student_turns: u32, duration_seconds: i64) -> bool {
    valid_student_turns >= 3 && duration_seconds >= 120
}

pub fn calculate_lesson_xp(valid_student_turns: u32, duration_seconds: i64) -> u32 {
    if !is_qualifying_lesson(valid_student_turns, duration_seconds) {
        return 0;
    }
    40 + (valid_student_turns.saturating_mul(2)).min(20) + ((duration_seconds / 60) as u32).min(30)
}

pub fn xp_threshold_for_level(level: u64) -> u64 {
    if level <= 1 {
        return 0;
    }
    100_u64.saturating_mul(level.saturating_sub(1).saturating_mul(level) / 2)
}

pub fn practice_level_from_xp(xp: u64) -> u64 {
    let mut low = 1_u64;
    let mut high = 2_u64;
    while xp_threshold_for_level(high) <= xp && high < 1_000_000_000 {
        high = high.saturating_mul(2);
    }
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        if xp_threshold_for_level(middle) <= xp {
            low = middle;
        } else {
            high = middle;
        }
    }
    low
}

pub fn level_progress_from_xp(xp: u64) -> LevelProgress {
    let practice_level = practice_level_from_xp(xp);
    let current = xp_threshold_for_level(practice_level);
    let next = xp_threshold_for_level(practice_level + 1);
    LevelProgress {
        practice_level,
        current_level_threshold: current,
        next_level_threshold: next,
        xp_into_current_level: xp - current,
        xp_needed_for_next_level: next - xp,
    }
}

pub fn unique_sorted_days(days: &[LocalDate]) -> Vec<LocalDate> {
    days.iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn current_streak(days: &[LocalDate], reference_date: LocalDate) -> u64 {
    let unique = unique_sorted_days(days);
    let end = if unique.binary_search(&reference_date).is_ok() {
        reference_date
    } else if unique.binary_search(&reference_date.add_days(-1)).is_ok() {
        reference_date.add_days(-1)
    } else {
        return 0;
    };
    let set = unique.into_iter().collect::<BTreeSet<_>>();
    let mut streak = 0;
    let mut cursor = end;
    while set.contains(&cursor) {
        streak += 1;
        cursor = cursor.add_days(-1);
    }
    streak
}

pub fn longest_streak(days: &[LocalDate]) -> u64 {
    let unique = unique_sorted_days(days);
    let mut longest = 0_u64;
    let mut running = 0_u64;
    let mut previous = None;
    for day in unique {
        running = if previous.is_some_and(|value: LocalDate| day.days_since(value) == 1) {
            running + 1
        } else {
            1
        };
        longest = longest.max(running);
        previous = Some(day);
    }
    longest
}

pub fn weekly_goal_progress(goal_minutes: u32, practiced_minutes: u64) -> WeeklyGoalProgress {
    let percent =
        ((practiced_minutes.saturating_mul(100)) / u64::from(goal_minutes)).min(100) as u32;
    WeeklyGoalProgress {
        goal_minutes,
        practiced_minutes,
        progress_percent: percent,
        reached: practiced_minutes >= u64::from(goal_minutes),
    }
}

pub fn validate_weekly_goal(minutes: u32) -> Result<(), String> {
    if !(30..=600).contains(&minutes) || minutes % 15 != 0 {
        return Err("Weekly goal must be 30–600 minutes in 15-minute increments.".to_owned());
    }
    Ok(())
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = year - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = i64::from(month) + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}
#[cfg(test)]
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    y += if m <= 2 { 1 } else { 0 };
    (y, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn dates(values: &[&str]) -> Vec<LocalDate> {
        values
            .iter()
            .map(|v| LocalDate::parse(v).unwrap())
            .collect()
    }
    #[test]
    fn xp_qualifier_and_caps() {
        assert_eq!(calculate_lesson_xp(3, 120), 48);
        assert_eq!(calculate_lesson_xp(30, 3600), 90);
        assert_eq!(calculate_lesson_xp(2, 120), 0);
        assert_eq!(calculate_lesson_xp(3, 119), 0);
    }
    #[test]
    fn level_thresholds_are_exact_and_large_values_do_not_overflow() {
        for (xp, level) in [(0, 1), (99, 1), (100, 2), (299, 2), (300, 3), (600, 4)] {
            assert_eq!(practice_level_from_xp(xp), level);
        }
        assert!(practice_level_from_xp(u64::MAX) > 1);
    }
    #[test]
    fn streak_rules_cover_today_yesterday_gap_longest_and_duplicate_days() {
        let reference = LocalDate::parse("2026-08-21").unwrap();
        assert_eq!(
            current_streak(
                &dates(&["2026-08-19", "2026-08-20", "2026-08-21"]),
                reference
            ),
            3
        );
        assert_eq!(
            current_streak(
                &dates(&["2026-08-18", "2026-08-19", "2026-08-20"]),
                reference
            ),
            3
        );
        assert_eq!(
            current_streak(&dates(&["2026-08-17", "2026-08-18"]), reference),
            0
        );
        assert_eq!(
            longest_streak(&dates(&[
                "2026-08-01",
                "2026-08-02",
                "2026-08-03",
                "2026-08-05",
                "2026-08-06",
                "2026-08-07",
                "2026-08-08",
                "2026-08-08"
            ])),
            4
        );
    }
    #[test]
    fn iso_week_and_goal_are_deterministic() {
        let day = LocalDate::parse("2026-08-21").unwrap();
        assert_eq!(day.iso_week_start().as_string(), "2026-08-17");
        assert_eq!(weekly_goal_progress(90, 45).progress_percent, 50);
        assert!(weekly_goal_progress(90, 120).reached);
        for value in [15, 615, 95] {
            assert!(validate_weekly_goal(value).is_err());
        }
        for value in [30, 600] {
            assert!(validate_weekly_goal(value).is_ok());
        }
    }
}
