use crate::{
    database,
    lesson_analysis::{LessonAnalysis, LessonAnalysisPayload, LessonAnalysisScores},
    lesson_analysis_repository::LessonAnalysisRepository,
    lesson_modes::{legacy_configuration, LessonConfigurationDto, LessonConfigurationRepository},
    lesson_repository::{CorrectionCandidate, Lesson, LessonRepository, TranscriptMessage},
    placement_repository::PlacementRepository,
    student_profile_repository::{LessonStudentProfileSnapshotDto, StudentProfileRepository},
};
use rusqlite::{OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone)]
pub struct LearningRepository {
    database: PathBuf,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonHistoryFilter {
    All,
    Completed,
    Interrupted,
    Analyzed,
    Unanalyzed,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonHistoryItem {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_seconds: Option<i64>,
    pub status: String,
    pub topic: Option<String>,
    pub mode: String,
    pub mode_id: String,
    pub mode_title: String,
    pub custom_title: Option<String>,
    pub student_turn_count: u32,
    pub teacher_turn_count: u32,
    pub correction_count: u32,
    pub analysis_status: Option<String>,
    pub overall_score: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonHistoryPage {
    pub items: Vec<LessonHistoryItem>,
    pub total: u32,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardLatestAnalysis {
    pub lesson_id: String,
    pub started_at: String,
    pub duration_seconds: Option<i64>,
    pub overall_score: i32,
    pub scores: LessonAnalysisScores,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub total_lessons: u32,
    pub completed_lessons: u32,
    pub total_practice_seconds: Option<i64>,
    pub total_student_turns: u32,
    pub total_corrections: u32,
    pub analyzed_lessons: u32,
    pub average_overall_score: Option<i32>,
    pub latest_lesson: Option<LessonHistoryItem>,
    pub latest_analyzed_lesson: Option<DashboardLatestAnalysis>,
    pub latest_recommendation: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreDimension {
    Fluency,
    Grammar,
    Vocabulary,
    Comprehension,
    Interaction,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressAverages {
    pub overall: i32,
    pub fluency: i32,
    pub grammar: i32,
    pub vocabulary: i32,
    pub comprehension: i32,
    pub interaction: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressPoint {
    pub lesson_id: String,
    pub date: String,
    pub duration_seconds: Option<i64>,
    pub overall: i32,
    pub fluency: i32,
    pub grammar: i32,
    pub vocabulary: i32,
    pub comprehension: i32,
    pub interaction: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressOverview {
    pub analyzed_lesson_count: u32,
    pub averages: Option<ProgressAverages>,
    pub strongest_areas: Vec<ScoreDimension>,
    pub focus_areas: Vec<ScoreDimension>,
    pub points: Vec<ProgressPoint>,
    pub latest_recommendation: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonDetails {
    pub lesson: Lesson,
    pub configuration: LessonConfigurationDto,
    pub messages: Vec<TranscriptMessage>,
    pub correction_candidates: Vec<CorrectionCandidate>,
    pub analysis: Option<LessonAnalysis>,
    pub student_profile_snapshot: Option<LessonStudentProfileSnapshotDto>,
}

impl LearningRepository {
    pub fn new(database: PathBuf) -> Self {
        Self { database }
    }

    pub fn dashboard_summary(&self) -> Result<DashboardSummary, String> {
        let connection = database::open(&self.database)?;
        let (total_lessons, completed_lessons, practice_seconds, student_turns, corrections): (
            u32,
            u32,
            Option<i64>,
            u32,
            u32,
        ) = connection
            .query_row(
                "SELECT
                   COUNT(*) FILTER (WHERE status IN ('completed', 'interrupted') AND student_turn_count > 0),
                   COUNT(*) FILTER (WHERE status = 'completed' AND student_turn_count > 0),
                   CASE
                     WHEN COUNT(*) FILTER (WHERE status IN ('completed', 'interrupted') AND student_turn_count > 0)
                        = COUNT(duration_seconds) FILTER (WHERE status IN ('completed', 'interrupted') AND student_turn_count > 0)
                     THEN SUM(CASE WHEN status IN ('completed', 'interrupted') AND student_turn_count > 0
                                        THEN duration_seconds END)
                     ELSE NULL
                   END,
                   COALESCE(SUM(CASE WHEN status IN ('completed', 'interrupted') AND student_turn_count > 0
                                     THEN student_turn_count ELSE 0 END), 0),
                   COALESCE(SUM(CASE WHEN status IN ('completed', 'interrupted') AND student_turn_count > 0
                                     THEN correction_count ELSE 0 END), 0)
                 FROM lesson",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .map_err(|error| format!("Could not calculate dashboard lesson totals: {error}"))?;
        let (analyzed_lessons, average_overall): (u32, Option<f64>) = connection
            .query_row(
                "SELECT COUNT(*), AVG(overall_score) FROM lesson_analysis
                 WHERE status = 'completed' AND overall_score IS NOT NULL",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|error| format!("Could not calculate dashboard analysis totals: {error}"))?;

        let latest_lesson = connection
            .query_row(
                &format!(
                    "{} WHERE l.status IN ('completed', 'interrupted') AND l.student_turn_count > 0
                     ORDER BY l.started_at DESC, l.created_at DESC, l.id DESC LIMIT 1",
                    history_select()
                ),
                [],
                history_row,
            )
            .optional()
            .map_err(|error| format!("Could not read latest lesson: {error}"))?;

        let latest_analysis_row: Option<(DashboardLatestAnalysis, String)> = connection
            .query_row(
                "SELECT l.id, l.started_at, l.duration_seconds, la.overall_score,
                        la.fluency_score, la.grammar_score, la.vocabulary_score,
                        la.comprehension_score, la.interaction_score, la.raw_json
                 FROM lesson_analysis la JOIN lesson l ON l.id = la.lesson_id
                 WHERE la.status = 'completed' AND la.overall_score IS NOT NULL
                   AND la.fluency_score IS NOT NULL AND la.grammar_score IS NOT NULL
                   AND la.vocabulary_score IS NOT NULL AND la.comprehension_score IS NOT NULL
                   AND la.interaction_score IS NOT NULL AND la.raw_json IS NOT NULL
                 ORDER BY l.started_at DESC, l.created_at DESC, l.id DESC LIMIT 1",
                [],
                |row| {
                    Ok((
                        DashboardLatestAnalysis {
                            lesson_id: row.get(0)?,
                            started_at: row.get(1)?,
                            duration_seconds: row.get(2)?,
                            overall_score: row.get(3)?,
                            scores: LessonAnalysisScores {
                                fluency: row.get(4)?,
                                grammar: row.get(5)?,
                                vocabulary: row.get(6)?,
                                comprehension: row.get(7)?,
                                interaction: row.get(8)?,
                                pronunciation: None,
                            },
                        },
                        row.get(9)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("Could not read latest analyzed lesson: {error}"))?;
        let (latest_analyzed_lesson, latest_recommendation) =
            latest_analysis_row.map_or(Ok((None, None)), |(latest, raw)| {
                let payload: LessonAnalysisPayload = serde_json::from_str(&raw)
                    .map_err(|error| format!("Latest analysis JSON is invalid: {error}"))?;
                Ok::<_, String>((
                    Some(latest),
                    payload.next_lesson_recommendations.first().cloned(),
                ))
            })?;

        Ok(DashboardSummary {
            total_lessons,
            completed_lessons,
            total_practice_seconds: practice_seconds,
            total_student_turns: student_turns,
            total_corrections: corrections,
            analyzed_lessons,
            average_overall_score: average_overall.map(round_score),
            latest_lesson,
            latest_analyzed_lesson,
            latest_recommendation,
        })
    }

    pub fn list_lessons(
        &self,
        filter: LessonHistoryFilter,
        limit: u32,
        offset: u32,
    ) -> Result<LessonHistoryPage, String> {
        if !(1..=100).contains(&limit) {
            return Err("History limit must be between 1 and 100.".to_owned());
        }
        let connection = database::open(&self.database)?;
        let condition = history_condition(filter);
        let total: u32 = connection
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM lesson l LEFT JOIN lesson_analysis la ON la.lesson_id = l.id WHERE {condition}"
                ),
                [],
                |row| row.get(0),
            )
            .map_err(|error| format!("Could not count lesson history: {error}"))?;
        let mut statement = connection
            .prepare(&format!(
                "{} WHERE {condition}
                 ORDER BY l.started_at DESC, l.created_at DESC, l.id DESC LIMIT ?1 OFFSET ?2",
                history_select()
            ))
            .map_err(|error| format!("Could not prepare lesson history: {error}"))?;
        let items = statement
            .query_map([limit, offset], history_row)
            .map_err(|error| format!("Could not query lesson history: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Could not read lesson history: {error}"))?;
        Ok(LessonHistoryPage {
            items,
            total,
            limit,
            offset,
        })
    }

    pub fn lesson_details(&self, lesson_id: &str) -> Result<Option<LessonDetails>, String> {
        let lessons = LessonRepository::new(self.database.clone());
        let Some(lesson) = lessons.get_lesson(lesson_id)? else {
            return Ok(None);
        };
        let messages = lessons.get_lesson_messages(lesson_id)?;
        let correction_candidates = lessons.get_correction_candidates(lesson_id)?;
        let analysis =
            LessonAnalysisRepository::new(self.database.clone()).get_by_lesson(lesson_id)?;
        let configuration = LessonConfigurationRepository::new(self.database.clone())
            .get(lesson_id)?
            .unwrap_or_else(|| legacy_configuration(lesson_id, lesson.topic.clone()));
        let placement = PlacementRepository::new(self.database.clone())?;
        let student_profile_snapshot =
            StudentProfileRepository::new(self.database.clone(), placement).snapshot(lesson_id)?;
        Ok(Some(LessonDetails {
            lesson,
            configuration,
            messages,
            correction_candidates,
            analysis,
            student_profile_snapshot,
        }))
    }

    pub fn progress_overview(&self) -> Result<ProgressOverview, String> {
        let connection = database::open(&self.database)?;
        let mut statement = connection
            .prepare(
                "SELECT l.id, l.started_at, l.duration_seconds, la.overall_score,
                        la.fluency_score, la.grammar_score, la.vocabulary_score,
                        la.comprehension_score, la.interaction_score, la.raw_json
                 FROM lesson_analysis la JOIN lesson l ON l.id = la.lesson_id
                 WHERE la.status = 'completed' AND la.overall_score IS NOT NULL
                   AND la.fluency_score IS NOT NULL AND la.grammar_score IS NOT NULL
                   AND la.vocabulary_score IS NOT NULL AND la.comprehension_score IS NOT NULL
                   AND la.interaction_score IS NOT NULL AND la.raw_json IS NOT NULL
                 ORDER BY l.started_at ASC, l.created_at ASC, l.id ASC",
            )
            .map_err(|error| format!("Could not prepare progress query: {error}"))?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    ProgressPoint {
                        lesson_id: row.get(0)?,
                        date: row.get(1)?,
                        duration_seconds: row.get(2)?,
                        overall: row.get(3)?,
                        fluency: row.get(4)?,
                        grammar: row.get(5)?,
                        vocabulary: row.get(6)?,
                        comprehension: row.get(7)?,
                        interaction: row.get(8)?,
                    },
                    row.get::<_, String>(9)?,
                ))
            })
            .map_err(|error| format!("Could not query progress: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Could not read progress: {error}"))?;

        let mut points = Vec::with_capacity(rows.len());
        let mut latest_recommendation = None;
        for (point, raw) in rows {
            let payload: LessonAnalysisPayload = serde_json::from_str(&raw)
                .map_err(|error| format!("Progress analysis JSON is invalid: {error}"))?;
            latest_recommendation = payload.next_lesson_recommendations.first().cloned();
            points.push(point);
        }
        let (averages, strongest_areas, focus_areas) = calculate_progress(&points);
        Ok(ProgressOverview {
            analyzed_lesson_count: points.len() as u32,
            averages,
            strongest_areas,
            focus_areas,
            points,
            latest_recommendation,
        })
    }
}

fn history_select() -> &'static str {
    "SELECT l.id, l.started_at, l.ended_at, l.duration_seconds, l.status, l.topic, l.mode,
            l.student_turn_count, l.teacher_turn_count, l.correction_count,
            la.status, CASE WHEN la.status = 'completed' THEN la.overall_score END,
            COALESCE(lcs.mode_id, l.mode),
            CASE COALESCE(lcs.mode_id, l.mode)
              WHEN 'everyday_english' THEN 'Everyday English' WHEN 'travel_english' THEN 'Travel English'
              WHEN 'job_interview' THEN 'Job Interview' WHEN 'university_academic' THEN 'University / Academic English'
              WHEN 'debate_opinions' THEN 'Debate & Opinions' WHEN 'custom' THEN 'Custom Lesson'
              ELSE 'Free Conversation' END,
            lcs.custom_title
     FROM lesson l LEFT JOIN lesson_analysis la ON la.lesson_id = l.id
       LEFT JOIN lesson_configuration_snapshot lcs ON lcs.lesson_id = l.id"
}

fn history_condition(filter: LessonHistoryFilter) -> &'static str {
    match filter {
        LessonHistoryFilter::All => "1 = 1",
        LessonHistoryFilter::Completed => "l.status = 'completed'",
        LessonHistoryFilter::Interrupted => "l.status = 'interrupted'",
        LessonHistoryFilter::Analyzed => "la.id IS NOT NULL",
        LessonHistoryFilter::Unanalyzed => "la.id IS NULL",
    }
}

fn history_row(row: &Row<'_>) -> rusqlite::Result<LessonHistoryItem> {
    Ok(LessonHistoryItem {
        id: row.get(0)?,
        started_at: row.get(1)?,
        ended_at: row.get(2)?,
        duration_seconds: row.get(3)?,
        status: row.get(4)?,
        topic: row.get(5)?,
        mode: row.get(6)?,
        student_turn_count: row.get(7)?,
        teacher_turn_count: row.get(8)?,
        correction_count: row.get(9)?,
        analysis_status: row.get(10)?,
        overall_score: row.get(11)?,
        mode_id: row.get(12)?,
        mode_title: row.get(13)?,
        custom_title: row.get(14)?,
    })
}

fn round_score(value: f64) -> i32 {
    value.round() as i32
}

fn calculate_progress(
    points: &[ProgressPoint],
) -> (
    Option<ProgressAverages>,
    Vec<ScoreDimension>,
    Vec<ScoreDimension>,
) {
    if points.is_empty() {
        return (None, Vec::new(), Vec::new());
    }
    let sums = points.iter().fold([0_i64; 6], |mut sums, point| {
        for (slot, value) in sums.iter_mut().zip([
            point.overall,
            point.fluency,
            point.grammar,
            point.vocabulary,
            point.comprehension,
            point.interaction,
        ]) {
            *slot += i64::from(value);
        }
        sums
    });
    let count = points.len() as f64;
    let averages = ProgressAverages {
        overall: round_score(sums[0] as f64 / count),
        fluency: round_score(sums[1] as f64 / count),
        grammar: round_score(sums[2] as f64 / count),
        vocabulary: round_score(sums[3] as f64 / count),
        comprehension: round_score(sums[4] as f64 / count),
        interaction: round_score(sums[5] as f64 / count),
    };
    let dimension_sums = [
        (ScoreDimension::Fluency, sums[1]),
        (ScoreDimension::Grammar, sums[2]),
        (ScoreDimension::Vocabulary, sums[3]),
        (ScoreDimension::Comprehension, sums[4]),
        (ScoreDimension::Interaction, sums[5]),
    ];
    let maximum = dimension_sums.iter().map(|(_, value)| value).max().unwrap();
    let minimum = dimension_sums.iter().map(|(_, value)| value).min().unwrap();
    let strongest = dimension_sums
        .iter()
        .filter_map(|(dimension, value)| (*value == *maximum).then_some(*dimension))
        .collect();
    let focus = dimension_sums
        .iter()
        .filter_map(|(dimension, value)| (*value == *minimum).then_some(*dimension))
        .collect();
    (Some(averages), strongest, focus)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lesson_analysis::{
        LessonAnalysisCorrection, LessonAnalysisCorrectionCategory, LessonAnalysisStrength,
    };
    use rusqlite::params;

    fn repository() -> (PathBuf, PathBuf, LearningRepository) {
        let directory = std::env::temp_dir().join(format!(
            "english-ai-coach-learning-repository-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let database_path = directory.join("learning.sqlite3");
        database::migrate(&database_path).unwrap();
        (
            directory,
            database_path.clone(),
            LearningRepository::new(database_path),
        )
    }

    fn seed_lesson(
        database_path: &std::path::Path,
        id: &str,
        started_at: &str,
        status: &str,
        duration: Option<i64>,
        student_turns: u32,
    ) {
        let connection = database::open(database_path).unwrap();
        connection
            .execute(
                "INSERT INTO lesson (id, started_at, ended_at, status, topic, mode,
                   duration_seconds, student_turn_count, teacher_turn_count, correction_count,
                   whisper_model, whisper_threads, ollama_model, piper_voice,
                   voice_engine_version, created_at, updated_at)
                 VALUES (?1, ?2, ?2, ?3, NULL, 'free_conversation', ?4, ?5, ?5, ?5,
                         'whisper', 12, 'qwen3.5:4b', 'lessac', 'voice-v2', ?2, ?2)",
                params![id, started_at, status, duration, student_turns],
            )
            .unwrap();
    }

    fn payload(scores: [i32; 5], recommendation: &str) -> LessonAnalysisPayload {
        LessonAnalysisPayload {
            schema_version: 1,
            scores: LessonAnalysisScores {
                fluency: scores[0],
                grammar: scores[1],
                vocabulary: scores[2],
                comprehension: scores[3],
                interaction: scores[4],
                pronunciation: None,
            },
            strengths: vec![LessonAnalysisStrength {
                title: "Boa interação".to_owned(),
                evidence: "Hello teacher".to_owned(),
            }],
            priority_improvements: vec![],
            corrections: vec![LessonAnalysisCorrection {
                original: "I like cook".to_owned(),
                corrected: "I like cooking".to_owned(),
                explanation: "Use o gerúndio.".to_owned(),
                category: LessonAnalysisCorrectionCategory::Grammar,
            }],
            natural_alternatives: vec![],
            vocabulary: vec![],
            recurring_patterns: vec![],
            next_lesson_recommendations: vec![recommendation.to_owned()],
            summary: "Resumo pedagógico local suficientemente completo.".to_owned(),
            pronunciation_available: false,
        }
    }

    fn seed_completed_analysis(
        database_path: &std::path::Path,
        lesson_id: &str,
        overall: i32,
        scores: [i32; 5],
        recommendation: &str,
    ) {
        let connection = database::open(database_path).unwrap();
        let raw = serde_json::to_string(&payload(scores, recommendation)).unwrap();
        connection
            .execute(
                "INSERT INTO lesson_analysis (id, lesson_id, status, schema_version,
                   prompt_version, analyzer_model, started_at, completed_at, overall_score,
                   fluency_score, grammar_score, vocabulary_score, comprehension_score,
                   interaction_score, pronunciation_score, summary, raw_json, created_at, updated_at)
                 VALUES (?1, ?2, 'completed', 1, 1, 'qwen3.5:4b', '2026-01-01', '2026-01-01',
                         ?3, ?4, ?5, ?6, ?7, ?8, NULL, 'summary', ?9, '2026-01-01', '2026-01-01')",
                params![
                    format!("analysis-{lesson_id}"),
                    lesson_id,
                    overall,
                    scores[0],
                    scores[1],
                    scores[2],
                    scores[3],
                    scores[4],
                    raw,
                ],
            )
            .unwrap();
    }

    #[test]
    fn dashboard_handles_empty_unanalyzed_interrupted_and_valid_analysis() {
        let (directory, database_path, repository) = repository();
        assert_eq!(repository.dashboard_summary().unwrap().total_lessons, 0);
        seed_lesson(
            &database_path,
            "completed",
            "2026-01-01",
            "completed",
            Some(60),
            2,
        );
        seed_lesson(
            &database_path,
            "interrupted",
            "2026-01-02",
            "interrupted",
            Some(30),
            1,
        );
        seed_lesson(
            &database_path,
            "failed",
            "2026-01-03",
            "failed",
            Some(99),
            4,
        );
        seed_completed_analysis(
            &database_path,
            "completed",
            80,
            [80, 70, 75, 90, 85],
            "Praticar o passado.",
        );
        let dashboard = repository.dashboard_summary().unwrap();
        assert_eq!(dashboard.total_lessons, 2);
        assert_eq!(dashboard.completed_lessons, 1);
        assert_eq!(dashboard.total_practice_seconds, Some(90));
        assert_eq!(dashboard.total_student_turns, 3);
        assert_eq!(dashboard.total_corrections, 3);
        assert_eq!(dashboard.analyzed_lessons, 1);
        assert_eq!(dashboard.average_overall_score, Some(80));
        assert_eq!(dashboard.latest_lesson.unwrap().id, "interrupted");
        assert_eq!(
            dashboard.latest_recommendation.as_deref(),
            Some("Praticar o passado.")
        );
        seed_lesson(
            &database_path,
            "unknown-duration",
            "2026-01-04",
            "completed",
            None,
            1,
        );
        assert_eq!(
            repository
                .dashboard_summary()
                .unwrap()
                .total_practice_seconds,
            None
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn averages_ignore_missing_failed_and_insufficient_analyses() {
        let (directory, database_path, repository) = repository();
        for (id, date) in [
            ("a", "2026-01-01"),
            ("b", "2026-01-02"),
            ("c", "2026-01-03"),
        ] {
            seed_lesson(&database_path, id, date, "completed", Some(60), 1);
        }
        seed_completed_analysis(&database_path, "a", 80, [80, 60, 70, 90, 90], "Primeira");
        seed_completed_analysis(&database_path, "b", 90, [90, 80, 70, 80, 90], "Segunda");
        let connection = database::open(&database_path).unwrap();
        connection.execute(
            "INSERT INTO lesson_analysis (id, lesson_id, status, schema_version, prompt_version,
             analyzer_model, created_at, updated_at) VALUES ('failed-analysis', 'c', 'failed', 1, 1,
             'qwen', '2026-01-03', '2026-01-03')", [],
        ).unwrap();
        drop(connection);
        let dashboard = repository.dashboard_summary().unwrap();
        assert_eq!(dashboard.average_overall_score, Some(85));
        assert_eq!(dashboard.analyzed_lessons, 2);
        let progress = repository.progress_overview().unwrap();
        assert_eq!(progress.analyzed_lesson_count, 2);
        assert_eq!(progress.averages.as_ref().unwrap().grammar, 70);
        assert_eq!(progress.strongest_areas, vec![ScoreDimension::Interaction]);
        assert_eq!(
            progress.focus_areas,
            vec![ScoreDimension::Grammar, ScoreDimension::Vocabulary]
        );
        assert_eq!(progress.points[0].lesson_id, "a");
        assert_eq!(progress.points[1].lesson_id, "b");
        assert_eq!(progress.latest_recommendation.as_deref(), Some("Segunda"));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn progress_has_no_trend_data_when_empty_and_preserves_one_real_point() {
        let (directory, database_path, repository) = repository();
        let empty = repository.progress_overview().unwrap();
        assert!(empty.averages.is_none());
        assert!(empty.points.is_empty());
        seed_lesson(
            &database_path,
            "one",
            "2026-01-01",
            "completed",
            Some(40),
            1,
        );
        seed_completed_analysis(&database_path, "one", 81, [85, 70, 80, 90, 80], "Foco real");
        let one = repository.progress_overview().unwrap();
        assert_eq!(one.analyzed_lesson_count, 1);
        assert_eq!(one.points.len(), 1);
        assert_eq!(one.points[0].overall, 81);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn history_filters_paginates_and_orders_newest_first() {
        let (directory, database_path, repository) = repository();
        for index in 0..25 {
            let status = if index % 2 == 0 {
                "completed"
            } else {
                "interrupted"
            };
            seed_lesson(
                &database_path,
                &format!("lesson-{index:02}"),
                &format!("2026-01-{:02}T00:00:00Z", index + 1),
                status,
                Some(60),
                1,
            );
        }
        seed_completed_analysis(&database_path, "lesson-24", 80, [80; 5], "Foco");
        let first = repository
            .list_lessons(LessonHistoryFilter::All, 20, 0)
            .unwrap();
        assert_eq!(first.total, 25);
        assert_eq!(first.items.len(), 20);
        assert_eq!(first.items[0].id, "lesson-24");
        let second = repository
            .list_lessons(LessonHistoryFilter::All, 20, 20)
            .unwrap();
        assert_eq!(second.items.len(), 5);
        assert_eq!(
            repository
                .list_lessons(LessonHistoryFilter::Completed, 100, 0)
                .unwrap()
                .total,
            13
        );
        assert_eq!(
            repository
                .list_lessons(LessonHistoryFilter::Interrupted, 100, 0)
                .unwrap()
                .total,
            12
        );
        assert_eq!(
            repository
                .list_lessons(LessonHistoryFilter::Analyzed, 100, 0)
                .unwrap()
                .total,
            1
        );
        assert_eq!(
            repository
                .list_lessons(LessonHistoryFilter::Unanalyzed, 100, 0)
                .unwrap()
                .total,
            24
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn details_are_ordered_related_and_isolated_to_one_lesson() {
        let (directory, database_path, repository) = repository();
        seed_lesson(
            &database_path,
            "target",
            "2026-01-01",
            "completed",
            Some(60),
            1,
        );
        seed_lesson(
            &database_path,
            "other",
            "2026-01-02",
            "completed",
            Some(60),
            1,
        );
        let connection = database::open(&database_path).unwrap();
        for (id, lesson, sequence, role, text) in [
            ("teacher", "target", 2, "teacher", "Try: I like cooking."),
            ("student", "target", 1, "student", "I like cook."),
            (
                "other-message",
                "other",
                1,
                "student",
                "Not part of target.",
            ),
        ] {
            connection.execute(
                "INSERT INTO transcript_message (id, lesson_id, sequence_index, turn_index, role,
                 text, source, engine_event_type, created_at) VALUES (?1, ?2, ?3, 1, ?4, ?5,
                 'test', 'test', '2026-01-01')",
                params![id, lesson, sequence, role, text],
            ).unwrap();
        }
        connection
            .execute(
                "INSERT INTO correction_candidate (id, lesson_id, student_message_id,
             teacher_message_id, student_text, teacher_response_text, detection_method, created_at)
             VALUES ('correction', 'target', 'student', 'teacher', 'I like cook.',
             'Try: I like cooking.', 'test', '2026-01-01')",
                [],
            )
            .unwrap();
        drop(connection);
        seed_completed_analysis(&database_path, "target", 81, [85, 70, 80, 90, 80], "Foco");
        let details = repository.lesson_details("target").unwrap().unwrap();
        assert_eq!(details.messages.len(), 2);
        assert_eq!(details.messages[0].id, "student");
        assert_eq!(details.messages[1].id, "teacher");
        assert!(details.configuration.legacy);
        assert_eq!(details.configuration.mode_title, "Free Conversation");
        assert_eq!(
            details.correction_candidates[0].teacher_message_id,
            "teacher"
        );
        assert_eq!(details.analysis.unwrap().overall_score, Some(81));
        assert!(repository.lesson_details("missing").unwrap().is_none());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    #[ignore = "manual read-only audit of Phase F against the user's physical SQLite database"]
    fn physical_phase_f_views_read_the_real_analyzed_lesson() {
        let local_app_data = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .expect("LOCALAPPDATA");
        let database_path = local_app_data
            .join("com.englishaicoach.desktop")
            .join("database")
            .join("english-ai-coach.sqlite3");
        let repository = LearningRepository::new(database_path);
        let dashboard = repository.dashboard_summary().expect("physical dashboard");
        let history = repository
            .list_lessons(LessonHistoryFilter::All, 100, 0)
            .expect("physical history");
        let lesson_id = "98d5e6f6-9c1a-47a1-8e1b-cba5421a0f34";
        let item = history
            .items
            .iter()
            .find(|item| item.id == lesson_id)
            .expect("real lesson in history");
        assert_eq!(item.overall_score, Some(81));
        assert_eq!(item.analysis_status.as_deref(), Some("completed"));
        let details = repository
            .lesson_details(lesson_id)
            .expect("physical details")
            .expect("real lesson details");
        assert_eq!(details.messages.len(), 6);
        assert_eq!(details.correction_candidates.len(), 1);
        let analysis = details.analysis.as_ref().expect("physical analysis");
        assert_eq!(analysis.overall_score, Some(81));
        let scores = analysis.scores.as_ref().expect("physical scores");
        assert_eq!(
            [
                scores.fluency,
                scores.grammar,
                scores.vocabulary,
                scores.comprehension,
                scores.interaction,
            ],
            [85, 70, 80, 90, 80]
        );
        assert_eq!(scores.pronunciation, None);
        assert_eq!(analysis.strengths[0].title, "Boa abertura e engajamento");
        assert!(analysis.priority_improvements[0]
            .better_alternative
            .contains("terrible at cooking"));
        assert_eq!(analysis.vocabulary[0].word_or_phrase, "terrible at");
        let progress = repository.progress_overview().expect("physical progress");
        assert_eq!(progress.analyzed_lesson_count, 1);
        assert_eq!(progress.points[0].overall, 81);
        assert_eq!(
            dashboard
                .latest_analyzed_lesson
                .as_ref()
                .unwrap()
                .overall_score,
            81
        );
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "dashboard": dashboard,
                "historyTotal": history.total,
                "realHistoryItem": item,
                "messageCount": details.messages.len(),
                "correctionCount": details.correction_candidates.len(),
                "analysis": analysis,
                "progress": progress,
            }))
            .unwrap()
        );
    }
}
