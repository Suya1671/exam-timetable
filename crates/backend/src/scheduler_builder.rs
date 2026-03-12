use std::collections::HashMap;

use entity::id::{ExamId, SessionId, StudentId, TimeslotId};
use itertools::Itertools;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder};
use solver::ExamScheduler;

use crate::solver_adapter::SolverAdapter;
use crate::SolveError;
use entity::entity::timeslots;
use entity::id::TimeslotSlot;

/// Backend-facing scheduler builder.
pub struct SchedulerBuilder<'a> {
    mappings: &'a SolverAdapter,
    scheduler: &'a mut ExamScheduler,
}

impl<'a> SchedulerBuilder<'a> {
    /// Build a scheduler builder with adapter context.
    pub fn new(mappings: &'a SolverAdapter, scheduler: &'a mut ExamScheduler) -> Self {
        Self {
            mappings,
            scheduler,
        }
    }

    /// Apply student clash constraints from session lists.
    pub fn apply_student_clashes(
        &mut self,
        sessions_per_student: &HashMap<StudentId, Vec<SessionId>>,
    ) {
        self.scheduler.setup_students(sessions_per_student);
    }

    /// Load and apply student clash constraints from the database.
    /// AI-generated (GPT-5.2-codex).
    pub async fn apply_student_clashes_from_db(
        &mut self,
        db: &DatabaseConnection,
    ) -> Result<(), SolveError> {
        let sessions = entity::entity::sessions::Entity::find().all(db).await?;
        let exams = entity::entity::exams::Entity::find().all(db).await?;
        let enrollments = entity::entity::enrolled_students::Entity::find()
            .all(db)
            .await?;

        let subject_by_exam = exams
            .into_iter()
            .map(|exam| (exam.id, exam.subject_id))
            .collect::<HashMap<_, _>>();

        let mut sessions_per_student: HashMap<StudentId, Vec<SessionId>> = HashMap::new();
        for session in sessions {
            if let Some(subject_id) = subject_by_exam.get(&session.exam_id) {
                for enrollment in enrollments
                    .iter()
                    .filter(|enrollment| enrollment.subject_id == *subject_id)
                {
                    sessions_per_student
                        .entry(enrollment.student_id)
                        .or_default()
                        .push(session.id);
                }
            }
        }

        self.apply_student_clashes(&sessions_per_student);
        Ok(())
    }

    /// Load and apply allowed/disallowed timeslot constraints per exam.
    /// AI-generated (GPT-5.2-codex).
    pub async fn apply_timeslot_restrictions_for_exams_from_db(
        &mut self,
        db: &DatabaseConnection,
    ) -> Result<(), SolveError> {
        let session_ids = entity::entity::sessions::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(|session| session.id)
            .collect::<Vec<_>>();

        for session_id in session_ids {
            let allowed_timeslots = build_allowed_timeslots_for_session(db, session_id).await?;
            let disallowed_timeslots =
                build_disallowed_timeslots_for_session(db, session_id).await?;
            self.mappings.apply_timeslot_restrictions(
                self.scheduler,
                session_id,
                &allowed_timeslots,
                &disallowed_timeslots,
            );
        }
        Ok(())
    }

    /// Load and apply distance preferences between exams of the same subject.
    /// AI-generated (GPT-5.2-codex).
    pub async fn apply_subject_exam_distance_from_db(
        &self,
        db: &DatabaseConnection,
    ) -> Result<(), SolveError> {
        let subject_exam_groups = entity::entity::exams::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .into_grouping_map_by(|exam| exam.subject_id)
            .fold(Vec::new(), |mut acc, _subject, exam| {
                acc.push(exam.id);
                acc
            });

        for (exam1, exam2) in subject_exam_groups
            .values()
            .flat_map(|exams| exams.iter().tuple_combinations())
        {
            apply_distance_preference_from_db(self.scheduler, db, *exam1, *exam2).await?;
        }
        Ok(())
    }

    /// Load and apply same-day constraints between exam pairs.
    /// AI-generated (GPT-5.2-codex).
    pub async fn apply_same_day_constraints_from_db(
        &mut self,
        db: &DatabaseConnection,
    ) -> Result<(), SolveError> {
        let day_groups = group_days(db).await?.into_values();
        let days = self.mappings.day_pairs(day_groups);
        let morning_slots = morning_slots(db).await?;

        let same_day_pairs = entity::entity::same_day_exams::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(|row| (row.first_slot_exam_id, row.second_slot_exam_id))
            .collect::<Vec<_>>();

        for (first_exam, second_exam) in same_day_pairs {
            apply_same_day_constraints_from_db(
                self.scheduler,
                db,
                first_exam,
                second_exam,
                &morning_slots,
                &days,
            )
            .await?;
        }
        Ok(())
    }

