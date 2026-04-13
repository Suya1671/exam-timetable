mod constraint_tracking;
mod diagnostics;
mod int_extensions;

pub use crate::diagnostics::ConstraintError;

use crate::constraint_tracking::ConstraintTracker;
use crate::int_extensions::IntExtensions;
pub use entity::id::SessionId;
use entity::id::StudentId;
use std::collections::HashMap;
use std::fmt::Debug;
use std::thread::available_parallelism;
use std::time::Duration;
use z3::Params;
use z3::{
    ast::{Ast, Bool, Int},
    Model, Optimize, SatResult, Solvable,
};

/// Solver timeslot index (ordered by date, slot).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    derive_more::Display,
    derive_more::From,
    derive_more::Into,
    specta::Type,
)]
pub struct TimeslotIndex(pub u64);

/// An exam scheduler
pub struct ExamScheduler {
    /// The Z3 solver/optimizer instance
    optimizer: Optimize,

    /// The decision variable: one [Int] per session, value = timeslot index
    ///
    /// assignment\[session_id] = timeslot_index
    assignment: HashMap<SessionId, Int>,

    /// Tracks the constraints that have been added to the optimizer,
    ///
    /// This is used to provide better diagnostics in case of infeasibility or other errors
    tracker: ConstraintTracker,
}

impl ExamScheduler {
    /// Initialises an exam scheduler with all the exams to be processed
    ///
    /// # Params
    /// - n_timeslots: the number of available timeslots. Used during assignment
    /// - session_ids: the list of sessions to be scheduled. Used during assignment
    ///
    /// # Constraints setup
    /// - Each session must be assigned to a valid timeslot (i.e. within index bounds)
    /// - All students which take a certain exam must write it at the same time (i.e. 1 exam cannot be scheduled in 2 different timeslots)
    ///     - Implicitly satisfied by the fact that we only have 1 variable per session, so it can only be assigned to 1 timeslot
    ///     - This also means each session is scheduled at exactly one timeslot
    pub fn new(session_ids: impl Iterator<Item = SessionId>, n_timeslots: u64) -> Self {
        Self::new_with_timeout(session_ids, n_timeslots, Duration::from_hours(2))
    }

    /// Initialises an exam scheduler with a custom timeout.
    pub fn new_with_timeout(
        session_ids: impl Iterator<Item = SessionId>,
        n_timeslots: u64,
        timeout: Duration,
    ) -> Self {
        let mut params = Params::new();

        let avail_parallel = available_parallelism()
            .ok()
            .map(|v| v.get())
            .map(|v| v as u32)
            .unwrap_or(1);
        dbg!(avail_parallel);

        params.set_bool("parallel.enable", true);
        params.set_u32("parallel.threads.max", avail_parallel);
        params.set_u32("smt.threads", avail_parallel);

        params.set_f64("timeout", timeout.as_millis() as f64);

        params.set_f64("sat.random_freq", 0.02);

        params.set_bool("sat.simplify_implies", true);
        params.set_bool("sat.enable_pre_simplify", true);
        params.set_bool("sat.force_cleanup", true);

        params.set_symbol("sat.solver", "cadical");
        params.set_bool("sat.euf", true);

        params.set_symbol("opt.priority", "box");

        let optimizer = Optimize::new();
        optimizer.set_params(&params);

        let assignment = session_ids
            .map(|session| {
                let var = Int::fresh_const(&format!("session_{}", session.0));

                (session, var)
            })
            .collect::<HashMap<_, _>>();

        let mut scheduler = Self {
            optimizer,
            assignment,
            tracker: ConstraintTracker::new(),
        };

        for (&session, session_var) in &scheduler.assignment {
            // Domain constraint: 0 <= session_var < n_timeslots (combined into one constraint)
            let lower = session_var.ge(0);
            let upper = session_var.lt(n_timeslots);
            let bounds = lower & upper; // Use & for Bool AND

            scheduler.tracker.assert_hard(
                &scheduler.optimizer,
                &bounds,
                ConstraintError::DomainBounds {
                    session,
                    n_timeslots,
                },
            );
        }

        scheduler
    }

    /// Add allowed timeslots for an exam session
    ///
    /// # Params
    /// - session: the session to add the constraint for
    /// - allowed_timeslots: the list of timeslot indices that the exam can be scheduled
    ///
    /// # Constraints setup
    /// - The exam session must be scheduled in one of the allowed timeslots
    ///
    /// This can be used for hard-limiting certain exams to specific timeslots,
    /// e.g. if session must be scheduled in the morning, then we can only allow timeslots that are in the morning
    pub fn add_allowed_timeslots(
        &mut self,
        session: SessionId,
        allowed_timeslots: Vec<TimeslotIndex>,
    ) {
        let exam_timeslot = self.assignment.get(&session).unwrap();

        // Use OR of equals instead of array lookup for better performance
        let is_allowed = Bool::or(
            &allowed_timeslots
                .iter()
                .map(|&t| exam_timeslot.eq(t.0))
                .collect::<Vec<_>>(),
        );

        self.tracker.assert_hard(
            &self.optimizer,
            &is_allowed,
            ConstraintError::AllowedTimeslots {
                session,
                timeslots: allowed_timeslots,
            },
        );
    }

