use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    SqliteConnection,
};
use std::collections::{HashMap, HashSet};

use entity::exams::TimeslotRestrictionMode;
use entity::id::{ExamId, SessionId, StudentId, SubjectId, TimeslotId};
use entity::schema::{
    different_week_exams, enrolled_student, exam, exam_timeslot_restriction, same_day_exam,
    same_time_exam, session, student, subject, timeslot,
};
use solver::{ExamScheduler, SolverError};
mod scheduler_builder;
mod solver_adapter;

use crate::scheduler_builder::SchedulerBuilder;
use crate::solver_adapter::SolverAdapter;

/// AI-generated (GPT-5.3-codex).
type ExamRestrictionMap = HashMap<ExamId, (Option<TimeslotRestrictionMode>, HashSet<TimeslotId>)>;

#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum SolveError {
    #[error("Database error: {0}")]
    DatabaseError(
        #[from]
        #[serde(serialize_with = "serialize_display")]
        #[specta(type = String)]
        diesel::result::Error,
    ),
    #[error("Solver error: {0}")]
    SolverError(#[from] SolverError),
    /// AI-generated (GPT-5.3-codex).
    #[error("Feasibility precheck failed: {reason}")]
    PrecheckFailed { reason: String },
}

fn serialize_display<T, S>(value: &T, s: S) -> Result<S::Ok, S::Error>
where
    T: std::fmt::Display,
    S: serde::Serializer,
{
    s.serialize_str(&value.to_string())
}

/// AI-generated (GPT-5.3-codex).
fn run_feasibility_precheck(db: &mut SqliteConnection, n_timeslots: u64) -> Result<(), SolveError> {
    if n_timeslots == 0 {
        return Err(SolveError::PrecheckFailed {
            reason: "No timeslots exist".to_string(),
        });
    }

    precheck_enrolled_subjects_have_sessions(db)?;

    let all_timeslots = load_all_timeslot_ids(db)?;
    let restrictions = load_exam_restrictions(db)?;

    precheck_exam_timeslot_domains(&all_timeslots, &restrictions)?;
    precheck_same_time_pairs(&all_timeslots, &restrictions, db)?;
    precheck_same_day_pairs(&all_timeslots, &restrictions, db)?;
    precheck_pair_constraint_contradictions(db)?;

    let rows = student::table
        .inner_join(enrolled_student::table)
        .inner_join(subject::table.on(enrolled_student::subject_id.eq(subject::id)))
        .inner_join(exam::table.on(exam::subject_id.eq(subject::id)))
        .inner_join(session::table.on(session::exam_id.eq(exam::id)))
        .select((student::id, student::name, session::id))
        .load::<(StudentId, String, SessionId)>(db)?;

    let mut sessions_by_student: HashMap<StudentId, (String, HashSet<SessionId>)> = HashMap::new();

    for (student_id, student_name, session_id) in rows {
        let (_, sessions) = sessions_by_student
            .entry(student_id)
            .or_insert_with(|| (student_name, HashSet::new()));
        sessions.insert(session_id);
    }

    if sessions_by_student.is_empty() {
        return Err(SolveError::PrecheckFailed {
            reason: "No student-session assignments found".to_string(),
        });
    }

    for (_, (student_name, sessions)) in sessions_by_student {
        let session_count = sessions.len() as u64;
        if session_count > n_timeslots {
            return Err(SolveError::PrecheckFailed {
                reason: format!(
                    "{student_name} has {session_count} sessions but only {n_timeslots} timeslots are available"
                ),
            });
        }
    }

    Ok(())
}

/// AI-generated (GPT-5.3-codex).
fn precheck_enrolled_subjects_have_sessions(db: &mut SqliteConnection) -> Result<(), SolveError> {
    let enrolled_subjects = enrolled_student::table
        .select(enrolled_student::subject_id)
        .distinct()
        .load::<SubjectId>(db)?;

    let subjects_with_sessions = exam::table
        .inner_join(session::table.on(session::exam_id.eq(exam::id)))
        .select(exam::subject_id)
        .distinct()
        .load::<SubjectId>(db)?;
    let session_subjects = subjects_with_sessions.into_iter().collect::<HashSet<_>>();

    for subject_id in enrolled_subjects {
        if !session_subjects.contains(&subject_id) {
            return Err(SolveError::PrecheckFailed {
                reason: format!("Subject {} has enrolments but no sessions", subject_id.0),
            });
        }
    }

    Ok(())
}

/// AI-generated (GPT-5.3-codex).
fn load_all_timeslot_ids(db: &mut SqliteConnection) -> Result<HashSet<TimeslotId>, SolveError> {
    let ids = timeslot::table
        .select(timeslot::id)
        .load::<TimeslotId>(db)?;
    Ok(ids.into_iter().collect())
}

