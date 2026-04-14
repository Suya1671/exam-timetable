use diesel::{
    dsl::{exists, not},
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
    RunQueryDsl, SqliteConnection,
};
use std::collections::{HashMap, HashSet};

use entity::id::{ExamId, SessionId, StudentId, SubjectId, TimeslotId};
use entity::schema::{
    enrolled_student, exam, exam_time_constraint, exam_timeslot_restriction, session, student,
    subject, timeslot,
};
use entity::{exam_time_constraints::ExamConstraintType, exams::TimeslotRestrictionMode};
use solver::{ExamScheduler, SolverError};
mod scheduler_builder;
mod solver_adapter;

use crate::scheduler_builder::SchedulerBuilder;
use crate::solver_adapter::SolverAdapter;

/// AI-generated (GPT-5.3-codex).
type ExamRestrictionMap = HashMap<ExamId, (Option<TimeslotRestrictionMode>, HashSet<TimeslotId>)>;

#[derive(Debug, Clone)]
struct ExamDetails {
    name: String,
    grade: i32,
    paper: i32,
}

impl std::fmt::Display for ExamDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} Paper {} (Grade {})",
            self.name, self.paper, self.grade
        )
    }
}

/// AI-generated (GPT-5.3-codex).
struct PrecheckContext {
    all_timeslots: HashSet<TimeslotId>,
    restrictions: ExamRestrictionMap,
    exam_details: HashMap<ExamId, ExamDetails>,
    morning_slots_by_date: HashMap<time::Date, HashSet<TimeslotId>>,
    all_slots_by_date: HashMap<time::Date, HashSet<TimeslotId>>,
}

impl PrecheckContext {
    fn load(db: &mut SqliteConnection, _n_timeslots: u64) -> Result<Self, SolveError> {
        let all_timeslots = load_all_timeslot_ids(db)?;
        let restrictions = load_exam_restrictions(db)?;
        let exam_details = load_exam_details(db)?;

        let timeslot_rows = timeslot::table
            .select((timeslot::id, timeslot::date, timeslot::slot))
            .load::<(TimeslotId, time::Date, i32)>(db)?;

        let mut morning_slots_by_date: HashMap<time::Date, HashSet<TimeslotId>> = HashMap::new();
        let mut all_slots_by_date: HashMap<time::Date, HashSet<TimeslotId>> = HashMap::new();
        for (id, date, slot) in timeslot_rows {
            all_slots_by_date.entry(date).or_default().insert(id);
            if slot == 0 {
                morning_slots_by_date.entry(date).or_default().insert(id);
            }
        }

        Ok(Self {
            all_timeslots,
            restrictions,
            exam_details,
            morning_slots_by_date,
            all_slots_by_date,
        })
    }

    fn exam_details(&self, exam_id: ExamId) -> String {
        self.exam_details
            .get(&exam_id)
            .map(|e| e.to_string())
            .unwrap_or_else(|| format!("ID {}", exam_id.0))
    }

    fn effective_timeslots(
        &self,
        mode: Option<TimeslotRestrictionMode>,
        selected: &HashSet<TimeslotId>,
    ) -> HashSet<TimeslotId> {
        match mode {
            Some(TimeslotRestrictionMode::Allow) => selected.clone(),
            Some(TimeslotRestrictionMode::Deny) => self
                .all_timeslots
                .iter()
                .copied()
                .filter(|id| !selected.contains(id))
                .collect(),
            None => self.all_timeslots.clone(),
        }
    }
}

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
    precheck_students_have_exams_at_their_grade(db)?;

    let ctx = PrecheckContext::load(db, n_timeslots)?;

    ctx.check_exam_timeslot_domains()?;
    ctx.check_same_time_pairs(db)?;
    ctx.check_same_day_pairs(db)?;
    ctx.check_pair_constraint_contradictions(db)?;

    let rows = student::table
        .inner_join(enrolled_student::table)
        .inner_join(
            exam::table.on(exam::subject_id
                .eq(enrolled_student::subject_id)
                .and(exam::grade.eq(student::grade))),
        )
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

impl PrecheckContext {
    fn check_exam_timeslot_domains(&self) -> Result<(), SolveError> {
        for (&exam_id, &(mode, ref selected)) in &self.restrictions {
            let feasible = self.effective_timeslots(mode, selected);
            if feasible.is_empty() {
                let details = self.exam_details(exam_id);
                return Err(SolveError::PrecheckFailed {
                    reason: format!(
                        "Exam '{details}' has no feasible timeslots. It is restricted to {} timeslots but none are available.",
                        match mode {
                            Some(TimeslotRestrictionMode::Allow) => format!("one of {} allowed", selected.len()),
                            Some(TimeslotRestrictionMode::Deny) => "all but".to_string(),
                            None => "no".to_string(),
                        }
                    ),
                });
            }
        }
        Ok(())
    }

