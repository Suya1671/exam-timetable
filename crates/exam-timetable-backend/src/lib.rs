use std::collections::HashMap;

use exam_timetable_model::{ExamId, TimeslotId};
use exam_timetable_solver::{ExamScheduler, SolverError};
use sqlx::{query, query_scalar, Sqlite, SqlitePool, Transaction};

#[derive(Debug, thiserror::Error)]
pub enum SolveError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Solver error: {0}")]
    SolverError(#[from] SolverError),
}

pub async fn solve(
    mut txn: Transaction<'_, Sqlite>,
) -> Result<HashMap<ExamId, TimeslotId>, SolveError> {
    let exam_ids = query_scalar!("SELECT id FROM exams",)
        .fetch_all(&mut *txn)
        .await?;

    let n_timeslots = query_scalar!("SELECT COUNT(*) as n_timeslots FROM timeslots",)
        .fetch_one(&mut *txn)
        .await?;

    let scheduler = ExamScheduler::new(&exam_ids, n_timeslots);

    let results = scheduler.solve()?;

    Ok(results)
}

/// Groups timeslots by the weekday of their calendar date
pub async fn group_by_weekday(
    mut txn: Transaction<'_, Sqlite>,
) -> Result<HashMap<time::Weekday, Vec<TimeslotId>>, SolveError> {
    let timeslots = query!("SELECT id, date as 'date: time::Date', slot FROM timeslots",)
        .fetch_all(&mut *txn)
        .await?;

    let chunks = timeslots
        .chunk_by(|a, b| a.date.weekday() == b.date.weekday())
        .map(|group| {
            let weekday = group[0].date.weekday();
            let timeslot_ids = group.iter().map(|ts| ts.id).collect();
            (weekday, timeslot_ids)
        })
        .collect();

    Ok(chunks)
}

/// Groups timeslots by the calendar date
pub async fn group_days(
    mut txn: Transaction<'_, Sqlite>,
) -> Result<Vec<Vec<TimeslotId>>, SolveError> {
    let timeslots = query!("SELECT id, date as 'date: time::Date', slot FROM timeslots",)
        .fetch_all(&mut *txn)
        .await?;

    let chunks = timeslots
        .chunk_by(|a, b| a.date == b.date)
        .map(|group| group.iter().map(|ts| ts.id).collect())
        .collect();

    Ok(chunks)
}
