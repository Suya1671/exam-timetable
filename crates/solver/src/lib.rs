mod constraint_tracking;
mod diagnostics;
mod int_extensions;

pub use crate::diagnostics::{ConstraintError, SolverDebugInfo};

use std::collections::HashMap;

use model::{ExamId, StudentId, TimeslotId};
use z3::{
    ast::{Ast, Bool, Int},
    Optimize, SatResult,
};

use crate::constraint_tracking::ConstraintTracker;
use crate::int_extensions::IntExtensions;

// implementation notes
// - Always ensure that timeslots are ordered by their date and then by slot number to ensure consistent variable indexing

/// An exam scheduler
pub struct ExamScheduler {
    /// The Z3 solver/optimizer instance
    optimizer: Optimize,

    /// The decision variable: one [Int] per exam, value = timeslot index
    ///
    /// assignment\[exam_id] = timeslot_id
    assignment: HashMap<ExamId, Int>,

    /// Tracks the constraints that have been added to the optimizer,
    ///
    /// This is used to provide better diagnostics in case of infeasibility or other errors
    tracker: ConstraintTracker,
}

impl ExamScheduler {
    /// Initialises an exam scheduler with all of the exams to be processed
    ///
    /// # Params
    /// - n_timeslots: the number of available timeslots. Used during assignment
    /// - exam_ids: the list of exams to be scheduled. Used during assignment
    ///
    /// # Constraints setup
    /// - Each exam must be assigned to a valid timeslot (i.e. within index bounds)
    /// - All students which take a certain exam must write it at the same time (i.e. 1 exam cannot be scheduled in 2 different timeslots)
    ///     - Implicitly satisfied by the fact that we only have 1 variable per exam, so it can only be assigned to 1 timeslot
    ///     - This also means all students taking the same exam _must_ have the same ExamId
    pub fn new(exam_ids: &[ExamId], n_timeslots: i64) -> Self {
        let optimizer = Optimize::new();

        let assignment = exam_ids
            .iter()
            .map(|&exam| {
                let var = Int::fresh_const(&format!("exam_{}", exam));

                (exam, var)
            })
            .collect::<HashMap<_, _>>();

        let mut scheduler = Self {
            optimizer,
            assignment,
            tracker: ConstraintTracker::new(),
        };

        for &exam in exam_ids {
            let exam_var = scheduler.assignment.get(&exam).unwrap();

            // Domain constraint: each exam must be assigned to a valid timeslot (i.e. within index bounds)
            scheduler.tracker.assert_hard(
                &scheduler.optimizer,
                &exam_var.ge(Int::from_i64(0)),
                ConstraintError::DomainLowerBound { exam },
            );
            scheduler.tracker.assert_hard(
                &scheduler.optimizer,
                &exam_var.lt(Int::from_i64(n_timeslots)),
                ConstraintError::DomainUpperBound { exam, n_timeslots },
            );
        }

        scheduler
    }

    /// Add allowed timeslots for an exam
    ///
    /// # Params
    /// - exam: the exam to add the constraint for
    /// - allowed_timeslots: the list of timeslot indices that the exam can be scheduled
    ///
    /// # Constraints setup
    /// - The exam must be scheduled in one of the allowed timeslots
    ///
    /// This can be used for hard-limiting certain exams to specific timeslots,
    /// e.g. if an exam must be scheduled in the morning, then we can only allow timeslots that are in the morning
    /// or for geography, where paper 1 and 2 must be on the same day
    pub fn add_allowed_timeslots(&mut self, exam: ExamId, allowed_timeslots: &[TimeslotId]) {
        let exam_timeslot = self.assignment.get(&exam).unwrap();

        // Constraint: the exam must be scheduled in one of the allowed timeslots
        self.tracker.assert_hard(
            &self.optimizer,
            &Bool::or(
                &allowed_timeslots
                    .iter()
                    .map(|&timeslot| exam_timeslot.eq(Int::from_i64(timeslot)))
                    .collect::<Box<[_]>>(),
            ),
            ConstraintError::AllowedTimeslots {
                exam,
                timeslots: allowed_timeslots.to_vec(),
            },
        );
    }

