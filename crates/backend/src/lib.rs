use std::collections::HashMap;

use entity::id::{SessionId, TimeslotId};
use solver::{ExamScheduler, SolverError};
mod scheduler_builder;
mod solver_adapter;

use crate::scheduler_builder::SchedulerBuilder;
use crate::solver_adapter::SolverAdapter;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, PaginatorTrait};

#[derive(Debug, thiserror::Error)]
pub enum SolveError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] DbErr),
    #[error("Solver error: {0}")]
    SolverError(#[from] SolverError),
}

pub async fn solve(
    db: &DatabaseConnection,
) -> Result<HashMap<SessionId, TimeslotId>, SolveError> {
    let mappings = SolverAdapter::new(db).await?;
    let n_timeslots = entity::entity::timeslots::Entity::find()
        .count(db)
        .await?;

    let mut scheduler = ExamScheduler::new(
        mappings.session_ids(),
        n_timeslots
            .try_into()
            .map_err(|_| DbErr::Custom("timeslot count overflow".to_string()))?,
    );

    // Retrieve every exam that each student is enrolled in.
    // A student may be enrolled in multiple subjects, and each subject can have multiple exams.
    let mut builder = SchedulerBuilder::new(&mappings, &mut scheduler);
    builder.apply_student_clashes_from_db(db).await?;
    builder
        .apply_timeslot_restrictions_for_exams_from_db(db)
        .await?;
    builder
        .apply_subject_exam_distance_from_db(db)
        .await?;
    builder.apply_same_day_constraints_from_db(db).await?;
    builder.apply_week_separation_from_db(db).await?;

    let results = scheduler.solve()?;
    let scheduled = mappings.map_solution(results);

    Ok(scheduled)
}
