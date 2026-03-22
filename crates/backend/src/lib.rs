use diesel::{QueryDsl, RunQueryDsl, SqliteConnection};
use std::collections::HashMap;

use entity::id::{SessionId, TimeslotId};
use entity::schema::timeslot;
use solver::{ExamScheduler, SolverError};
mod scheduler_builder;
mod solver_adapter;

use crate::scheduler_builder::SchedulerBuilder;
use crate::solver_adapter::SolverAdapter;

#[derive(Debug, thiserror::Error)]
pub enum SolveError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Solver error: {0}")]
    SolverError(#[from] SolverError),
}

pub fn solve(
    db: &mut SqliteConnection,
) -> Result<impl Iterator<Item = HashMap<SessionId, TimeslotId>>, SolveError> {
    let mappings = SolverAdapter::new(db)?;
    let n_timeslots: i64 = timeslot::table.count().get_result(db)?;

    let mut scheduler = ExamScheduler::new(
        mappings.session_ids().iter().copied(),
        n_timeslots.try_into().unwrap(),
    );

    // Retrieve every exam that each student is enrolled in.
    // A student may be enrolled in multiple subjects, and each subject can have multiple exams.
    let mut builder = SchedulerBuilder::new(&mappings, &mut scheduler);
    builder.apply_student_clashes_from_db(db)?;
    builder.apply_timeslot_restrictions_for_exams_from_db(db)?;
    builder.apply_subject_exam_distance_from_db(db)?;
    builder.apply_same_day_constraints_from_db(db)?;
    builder.apply_week_separation_from_db(db)?;

    let solutions = scheduler.solve()?;
    let scheduled = solutions.map(move |solution| mappings.map_solution(solution));

    Ok(scheduled)
}