    /// Add disallowed timeslots for an exam
    ///
    /// # Params
    /// - exam: the exam to add the constraint for
    /// - disallowed_timeslots: the list of timeslot indices that the exam cannot be
    ///
    /// # Constraints setup
    /// - The exam must not be scheduled in any of the disallowed timeslots
    pub fn add_disallowed_timeslots(
        &mut self,
        session: SessionId,
        disallowed_timeslots: Vec<TimeslotIndex>,
    ) {
        let exam_timeslot = self.assignment.get(&session).unwrap();

        // Use AND of not-equals for better performance
        let not_disallowed = Bool::and(
            &disallowed_timeslots
                .iter()
                .map(|&t| exam_timeslot.ne(t.0))
                .collect::<Vec<_>>(),
        );

        self.tracker.assert_hard(
            &self.optimizer,
            &not_disallowed,
            ConstraintError::DisallowedTimeslots {
                session,
                timeslots: disallowed_timeslots,
            },
        );
    }

    /// Requires that two exams must be scheduled in any pair of timeslots from a given list
    ///
    /// This can be used for geography, where paper 1 and 2 must be in the same day
    pub fn add_pair_constraint(
        &mut self,
        session1: SessionId,
        session2: SessionId,
        allowed_timeslot_pairs: Vec<(TimeslotIndex, TimeslotIndex)>,
    ) {
        let timeslot1 = self.assignment.get(&session1).unwrap();
        let timeslot2 = self.assignment.get(&session2).unwrap();

        // Constraint: the two exams must be scheduled in any pair of timeslots from the given list
        self.tracker.assert_hard(
            &self.optimizer,
            &Bool::or(
                &allowed_timeslot_pairs
                    .iter()
                    .map(|&(t1, t2)| timeslot1.eq(t1.0) & timeslot2.eq(t2.0))
                    .collect::<Vec<_>>(),
            ),
            ConstraintError::PairConstraint {
                session1,
                session2,
                allowed_pairs: allowed_timeslot_pairs,
            },
        );
    }

    /// Require two sessions to run in the same timeslot.
    /// AI-generated (GPT-5.3-codex).
    pub fn require_same_time(&mut self, session1: SessionId, session2: SessionId) {
        let timeslot1 = self.assignment.get(&session1).unwrap();
        let timeslot2 = self.assignment.get(&session2).unwrap();

        self.tracker.assert_hard(
            &self.optimizer,
            &timeslot1.eq(timeslot2),
            ConstraintError::SameTime { session1, session2 },
        );
    }

    pub fn require_different_time(&mut self, session1: SessionId, session2: SessionId) {
        let timeslot1 = self.assignment.get(&session1).unwrap();
        let timeslot2 = self.assignment.get(&session2).unwrap();

        self.tracker.assert_hard(
            &self.optimizer,
            &timeslot1.ne(timeslot2),
            ConstraintError::DifferentTime { session1, session2 },
        );
    }

    /// Add preferred timeslots for an exam with a certain priority
    ///
    /// # Params
    /// - exam: the exam to add the constraint for
    /// - timeslots: the list of timeslot indices that the exam should preferably be scheduled
    /// - priority: the priority of this preference (higher = more important)
    ///
    /// # Constraints setup
    /// - Soft constraint: the exam should be scheduled in the preferred timeslot
    pub fn prioritise_exam(&self, session: SessionId, timeslots: &[TimeslotIndex], priority: u64) {
        let exam_timeslot = self.assignment.get(&session).unwrap();

        for &timeslot in timeslots {
            // Soft constraint: the exam should be scheduled in the preferred timeslot
            self.optimizer
                .assert_soft(&exam_timeslot.eq(timeslot.0), priority, None);
        }
    }

