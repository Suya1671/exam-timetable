use std::collections::HashMap;
use std::hash::Hash;

use exam_timetable_model::{ExamId, StudentId, TimeslotId};
use exam_timetable_solver::{ExamScheduler, SolverError};
use itertools::Itertools;
use sqlx::{query, query_scalar, Sqlite, Transaction};
use time::Date;

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

    let mut scheduler = ExamScheduler::new(&exam_ids, n_timeslots);

    // Retrieve every exam that each student is enrolled in.
    // A student may be enrolled in multiple subjects, and each subject can have multiple exams.
    let exams_per_student = query!(
        "
        SELECT
            es.student_id as 'student_id: StudentId',
            e.id AS 'exam_id: ExamId'
        FROM enrolled_students AS es
        JOIN exams AS e
            ON es.subject_id = e.subject_id
        "
    )
    .fetch_all(&mut *txn)
    .await?
    .into_iter()
    .into_grouping_map_by(|row| row.student_id)
    .fold(Vec::new(), |mut acc, _student, row| {
        acc.push(row.exam_id);
        acc
    });

    scheduler.setup_students(&exams_per_student);

    let results = scheduler.solve()?;

    Ok(results)
}

/// Groups timeslots by the weekday of their calendar date
pub async fn group_by_weekday(
    mut txn: Transaction<'_, Sqlite>,
) -> Result<HashMap<time::Weekday, Vec<TimeslotId>>, SolveError> {
    let timeslots = fetch_timeslots(&mut txn).await?;
    Ok(group_timeslots_by(timeslots, |ts| ts.date.weekday()))
}

/// Groups timeslots by the calendar date
pub async fn group_days(
    mut txn: Transaction<'_, Sqlite>,
) -> Result<HashMap<Date, Vec<TimeslotId>>, SolveError> {
    let timeslots = fetch_timeslots(&mut txn).await?;
    Ok(group_timeslots_by(timeslots, |ts| ts.date))
}

struct TimeslotRow {
    id: TimeslotId,
    date: Date,
}

async fn fetch_timeslots(
    txn: &mut Transaction<'_, Sqlite>,
) -> Result<Vec<TimeslotRow>, sqlx::Error> {
    let rows = query!(
        "SELECT id as 'id: TimeslotId', date as 'date: time::Date' FROM timeslots ORDER BY date, slot",
    )
    .fetch_all(&mut **txn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| TimeslotRow {
            id: row.id,
            date: row.date,
        })
        .collect())
}

fn group_timeslots_by<K: Eq + Hash>(
    timeslots: Vec<TimeslotRow>,
    key_fn: impl Fn(&TimeslotRow) -> K,
) -> HashMap<K, Vec<TimeslotId>> {
    let mut grouped = HashMap::new();
    for timeslot in timeslots {
        grouped
            .entry(key_fn(&timeslot))
            .or_insert_with(Vec::new)
            .push(timeslot.id);
    }
    grouped
}