    /// Load and apply week separation constraints between exam pairs.
    /// AI-generated (GPT-5.2-codex).
    pub async fn apply_week_separation_from_db(
        &self,
        db: &DatabaseConnection,
    ) -> Result<(), SolveError> {
        let week_entries = timeslots::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(|row| (row.id, row.date.iso_week()))
            .collect::<Vec<_>>();
        let week_map = self.mappings.week_map(week_entries);

        let week_pairs = entity::entity::different_week_exams::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(|row| (row.exam1_id, row.exam2_id))
            .collect::<Vec<_>>();

        for (exam1, exam2) in week_pairs {
            apply_week_separation_from_db(self.scheduler, db, exam1, exam2, &week_map).await?;
        }
        Ok(())
    }
}

async fn build_allowed_timeslots_for_session(
    db: &DatabaseConnection,
    session_id: SessionId,
) -> Result<Vec<TimeslotId>, SolveError> {
    let exam_id = entity::entity::sessions::Entity::find_by_id(session_id)
        .one(db)
        .await?
        .map(|session| session.exam_id)
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("session missing".to_string()))?;

    build_allowed_timeslots(db, exam_id).await
}

async fn build_disallowed_timeslots_for_session(
    db: &DatabaseConnection,
    session_id: SessionId,
) -> Result<Vec<TimeslotId>, SolveError> {
    let exam_id = entity::entity::sessions::Entity::find_by_id(session_id)
        .one(db)
        .await?
        .map(|session| session.exam_id)
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("session missing".to_string()))?;

    build_disallowed_timeslots(db, exam_id).await
}

async fn apply_distance_preference_from_db(
    scheduler: &ExamScheduler,
    db: &DatabaseConnection,
    exam1: ExamId,
    exam2: ExamId,
) -> Result<(), SolveError> {
    let session1 = entity::entity::sessions::Entity::find()
        .filter(entity::entity::sessions::Column::ExamId.eq(exam1))
        .filter(entity::entity::sessions::Column::Sequence.eq(0))
        .one(db)
        .await?
        .map(|session| session.id)
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("session missing".to_string()))?;
    let session2 = entity::entity::sessions::Entity::find()
        .filter(entity::entity::sessions::Column::ExamId.eq(exam2))
        .filter(entity::entity::sessions::Column::Sequence.eq(0))
        .one(db)
        .await?
        .map(|session| session.id)
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("session missing".to_string()))?;

    scheduler.maximize_exam_distance(session1, session2);
    Ok(())
}

async fn apply_same_day_constraints_from_db(
    scheduler: &mut ExamScheduler,
    db: &DatabaseConnection,
    first_exam: ExamId,
    second_exam: ExamId,
    _morning_slots: &[TimeslotId],
    days: &[(solver::TimeslotIndex, solver::TimeslotIndex)],
) -> Result<(), SolveError> {
    let first_session = entity::entity::sessions::Entity::find()
        .filter(entity::entity::sessions::Column::ExamId.eq(first_exam))
        .filter(entity::entity::sessions::Column::Sequence.eq(0))
        .one(db)
        .await?
        .map(|session| session.id)
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("session missing".to_string()))?;
    let second_session = entity::entity::sessions::Entity::find()
        .filter(entity::entity::sessions::Column::ExamId.eq(second_exam))
        .filter(entity::entity::sessions::Column::Sequence.eq(0))
        .one(db)
        .await?
        .map(|session| session.id)
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("session missing".to_string()))?;

    let morning_indices = timeslots::Entity::find()
        .filter(timeslots::Column::Slot.eq(TimeslotSlot::First))
        .order_by_asc(timeslots::Column::Date)
        .order_by_asc(timeslots::Column::Slot)
        .all(db)
        .await?
        .into_iter()
        .enumerate()
        .map(|(idx, _)| solver::TimeslotIndex(idx as i64))
        .collect::<Vec<_>>();

    scheduler.add_allowed_timeslots(first_session, &morning_indices);
    scheduler.add_pair_constraint(first_session, second_session, days);
    Ok(())
}

async fn apply_week_separation_from_db(
    scheduler: &ExamScheduler,
    db: &DatabaseConnection,
    exam1: ExamId,
    exam2: ExamId,
    week_map: &HashMap<solver::TimeslotIndex, i64>,
) -> Result<(), SolveError> {
    let session1 = entity::entity::sessions::Entity::find()
        .filter(entity::entity::sessions::Column::ExamId.eq(exam1))
        .filter(entity::entity::sessions::Column::Sequence.eq(0))
        .one(db)
        .await?
        .map(|session| session.id)
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("session missing".to_string()))?;
    let session2 = entity::entity::sessions::Entity::find()
        .filter(entity::entity::sessions::Column::ExamId.eq(exam2))
        .filter(entity::entity::sessions::Column::Sequence.eq(0))
        .one(db)
        .await?
        .map(|session| session.id)
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("session missing".to_string()))?;

    scheduler.separate_exam_groups(session1, session2, week_map);
    Ok(())
}

async fn morning_slots(db: &DatabaseConnection) -> Result<Vec<TimeslotId>, SolveError> {
    let results = timeslots::Entity::find()
        .filter(timeslots::Column::Slot.eq(TimeslotSlot::First))
        .all(db)
        .await?
        .into_iter()
        .map(|row| row.id)
        .collect();

    Ok(results)
}