    /// Constrains two exams to have different values for a given timeslot property.
    ///
    /// For example, given a week mapping, this ensures P1 and P2 of the same subject
    /// are scheduled in different weeks. Given a day mapping, it ensures two exams
    /// don't fall on the same day.
    ///
    /// # Params
    /// - exam_a: the first exam
    /// - exam_b: the second exam
    /// - mapping: a map of timeslot ID -> property value to separate by (e.g. TimeslotId -> week number)
    pub fn separate_exam_groups(
        &mut self,
        session1: SessionId,
        session2: SessionId,
        mapping: HashMap<TimeslotIndex, u64>,
    ) {
        let a = self.assignment.get(&session1).unwrap();
        let b = self.assignment.get(&session2).unwrap();

        let mut week_timeslots: HashMap<u64, Vec<u64>> = HashMap::new();
        for (&timeslot, &week) in &mapping {
            week_timeslots.entry(week).or_default().push(timeslot.0);
        }

        let mut week_constraints = Vec::new();
        for (_, timeslots) in week_timeslots {
            if timeslots.is_empty() {
                continue;
            }
            let a_in_week = Bool::or(&timeslots.iter().map(|&t| a.eq(t)).collect::<Vec<_>>());
            let b_in_week = Bool::or(&timeslots.iter().map(|&t| b.eq(t)).collect::<Vec<_>>());
            week_constraints.push(Bool::or(&[&a_in_week.not(), &b_in_week.not()]));
        }

        let combined = Bool::and(&week_constraints);
        self.tracker.assert_hard(
            &self.optimizer,
            &combined,
            ConstraintError::SeparateExamGroups {
                session1,
                session2,
                mapping,
            },
        );
    }

    // TODO: make this more "generic"/not student based.
    /// Add a soft constraint to minimize the number of exams a student has on the same day
    ///
    /// # Params
    /// - students: the list of students to consider for this constraint
    /// - days: the list of days, where each day is a list of timeslots that belong to the same day
    /// - student_weight: a function that takes a student and returns a weight for how important it is to minimize the number of exams on the same day for that student
    ///
    /// # Constraints setup
    /// - Soft constraint: for each student and each day, if the student has more than 1 exam on that day, add a penalty proportional to the number of excess exams and the student weight. This encourages the solver to spread a student's exams across different days, but does not require it (i.e. it's a soft constraint).
    pub fn minimize_exams_per_day(
        &self,
        students: &HashMap<StudentId, Vec<SessionId>>,
        days: &[&[TimeslotIndex]],
        student_weight: impl Fn(StudentId) -> u64,
    ) {
        let total_penalty: Int = students
            .iter()
            .flat_map(|(&student_id, exams)| {
                let weight = student_weight(student_id);
                days.iter().map(move |day_timeslots| {
                    let count = self.exams_on_day(exams, day_timeslots);

                    // Penalty = weight * max(0, count - 1)
                    // i.e. 0 if 1 or fewer exams, scaled count otherwise
                    let excess = count - 1;

                    let is_in_excess = excess.gt(0);
                    is_in_excess.ite(&(excess * weight), &Int::from_u64(0))
                })
            })
            .reduce(|acc, x| acc + x)
            .unwrap();

        self.optimizer.minimize(&total_penalty);
    }

    /// Add a soft constraint to maximise the distance between two exams
    ///
    /// # Params
    /// - exam1: the first exam
    /// - exam2: the second exam
    ///
    /// # Constraints setup
    /// - Optimize: the time between the two exams must be maximised
    pub fn maximize_exam_distance(&self, session1: SessionId, session2: SessionId) {
        let timeslot1 = self.assignment.get(&session1).unwrap();
        let timeslot2 = self.assignment.get(&session2).unwrap();

        // Soft constraint: maximize the distance between the two exams
        let distance = (timeslot1 - timeslot2).abs();

        self.optimizer.maximize(&distance);
    }

    /// Count the number of exams within in a list is done on a given day
    ///
    /// # Params
    /// - sessions: the exam sessions to count
    /// - day_timeslots: the list of timeslot indices that belong to the same day
    ///
    /// # Returns
    /// - The number of sessions that happen on that day
    fn exams_on_day(&self, sessions: &[SessionId], day_timeslots: &[TimeslotIndex]) -> Int {
        sessions
            .iter()
            .map(|session| {
                let var = self.assignment.get(session).unwrap();

                let is_on_day = Bool::or(
                    &day_timeslots
                        .iter()
                        .map(|&t| var.eq(t.0))
                        .collect::<Vec<_>>(),
                );

                is_on_day.ite(&Int::from_u64(1), &Int::from_u64(0))
            })
            .reduce(|acc, x| acc + x)
            .unwrap()
    }

    // TODO: make this more generic/not student based.
    // The custom error reporting per-student group is nice though...
    // I wonder if adding a way to attach context to all potential errors would be best (and what would be the best way of going about that...)
    /// Setup student constraints
    ///
    /// # Params
    /// - students: all students which have the same exam sessions
    /// - sessions: the exam sessions for all the students
    ///
    /// # Constraints setup
    /// - Students cannot have two exams in the same timeslot
    ///
    /// AI-generated (minimax-m2.5).
    pub fn setup_students(&mut self, students: Vec<StudentId>, sessions: Vec<SessionId>) {
        let exam_bools = sessions
            .iter()
            .map(|exam| self.assignment.get(exam).unwrap())
            .collect::<Vec<_>>();

        // Constraint: a student cannot have two exams in the same timeslot
        // Therefore, all exams taken by a student must be in different timeslots
        self.tracker.assert_hard(
            &self.optimizer,
            &Int::distinct(&exam_bools),
            ConstraintError::StudentDistinct { students, sessions },
        );
    }

