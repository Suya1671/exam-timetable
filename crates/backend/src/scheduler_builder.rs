use diesel::{
    alias, connection::DefaultLoadingMode, BoolExpressionMethods, ExpressionMethods, JoinOnDsl,
    NullableExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection,
};
use entity::{
    exam_constraints::ExamConstraintType,
    exams::TimeslotRestrictionMode,
    id::{SessionId, StudentId, SubjectId, TimeslotId},
    schema::{
        enrolled_student, exam, exam_constraint, exam_timeslot_restriction, session, student,
        timeslot,
    },
};
use itertools::Itertools;
use solver::{ExamScheduler, TimeslotIndex as SolverTimeslotIndex};
use std::collections::{HashMap, HashSet};

use crate::{solver_adapter::SolverAdapter, SolveError};

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

    /// Load and apply student clash constraints from the database.
    /// Groups students with identical session sets to deduplicate identical constraints.
    pub fn apply_student_clashes_from_db(
        &mut self,
        db: &mut SqliteConnection,
    ) -> Result<(), SolveError> {
        let rows = student::table
            .inner_join(enrolled_student::table)
            .inner_join(
                exam::table.on(exam::subject_id
                    .eq(enrolled_student::subject_id)
                    .and(exam::grade.eq(student::grade))),
            )
            .inner_join(session::table.on(session::exam_id.eq(exam::id)))
            .select((student::id, session::id))
            .distinct()
            .load_iter::<(StudentId, SessionId), DefaultLoadingMode>(db)?;

        let mut student_sessions: HashMap<StudentId, Vec<SessionId>> = HashMap::new();

        for row in rows {
            let (student_id, session_id) = row?;
            student_sessions
                .entry(student_id)
                .or_default()
                .push(session_id);
        }

        let mut session_to_students: HashMap<Vec<SessionId>, Vec<StudentId>> = HashMap::new();

        for (student, mut sessions) in student_sessions {
            sessions.sort();
            sessions.dedup();
            session_to_students
                .entry(sessions)
                .or_default()
                .push(student);
        }

        for (sessions, students) in session_to_students {
            self.scheduler.setup_students(students, sessions);
        }

        Ok(())
    }

    /// Load and apply allowed/disallowed timeslot constraints per exam.
    /// Only applies to first session (sequence=0) of each exam to avoid duplicates.
    pub fn apply_timeslot_restrictions_for_exams_from_db(
        &mut self,
        db: &mut SqliteConnection,
    ) -> Result<(), SolveError> {
        let rows = exam::table
            .inner_join(
                session::table.on(session::exam_id.eq(exam::id).and(session::sequence.eq(0))),
            )
            .left_join(
                exam_timeslot_restriction::table
                    .on(exam_timeslot_restriction::exam_id.eq(exam::id)),
            )
            .select((
                session::id,
                exam::timeslot_restriction_mode,
                exam_timeslot_restriction::timeslot_id.nullable(),
            ))
            .load_iter::<(
                SessionId,
                Option<TimeslotRestrictionMode>,
                Option<TimeslotId>,
            ), _>(db)?;

        let mut restrictions: HashMap<
            (SessionId, Option<TimeslotRestrictionMode>),
            HashSet<TimeslotId>,
        > = HashMap::new();

        for row in rows {
            let (session_id, mode, timeslot_id) = row?;
            let entry = restrictions.entry((session_id, mode)).or_default();

            if let Some(timeslot_id) = timeslot_id {
                entry.insert(timeslot_id);
            }
        }

        for ((session_id, mode), selected_timeslots) in restrictions {
            match mode {
                Some(TimeslotRestrictionMode::Allow) => self.mappings.apply_allowed_timeslots(
                    self.scheduler,
                    session_id,
                    selected_timeslots.iter().copied(),
                ),
                Some(TimeslotRestrictionMode::Deny) => self.mappings.apply_disallowed_timeslots(
                    self.scheduler,
                    session_id,
                    selected_timeslots.iter().copied(),
                ),
                None => {}
            }
        }
        Ok(())
    }

    /// Load and apply distance preferences between exams of the same subject.
    pub fn apply_subject_exam_distance_from_db(
        &self,
        db: &mut SqliteConnection,
    ) -> Result<(), SolveError> {
        let rows = exam::table
            .inner_join(
                session::table.on(session::exam_id.eq(exam::id).and(session::sequence.eq(0))),
            )
            .select((session::id, exam::subject_id))
            .load_iter::<(SessionId, SubjectId), _>(db)?;

        let mut subject_session_groups: HashMap<_, Vec<_>> = HashMap::new();

        for row in rows {
            let (session_id, subject_id) = row?;
            subject_session_groups
                .entry(subject_id)
                .or_default()
                .push(session_id);
        }

        for (session1, session2) in subject_session_groups
            .into_values()
            .flat_map(|sessions| sessions.into_iter().tuple_combinations())
        {
            self.scheduler.maximize_exam_distance(session1, session2);
        }

        Ok(())
    }

    /// Load and apply constraints between exam pairs.
    pub fn apply_exam_constraints_from_db(
        &mut self,
        db: &mut SqliteConnection,
    ) -> Result<(), SolveError> {
        // This can probably be simplified down into 1 sql call, but I am too lazy to do that refactor rn
        let day_groups = group_days(db)?.into_values();
        let days = self.mappings.combination_pairs(day_groups);

        let week_groups = group_weeks(db)?.into_values();
        let weeks = self.mappings.combination_pairs(week_groups);

        let morning_timeslots: Vec<_> = timeslot::table
            .filter(timeslot::slot.eq(0))
            .select(timeslot::id)
            .load_iter::<TimeslotId, DefaultLoadingMode>(db)?
            .map_ok(|slot_id| self.mappings.timeslot_index_for_id(slot_id))
            .try_collect()?;

        let week_entries: Vec<_> = timeslot::table
            .select((timeslot::id, timeslot::date))
            .load_iter::<(TimeslotId, time::Date), DefaultLoadingMode>(db)?
            .map_ok(|(id, date)| (id, date.iso_week().into()))
            .try_collect()?;
        let week_map = self.mappings.group_map(week_entries);

        let day_entries: Vec<_> = timeslot::table
            .select((timeslot::id, timeslot::date))
            .load_iter::<(TimeslotId, time::Date), DefaultLoadingMode>(db)?
            .map_ok(|(id, date): (TimeslotId, time::Date)| (id, date.ordinal().into()))
            .try_collect()?;
        let day_map = self.mappings.group_map(day_entries);

        let (first_session, second_session) =
            alias!(session as first_session, session as second_session);

        let (first_id, first_exam_id, first_sequence) =
            first_session.fields((session::id, session::exam_id, session::sequence));
        let (second_id, second_exam_id, second_sequence) =
            second_session.fields((session::id, session::exam_id, session::sequence));

        let constraint_rows = exam_constraint::table
            .inner_join(
                first_session.on(first_exam_id
                    .eq(exam_constraint::exam1_id)
                    .and(first_sequence.eq(0))),
            )
            .inner_join(
                second_session.on(second_exam_id
                    .eq(exam_constraint::exam2_id)
                    .and(second_sequence.eq(0))),
            )
            .select((first_id, second_id, exam_constraint::constraint_type))
            .load_iter::<(SessionId, SessionId, ExamConstraintType), _>(db)?;

        for row in constraint_rows {
            let (first_session_id, second_session_id, constraint_type) = row?;

            match constraint_type {
                ExamConstraintType::SameTime => {
                    self.scheduler
                        .require_same_time(first_session_id, second_session_id);
                }
                ExamConstraintType::SameDay => {
                    self.scheduler
                        .add_allowed_timeslots(first_session_id, morning_timeslots.clone());

                    self.scheduler.add_pair_constraint(
                        first_session_id,
                        second_session_id,
                        days.clone(),
                    );
                }
                ExamConstraintType::SameWeek => {
                    self.scheduler.add_pair_constraint(
                        first_session_id,
                        second_session_id,
                        weeks.clone(),
                    );
                }
                ExamConstraintType::DifferentTime => {
                    self.scheduler
                        .require_different_time(first_session_id, second_session_id);
                }
                ExamConstraintType::DifferentDay => {
                    self.scheduler.separate_exam_groups(
                        first_session_id,
                        second_session_id,
                        day_map.clone(),
                    );
                }
                ExamConstraintType::DifferentWeek => {
                    self.scheduler.separate_exam_groups(
                        first_session_id,
                        second_session_id,
                        week_map.clone(),
                    );
                }
            }
        }

        Ok(())
    }

    /// Add soft constraint to minimize exams per day for each student.
    pub fn apply_minimize_exams_per_day(
        &self,
        db: &mut SqliteConnection,
    ) -> Result<(), SolveError> {
        let day_groups = group_days(db)?;
        let days: Vec<Vec<SolverTimeslotIndex>> = day_groups
            .into_values()
            .map(|timeslots| {
                timeslots
                    .iter()
                    .map(|&id| self.mappings.timeslot_index_for_id(id))
                    .collect()
            })
            .collect();

        if days.is_empty() {
            return Ok(());
        }

        let days_ref: Vec<&[SolverTimeslotIndex]> = days.iter().map(|v| v.as_slice()).collect();

        let rows = student::table
            .inner_join(enrolled_student::table)
            .inner_join(
                exam::table.on(exam::subject_id
                    .eq(enrolled_student::subject_id)
                    .and(exam::grade.eq(student::grade))),
            )
            .inner_join(session::table.on(session::exam_id.eq(exam::id)))
            .select((student::id, session::id))
            .load_iter::<(StudentId, SessionId), DefaultLoadingMode>(db)?;

        let mut student_sessions: HashMap<StudentId, Vec<SessionId>> = HashMap::new();
        for row in rows {
            let (student_id, session_id) = row?;
            student_sessions
                .entry(student_id)
                .or_default()
                .push(session_id);
        }

        if !student_sessions.is_empty() {
            self.scheduler
                .minimize_exams_per_day(&student_sessions, &days_ref, |_| 10);
        }

        Ok(())
    }
}

/// Groups timeslots by the calendar date.
fn group_days(
    db: &mut SqliteConnection,
) -> Result<HashMap<time::Date, Vec<TimeslotId>>, SolveError> {
    let rows = timeslot::table
        .order((timeslot::date.asc(), timeslot::slot.asc()))
        .select((timeslot::id, timeslot::date))
        .load_iter::<(TimeslotId, time::Date), _>(db)?;

    let mut groups: HashMap<_, Vec<_>> = HashMap::new();

    for row in rows {
        let (id, date) = row?;
        groups.entry(date).or_default().push(id);
    }

    Ok(groups)
}

/// Groups timeslots by the calendar week.
pub fn group_weeks(db: &mut SqliteConnection) -> Result<HashMap<u8, Vec<TimeslotId>>, SolveError> {
    let rows = timeslot::table
        .order((timeslot::date.asc(), timeslot::slot.asc()))
        .select((timeslot::id, timeslot::date))
        .load_iter::<(TimeslotId, time::Date), _>(db)?;

    let mut groups: HashMap<_, Vec<_>> = HashMap::new();

    for row in rows {
        let (id, date): (TimeslotId, time::Date) = row?;
        groups.entry(date.iso_week()).or_default().push(id);
    }

    Ok(groups)
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