async fn build_allowed_timeslots(
    db: &DatabaseConnection,
    exam: ExamId,
) -> Result<Vec<TimeslotId>, SolveError> {
    let mut results = Vec::new();
    let direct = entity::entity::exam_allowed_timeslots::Entity::find()
        .filter(entity::entity::exam_allowed_timeslots::Column::ExamId.eq(exam))
        .all(db)
        .await?
        .into_iter()
        .map(|row| row.timeslot_id);
    results.extend(direct);

    let subject_id = entity::entity::exams::Entity::find_by_id(exam)
        .one(db)
        .await?
        .map(|row| row.subject_id)
        .ok_or_else(|| DbErr::RecordNotFound("exam missing".to_string()))?;

    let student_ids = entity::entity::enrolled_students::Entity::find()
        .filter(entity::entity::enrolled_students::Column::SubjectId.eq(subject_id))
        .all(db)
        .await?
        .into_iter()
        .map(|row| row.student_id)
        .collect::<Vec<_>>();
    let student = entity::entity::student_allowed_timeslots::Entity::find()
        .filter(entity::entity::student_allowed_timeslots::Column::StudentId.is_in(student_ids))
        .all(db)
        .await?
        .into_iter()
        .map(|row| row.timeslot_id);
    results.extend(student);

    let subject = entity::entity::subject_allowed_timeslots::Entity::find()
        .filter(entity::entity::subject_allowed_timeslots::Column::SubjectId.eq(subject_id))
        .all(db)
        .await?
        .into_iter()
        .map(|row| row.timeslot_id);
    results.extend(subject);

    Ok(results.into_iter().unique().collect())
}

async fn build_disallowed_timeslots(
    db: &DatabaseConnection,
    exam: ExamId,
) -> Result<Vec<TimeslotId>, SolveError> {
    let mut results = Vec::new();
    let direct = entity::entity::exam_denied_timeslots::Entity::find()
        .filter(entity::entity::exam_denied_timeslots::Column::ExamId.eq(exam))
        .all(db)
        .await?
        .into_iter()
        .map(|row| row.timeslot_id);
    results.extend(direct);

    let subject_id = entity::entity::exams::Entity::find_by_id(exam)
        .one(db)
        .await?
        .map(|row| row.subject_id)
        .ok_or_else(|| DbErr::RecordNotFound("exam missing".to_string()))?;

    let student_ids = entity::entity::enrolled_students::Entity::find()
        .filter(entity::entity::enrolled_students::Column::SubjectId.eq(subject_id))
        .all(db)
        .await?
        .into_iter()
        .map(|row| row.student_id)
        .collect::<Vec<_>>();
    let student = entity::entity::student_denied_timeslots::Entity::find()
        .filter(entity::entity::student_denied_timeslots::Column::StudentId.is_in(student_ids))
        .all(db)
        .await?
        .into_iter()
        .map(|row| row.timeslot_id);
    results.extend(student);

    let subject = entity::entity::subject_denied_timeslots::Entity::find()
        .filter(entity::entity::subject_denied_timeslots::Column::SubjectId.eq(subject_id))
        .all(db)
        .await?
        .into_iter()
        .map(|row| row.timeslot_id);
    results.extend(subject);

    Ok(results.into_iter().unique().collect())
}

/// Groups timeslots by the calendar date.
async fn group_days(
    db: &DatabaseConnection,
) -> Result<HashMap<time::Date, Vec<TimeslotId>>, SolveError> {
    let rows = timeslots::Entity::find()
        .order_by_asc(timeslots::Column::Date)
        .order_by_asc(timeslots::Column::Slot)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.date, row.id))
        .into_grouping_map_by(|(date, _)| *date)
        .fold(Vec::new(), |mut acc, _date, (_parsed_date, id)| {
            acc.push(id);
            acc
        }))
}

// TODO: move as integration test/final solver test
// #[cfg(test)]
// mod tests {
//     use crate::solver_adapter::{SolverExam, TimeslotIndex};

//     use super::*;

//     #[test]
//     fn apply_timeslot_restrictions_delegates_to_adapter() {
//         let exams = vec![SolverExam {
//             id: ExamId(1),
//             sessions: 1,
//         }];

//         let mappings = SolverAdapter::new(
//             &exams,
//             TimeslotIndex::new(
//                 vec![TimeslotId(10), TimeslotId(20)],
//                 HashMap::from([(TimeslotId(10), 0), (TimeslotId(20), 1)]),
//             ),
//         );
//         let mut scheduler = ExamScheduler::new(mappings.session_ids(), 2);
//         let mut builder = SchedulerBuilder::new(&mappings, &mut scheduler);

//         builder.apply_timeslot_restrictions(ExamId(1), &[TimeslotId(10)], &[]);

//         let results = scheduler.solve().expect("expected sat schedule");
//         let mapped = mappings.map_solution(results);
//         assert_eq!(mapped.get(&ExamId(1)), Some(&TimeslotId(10)));
//     }
// }