/// AI-generated (GPT-5.3-codex).
fn load_exam_restrictions(db: &mut SqliteConnection) -> Result<ExamRestrictionMap, SolveError> {
    let rows = exam::table
        .left_join(
            exam_timeslot_restriction::table.on(exam_timeslot_restriction::exam_id.eq(exam::id)),
        )
        .select((
            exam::id,
            exam::timeslot_restriction_mode,
            exam_timeslot_restriction::timeslot_id.nullable(),
        ))
        .load::<(ExamId, Option<TimeslotRestrictionMode>, Option<TimeslotId>)>(db)?;

    let mut restrictions: ExamRestrictionMap = HashMap::new();

    for (exam_id, mode, timeslot_id) in rows {
        let entry = restrictions
            .entry(exam_id)
            .or_insert_with(|| (mode, HashSet::new()));
        if let Some(id) = timeslot_id {
            entry.1.insert(id);
        }
    }

    Ok(restrictions)
}

/// AI-generated (GPT-5.3-codex).
fn effective_exam_timeslots(
    all_timeslots: &HashSet<TimeslotId>,
    mode: Option<TimeslotRestrictionMode>,
    selected: &HashSet<TimeslotId>,
) -> HashSet<TimeslotId> {
    match mode {
        Some(TimeslotRestrictionMode::Allow) => selected.clone(),
        Some(TimeslotRestrictionMode::Deny) => all_timeslots
            .iter()
            .copied()
            .filter(|id| !selected.contains(id))
            .collect(),
        None => all_timeslots.clone(),
    }
}

/// AI-generated (GPT-5.3-codex).
fn precheck_exam_timeslot_domains(
    all_timeslots: &HashSet<TimeslotId>,
    restrictions: &ExamRestrictionMap,
) -> Result<(), SolveError> {
    for (&exam_id, &(mode, ref selected)) in restrictions {
        let feasible = effective_exam_timeslots(all_timeslots, mode, selected);
        if feasible.is_empty() {
            return Err(SolveError::PrecheckFailed {
                reason: format!("Exam {} has no feasible timeslots", exam_id.0),
            });
        }
    }

    Ok(())
}

/// AI-generated (GPT-5.3-codex).
fn precheck_same_time_pairs(
    all_timeslots: &HashSet<TimeslotId>,
    restrictions: &ExamRestrictionMap,
    db: &mut SqliteConnection,
) -> Result<(), SolveError> {
    let pairs = same_time_exam::table
        .select((same_time_exam::exam1_id, same_time_exam::exam2_id))
        .load::<(ExamId, ExamId)>(db)?;

    for (exam1_id, exam2_id) in pairs {
        let (mode1, selected1) =
            restrictions
                .get(&exam1_id)
                .ok_or_else(|| SolveError::PrecheckFailed {
                    reason: format!("Exam {} missing from restrictions", exam1_id.0),
                })?;
        let (mode2, selected2) =
            restrictions
                .get(&exam2_id)
                .ok_or_else(|| SolveError::PrecheckFailed {
                    reason: format!("Exam {} missing from restrictions", exam2_id.0),
                })?;

        let feasible1 = effective_exam_timeslots(all_timeslots, *mode1, selected1);
        let feasible2 = effective_exam_timeslots(all_timeslots, *mode2, selected2);

        let overlap_exists = feasible1.iter().any(|slot| feasible2.contains(slot));
        if !overlap_exists {
            return Err(SolveError::PrecheckFailed {
                reason: format!(
                    "Same-time pair ({}, {}) has no common feasible timeslot",
                    exam1_id.0, exam2_id.0
                ),
            });
        }
    }

    Ok(())
}

/// AI-generated (GPT-5.3-codex).
fn precheck_same_day_pairs(
    all_timeslots: &HashSet<TimeslotId>,
    restrictions: &ExamRestrictionMap,
    db: &mut SqliteConnection,
) -> Result<(), SolveError> {
    let timeslot_rows = timeslot::table
        .select((timeslot::id, timeslot::date, timeslot::slot))
        .load::<(TimeslotId, time::Date, i32)>(db)?;

    let mut by_date: HashMap<time::Date, (Vec<TimeslotId>, Vec<TimeslotId>)> = HashMap::new();
    for (id, date, slot) in timeslot_rows {
        let entry = by_date
            .entry(date)
            .or_insert_with(|| (Vec::new(), Vec::new()));
        if slot == 0 {
            entry.0.push(id);
        } else {
            entry.1.push(id);
        }
    }

    let pairs = same_day_exam::table
        .select((
            same_day_exam::first_slot_exam_id,
            same_day_exam::second_slot_exam_id,
        ))
        .load::<(ExamId, ExamId)>(db)?;

    for (first_exam_id, second_exam_id) in pairs {
        let (mode1, selected1) =
            restrictions
                .get(&first_exam_id)
                .ok_or_else(|| SolveError::PrecheckFailed {
                    reason: format!("Exam {} missing from restrictions", first_exam_id.0),
                })?;
        let (mode2, selected2) =
            restrictions
                .get(&second_exam_id)
                .ok_or_else(|| SolveError::PrecheckFailed {
                    reason: format!("Exam {} missing from restrictions", second_exam_id.0),
                })?;

        let feasible_first = effective_exam_timeslots(all_timeslots, *mode1, selected1);
        let feasible_second = effective_exam_timeslots(all_timeslots, *mode2, selected2);

        let day_pair_exists = by_date.values().any(|(morning, non_morning)| {
            morning.iter().any(|slot| feasible_first.contains(slot))
                && non_morning
                    .iter()
                    .any(|slot| feasible_second.contains(slot))
        });

        if !day_pair_exists {
            return Err(SolveError::PrecheckFailed {
                reason: format!(
                    "Same-day pair ({}, {}) has no feasible morning/day combination",
                    first_exam_id.0, second_exam_id.0
                ),
            });
        }
    }

    Ok(())
}