    /// Setup student constraints
    pub fn setup_student(&mut self, student: StudentId, sessions: Vec<SessionId>) {
        self.setup_students(vec![student], sessions);
    }

    /// Solve and return a single solution (the first one found).
    /// This is faster than `solve()` when you only need one schedule.
    /// AI-generated (minimax-m2.5).
    pub fn solve_one(self) -> Result<HashMap<SessionId, TimeslotIndex>, SolverError> {
        println!("Running solver with {} sessions", self.assignment.len());
        dbg!(self.optimizer.get_assertions());
        let sat_result = self.optimizer.check(&[]);

        match sat_result {
            SatResult::Sat => {
                let model = self.optimizer.get_model().ok_or(SolverError::NoModel)?;
                dbg!(self
                    .optimizer
                    .get_statistics()
                    .entries()
                    .collect::<Vec<_>>());

                let mut result = HashMap::new();
                for (&session, var) in &self.assignment {
                    if let Some(value) = model.eval(var, true) {
                        let idx = value.as_i64().ok_or_else(|| SolverError::ModelEvaluation {
                            session,
                            details: "solution value is not an integer".to_string(),
                        })?;
                        result.insert(session, TimeslotIndex(idx as u64));
                    } else {
                        return Err(SolverError::ModelEvaluation {
                            session,
                            details: "failed to evaluate variable".to_string(),
                        });
                    }
                }
                Ok(result)
            }
            SatResult::Unsat => Err(SolverError::Infeasible {
                unsat_core_constraints: self
                    .tracker
                    .unsat_core_constraints(&self.optimizer)
                    .cloned()
                    .collect(),
            }),
            SatResult::Unknown => Err(SolverError::Unknown {
                reason: self.optimizer.get_reason_unknown(),
            }),
        }
    }