    /// Requires that two exams must be scheduled in any pair of timeslots from a given list
    pub fn add_pair_constraint(
        &mut self,
        exam1: ExamId,
        exam2: ExamId,
        allowed_timeslot_pairs: &[(TimeslotId, TimeslotId)],
    ) {
        let timeslot1 = self.assignment.get(&exam1).unwrap();
        let timeslot2 = self.assignment.get(&exam2).unwrap();

        // Constraint: the two exams must be scheduled in any pair of timeslots from the given list
        self.tracker.assert_hard(
            &self.optimizer,
            &Bool::or(
                &allowed_timeslot_pairs
                    .iter()
                    .map(|&(t1, t2)| {
                        timeslot1.eq(Int::from_i64(t1)) & timeslot2.eq(Int::from_i64(t2))
                    })
                    .collect::<Box<[_]>>(),
            ),
            ConstraintError::PairConstraint {
                exam1,
                exam2,
                allowed_pairs: allowed_timeslot_pairs.to_vec(),
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
    pub fn add_disallowed_timeslots(&mut self, exam: ExamId, disallowed_timeslots: &[TimeslotId]) {
        let exam_timeslot = self.assignment.get(&exam).unwrap();

        for &timeslot in disallowed_timeslots {
            // Constraint: the exam must not be scheduled in any of the disallowed timeslots
            self.tracker.assert_hard(
                &self.optimizer,
                &exam_timeslot.eq(Int::from_i64(timeslot)).not(),
                ConstraintError::DisallowedTimeslot { exam, timeslot },
            );
        }
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
    pub fn prioritise_exam(&self, exam: ExamId, timeslots: &[TimeslotId], priority: i64) {
        let exam_timeslot = self.assignment.get(&exam).unwrap();

        for &timeslot in timeslots {
            // Soft constraint: the exam should be scheduled in the preferred timeslot
            self.optimizer
                .assert_soft(&exam_timeslot.eq(Int::from_i64(timeslot)), priority, None);
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
        &self,
        exam_a: ExamId,
        exam_b: ExamId,
        mapping: &HashMap<TimeslotId, i64>,
    ) {
        let a = self.assignment.get(&exam_a).unwrap();
        let b = self.assignment.get(&exam_b).unwrap();

        let property_a = self.timeslot_property_expr(a, mapping);
        let property_b = self.timeslot_property_expr(b, mapping);

        self.optimizer.assert(&property_a.eq(property_b).not());
    }

    /// Maps a symbolic timeslot assignment variable to a concrete property of that timeslot.
    ///
    /// Since the timeslot assignment is a Z3 variable (not a concrete value), we cannot simply
    /// index into a map at runtime. This function allows Z3 to reason about timeslot properties
    /// symbolically — for example, "what week is this exam in?" or "what day is this exam on?" —
    /// so that those properties can be used in further constraints.
    ///
    /// # Params
    /// - var: the Z3 timeslot assignment variable for an exam
    /// - mapping: a map of timeslot ID -> property value (e.g. TimeslotId -> week number)
    ///
    /// # Example
    /// ```
    /// // Constrain two exams to be in different weeks
    /// let week_of_p1 = self.timeslot_property_expr(&self.assignment[&p1], &week_map);
    /// let week_of_p2 = self.timeslot_property_expr(&self.assignment[&p2], &week_map);
    /// self.solver.assert(week_of_p1.eq(week_of_p2).not());
    /// ```
    fn timeslot_property_expr(&self, var: &Int, mapping: &HashMap<TimeslotId, i64>) -> Int {
        mapping
            .iter()
            .fold(Int::from_i64(-1), |acc, (&timeslot, &value)| {
                Bool::ite(
                    &var.eq(Int::from_i64(timeslot)),
                    &Int::from_i64(value),
                    &acc,
                )
            })
    }

    /// Add a constraint that an exam must be scheduled in multiple timeslots
    ///
    /// # Params
    /// - exam: the exam to add the constraint for
    /// - timeslot_combinations: a list of allowed combinations of timeslots for the exam. Each combination is a list of timeslot indices that the exam can be scheduled over
    pub fn add_multi_slot_exam_constraint(
        &mut self,
        exam: ExamId,
        timeslot_combinations: &[&[TimeslotId]],
    ) {
        let exam_timeslot = self.assignment.get(&exam).unwrap();

        let mut allowed_starts = Vec::new();
        for combination in timeslot_combinations {
            if let Some(&start) = combination.first() {
                allowed_starts.push(start);
            }
        }

        let slots_required = timeslot_combinations
            .first()
            .map_or(0_u32, |combo| combo.len() as u32);

        let expr = if allowed_starts.is_empty() {
            Bool::from_bool(false)
        } else {
            Bool::or(
                &allowed_starts
                    .iter()
                    .map(|&timeslot| exam_timeslot.eq(Int::from_i64(timeslot)))
                    .collect::<Box<[_]>>(),
            )
        };

        self.tracker.assert_hard(
            &self.optimizer,
            &expr,
            ConstraintError::MultiSlotStart {
                exam,
                slots_required,
                allowed_starts: allowed_starts.clone(),
            },
        );
    }

    /// Prevent other exams from falling inside a multi-slot exam block.
    ///
    /// This treats the exam's assignment variable as the start of a consecutive
    /// block of `slots_required` timeslots, based on the supplied position map.
    ///
    /// # Params
    /// - block_exam: the multi-slot exam defining the blocked window
    /// - other_exam: the exam that must remain outside the block
    /// - slots_required: number of consecutive slots required by the block exam
    /// - timeslot_positions: map of TimeslotId to ordered position index
    /// AI-generated (GPT-5.2-codex).
    pub fn prevent_block_overlap(
        &mut self,
        block_exam: ExamId,
        other_exam: ExamId,
        slots_required: u32,
        timeslot_positions: &HashMap<TimeslotId, i64>,
    ) {
        let block_var = self.assignment.get(&block_exam).unwrap();
        let other_var = self.assignment.get(&other_exam).unwrap();

        let block_pos = self.timeslot_property_expr(block_var, timeslot_positions);
        let other_pos = self.timeslot_property_expr(other_var, timeslot_positions);

        let block_end = &block_pos + Int::from_i64(i64::from(slots_required) - 1);

        let outside_block = other_pos.lt(&block_pos) | other_pos.gt(&block_end);

        self.tracker.assert_hard(
            &self.optimizer,
            &outside_block,
            ConstraintError::BlockExclusion {
                block_exam,
                other_exam,
                slots_required,
            },
        );
    }

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
        students: &HashMap<StudentId, Vec<ExamId>>,
        days: &[&[TimeslotId]],
        student_weight: impl Fn(StudentId) -> u64,
    ) {
        let total_penalty: Int = students
            .iter()
            .flat_map(|(&student_id, exams)| {
                let weight = student_weight(student_id) as i64;
                days.iter().map(move |day_timeslots| {
                    let count = self.exams_on_day(exams, day_timeslots);

                    // Penalty = weight * max(0, count - 1)
                    // i.e. 0 if 1 or fewer exams, scaled count otherwise
                    let excess = count - Int::from_i64(1);

                    let is_in_excess = excess.gt(Int::from_i64(0));
                    is_in_excess.ite(&(excess * Int::from_i64(weight)), &Int::from_i64(0))
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
    pub fn maximize_exam_distance(&self, exam1: ExamId, exam2: ExamId) {
        let timeslot1 = self.assignment.get(&exam1).unwrap();
        let timeslot2 = self.assignment.get(&exam2).unwrap();

        // Soft constraint: maximize the distance between the two exams
        let distance = (timeslot1 - timeslot2).abs();

        self.optimizer.maximize(&distance);
    }

    /// Count the number of exams within in a list is done on a given day
    ///
    /// # Params
    /// - exams: the exams to count
    /// - day_timeslots: the list of timeslot indices that belong to the same day
    ///
    /// # Returns
    /// - The number of exams that happen on that day
    fn exams_on_day(&self, exams: &[ExamId], day_timeslots: &[TimeslotId]) -> Int {
        exams
            .iter()
            .map(|exam| {
                let var = self.assignment.get(exam).unwrap();
                let on_day = Bool::or(
                    &day_timeslots
                        .iter()
                        .map(|&t| var.eq(Int::from_i64(t)))
                        .collect::<Box<[_]>>(),
                );
                on_day.ite(&Int::from_i64(1), &Int::from_i64(0))
            })
            .reduce(|acc, x| acc + x)
            .unwrap()
    }

    /// Setup student constraints
    ///
    /// # Constraints setup
    /// - Students cannot have two exams in the same timeslot
    pub fn setup_students(&mut self, students: &HashMap<StudentId, Vec<ExamId>>) {
        for (&student_id, exams) in students {
            let exam_bools = exams
                .iter()
                .map(|exam| self.assignment.get(exam).unwrap())
                .collect::<Box<[_]>>();

            // Constraint: a student cannot have two exams in the same timeslot
            // Therefore, all exams taken by a student must be in different timeslots
            self.tracker.assert_hard(
                &self.optimizer,
                &Int::distinct(&exam_bools),
                ConstraintError::StudentDistinct {
                    student: student_id,
                    exams: exams.clone(),
                },
            );
        }
    }

    pub fn solve(&self) -> Result<HashMap<ExamId, TimeslotId>, SolverError> {
        let sat_result = self.optimizer.check(&[]);

        match sat_result {
            SatResult::Sat => {
                let model = self
                    .optimizer
                    .get_model()
                    .ok_or_else(|| SolverError::NoModel {
                        debug: self.tracker.build_debug_info(&self.optimizer),
                    })?;

                let result = self
                    .assignment
                    .iter()
                    .map(|(&exam, var)| {
                        let value =
                            model
                                .eval(var, true)
                                .ok_or_else(|| SolverError::ModelEvaluation {
                                    exam,
                                    details: format!("Model did not produce a value for {var}"),
                                    debug: self.tracker.build_debug_info(&self.optimizer),
                                })?;

                        let timeslot =
                            value.as_i64().ok_or_else(|| SolverError::ModelEvaluation {
                                exam,
                                details: format!("Model value for {var} was not an i64: {}", value),
                                debug: self.tracker.build_debug_info(&self.optimizer),
                            })? as TimeslotId;

                        Ok((exam, timeslot))
                    })
                    .collect::<Result<HashMap<ExamId, TimeslotId>, SolverError>>()?;

                Ok(result)
            }
            SatResult::Unsat => Err(SolverError::Infeasible {
                unsat_core_constraints: self
                    .tracker
                    .unsat_core_constraints(&self.optimizer, sat_result),
                debug: self.tracker.build_debug_info(&self.optimizer),
            }),
            SatResult::Unknown => Err(SolverError::Unknown {
                reason: self.optimizer.get_reason_unknown(),
                debug: self.tracker.build_debug_info(&self.optimizer),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solved_timeslot(solution: &HashMap<ExamId, TimeslotId>, exam: ExamId) -> TimeslotId {
        *solution
            .get(&exam)
            .unwrap_or_else(|| panic!("missing exam {exam}"))
    }

    #[test]
    fn basic_solve() {
        let exam_ids = vec![1, 2, 3];
        let n_timeslots = 3;

        let scheduler = ExamScheduler::new(&exam_ids, n_timeslots);
        let solution = scheduler.solve().expect("Expected a valid solution");

        assert_eq!(solution.len(), exam_ids.len());
        for exam in exam_ids {
            let t = solved_timeslot(&solution, exam);
            assert!(t >= 0 && t < n_timeslots);
        }
    }

    #[test]
    fn add_allowed_timeslots_restricts_domain() {
        let exam_ids = vec![1];
        let mut scheduler = ExamScheduler::new(&exam_ids, 4);

        scheduler.add_allowed_timeslots(1, &[2, 3]);

        let solution = scheduler.solve().expect("Expected a valid solution");
        let t = solved_timeslot(&solution, 1);

        assert!([2, 3].contains(&t));
    }

    #[test]
    fn add_allowed_timeslots_conflict_is_infeasible() {
        let exam_ids = vec![1];
        let mut scheduler = ExamScheduler::new(&exam_ids, 4);

        scheduler.add_allowed_timeslots(1, &[0]);
        scheduler.add_disallowed_timeslots(1, &[0]);

        let err = scheduler
            .solve()
            .expect_err("Expected infeasible constraints");

        match err {
            SolverError::Infeasible {
                unsat_core_constraints,
                ..
            } => {
                assert!(unsat_core_constraints.iter().any(|c| {
                    matches!(
                        c,
                        ConstraintError::AllowedTimeslots {
                            exam: 1,
                            timeslots
                        } if timeslots == &vec![0]
                    )
                }));
                assert!(unsat_core_constraints.iter().any(|c| {
                    matches!(
                        c,
                        ConstraintError::DisallowedTimeslot {
                            exam: 1,
                            timeslot: 0
                        }
                    )
                }));
            }
            _ => panic!("Expected an infeasibility error"),
        }
    }

    #[test]
    fn add_pair_constraint_enforces_allowed_pairs() {
        let exam_ids = vec![1, 2];
        let mut scheduler = ExamScheduler::new(&exam_ids, 3);

        scheduler.add_pair_constraint(1, 2, &[(0, 1), (2, 0)]);

        let solution = scheduler.solve().expect("Expected a valid solution");
        let pair = (solved_timeslot(&solution, 1), solved_timeslot(&solution, 2));

        assert!(pair == (0, 1) || pair == (2, 0));
    }

    #[test]
    fn add_pair_constraint_conflict_is_infeasible() {
        let exam_ids = vec![1, 2];
        let mut scheduler = ExamScheduler::new(&exam_ids, 2);

        scheduler.add_pair_constraint(1, 2, &[(0, 1)]);
        scheduler.add_pair_constraint(1, 2, &[(1, 0)]);

        let err = scheduler
            .solve()
            .expect_err("Expected infeasible constraints");

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
                                exam1: 1,
                                exam2: 2,
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
        let exam_ids = vec![1];
        let mut scheduler = ExamScheduler::new(&exam_ids, 3);

        scheduler.add_disallowed_timeslots(1, &[0, 1]);

        let solution = scheduler.solve().expect("Expected a valid solution");
        assert_eq!(solved_timeslot(&solution, 1), 2);
    }

    #[test]
    fn prioritise_exam_prefers_weighted_soft_constraints() {
        let exam_ids = vec![1, 2];
        let scheduler = ExamScheduler::new(&exam_ids, 3);

        scheduler.prioritise_exam(1, &[0], 100);
        scheduler.prioritise_exam(2, &[2], 100);

        let solution = scheduler.solve().expect("Expected a valid solution");
        assert_eq!(solved_timeslot(&solution, 1), 0);
        assert_eq!(solved_timeslot(&solution, 2), 2);
    }

    #[test]
    fn separate_exam_groups_uses_mapping_property() {
        let exam_ids = vec![1, 2];
        let scheduler = ExamScheduler::new(&exam_ids, 4);

        // 0,1 -> day 0 ; 2,3 -> day 1
        let day_map = HashMap::from([(0, 0), (1, 0), (2, 1), (3, 1)]);
        scheduler.separate_exam_groups(1, 2, &day_map);

        let solution = scheduler.solve().expect("Expected a valid solution");

        let t1 = solved_timeslot(&solution, 1);
        let t2 = solved_timeslot(&solution, 2);

        assert_ne!(day_map.get(&t1), day_map.get(&t2));
    }

    #[test]
    fn setup_students_prevents_clashes() {
        let exam_ids = vec![1, 2, 3];
        let n_timeslots = 2;

        let mut scheduler = ExamScheduler::new(&exam_ids, n_timeslots);
        scheduler.setup_students(&HashMap::from([(1, vec![1, 2]), (2, vec![2, 3])]));

        let solution = scheduler.solve().expect("Expected a solution");

        let exam_1 = solved_timeslot(&solution, 1);
        let exam_2 = solved_timeslot(&solution, 2);
        let exam_3 = solved_timeslot(&solution, 3);

        assert_ne!(exam_1, exam_2, "student 1 has a clash");
        assert_ne!(exam_2, exam_3, "student 2 has a clash");
    }

    #[test]
    fn basic_fail_not_enough_timeslots() {
        let exam_ids = vec![1, 2, 3];
        let n_timeslots = 2;

        let mut scheduler = ExamScheduler::new(&exam_ids, n_timeslots);
        scheduler.setup_students(&HashMap::from([(1, vec![1, 2, 3])]));

        let errors = scheduler.solve().expect_err("Expected an error");

        match errors {
            SolverError::Infeasible {
                unsat_core_constraints,
                ..
            } => {
                assert!(unsat_core_constraints
                    .iter()
                    .any(|c| matches!(c, ConstraintError::StudentDistinct { student: 1, .. })));
            }
            _ => panic!("Expected an infeasibility error"),
        }
    }

    #[test]
    fn minimize_exams_per_day_spreads_exams_for_student() {
        let exam_ids = vec![1, 2, 3];
        let n_timeslots = 3;
        let scheduler = ExamScheduler::new(&exam_ids, n_timeslots);

        // Day 0: slots 0,1 ; Day 1: slot 2
        let days: [&[TimeslotId]; 2] = [&[0, 1], &[2]];
        let students = HashMap::from([(10, vec![1, 2])]);

        scheduler.minimize_exams_per_day(&students, &days, |_| 10);

        let solution = scheduler.solve().expect("Expected a valid solution");
        let t1 = solved_timeslot(&solution, 1);
        let t2 = solved_timeslot(&solution, 2);

        let same_day = (days[0].contains(&t1) && days[0].contains(&t2))
            || (days[1].contains(&t1) && days[1].contains(&t2));
        assert!(!same_day, "Expected exams to be spread across days");
    }

    #[test]
    fn maximize_exam_distance_pushes_exams_apart() {
        let exam_ids = vec![1, 2];
        let scheduler = ExamScheduler::new(&exam_ids, 4);

        scheduler.maximize_exam_distance(1, 2);

        let solution = scheduler.solve().expect("Expected a valid solution");
        let t1 = solved_timeslot(&solution, 1);
        let t2 = solved_timeslot(&solution, 2);

        let distance = (t1 - t2).abs();
        assert_eq!(distance, 3, "Expected maximal distance in 4 timeslots");
    }

    #[test]
    fn combined_constraints_and_optimizations_work_together() {
        let exam_ids = vec![1, 2, 3];
        let mut scheduler = ExamScheduler::new(&exam_ids, 4);

        scheduler.add_allowed_timeslots(1, &[0, 1]);
        scheduler.add_disallowed_timeslots(2, &[0]);
        scheduler.add_pair_constraint(1, 3, &[(0, 2), (1, 3)]);
        scheduler.setup_students(&HashMap::from([(42, vec![1, 2])]));
        scheduler.maximize_exam_distance(1, 2);

        let solution = scheduler.solve().expect("Expected a valid solution");

        let t1 = solved_timeslot(&solution, 1);
        let t2 = solved_timeslot(&solution, 2);
        let t3 = solved_timeslot(&solution, 3);

        assert!([0, 1].contains(&t1));
        assert_ne!(t2, 0);
        assert!(t1 != t2);

        assert!((t1 == 0 && t3 == 2) || (t1 == 1 && t3 == 3));
    }
}

// pub enum SolverSolution {
//     /// A valid schedule that satisfies all hard constraints and optimizes soft constraints as much as possible.
//     Optimal(HashMap<ExamId, TimeslotId>),
//     /// A valid schedule that satisfies all hard constraints, but may not optimize soft constraints.
//     Inoptimal(HashMap<ExamId, TimeslotId>),
// }

#[derive(Debug, thiserror::Error)]
pub enum SolverError {
    /// The query is unsatisfiable. No schedule that satisfies all hard constraints exists.
    #[error("The query is unsatisfiable. No schedule that satisfies all hard constraints exists")]
    Infeasible {
        /// Hard constraints participating in the unsat core.
        ///
        /// This is typically a minimal or near-minimal conflicting subset and is the
        /// primary signal for explaining infeasibility.
        unsat_core_constraints: Vec<ConstraintError>,
        /// Shared debug context captured when the error was produced.
        debug: SolverDebugInfo,
    },

    /// Z3 reported SAT but no model was available.
    #[error("The query was satisfiable but no model was returned")]
    NoModel {
        /// Shared debug context captured when the error was produced.
        debug: SolverDebugInfo,
    },

    /// Z3 returned a model that could not be converted to a timeslot id.
    #[error("Failed to evaluate model for exam {exam}: {details}")]
    ModelEvaluation {
        /// The exam whose model value failed to evaluate or convert.
        exam: ExamId,
        /// Human-readable details describing the model evaluation/conversion failure.
        details: String,
        /// Shared debug context captured when the error was produced.
        debug: SolverDebugInfo,
    },

    /// The query was interrupted, timed out or otherwise failed.
    #[error("The query was interrupted, timed out or otherwise failed")]
    Unknown {
        /// Optional reason returned by Z3 for the unknown result.
        reason: Option<String>,
        /// Shared debug context captured when the error was produced.
        debug: SolverDebugInfo,
    },
}
