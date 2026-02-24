mod int_extensions;

use std::collections::HashMap;

use exam_timetable_model::{ExamId, StudentId, TimeslotId};
use z3::{
    ast::{Ast, Bool, Int},
    Optimize, SatResult,
};

use crate::int_extensions::IntExtensions;

pub struct SchedulerStudent {
    id: StudentId,
    exams: Vec<ExamId>,
}

// implementation notes
// - Always ensure that timeslots are ordered by their date and then by slot number to ensure consistent variable indexing

/// An exam scheduler
///
/// Note: the information used in functions _must not change_
/// i.e. you cannot just add a new timeslot in the middle of setting up the scheduler
pub struct ExamScheduler {
    /// The Z3 solver/optimizer instance
    optimizer: Optimize,

    /// The decision variable: one [Int] per exam, value = timeslot index
    ///
    /// assignment\[exam_id] = timeslot_id
    assignment: HashMap<ExamId, Int>,
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
            .map(|exam| {
                let var = Int::fresh_const(&format!("exam_{}", exam));

                // Domain constraint: each exam must be assigned to a valid timeslot (i.e. within index bounds)
                optimizer.assert(&var.ge(Int::from_i64(0)));
                optimizer.assert(&var.lt(Int::from_i64(n_timeslots)));

                (*exam, var)
            })
            .collect();

        Self {
            optimizer,
            assignment,
        }
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
    pub fn add_allowed_timeslots(&self, exam: ExamId, allowed_timeslots: &[TimeslotId]) {
        let exam_timeslot = self.assignment.get(&exam).unwrap();

        // Constraint: the exam must be scheduled in one of the allowed timeslots
        self.optimizer.assert(&Bool::or(
            &allowed_timeslots
                .iter()
                .map(|timeslot| exam_timeslot.eq(Int::from_i64(*timeslot)))
                .collect::<Box<[_]>>(),
        ));
    }