    pub fn solve(
        self,
    ) -> Result<impl Iterator<Item = HashMap<SessionId, TimeslotIndex>>, SolverError> {
        let sat_result = self.optimizer.check(&[]);

        match sat_result {
            SatResult::Sat => {
                let solutions = ExamSchedulerSolution(self.assignment);
                Ok(self
                    .optimizer
                    .into_solutions(solutions, true)
                    .map(|solution| solution.0)
                    .map(|solution| {
                        solution
                            .into_iter()
                            .map(|(session, var)| {
                                (
                                    session,
                                    TimeslotIndex(var.as_u64().expect("solution to have a value")),
                                )
                            })
                            .collect()
                    }))
            }
            SatResult::Unsat => Err(SolverError::Infeasible {
                unsat_core_constraints: self
                    .tracker
                    .unsat_core_constraints(&self.optimizer)
                    .cloned()
                    .collect(),
            }),
            SatResult::Unknown => Err(SolverError::Unknown {
                reason: self.optimizer.get_reason_unknown(),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExamSchedulerSolution(HashMap<SessionId, Int>);

impl Solvable for ExamSchedulerSolution {
    type ModelInstance = Self;

    fn read_from_model(&self, model: &Model, _model_completion: bool) -> Option<Self> {
        let values = self
            .0
            .iter()
            .map(|(&session_id, session)| {
                model.eval(session, true).map(|value| (session_id, value))
            })
            .collect::<Option<HashMap<_, _>>>()?;

        Some(ExamSchedulerSolution(values))
    }

    fn generate_constraint(&self, model_instance: &Self) -> Bool {
        let neq_constraints: Vec<_> = model_instance
            .0
            .iter()
            .map(|(&session, timeslot)| {
                let var = self.0.get(&session).unwrap();
                var.eq(timeslot).not()
            })
            .collect();

        Bool::or(&neq_constraints)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solved_timeslot(
        solution: &HashMap<SessionId, TimeslotIndex>,
        session: SessionId,
    ) -> TimeslotIndex {
        *solution
            .get(&session)
            .unwrap_or_else(|| panic!("missing session {session}"))
    }

    #[test]
    fn basic_solve() {
        let exam_ids = [SessionId(1), SessionId(2), SessionId(3)];
        let n_timeslots = 3;

        let scheduler = ExamScheduler::new(exam_ids.iter().copied(), n_timeslots);

        let solutions: Vec<_> = scheduler
            .solve()
            .expect("Expected a valid solution")
            .collect();

        // With 3 exams and 3 timeslots, there are 3^3 = 27 possible solutions
        assert_eq!(solutions.len(), 27);

        // Verify uniqueness
        for i in 0..solutions.len() {
            for j in (i + 1)..solutions.len() {
                assert_ne!(solutions[i], solutions[j]);
            }
        }

        // Verify each solution assigns all sessions
        for solution in &solutions {
            assert_eq!(solution.len(), 3);
            assert!(solution.contains_key(&SessionId(1)));
            assert!(solution.contains_key(&SessionId(2)));
            assert!(solution.contains_key(&SessionId(3)));

            // Verify all timeslots are within bounds
            for &timeslot in solution.values() {
                assert!(timeslot.0 < n_timeslots);
            }
        }
    }
    #[test]
    fn add_allowed_timeslots_restricts_domain() {
        let exam_ids = [SessionId(1)];
        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), 4);

        scheduler.add_allowed_timeslots(SessionId(1), vec![TimeslotIndex(2), TimeslotIndex(3)]);

        let solutions: Vec<_> = scheduler
            .solve()
            .expect("Expected a valid solution")
            .map(|solution| solved_timeslot(&solution, SessionId(1)))
            .collect();

        assert!(solutions.contains(&TimeslotIndex(2)));
        assert!(solutions.contains(&TimeslotIndex(3)));

        assert_eq!(solutions.len(), 2);
    }

    #[test]
    fn add_allowed_timeslots_conflict_is_infeasible() {
        let exam_ids = [SessionId(1)];
        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), 4);

        scheduler.add_allowed_timeslots(SessionId(1), vec![TimeslotIndex(0)]);
        scheduler.add_disallowed_timeslots(SessionId(1), vec![TimeslotIndex(0)]);

        let Err(err) = scheduler.solve() else {
            unreachable!("Expected no valid solutions");
        };

        match err {
            SolverError::Infeasible {
                unsat_core_constraints,
                ..
            } => {
                assert!(unsat_core_constraints.iter().any(|c| {
                    matches!(
                        c,
                        ConstraintError::AllowedTimeslots {
                            session: SessionId(1),
                            timeslots
                        } if timeslots == &vec![TimeslotIndex(0)]
                    )
                }));

                assert!(unsat_core_constraints.iter().any(|c| {
                    matches!(
                        c,
                        ConstraintError::DisallowedTimeslots {
                            session: SessionId(1),
                            timeslots
                        } if timeslots == &vec![TimeslotIndex(0)]
                    )
                }));
            }
            _ => panic!("Expected an infeasibility error"),
        }
    }

    #[test]
    fn add_pair_constraint_enforces_allowed_pairs() {
        let exam_ids = [SessionId(1), SessionId(2)];
        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), 3);

        scheduler.add_pair_constraint(
            SessionId(1),
            SessionId(2),
            vec![
                (TimeslotIndex(0), TimeslotIndex(1)),
                (TimeslotIndex(2), TimeslotIndex(0)),
            ],
        );

        let solution = scheduler
            .solve()
            .expect("Expected a valid solution")
            .next()
            .expect("solution to exist");
        let pair = (
            solved_timeslot(&solution, SessionId(1)),
            solved_timeslot(&solution, SessionId(2)),
        );

        assert!(
            pair == (TimeslotIndex(0), TimeslotIndex(1))
                || pair == (TimeslotIndex(2), TimeslotIndex(0))
        );
    }