/// AI-generated (GPT-5.3-codex).
fn precheck_pair_constraint_contradictions(db: &mut SqliteConnection) -> Result<(), SolveError> {
    let same_time_pairs = same_time_exam::table
        .select((same_time_exam::exam1_id, same_time_exam::exam2_id))
        .load::<(ExamId, ExamId)>(db)?
        .into_iter()
        .map(normalize_exam_pair)
        .collect::<HashSet<_>>();

    let different_week_pairs = different_week_exams::table
        .select((
            different_week_exams::exam1_id,
            different_week_exams::exam2_id,
        ))
        .load::<(ExamId, ExamId)>(db)?
        .into_iter()
        .map(normalize_exam_pair)
        .collect::<HashSet<_>>();

    if let Some((exam1, exam2)) = same_time_pairs
        .intersection(&different_week_pairs)
        .next()
        .copied()
    {
        return Err(SolveError::PrecheckFailed {
            reason: format!(
                "Pair ({}, {}) cannot be both same-time and different-week",
                exam1.0, exam2.0
            ),
        });
    }

    Ok(())
}

/// AI-generated (GPT-5.3-codex).
fn normalize_exam_pair((exam1, exam2): (ExamId, ExamId)) -> (ExamId, ExamId) {
    if exam1 <= exam2 {
        (exam1, exam2)
    } else {
        (exam2, exam1)
    }
}

/// AI-generated (GPT-5.3-codex).
fn validate_locked_assignments(
    session_ids: &[SessionId],
    all_timeslots: &HashSet<TimeslotId>,
    locked_assignments: &[(SessionId, TimeslotId)],
) -> Result<(), SolveError> {
    let known_sessions = session_ids.iter().copied().collect::<HashSet<_>>();

    for (session_id, timeslot_id) in locked_assignments {
        if !known_sessions.contains(session_id) {
            return Err(SolveError::PrecheckFailed {
                reason: format!("Locked session {} does not exist", session_id.0),
            });
        }

        if !all_timeslots.contains(timeslot_id) {
            return Err(SolveError::PrecheckFailed {
                reason: format!("Locked timeslot {} does not exist", timeslot_id.0),
            });
        }
    }

    Ok(())
}

pub fn solve(
    db: &mut SqliteConnection,
) -> Result<impl Iterator<Item = HashMap<SessionId, TimeslotId>>, SolveError> {
    solve_with_locked_assignments(db, &[])
}

/// AI-generated (GPT-5.3-codex).
pub fn solve_with_locked_assignments(
    db: &mut SqliteConnection,
    locked_assignments: &[(SessionId, TimeslotId)],
) -> Result<impl Iterator<Item = HashMap<SessionId, TimeslotId>>, SolveError> {
    let mappings = SolverAdapter::new(db)?;
    let n_timeslots: i64 = timeslot::table.count().get_result(db)?;
    let n_timeslots_u64: u64 = n_timeslots.try_into().unwrap();
    let all_timeslots = load_all_timeslot_ids(db)?;

    run_feasibility_precheck(db, n_timeslots_u64)?;
    validate_locked_assignments(mappings.session_ids(), &all_timeslots, locked_assignments)?;

    let mut scheduler = ExamScheduler::new(mappings.session_ids().iter().copied(), n_timeslots_u64);

    // Retrieve every exam that each student is enrolled in.
    // A student may be enrolled in multiple subjects, and each subject can have multiple exams.
    let mut builder = SchedulerBuilder::new(&mappings, &mut scheduler);
    builder.apply_student_clashes_from_db(db)?;
    builder.apply_timeslot_restrictions_for_exams_from_db(db)?;
    builder.apply_subject_exam_distance_from_db(db)?;
    builder.apply_same_time_constraints_from_db(db)?;
    builder.apply_same_day_constraints_from_db(db)?;
    builder.apply_week_separation_from_db(db)?;

    for (session_id, timeslot_id) in locked_assignments {
        mappings.apply_allowed_timeslots(
            &mut scheduler,
            *session_id,
            std::iter::once(*timeslot_id),
        );
    }

    let solutions = scheduler.solve()?;
    let scheduled = solutions.map(move |solution| mappings.map_solution(solution));

    Ok(scheduled)
}