    // TODO: generalise this to checking all types of pairs
    fn check_same_time_pairs(&self, db: &mut SqliteConnection) -> Result<(), SolveError> {
        let pairs = exam_time_constraint::table
            .select((
                exam_time_constraint::exam1_id,
                exam_time_constraint::exam2_id,
            ))
            .filter(exam_time_constraint::constraint_type.eq(ExamConstraintType::SameTime))
            .load::<(ExamId, ExamId)>(db)?;

        for (exam1_id, exam2_id) in pairs {
            let (mode1, selected1) =
                self.restrictions
                    .get(&exam1_id)
                    .ok_or_else(|| SolveError::PrecheckFailed {
                        reason: format!("Exam {} missing from restrictions", exam1_id.0),
                    })?;
            let (mode2, selected2) =
                self.restrictions
                    .get(&exam2_id)
                    .ok_or_else(|| SolveError::PrecheckFailed {
                        reason: format!("Exam {} missing from restrictions", exam2_id.0),
                    })?;

            let feasible1 = self.effective_timeslots(*mode1, selected1);
            let feasible2 = self.effective_timeslots(*mode2, selected2);

            let overlap_exists = feasible1.iter().any(|slot| feasible2.contains(slot));
            if !overlap_exists {
                return Err(SolveError::PrecheckFailed {
                    reason: format!(
                        "Same-time pair '{}' and '{}' has no common feasible timeslot. They must be scheduled together but have no overlapping available timeslots.",
                        self.exam_details(exam1_id),
                        self.exam_details(exam2_id),
                    ),
                });
            }
        }
        Ok(())
    }

    fn check_same_day_pairs(&self, db: &mut SqliteConnection) -> Result<(), SolveError> {
        let pairs = exam_time_constraint::table
            .select((
                exam_time_constraint::exam1_id,
                exam_time_constraint::exam2_id,
            ))
            .filter(exam_time_constraint::constraint_type.eq(ExamConstraintType::SameDay))
            .load::<(ExamId, ExamId)>(db)?;

        for (first_exam_id, second_exam_id) in pairs {
            let (mode1, selected1) = self.restrictions.get(&first_exam_id).ok_or_else(|| {
                SolveError::PrecheckFailed {
                    reason: format!("Exam {} missing from restrictions", first_exam_id.0),
                }
            })?;
            let (mode2, selected2) = self.restrictions.get(&second_exam_id).ok_or_else(|| {
                SolveError::PrecheckFailed {
                    reason: format!("Exam {} missing from restrictions", second_exam_id.0),
                }
            })?;

            let feasible_first = self.effective_timeslots(*mode1, selected1);
            let feasible_second = self.effective_timeslots(*mode2, selected2);

            let mut found_valid_day = false;
            let days_checked = self.all_slots_by_date.len();

            for (date, all_day_slots) in &self.all_slots_by_date {
                let morning_on_day = self.morning_slots_by_date.get(date);

                let first_in_morning = morning_on_day
                    .map(|m| m.iter().any(|slot| feasible_first.contains(slot)))
                    .unwrap_or(false);
                let second_in_afternoon = all_day_slots.iter().any(|slot| {
                    feasible_second.contains(slot)
                        && !morning_on_day.map(|m| m.contains(slot)).unwrap_or(true)
                });

                if first_in_morning && second_in_afternoon {
                    found_valid_day = true;
                    break;
                }
            }

            if !found_valid_day {
                let first_restriction_desc = match mode1 {
                    Some(TimeslotRestrictionMode::Allow) => {
                        if selected1.is_empty() {
                            "no allowed timeslots".to_string()
                        } else {
                            format!("limited to {} timeslots", selected1.len())
                        }
                    }
                    Some(TimeslotRestrictionMode::Deny) => {
                        format!("denied {} timeslots", selected1.len())
                    }
                    None => "no restrictions".to_string(),
                };
                let second_restriction_desc = match mode2 {
                    Some(TimeslotRestrictionMode::Allow) => {
                        if selected2.is_empty() {
                            "no allowed timeslots".to_string()
                        } else {
                            format!("limited to {} timeslots", selected2.len())
                        }
                    }
                    Some(TimeslotRestrictionMode::Deny) => {
                        format!("denied {} timeslots", selected2.len())
                    }
                    None => "no restrictions".to_string(),
                };

                return Err(SolveError::PrecheckFailed {
                    reason: format!(
                        "Same-day pair '{}' and '{}' has no feasible day combination. \
The first exam ({}) must be in a morning slot, and the second exam ({}) must be in an afternoon slot on the same day. \
Checked {} days but found no valid morning+afternoon combination.",
                        self.exam_details(first_exam_id),
                        self.exam_details(second_exam_id),
                        first_restriction_desc,
                        second_restriction_desc,
                        days_checked,
                    ),
                });
            }
        }
        Ok(())
    }