    #[test]
    fn add_pair_constraint_conflict_is_infeasible() {
        let exam_ids = [SessionId(1), SessionId(2)];
        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), 2);

        scheduler.add_pair_constraint(
            SessionId(1),
            SessionId(2),
            vec![(TimeslotIndex(0), TimeslotIndex(1))],
        );
        scheduler.add_pair_constraint(
            SessionId(1),
            SessionId(2),
            vec![(TimeslotIndex(1), TimeslotIndex(0))],
        );

        let Err(err) = scheduler.solve() else {
            unreachable!("Expected no solutions");
        };

        match err {
            SolverError::Infeasible {
                unsat_core_constraints,
                ..
            } => {
                let count = unsat_core_constraints
                    .iter()
                    .filter(|c| {
                        matches!(
                            c,
                            ConstraintError::PairConstraint {
                                session1: SessionId(1),
                                session2: SessionId(2),
                                ..
                            }
                        )
                    })
                    .count();

                assert!(count >= 2, "Expected both pair constraints in unsat core");
            }
            _ => panic!("Expected an infeasibility error"),
        }
    }

    #[test]
    fn add_disallowed_timeslots_excludes_values() {
        let exam_ids = [SessionId(1)];
        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), 3);

        scheduler.add_disallowed_timeslots(SessionId(1), vec![TimeslotIndex(0), TimeslotIndex(1)]);

        let solution = scheduler
            .solve()
            .expect("Expected a valid solution")
            .next()
            .expect("solution to exist");
        assert_eq!(solved_timeslot(&solution, SessionId(1)), TimeslotIndex(2));
    }

    #[test]
    fn prioritise_exam_prefers_weighted_soft_constraints() {
        let exam_ids = [SessionId(1), SessionId(2)];
        let scheduler = ExamScheduler::new(exam_ids.iter().copied(), 3);

        scheduler.prioritise_exam(SessionId(1), &[TimeslotIndex(0)], 100);
        scheduler.prioritise_exam(SessionId(2), &[TimeslotIndex(2)], 100);

        let solution = scheduler
            .solve()
            .expect("Expected a valid solution")
            .next()
            .expect("solution to exist");

        assert_eq!(solved_timeslot(&solution, SessionId(1)), TimeslotIndex(0));
        assert_eq!(solved_timeslot(&solution, SessionId(2)), TimeslotIndex(2));
    }

    #[test]
    fn separate_exam_groups_uses_mapping_property() {
        let exam_ids = [SessionId(1), SessionId(2)];
        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), 4);

        // 0,1 -> day 0 ; 2,3 -> day 1
        let day_map = HashMap::from([
            (TimeslotIndex(0), 0),
            (TimeslotIndex(1), 0),
            (TimeslotIndex(2), 1),
            (TimeslotIndex(3), 1),
        ]);
        scheduler.separate_exam_groups(SessionId(1), SessionId(2), day_map.clone());

        let solution = scheduler
            .solve()
            .expect("Expected a valid solution")
            .next()
            .expect("solution to exist");

        let t1 = solved_timeslot(&solution, SessionId(1));
        let t2 = solved_timeslot(&solution, SessionId(2));

        assert_ne!(day_map.get(&t1), day_map.get(&t2));
    }

    /// AI-generated (GPT-5.3-codex).
    #[test]
    fn require_same_time_assigns_both_sessions_to_same_slot() {
        let exam_ids = [SessionId(1), SessionId(2)];
        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), 3);

        scheduler.require_same_time(SessionId(1), SessionId(2));

        let solution = scheduler
            .solve()
            .expect("Expected a valid solution")
            .next()
            .expect("solution to exist");

        assert_eq!(
            solved_timeslot(&solution, SessionId(1)),
            solved_timeslot(&solution, SessionId(2))
        );
    }

    #[test]
    fn require_different_time_assigns_both_sessions_to_different_timeslots() {
        let exam_ids = [SessionId(1), SessionId(2)];
        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), 3);
        scheduler.require_different_time(SessionId(1), SessionId(2));

        let solution = scheduler
            .solve()
            .expect("Expected a valid solution")
            .next()
            .expect("solution to exist");

        assert_ne!(
            solved_timeslot(&solution, SessionId(1)),
            solved_timeslot(&solution, SessionId(2))
        );
    }

    #[test]
    fn setup_students_prevents_clashes() {
        let exam_ids = [SessionId(1), SessionId(2), SessionId(3)];
        let n_timeslots = 2;

        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), n_timeslots);
        scheduler.setup_student(StudentId(1), vec![SessionId(1), SessionId(2)]);
        scheduler.setup_student(StudentId(2), vec![SessionId(2), SessionId(3)]);

        let solution = scheduler
            .solve()
            .expect("Expected a solution")
            .next()
            .expect("solution to exist");

        let exam_1 = solved_timeslot(&solution, SessionId(1));
        let exam_2 = solved_timeslot(&solution, SessionId(2));
        let exam_3 = solved_timeslot(&solution, SessionId(3));

        assert_ne!(exam_1, exam_2, "student 1 has a clash");
        assert_ne!(exam_2, exam_3, "student 2 has a clash");
    }

    /// AI-generated (GPT-5.2-codex).
    #[test]
    fn setup_students_prevents_multiway_overlaps() {
        let exam_ids = [SessionId(1), SessionId(2), SessionId(3), SessionId(4)];
        let n_timeslots = 4;

        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), n_timeslots);
        scheduler.setup_student(StudentId(1), vec![SessionId(1), SessionId(2), SessionId(3)]);
        scheduler.setup_student(StudentId(2), vec![SessionId(2), SessionId(4)]);

        let solution = scheduler
            .solve()
            .expect("Expected a solution")
            .next()
            .expect("solution to exist");

        let s1 = [
            solved_timeslot(&solution, SessionId(1)),
            solved_timeslot(&solution, SessionId(2)),
            solved_timeslot(&solution, SessionId(3)),
        ];
        assert!(s1[0] != s1[1] && s1[0] != s1[2] && s1[1] != s1[2]);

        let s2 = [
            solved_timeslot(&solution, SessionId(2)),
            solved_timeslot(&solution, SessionId(4)),
        ];
        assert_ne!(s2[0], s2[1]);
    }

    /// AI-generated (GPT-5.2-codex).
    #[test]
    fn setup_students_allows_non_overlapping_shared_sessions() {
        let exam_ids = [SessionId(1), SessionId(2), SessionId(3), SessionId(4)];
        let n_timeslots = 4;

        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), n_timeslots);
        scheduler.setup_student(StudentId(1), vec![SessionId(1), SessionId(2)]);
        scheduler.setup_student(StudentId(2), vec![SessionId(2), SessionId(3)]);
        scheduler.setup_student(StudentId(3), vec![SessionId(1), SessionId(4)]);

        let solution = scheduler
            .solve()
            .expect("Expected a solution")
            .next()
            .expect("solution to exist");

        let s1 = [
            solved_timeslot(&solution, SessionId(1)),
            solved_timeslot(&solution, SessionId(2)),
        ];
        assert_ne!(s1[0], s1[1]);

        let s2 = [
            solved_timeslot(&solution, SessionId(2)),
            solved_timeslot(&solution, SessionId(3)),
        ];
        assert_ne!(s2[0], s2[1]);

        let s3 = [
            solved_timeslot(&solution, SessionId(1)),
            solved_timeslot(&solution, SessionId(4)),
        ];
        assert_ne!(s3[0], s3[1]);
    }

    /// AI-generated (GPT-5.2-codex).
    #[test]
    fn multi_session_overlap_is_infeasible() {
        let exam_ids = [
            SessionId(1),
            SessionId(2),
            SessionId(3),
            SessionId(4),
            SessionId(5),
        ];
        let n_timeslots = 4;

        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), n_timeslots);
        scheduler.setup_student(
            StudentId(1),
            vec![
                SessionId(1),
                SessionId(2),
                SessionId(3),
                SessionId(4),
                SessionId(5),
            ],
        );

        scheduler.add_allowed_timeslots(SessionId(1), vec![TimeslotIndex(0)]);
        scheduler.add_allowed_timeslots(SessionId(2), vec![TimeslotIndex(1)]);
        scheduler.add_allowed_timeslots(SessionId(3), vec![TimeslotIndex(2)]);
        scheduler.add_allowed_timeslots(SessionId(4), vec![TimeslotIndex(3)]);
        scheduler.add_allowed_timeslots(SessionId(5), vec![TimeslotIndex(2)]);

        let Err(err) = scheduler.solve() else {
            unreachable!("Expected no solutions")
        };

        match err {
            SolverError::Infeasible { .. } => {}
            _ => panic!("Expected an infeasibility error"),
        }
    }

    /// AI-generated (GPT-5.2-codex).
    #[test]
    fn consecutive_chain_with_overlap_is_infeasible() {
        let exam_ids = [SessionId(1), SessionId(2), SessionId(3), SessionId(4)];
        let n_timeslots = 3;

        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), n_timeslots);
        scheduler.setup_student(
            StudentId(1),
            vec![SessionId(1), SessionId(2), SessionId(3), SessionId(4)],
        );

        let consecutive_pairs = vec![
            (TimeslotIndex(0), TimeslotIndex(1)),
            (TimeslotIndex(1), TimeslotIndex(2)),
        ];
        scheduler.add_pair_constraint(SessionId(1), SessionId(2), consecutive_pairs.clone());
        scheduler.add_pair_constraint(SessionId(2), SessionId(3), consecutive_pairs);

        scheduler.add_allowed_timeslots(SessionId(1), vec![TimeslotIndex(0)]);
        scheduler.add_allowed_timeslots(SessionId(2), vec![TimeslotIndex(1)]);
        scheduler.add_allowed_timeslots(SessionId(3), vec![TimeslotIndex(2)]);
        scheduler.add_allowed_timeslots(SessionId(4), vec![TimeslotIndex(2)]);

        let Err(err) = scheduler.solve() else {
            unreachable!("Expected no solutions")
        };

        match err {
            SolverError::Infeasible { .. } => {}
            _ => panic!("Expected an infeasibility error"),
        }
    }

    #[test]
    fn basic_fail_not_enough_timeslots() {
        let exam_ids = [SessionId(1), SessionId(2), SessionId(3)];
        let n_timeslots = 2;

        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), n_timeslots);
        scheduler.setup_student(StudentId(1), vec![SessionId(1), SessionId(2), SessionId(3)]);

        let Err(errors) = scheduler.solve() else {
            unreachable!("Expected no solutions")
        };

        match errors {
            SolverError::Infeasible {
                unsat_core_constraints,
                ..
            } => {
                assert!(unsat_core_constraints.iter().any(|c| {
                    matches!(c, ConstraintError::StudentDistinct { students: _, .. })
                }));
            }
            _ => panic!("Expected an infeasibility error"),
        }
    }

    #[test]
    fn minimize_exams_per_day_spreads_exams_for_student() {
        let exam_ids = [SessionId(1), SessionId(2), SessionId(3)];
        let n_timeslots = 3;
        let scheduler = ExamScheduler::new(exam_ids.iter().copied(), n_timeslots);

        // Day 0: slots 0,1 ; Day 1: slot 2
        let days: [&[TimeslotIndex]; 2] =
            [&[TimeslotIndex(0), TimeslotIndex(1)], &[TimeslotIndex(2)]];
        let students = HashMap::from([(StudentId(10), vec![SessionId(1), SessionId(2)])]);

        scheduler.minimize_exams_per_day(&students, &days, |_| 10);

        let solution = scheduler
            .solve()
            .expect("Expected a valid solution")
            .next()
            .expect("solution to exist");
        let t1 = solved_timeslot(&solution, SessionId(1));
        let t2 = solved_timeslot(&solution, SessionId(2));

        let same_day = (days[0].contains(&t1) && days[0].contains(&t2))
            || (days[1].contains(&t1) && days[1].contains(&t2));
        assert!(!same_day, "Expected exams to be spread across days");
    }

    #[test]
    fn maximize_exam_distance_pushes_exams_apart() {
        let exam_ids = [SessionId(1), SessionId(2)];
        let scheduler = ExamScheduler::new(exam_ids.iter().copied(), 4);

        scheduler.maximize_exam_distance(SessionId(1), SessionId(2));

        let solution = scheduler
            .solve()
            .expect("Expected a valid solution")
            .next()
            .expect("solution to exist");
        let t1 = solved_timeslot(&solution, SessionId(1));
        let t2 = solved_timeslot(&solution, SessionId(2));

        let distance = (t1.0 as i64 - t2.0 as i64).abs();
        assert_eq!(distance, 3, "Expected maximal distance in 4 timeslots");
    }

    #[test]
    fn combined_constraints_and_optimizations_work_together() {
        let exam_ids = [SessionId(1), SessionId(2), SessionId(3)];
        let mut scheduler = ExamScheduler::new(exam_ids.iter().copied(), 4);

        scheduler.add_allowed_timeslots(SessionId(1), vec![TimeslotIndex(0), TimeslotIndex(1)]);
        scheduler.add_disallowed_timeslots(SessionId(2), vec![TimeslotIndex(0)]);
        scheduler.add_pair_constraint(
            SessionId(1),
            SessionId(3),
            vec![
                (TimeslotIndex(0), TimeslotIndex(2)),
                (TimeslotIndex(1), TimeslotIndex(3)),
            ],
        );
        scheduler.setup_student(StudentId(42), vec![SessionId(1), SessionId(2)]);
        scheduler.maximize_exam_distance(SessionId(1), SessionId(2));

        let solution = scheduler
            .solve()
            .expect("Expected a valid solution")
            .next()
            .expect("solution to exist");

        let t1 = solved_timeslot(&solution, SessionId(1));
        let t2 = solved_timeslot(&solution, SessionId(2));
        let t3 = solved_timeslot(&solution, SessionId(3));

        assert!([TimeslotIndex(0), TimeslotIndex(1)].contains(&t1));
        assert_ne!(t2, TimeslotIndex(0));
        assert_ne!(t1, t2);

        assert!(
            (t1 == TimeslotIndex(0) && t3 == TimeslotIndex(2))
                || (t1 == TimeslotIndex(1) && t3 == TimeslotIndex(3))
        );
    }
}

