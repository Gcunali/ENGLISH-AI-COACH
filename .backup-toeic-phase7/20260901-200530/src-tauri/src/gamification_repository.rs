use crate::{
    database,
    gamification::{
        calculate_lesson_xp, current_streak, level_progress_from_xp, longest_streak,
        validate_weekly_goal, weekly_goal_progress, AchievementCriterion, LocalDate, ACHIEVEMENTS,
        GAMIFICATION_SCHEMA_VERSION, GAMIFICATION_XP_RULE_VERSION, GUIDED_SESSION_XP,
        GUIDED_XP_RULE_VERSION,
    },
    lesson_repository::is_technical_transcript,
};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::Serialize;
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone)]
pub struct GamificationRepository {
    database: PathBuf,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamificationOverviewDto {
    pub schema_version: u32,
    pub total_xp: u64,
    pub practice_level: u64,
    pub current_level_threshold: u64,
    pub next_level_threshold: u64,
    pub xp_into_current_level: u64,
    pub xp_needed_for_next_level: u64,
    pub qualifying_lesson_count: u64,
    pub total_practice_minutes: u64,
    pub current_streak_days: u64,
    pub longest_streak_days: u64,
    pub weekly_goal: crate::gamification::WeeklyGoalProgress,
    pub unlocked_achievement_count: u64,
    pub total_achievement_count: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementDto {
    pub id: String,
    pub version: u32,
    pub title: String,
    pub description: String,
    pub category: String,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
    pub progress_current: u64,
    pub progress_target: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamificationProfileDto {
    pub schema_version: u32,
    pub weekly_goal_minutes: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamificationSyncResult {
    pub inspected_lessons: u64,
    pub qualifying_lessons: u64,
    pub ignored_lessons: u64,
    pub events_created: u64,
    pub achievements_unlocked: Vec<AchievementDto>,
    pub inspected_guided_sessions: u64,
    pub guided_events_created: u64,
}

#[derive(Clone)]
struct Event {
    ended_at: String,
    activity_day: LocalDate,
    duration_seconds: u64,
    source_type: &'static str,
}

#[derive(Default)]
struct GuidedProgressFacts {
    completed_sessions: Vec<String>,
    unique_lessons: BTreeMap<String, String>,
}

impl GamificationRepository {
    pub fn new(database: PathBuf) -> Self {
        Self { database }
    }

    pub fn sync(&self) -> Result<GamificationSyncResult, String> {
        let mut connection = database::open(&self.database)?;
        let transaction = connection.transaction().map_err(db_error)?;
        let lessons = completed_lessons(&transaction)?;
        let mut qualifying = 0_u64;
        let mut events_created = 0_u64;
        for (id, duration, ended_at) in &lessons {
            let turns = valid_student_turns(&transaction, id)?;
            let xp = calculate_lesson_xp(turns, *duration);
            if xp == 0 {
                continue;
            }
            qualifying += 1;
            let activity_day: String = transaction
                .query_row("SELECT date(?1, 'localtime')", [ended_at], |row| row.get(0))
                .map_err(db_error)?;
            events_created += transaction.execute(
                "INSERT OR IGNORE INTO gamification_xp_event
                 (id,event_type,source_type,source_id,rule_version,xp_amount,activity_day,created_at)
                 VALUES (?1,'qualifying_lesson_completed','lesson',?2,?3,?4,?5,?6)",
                params![uuid::Uuid::new_v4().to_string(), id, GAMIFICATION_XP_RULE_VERSION, xp, activity_day, ended_at],
            ).map_err(db_error)? as u64;
        }
        let guided = completed_guided_sessions(&transaction)?;
        let mut guided_events_created = 0_u64;
        for (id, completed_at) in &guided {
            let activity_day: String = transaction
                .query_row("SELECT date(?1,'localtime')", [completed_at], |row| {
                    row.get(0)
                })
                .map_err(db_error)?;
            guided_events_created += transaction
                .execute(
                    "INSERT OR IGNORE INTO guided_gamification_xp_event
                 (id,session_id,rule_version,xp_amount,activity_day,created_at)
                 VALUES(?1,?2,?3,?4,?5,?6)",
                    params![
                        uuid::Uuid::new_v4().to_string(),
                        id,
                        GUIDED_XP_RULE_VERSION,
                        GUIDED_SESSION_XP,
                        activity_day,
                        completed_at
                    ],
                )
                .map_err(db_error)? as u64;
        }
        let newly_unlocked = unlock_achievements(&transaction)?;
        transaction.commit().map_err(db_error)?;
        let achievements = self.achievements()?;
        let unlocked_ids = newly_unlocked;
        Ok(GamificationSyncResult {
            inspected_lessons: lessons.len() as u64,
            qualifying_lessons: qualifying,
            ignored_lessons: lessons.len() as u64 - qualifying,
            events_created,
            achievements_unlocked: achievements
                .into_iter()
                .filter(|item| unlocked_ids.iter().any(|id| id == &item.id))
                .collect(),
            inspected_guided_sessions: guided.len() as u64,
            guided_events_created,
        })
    }

    pub fn overview(&self) -> Result<GamificationOverviewDto, String> {
        self.sync()?;
        let connection = database::open(&self.database)?;
        let events = events(&connection)?;
        let total_xp = connection.query_row(
            "SELECT
               (SELECT COALESCE(SUM(xp_amount),0) FROM gamification_xp_event WHERE rule_version=?1)+
               (SELECT COALESCE(SUM(xp_amount),0) FROM guided_gamification_xp_event WHERE rule_version=?2)+
               (SELECT COALESCE(SUM(xp_amount),0) FROM learning_practice_xp_event WHERE rule_version=?3)",
            params![GAMIFICATION_XP_RULE_VERSION,GUIDED_XP_RULE_VERSION,crate::practice_repository::PRACTICE_XP_RULE_VERSION], |row| row.get::<_, i64>(0),
        ).map_err(db_error)?.max(0) as u64;
        let total_seconds = events
            .iter()
            .map(|event| event.duration_seconds)
            .sum::<u64>();
        let days = events
            .iter()
            .map(|event| event.activity_day)
            .collect::<Vec<_>>();
        let today_text: String = connection
            .query_row("SELECT date('now','localtime')", [], |row| row.get(0))
            .map_err(db_error)?;
        let today = LocalDate::parse(&today_text)?;
        let week_start = today.iso_week_start();
        let weekly_seconds = events
            .iter()
            .filter(|event| event.activity_day >= week_start && event.activity_day <= today)
            .map(|event| event.duration_seconds)
            .sum::<u64>();
        let goal = profile_with(&connection)?.weekly_goal_minutes;
        let level = level_progress_from_xp(total_xp);
        let unlocked = connection
            .query_row("SELECT COUNT(*) FROM achievement_unlock", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(db_error)?
            .max(0) as u64;
        Ok(GamificationOverviewDto {
            schema_version: GAMIFICATION_SCHEMA_VERSION,
            total_xp,
            practice_level: level.practice_level,
            current_level_threshold: level.current_level_threshold,
            next_level_threshold: level.next_level_threshold,
            xp_into_current_level: level.xp_into_current_level,
            xp_needed_for_next_level: level.xp_needed_for_next_level,
            qualifying_lesson_count: events.len() as u64,
            total_practice_minutes: total_seconds / 60,
            current_streak_days: current_streak(&days, today),
            longest_streak_days: longest_streak(&days),
            weekly_goal: weekly_goal_progress(goal, weekly_seconds / 60),
            unlocked_achievement_count: unlocked,
            total_achievement_count: ACHIEVEMENTS.len() as u64,
        })
    }

    pub fn profile(&self) -> Result<GamificationProfileDto, String> {
        let connection = database::open(&self.database)?;
        profile_with(&connection)
    }

    pub fn update_weekly_goal(&self, minutes: u32) -> Result<GamificationProfileDto, String> {
        validate_weekly_goal(minutes)?;
        let connection = database::open(&self.database)?;
        connection.execute(
            "UPDATE gamification_profile SET weekly_goal_minutes=?1, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE profile_key='default'",
            [minutes],
        ).map_err(db_error)?;
        profile_with(&connection)
    }

    pub fn achievements(&self) -> Result<Vec<AchievementDto>, String> {
        let connection = database::open(&self.database)?;
        achievements_with(&connection)
    }
}

fn completed_lessons(transaction: &Transaction<'_>) -> Result<Vec<(String, i64, String)>, String> {
    let mut statement = transaction.prepare(
        "SELECT id,COALESCE(duration_seconds,0),ended_at FROM lesson WHERE status='completed' AND ended_at IS NOT NULL ORDER BY ended_at,id"
    ).map_err(db_error)?;
    let rows = statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    Ok(rows)
}

fn completed_guided_sessions(
    transaction: &Transaction<'_>,
) -> Result<Vec<(String, String)>, String> {
    transaction
        .prepare(
            "SELECT id,completed_at FROM interactive_lesson_session
         WHERE status='completed' AND completed_at IS NOT NULL ORDER BY completed_at,id",
        )
        .map_err(db_error)?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)
}

fn valid_student_turns(transaction: &Transaction<'_>, lesson_id: &str) -> Result<u32, String> {
    let mut statement = transaction
        .prepare("SELECT text FROM transcript_message WHERE lesson_id=?1 AND role='student'")
        .map_err(db_error)?;
    let texts = statement
        .query_map([lesson_id], |row| row.get::<_, String>(0))
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    Ok(texts
        .into_iter()
        .filter(|text| !is_technical_transcript(text))
        .count() as u32)
}

fn events(connection: &Connection) -> Result<Vec<Event>, String> {
    let mut statement = connection
        .prepare(
            "SELECT created_at,activity_day,duration_seconds,source_type FROM (
               SELECT e.created_at,e.activity_day,COALESCE(l.duration_seconds,0) AS duration_seconds,
                      'standard' AS source_type,e.source_id AS source_id
               FROM gamification_xp_event e JOIN lesson l ON l.id=e.source_id
               WHERE e.rule_version=?1
               UNION ALL
               SELECT e.created_at,e.activity_day,
                      COALESCE((SELECT SUM(p.duration_seconds) FROM interactive_lesson_active_practice_event p WHERE p.session_id=e.session_id),0),
                      'guided',e.session_id
               FROM guided_gamification_xp_event e WHERE e.rule_version=?2
               UNION ALL
               SELECT e.created_at,e.activity_day,
                      COALESCE((SELECT SUM(p.duration_seconds) FROM learning_practice_active_time_event p WHERE p.session_id=e.session_id),0),
                      'practice',e.session_id
               FROM learning_practice_xp_event e WHERE e.rule_version=?3
             ) ORDER BY created_at,source_id",
        )
        .map_err(db_error)?;
    let raw = statement
        .query_map(
            params![
                GAMIFICATION_XP_RULE_VERSION,
                GUIDED_XP_RULE_VERSION,
                crate::practice_repository::PRACTICE_XP_RULE_VERSION
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    raw.into_iter()
        .map(|(ended_at, day, duration, source_type)| {
            Ok(Event {
                ended_at,
                activity_day: LocalDate::parse(&day)?,
                duration_seconds: duration.max(0) as u64,
                source_type: match source_type.as_str() {
                    "guided" => "guided",
                    "practice" => "practice",
                    _ => "standard",
                },
            })
        })
        .collect()
}

fn unlock_achievements(transaction: &Transaction<'_>) -> Result<Vec<String>, String> {
    let events = events(transaction)?;
    let guided = guided_progress_facts(transaction)?;
    let placement_at: Option<String> = transaction.query_row(
        "SELECT MIN(completed_at) FROM placement_attempt WHERE status='completed' AND completed_at IS NOT NULL",
        [], |row| row.get(0),
    ).map_err(db_error)?;
    let mut cumulative_seconds = 0_u64;
    let mut minute_crossings = BTreeMap::new();
    for event in &events {
        cumulative_seconds += event.duration_seconds;
        for target in [60_u64, 300, 600] {
            if cumulative_seconds >= target * 60 {
                minute_crossings
                    .entry(target)
                    .or_insert_with(|| event.ended_at.clone());
            }
        }
    }
    let mut day_end = BTreeMap::<LocalDate, String>::new();
    for event in &events {
        day_end
            .entry(event.activity_day)
            .or_insert_with(|| event.ended_at.clone());
    }
    let mut streak_crossings = BTreeMap::new();
    let mut previous = None;
    let mut run = 0_u64;
    for (day, ended_at) in day_end {
        run = if previous.is_some_and(|value: LocalDate| day.days_since(value) == 1) {
            run + 1
        } else {
            1
        };
        for target in [3_u64, 7, 14] {
            if run >= target {
                streak_crossings
                    .entry(target)
                    .or_insert_with(|| ended_at.clone());
            }
        }
        previous = Some(day);
    }
    let mut unlocked = Vec::new();
    for definition in ACHIEVEMENTS {
        let (value, timestamp) = match definition.criterion {
            AchievementCriterion::Lessons(target) => (
                events
                    .iter()
                    .filter(|event| event.source_type == "standard")
                    .count() as u64,
                events
                    .iter()
                    .filter(|event| event.source_type == "standard")
                    .nth(target.saturating_sub(1) as usize)
                    .map(|e| e.ended_at.clone()),
            ),
            AchievementCriterion::PracticeMinutes(target) => (
                cumulative_seconds / 60,
                minute_crossings.get(&target).cloned(),
            ),
            AchievementCriterion::LongestStreak(target) => (
                longest_streak(&events.iter().map(|e| e.activity_day).collect::<Vec<_>>()),
                streak_crossings.get(&target).cloned(),
            ),
            AchievementCriterion::PlacementCompleted => {
                (u64::from(placement_at.is_some()), placement_at.clone())
            }
            AchievementCriterion::GuidedLessons(target) => (
                guided.completed_sessions.len() as u64,
                guided
                    .completed_sessions
                    .get(target.saturating_sub(1) as usize)
                    .cloned(),
            ),
            AchievementCriterion::FirstGuidedUnitComplete => (
                u64::from(first_completed_unit_at(&guided).is_some()),
                first_completed_unit_at(&guided),
            ),
            AchievementCriterion::GuidedLevelComplete(level) => (
                guided_level_count(&guided, level).min(48) as u64,
                guided_level_complete_at(&guided, level),
            ),
            AchievementCriterion::GuidedCourseComplete => (
                guided.unique_lessons.len().min(288) as u64,
                guided_course_complete_at(&guided),
            ),
        };
        if let Some(unlocked_at) = timestamp {
            let inserted = transaction.execute(
                "INSERT OR IGNORE INTO achievement_unlock(achievement_id,achievement_version,unlocked_at,trigger_value,created_at) VALUES (?1,?2,?3,?4,?3)",
                params![definition.id, definition.version, unlocked_at, value.min(i64::MAX as u64) as i64],
            ).map_err(db_error)?;
            if inserted > 0 {
                unlocked.push(definition.id.to_owned());
            }
        }
    }
    Ok(unlocked)
}

fn achievements_with(connection: &Connection) -> Result<Vec<AchievementDto>, String> {
    let events = events(connection)?;
    let guided = guided_progress_facts(connection)?;
    let practice_minutes = events
        .iter()
        .map(|event| event.duration_seconds)
        .sum::<u64>()
        / 60;
    let longest = longest_streak(
        &events
            .iter()
            .map(|event| event.activity_day)
            .collect::<Vec<_>>(),
    );
    let placement_count = connection
        .query_row(
            "SELECT COUNT(*) FROM placement_attempt WHERE status='completed'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(db_error)?
        .max(0) as u64;
    ACHIEVEMENTS.iter().map(|definition| {
        let unlocked_at: Option<String> = connection.query_row(
            "SELECT unlocked_at FROM achievement_unlock WHERE achievement_id=?1 AND achievement_version=?2",
            params![definition.id, definition.version], |row| row.get(0),
        ).optional().map_err(db_error)?;
        let (current, target) = match definition.criterion {
            AchievementCriterion::Lessons(target) => (
                events.iter().filter(|event| event.source_type == "standard").count() as u64,
                target,
            ),
            AchievementCriterion::PracticeMinutes(target) => (practice_minutes, target),
            AchievementCriterion::LongestStreak(target) => (longest, target),
            AchievementCriterion::PlacementCompleted => (placement_count.min(1), 1),
            AchievementCriterion::GuidedLessons(target) => {
                (guided.completed_sessions.len() as u64, target)
            }
            AchievementCriterion::FirstGuidedUnitComplete => {
                (u64::from(first_completed_unit_at(&guided).is_some()), 1)
            }
            AchievementCriterion::GuidedLevelComplete(level) => {
                (guided_level_count(&guided, level).min(48) as u64, 48)
            }
            AchievementCriterion::GuidedCourseComplete => {
                (guided.unique_lessons.len().min(288) as u64, 288)
            }
        };
        Ok(AchievementDto {
            id: definition.id.to_owned(), version: definition.version, title: definition.title.to_owned(),
            description: definition.description.to_owned(), category: definition.category.to_owned(),
            unlocked: unlocked_at.is_some(), unlocked_at, progress_current: current.min(target), progress_target: target,
        })
    }).collect()
}

fn guided_progress_facts(connection: &Connection) -> Result<GuidedProgressFacts, String> {
    let rows = connection
        .prepare(
            "SELECT lesson_id,completed_at FROM interactive_lesson_session
             WHERE status='completed' AND completed_at IS NOT NULL ORDER BY completed_at,id",
        )
        .map_err(db_error)?
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    let mut facts = GuidedProgressFacts::default();
    for (lesson_id, completed_at) in rows {
        facts.completed_sessions.push(completed_at.clone());
        facts
            .unique_lessons
            .entry(lesson_id)
            .or_insert(completed_at);
    }
    Ok(facts)
}

fn guided_level_count(facts: &GuidedProgressFacts, level: &str) -> usize {
    let prefix = format!("{level}-");
    facts
        .unique_lessons
        .keys()
        .filter(|lesson_id| lesson_id.starts_with(&prefix))
        .count()
}

fn guided_level_complete_at(facts: &GuidedProgressFacts, level: &str) -> Option<String> {
    if guided_level_count(facts, level) < 48 {
        return None;
    }
    let prefix = format!("{level}-");
    facts
        .unique_lessons
        .iter()
        .filter(|(lesson_id, _)| lesson_id.starts_with(&prefix))
        .map(|(_, completed_at)| completed_at.clone())
        .max()
}

fn first_completed_unit_at(facts: &GuidedProgressFacts) -> Option<String> {
    let mut units = BTreeMap::<String, Vec<String>>::new();
    for (lesson_id, completed_at) in &facts.unique_lessons {
        let mut parts = lesson_id.split('-');
        let (Some(level), Some(unit)) = (parts.next(), parts.next()) else {
            continue;
        };
        if !unit.starts_with('u') {
            continue;
        }
        units
            .entry(format!("{level}-{unit}"))
            .or_default()
            .push(completed_at.clone());
    }
    units
        .into_values()
        .filter(|values| values.len() >= 6)
        .filter_map(|values| values.into_iter().max())
        .min()
}

fn guided_course_complete_at(facts: &GuidedProgressFacts) -> Option<String> {
    (facts.unique_lessons.len() >= 288)
        .then(|| facts.unique_lessons.values().cloned().max())
        .flatten()
}

fn profile_with(connection: &Connection) -> Result<GamificationProfileDto, String> {
    let minutes: u32 = connection
        .query_row(
            "SELECT weekly_goal_minutes FROM gamification_profile WHERE profile_key='default'",
            [],
            |row| row.get(0),
        )
        .map_err(db_error)?;
    Ok(GamificationProfileDto {
        schema_version: GAMIFICATION_SCHEMA_VERSION,
        weekly_goal_minutes: minutes,
    })
}

fn db_error(error: rusqlite::Error) -> String {
    format!("Gamification database error: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    fn repository() -> (PathBuf, GamificationRepository) {
        let directory =
            std::env::temp_dir().join(format!("gamification-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("test.sqlite3");
        database::migrate(&path).unwrap();
        (directory, GamificationRepository::new(path))
    }

    fn lesson(
        repo: &GamificationRepository,
        id: &str,
        ended: &str,
        duration: i64,
        messages: &[&str],
    ) {
        let c = database::open(&repo.database).unwrap();
        c.execute("INSERT INTO lesson(id,started_at,ended_at,status,mode,duration_seconds,whisper_model,whisper_threads,ollama_model,piper_voice,voice_engine_version,created_at,updated_at) VALUES(?1,?2,?2,'completed','free_conversation',?3,'w',1,'o','p','v',?2,?2)", params![id,ended,duration]).unwrap();
        for (index, text) in messages.iter().enumerate() {
            let sequence = (index + 1) as i64;
            c.execute("INSERT INTO transcript_message(id,lesson_id,sequence_index,turn_index,role,text,source,engine_event_type,created_at) VALUES(?1,?2,?3,?3,'student',?4,'voice','final',?5)", params![format!("{id}-{index}"),id,sequence,text,ended]).unwrap();
        }
    }

    #[test]
    fn historical_sync_is_idempotent_and_uses_valid_transcripts_only() {
        let (directory, repo) = repository();
        lesson(
            &repo,
            "one",
            "2026-08-18T12:00:00Z",
            180,
            &["hello", "[SILENCE]", "how are you", "great"],
        );
        lesson(
            &repo,
            "short",
            "2026-08-19T12:00:00Z",
            119,
            &["one", "two", "three"],
        );
        lesson(
            &repo,
            "two",
            "2026-08-19T12:00:00Z",
            240,
            &["one", "two", "three"],
        );
        lesson(
            &repo,
            "three",
            "2026-08-20T12:00:00Z",
            300,
            &["one", "two", "three"],
        );
        let first = repo.sync().unwrap();
        let second = repo.sync().unwrap();
        assert_eq!((first.qualifying_lessons, first.events_created), (3, 3));
        assert_eq!(second.events_created, 0);
        let overview = repo.overview().unwrap();
        assert_eq!(overview.qualifying_lesson_count, 3);
        assert_eq!(overview.total_xp, 49 + 50 + 51);
        assert!(
            repo.achievements()
                .unwrap()
                .iter()
                .find(|a| a.id == "first_conversation")
                .unwrap()
                .unlocked
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn weekly_goal_is_validated_and_persists() {
        let (directory, repo) = repository();
        assert!(repo.update_weekly_goal(95).is_err());
        assert_eq!(
            repo.update_weekly_goal(105).unwrap().weekly_goal_minutes,
            105
        );
        assert_eq!(
            GamificationRepository::new(repo.database.clone())
                .profile()
                .unwrap()
                .weekly_goal_minutes,
            105
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn guided_xp_is_exactly_once_per_session_and_repeat_sessions_are_legitimate_practice() {
        let (directory, repo) = repository();
        let connection = database::open(&repo.database).unwrap();
        for (id, completed) in [
            ("guided-one", "2026-08-24T12:00:00Z"),
            ("guided-repeat", "2026-08-25T12:00:00Z"),
        ] {
            connection.execute("INSERT INTO interactive_lesson_session(id,lesson_id,lesson_content_version,package_schema_version,lesson_flow_version,package_hash,engine_version,snapshot_version,status,stage_count,current_stage_index,package_snapshot_json,student_context_snapshot_json,started_at,updated_at,completed_at) VALUES(?1,'a1-u01-l01-hello-goodbye',1,1,1,?2,1,1,'completed',1,0,'{}','{}',?3,?3,?3)",params![id,"a".repeat(64),completed]).unwrap();
        }
        for (event, session, seconds) in [
            ("active-1", "guided-one", 30),
            ("active-2", "guided-one", 30),
            ("active-3", "guided-repeat", 15),
        ] {
            connection.execute("INSERT INTO interactive_lesson_active_practice_event(event_id,session_id,duration_seconds,recorded_at) VALUES(?1,?2,?3,'2026-08-25T12:00:00Z')",params![event,session,seconds]).unwrap();
        }
        drop(connection);
        let first = repo.sync().unwrap();
        let second = repo.sync().unwrap();
        assert_eq!(first.guided_events_created, 2);
        assert_eq!(second.guided_events_created, 0);
        let overview = repo.overview().unwrap();
        assert_eq!(overview.total_xp, 120);
        assert_eq!(overview.qualifying_lesson_count, 2);
        assert_eq!(overview.total_practice_minutes, 1);
        let achievements = repo.achievements().unwrap();
        assert!(
            achievements
                .iter()
                .find(|item| item.id == "first_guided_lesson")
                .unwrap()
                .unlocked
        );
        assert!(
            !achievements
                .iter()
                .find(|item| item.id == "ten_guided_lessons")
                .unwrap()
                .unlocked
        );
        let connection = database::open(&repo.database).unwrap();
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM placement_attempt", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM guided_gamification_xp_event",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            2
        );
        drop(connection);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    #[ignore = "manual migration, sync and goal persistence audit against the user's physical SQLite database"]
    fn physical_phase_l_migrates_syncs_and_preserves_goal() {
        let path = PathBuf::from(std::env::var_os("LOCALAPPDATA").expect("LOCALAPPDATA"))
            .join("com.englishaicoach.desktop")
            .join("database")
            .join("english-ai-coach.sqlite3");
        database::migrate(&path).expect("migrate physical database");
        let repository = GamificationRepository::new(path.clone());
        let first = repository.sync().expect("first physical sync");
        let second = repository.sync().expect("idempotent physical sync");
        assert_eq!(second.events_created, 0);
        repository
            .update_weekly_goal(105)
            .expect("set temporary goal");
        assert_eq!(
            GamificationRepository::new(path.clone())
                .profile()
                .unwrap()
                .weekly_goal_minutes,
            105
        );
        repository
            .update_weekly_goal(90)
            .expect("restore default goal");
        assert_eq!(
            GamificationRepository::new(path.clone())
                .profile()
                .unwrap()
                .weekly_goal_minutes,
            90
        );
        let overview = repository.overview().expect("physical overview");
        let achievements = repository.achievements().expect("physical achievements");
        let connection = database::open(&path).expect("open physical database");
        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migration", [], |row| {
                row.get(0)
            })
            .unwrap();
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        let foreign_keys: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, 9);
        assert_eq!(integrity, "ok");
        assert_eq!(foreign_keys, 0);
        println!("PHASE_L inspected={} qualifying={} ignored={} first_events={} repeat_events={} total_xp={} level={} total_minutes={} current_streak={} longest_streak={} weekly={}/{} achievements={:?}", first.inspected_lessons, first.qualifying_lessons, first.ignored_lessons, first.events_created, second.events_created, overview.total_xp, overview.practice_level, overview.total_practice_minutes, overview.current_streak_days, overview.longest_streak_days, overview.weekly_goal.practiced_minutes, overview.weekly_goal.goal_minutes, achievements.iter().filter(|item| item.unlocked).map(|item| item.id.as_str()).collect::<Vec<_>>());
    }
}
