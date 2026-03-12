use std::collections::HashMap;
use std::hash::Hash;

use futures::{StreamExt, TryStreamExt};
use itertools::Itertools;
use model::{ExamId, StudentId, SubjectId, TimeslotId};
use solver::{ExamScheduler, SolverError};
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

    let timeslot_index = build_timeslot_index(&mut txn).await?;

    for exam_id in exam_ids {
        let allowed_timeslots = build_allowed_timeslots(&mut txn, exam_id).await?;
        scheduler.add_allowed_timeslots(exam_id, &allowed_timeslots);

        let disallowed_timeslots = build_disallowed_timeslots(&mut txn, exam_id).await?;
        scheduler.add_disallowed_timeslots(exam_id, &disallowed_timeslots);
    }

    let multi_slot_exams = query!(
        "
        SELECT
            id AS 'id: ExamId',
            slots_required AS 'slots_required: i64'
        FROM exams
        WHERE slots_required > 1
        "
    )
    .fetch_all(&mut *txn)
    .await?;

    let ordered_timeslots = timeslot_index.ordered.clone();
    for exam in &multi_slot_exams {
        let slots_required = exam.slots_required as usize;
        let windows = build_consecutive_windows(&ordered_timeslots, slots_required);
        let window_refs = windows
            .iter()
            .map(|window| window.as_slice())
            .collect::<Vec<_>>();

        scheduler.add_multi_slot_exam_constraint(exam.id, &window_refs);
    }

    let exam_cohorts = build_exam_cohorts(&exams_per_student);

    for exam in &multi_slot_exams {
        let block_exam = exam.id;
        let slots_required = exam.slots_required as u32;

        for exams in &exam_cohorts {
            if !exams.contains(&block_exam) {
                continue;
            }

            for &other_exam in exams {
                if other_exam == block_exam {
                    continue;
                }

                scheduler.prevent_block_overlap(
                    block_exam,
                    other_exam,
                    slots_required,
                    &timeslot_index.positions,
                );
            }
        }
    }

    // distance maxing
    let subject_exam_groups = query!(
        "
        SELECT
            subject_id AS 'subject_id: SubjectId',
            id AS 'exam_id: ExamId'
        FROM exams
        "
    )
    .fetch_all(&mut *txn)
    .await?
    .into_iter()
    .into_grouping_map_by(|row| row.subject_id)
    .fold(Vec::new(), |mut acc, _subject, row| {
        acc.push(row.exam_id);
        acc
    });

    let days = group_days(&mut txn)
        .await?
        .into_values()
        .flat_map(|timeslots| timeslots.into_iter().tuple_combinations())
        .collect::<Box<_>>();

    for (exam1, exam2) in subject_exam_groups
        .values()
        .flat_map(|exams| exams.iter().tuple_combinations())
    {
        scheduler.maximize_exam_distance(*exam1, *exam2);
    }

    // Exams must happen on the same day
    while let Some(Ok((exam1, exam2))) = query!(
        "
        SELECT
            exam1_id AS 'exam1_id: ExamId',
            exam2_id AS 'exam2_id: ExamId'
        FROM same_day_exams
        "
    )
    .fetch(&mut *txn)
    .map_ok(|row| (row.exam1_id, row.exam2_id))
    .next()
    .await
    {
        scheduler.add_pair_constraint(exam1, exam2, &days);
    }

    let week_groups = sqlx::query!(
        "
        SELECT
            strftime('%W', date) AS 'week: i64',
            id AS 'id: TimeslotId'
        FROM timeslots
        "
    )
    .fetch_all(&mut *txn)
    .await?
    .into_iter()
    .map(|row| (row.id, row.week.expect("Invalid week number")))
    .collect::<HashMap<_, _>>();

    // Exam 1 and 2 must be in different weeks
    while let Some(Ok((exam1, exam2))) = query!(
        "
        SELECT
            exam1_id AS 'exam1_id: ExamId',
            exam2_id AS 'exam2_id: ExamId'
        FROM different_week_exams
        "
    )
    .fetch(&mut *txn)
    .map_ok(|row| (row.exam1_id, row.exam2_id))
    .next()
    .await
    {
        scheduler.separate_exam_groups(exam1, exam2, &week_groups);
    }

    let results = scheduler.solve()?;

    Ok(results)
}

struct TimeslotRow {
    id: TimeslotId,
    date: Date,
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

/// Chronological timeslot ordering plus a symbolic position map.
///
/// `ordered` is used to build consecutive windows, while `positions` is used
/// to build Z3 expressions that reason about "between" relationships without
/// relying on TimeslotId ordering.
/// AI-generated (GPT-5.2-codex).
struct TimeslotIndex {
    ordered: Vec<TimeslotId>,
    positions: HashMap<TimeslotId, i64>,
}

/// Build a chronological index for timeslots.
///
/// The returned positions map uses ordered list positions, not TimeslotId values.
/// AI-generated (GPT-5.2-codex).
async fn build_timeslot_index(
    txn: &mut Transaction<'_, Sqlite>,
) -> Result<TimeslotIndex, SolveError> {
    let timeslots = fetch_timeslots(txn).await?;
    let ordered: Vec<_> = timeslots.iter().map(|ts| ts.id).collect();
    let positions = ordered
        .iter()
        .enumerate()
        .map(|(idx, &id)| (id, idx as i64))
        .collect();

    Ok(TimeslotIndex { ordered, positions })
}

/// Build consecutive timeslot windows of a fixed size.
///
/// Windows are built using ordered list positions, so TimeslotId ordering is ignored.
/// AI-generated (GPT-5.2-codex).
fn build_consecutive_windows(
    ordered_timeslots: &[TimeslotId],
    slots_required: usize,
) -> Vec<Vec<TimeslotId>> {
    if slots_required == 0 || ordered_timeslots.len() < slots_required {
        return Vec::new();
    }

    let mut windows = Vec::new();
    for start in 0..=ordered_timeslots.len() - slots_required {
        windows.push(ordered_timeslots[start..start + slots_required].to_vec());
    }

    windows
}

/// Build unique exam cohorts from per-student exam lists.
///
/// Cohorts are normalized via sorting and deduplication to avoid redundant constraints.
/// AI-generated (GPT-5.2-codex).
fn build_exam_cohorts(exams_per_student: &HashMap<StudentId, Vec<ExamId>>) -> Vec<Vec<ExamId>> {
    let mut cohorts: HashMap<Vec<ExamId>, ()> = HashMap::new();

    for exams in exams_per_student.values() {
        let mut normalized = exams.clone();
        normalized.sort_unstable();
        normalized.dedup();

        cohorts.entry(normalized).or_insert(());
    }

    cohorts.into_keys().collect()
}