// pub enum SolverSolution {
//     /// A valid schedule that satisfies all hard constraints and optimizes soft constraints as much as possible.
//     Optimal(HashMap<ExamId, TimeslotId>),
//     /// A valid schedule that satisfies all hard constraints, but may not optimize soft constraints.
//     Inoptimal(HashMap<ExamId, TimeslotId>),
// }

#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum SolverError {
    /// The query is unsatisfiable. No schedule that satisfies all hard constraints exists.
    #[error("The query is unsatisfiable. No schedule that satisfies all hard constraints exists")]
    Infeasible {
        /// Hard constraints participating in the unsat core.
        ///
        /// This is typically a minimal or near-minimal conflicting subset and is the
        /// primary signal for explaining infeasibility.
        unsat_core_constraints: Vec<ConstraintError>,
    },

    /// Z3 reported SAT but no model was available.
    #[error("The query was satisfiable but no model was returned")]
    NoModel,

    /// Z3 returned a model that could not be converted to a timeslot index.
    #[error("Failed to evaluate model for session {session}: {details}")]
    ModelEvaluation {
        /// The session whose model value failed to evaluate or convert.
        session: SessionId,
        /// Human-readable details describing the model evaluation/conversion failure.
        details: String,
    },

    /// The query was interrupted, timed out or otherwise failed.
    #[error("The query was interrupted, timed out or otherwise failed")]
    Unknown {
        /// Optional reason returned by Z3 for the unknown result.
        reason: Option<String>,
    },
}