    fn check_pair_constraint_contradictions(
        &self,
        db: &mut SqliteConnection,
    ) -> Result<(), SolveError> {
        let same_time_pairs = exam_time_constraint::table
            .select((
                exam_time_constraint::exam1_id,
                exam_time_constraint::exam2_id,
            ))
            .filter(exam_time_constraint::constraint_type.eq(ExamConstraintType::SameTime))
            .load::<(ExamId, ExamId)>(db)?
            .into_iter()
            .map(|(e1, e2)| if e1 <= e2 { (e1, e2) } else { (e2, e1) })
            .collect::<HashSet<_>>();

        let different_week_pairs = exam_time_constraint::table
            .select((
                exam_time_constraint::exam1_id,
                exam_time_constraint::exam2_id,
            ))
            .filter(exam_time_constraint::constraint_type.eq(ExamConstraintType::DifferentWeek))
            .load::<(ExamId, ExamId)>(db)?
            .into_iter()
            .map(|(e1, e2)| if e1 <= e2 { (e1, e2) } else { (e2, e1) })
            .collect::<HashSet<_>>();

        if let Some((exam1, exam2)) = same_time_pairs
            .intersection(&different_week_pairs)
            .next()
            .copied()
        {
            return Err(SolveError::PrecheckFailed {
                reason: format!(
                    "Exams '{}' and '{}' are constrained to be both at the same time AND in different weeks. \
This is impossible: if they're at the same time, they're on the same day/week; if they're in different weeks, they can't be at the same time.",
                    self.exam_details(exam1),
                    self.exam_details(exam2),
                ),
            });
        }
        Ok(())
    }
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
fn precheck_students_have_exams_at_their_grade(
    db: &mut SqliteConnection,
) -> Result<(), SolveError> {
    let invalid_enrollments = student::table
        .inner_join(enrolled_student::table)
        .select((student::name, student::grade, enrolled_student::subject_id))
        .filter(not(exists(
            exam::table
                .filter(exam::subject_id.eq(enrolled_student::subject_id))
                .filter(exam::grade.eq(student::grade)),
        )))
        .load::<(String, i32, SubjectId)>(db)?;

    if let Some((student_name, grade, subject_id)) = invalid_enrollments.into_iter().next() {
        let subject_name: String = subject::table
            .filter(subject::id.eq(subject_id))
            .select(subject::name)
            .first(db)
            .unwrap_or_else(|_| format!("ID {}", subject_id.0));

        return Err(SolveError::PrecheckFailed {
            reason: format!(
                "Student '{}' (Grade {}) is enrolled in '{}' but no exam exists at their grade. \
Add exams for Grade {} or remove the subject from this student's enrolments.",
                student_name, grade, subject_name, grade
            ),
        });
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
fn load_exam_details(
    db: &mut SqliteConnection,
) -> Result<HashMap<ExamId, ExamDetails>, SolveError> {
    let rows = exam::table
        .inner_join(subject::table.on(exam::subject_id.eq(subject::id)))
        .select((exam::id, subject::name, exam::grade, exam::paper))
        .load::<(ExamId, String, i32, i32)>(db)?;

    Ok(rows
        .into_iter()
        .map(|(id, name, grade, paper)| (id, ExamDetails { name, grade, paper }))
        .collect())
}

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

/// Solve with a single solution (faster, no optimization).
/// AI-generated (minimax-m2.5).
pub fn solve_one(
    db: &mut SqliteConnection,
    locked_assignments: &[(SessionId, TimeslotId)],
) -> Result<HashMap<SessionId, TimeslotId>, SolveError> {
    let mappings = SolverAdapter::new(db)?;
    let n_timeslots: i64 = timeslot::table.count().get_result(db)?;
    let n_timeslots_u64: u64 = n_timeslots.try_into().unwrap();
    let all_timeslots = load_all_timeslot_ids(db)?;

    run_feasibility_precheck(db, n_timeslots_u64)?;
    validate_locked_assignments(mappings.session_ids(), &all_timeslots, locked_assignments)?;

    let mut scheduler = ExamScheduler::new(mappings.session_ids().iter().copied(), n_timeslots_u64);

    let mut builder = SchedulerBuilder::new(&mappings, &mut scheduler);
    builder.apply_student_clashes_from_db(db)?;
    builder.apply_timeslot_restrictions_for_exams_from_db(db)?;
    builder.apply_subject_exam_distance_from_db(db)?;
    builder.apply_exam_constraints_from_db(db)?;
    builder.apply_multi_session_constraints(db)?;
    builder.apply_minimize_exams_per_day(db)?;

    for (session_id, timeslot_id) in locked_assignments {
        mappings.apply_allowed_timeslots(
            &mut scheduler,
            *session_id,
            std::iter::once(*timeslot_id),
        );
    }

    let solution = scheduler.solve_one()?;
    Ok(mappings.map_solution(solution))
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
    builder.apply_exam_constraints_from_db(db)?;
    builder.apply_multi_session_constraints(db)?;
    builder.apply_minimize_exams_per_day(db)?;

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