    /// Add disallowed timeslots for an exam
    ///
    /// # Params
    /// - exam: the exam to add the constraint for
    /// - disallowed_timeslots: the list of timeslot indices that the exam cannot be
    ///
    /// # Constraints setup
    /// - The exam must not be scheduled in any of the disallowed timeslots
    pub fn add_disallowed_timeslots(&self, exam: ExamId, disallowed_timeslots: &[TimeslotId]) {
        let exam_timeslot = self.assignment.get(&exam).unwrap();

        for timeslot in disallowed_timeslots {
            // Constraint: the exam must not be scheduled in any of the disallowed timeslots
            self.optimizer
                .assert(&exam_timeslot.eq(Int::from_i64(*timeslot)).not());
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
    pub async fn prioritise_exam(&self, exam: ExamId, timeslots: &[TimeslotId], priority: i64) {
        let exam_timeslot = self.assignment.get(&exam).unwrap();

        for timeslot in timeslots {
            // Soft constraint: the exam should be scheduled in the preferred timeslot
            self.optimizer
                .assert_soft(&exam_timeslot.eq(Int::from_i64(*timeslot)), priority, None);
        }
    }

    /// Add a soft constraint to minimize the number of exams a student has on the same day
    ///
    /// # Params
    /// - students: the list of students to consider for this constraint
    /// - days: the list of days, where each day is a list of timeslots that belong to the same day
    /// - student_weight: a function that takes a student and returns a weight for how important it is to minimize the number of exams on the same day for that student
    ///
    /// # Constraints setup
    pub fn minimize_exams_per_day(
        &self,
        students: &[SchedulerStudent],
        days: &[&[TimeslotId]],
        student_weight: impl Fn(&SchedulerStudent) -> u64,
    ) {
        let total_penalty: Int = students
            .iter()
            .flat_map(|student| {
                let weight = student_weight(student) as i64;
                days.iter().map(move |day_timeslots| {
                    let count = self.exams_on_day(student, day_timeslots);

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

    /// Count the number of exams a student has on a given day
    ///
    /// # Params
    /// - student: the student to count the exams for
    /// - day_timeslots: the list of timeslot indices that belong to the same day
    ///
    /// # Returns
    /// - The number of exams the student has on the given day
    fn exams_on_day(&self, student: &SchedulerStudent, day_timeslots: &[TimeslotId]) -> Int {
        student
            .exams
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
    pub fn setup_students(&self, students: &[SchedulerStudent]) {
        for student in students {
            let exams = student
                .exams
                .iter()
                .map(|exam| self.assignment.get(exam).unwrap())
                .collect::<Box<[_]>>();

            // Constraint: a student cannot have two exams in the same timeslot
            // Therefore, all exams taken by a student must be in different timeslots
            self.optimizer.assert(&Int::distinct(&exams));
        }
    }

    pub fn solve(&self) -> Result<HashMap<ExamId, TimeslotId>, SolverError> {
        match self.optimizer.check(&[]) {
            SatResult::Sat => {
                let model = self.optimizer.get_model().unwrap();
                let result = self
                    .assignment
                    .iter()
                    .map(|(&exam, var)| {
                        let timeslot =
                            model.eval(var, true).unwrap().as_i64().unwrap() as TimeslotId;
                        (exam, timeslot)
                    })
                    .collect();
                Ok(result)
            }
            SatResult::Unsat => {
                let core = self.optimizer.get_unsat_core();
                let violated = core.iter().map(|b| b.to_string()).collect();
                Err(SolverError::Infeasible(violated))
            }
            SatResult::Unknown => Err(SolverError::Unknown),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // async fn create_testing_pool() -> SqlitePool {
    //     let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    //     sqlx::migrate!("../../migrations").run(&pool).await.unwrap();

    //     // example data
    //     sqlx::query!(
    //         r#"
    //         INSERT INTO subjects (id, name, grade) VALUES
    //             (1, 'Mathematics', 10),
    //             (2, 'Physics', 10),
    //             (3, 'Chemistry', 10)
    //         "#
    //     )
    //     .execute(&pool)
    //     .await
    //     .unwrap();

    //     pool
    // }

    #[test]
    fn basic_solve() {
        let exam_ids = vec![1, 2, 3];
        let n_timeslots = 3;

        let scheduler = ExamScheduler::new(&exam_ids, n_timeslots);
        assert!(scheduler.solve().is_ok());
    }

    #[test]
    fn basic_with_students() {
        let exam_ids = vec![1, 2, 3];
        let n_timeslots = 2;

        let scheduler = ExamScheduler::new(&exam_ids, n_timeslots);
        scheduler.setup_students(&[
            SchedulerStudent {
                id: 1,
                exams: vec![1, 2],
            },
            SchedulerStudent {
                id: 2,
                exams: vec![2, 3],
            },
        ]);

        for (exam, time) in scheduler.solve().expect("Expected a solution") {
            match exam {
                1 => assert_eq!(time, 0),
                2 => assert_eq!(time, 1),
                3 => assert_eq!(time, 0),
                _ => panic!("Unexpected exam id"),
            }
        }
    }

    #[test]
    fn basic_fail_not_enough_timeslots() {
        let exam_ids = vec![1, 2, 3];
        let n_timeslots = 2;

        let scheduler = ExamScheduler::new(&exam_ids, n_timeslots);
        scheduler.setup_students(&[SchedulerStudent {
            id: 1,
            exams: vec![1, 2, 3],
        }]);

        let errors = scheduler.solve().expect_err("Expected an error");

        match errors {
            SolverError::Infeasible(violated) => {
                dbg!(&violated);
                todo!("Check that the violated constraints contain the distinct constraint for the student's exams");
                // assert!(violated.iter().any(|c| c.contains("distinct")));
            }
            _ => panic!("Expected an infeasibility error"),
        }
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
    #[error(
        "The query is unsatisfiable. No schedule that satisfies all hard constraints exists: {0:?}"
    )]
    Infeasible(Vec<String>),
    /// The query was interrupted, timed out or otherwise failed.
    #[error("The query was interrupted, timed out or otherwise failed")]
    Unknown,
}
