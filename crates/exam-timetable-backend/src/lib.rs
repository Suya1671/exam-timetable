use std::collections::HashMap;
use std::hash::Hash;

use exam_timetable_model::{ExamId, StudentId, SubjectId, TimeslotId};
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

    for exam_id in exam_ids {
        let allowed_timeslots = build_allowed_timeslots(&mut txn, exam_id).await?;
        scheduler.add_allowed_timeslots(exam_id, &allowed_timeslots);

        let disallowed_timeslots = build_disallowed_timeslots(&mut txn, exam_id).await?;
        scheduler.add_disallowed_timeslots(exam_id, &disallowed_timeslots);
    }

    let results = scheduler.solve()?;

    Ok(results)
}

struct TimeslotRow {
    id: TimeslotId,
    date: Date,
}

/// Groups timeslots by the weekday of their calendar date
async fn group_by_weekday(
    mut txn: Transaction<'_, Sqlite>,
) -> Result<HashMap<time::Weekday, Vec<TimeslotId>>, SolveError> {
    let timeslots = fetch_timeslots(&mut txn).await?;
    Ok(group_timeslots_by(timeslots, |ts| ts.date.weekday()))
}

/// Groups timeslots by the calendar date
async fn group_days(
    txn: &mut Transaction<'_, Sqlite>,
) -> Result<HashMap<Date, Vec<TimeslotId>>, SolveError> {
    let timeslots = fetch_timeslots(txn).await?;
    Ok(group_timeslots_by(timeslots, |ts| ts.date))
}

async fn build_allowed_timeslots(
    txn: &mut Transaction<'_, Sqlite>,
    exam: ExamId,
) -> Result<Vec<TimeslotId>, SolveError> {
    let results = query_scalar!(
        "
            -- Exams
            SELECT
                timeslot_id as 'timeslot_id: TimeslotId'
            FROM exam_allowed_timeslots
            WHERE exam_id = $1
            -- Students
            UNION
            SELECT
                timeslot_id as 'timeslot_id: TimeslotId'
            FROM student_allowed_timeslots
            JOIN enrolled_students ON student_allowed_timeslots.student_id = enrolled_students.student_id
            JOIN exams ON enrolled_students.subject_id = exams.subject_id
            WHERE exams.id = $1
            -- Subjects
            UNION
            SELECT
                timeslot_id as 'timeslot_id: TimeslotId'
            FROM subject_allowed_timeslots
            JOIN exams ON subject_allowed_timeslots.subject_id = exams.subject_id
            WHERE exams.id = $1
        ",
        exam
    ).fetch_all(&mut **txn)
    .await?;

    Ok(results.into_iter().unique().collect())
}

async fn build_disallowed_timeslots(
    txn: &mut Transaction<'_, Sqlite>,
    exam: ExamId,
) -> Result<Vec<TimeslotId>, SolveError> {
    let results = query_scalar!(
        "
            -- Exams
            SELECT
                timeslot_id as 'timeslot_id: TimeslotId'
            FROM exam_denied_timeslots
            WHERE exam_id = $1
            -- Students
            UNION
            SELECT
                timeslot_id as 'timeslot_id: TimeslotId'
            FROM student_denied_timeslots
            JOIN enrolled_students ON student_denied_timeslots.student_id = enrolled_students.student_id
            JOIN exams ON enrolled_students.subject_id = exams.subject_id
            WHERE exams.id = $1
            -- Subjects
            UNION
            SELECT
                timeslot_id as 'timeslot_id: TimeslotId'
            FROM subject_denied_timeslots
            JOIN exams ON subject_denied_timeslots.subject_id = exams.subject_id
            WHERE exams.id = $1
        ",
        exam
    ).fetch_all(&mut **txn)
    .await?;

    Ok(results.into_iter().unique().collect())
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
