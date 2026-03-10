use exam_timetable_model::{ExamId, StudentId, TimeslotId};
use serde::{Deserialize, Serialize};

/// Structured description of a hard constraint attached to Z3.
///
/// These values are used to map unsat-core atoms back to domain-level scheduler
/// constraints for human-readable diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintError {
    /// An exam was constrained to be assigned to a non-negative timeslot index.
    DomainLowerBound {
        /// The exam for which the lower-bound domain constraint was asserted.
        exam: ExamId,
    },
    /// An exam was constrained to be assigned to a timeslot strictly below `n_timeslots`.
    DomainUpperBound {
        /// The exam for which the upper-bound domain constraint was asserted.
        exam: ExamId,
        /// The total number of available timeslots used as the exclusive upper bound.
        n_timeslots: i64,
    },
    /// An exam was constrained to be assigned to one of a finite set of allowed timeslots.
    AllowedTimeslots {
        /// The exam whose assignment was restricted.
        exam: ExamId,
        /// The set of timeslot identifiers that are permitted for this exam.
        timeslots: Vec<TimeslotId>,
    },
    /// An exam was constrained to avoid a specific timeslot.
    DisallowedTimeslot {
        /// The exam whose assignment was restricted.
        exam: ExamId,
        /// A timeslot identifier that is forbidden for the exam.
        timeslot: TimeslotId,
    },
    /// A student's exams were constrained to all occur in distinct timeslots.
    StudentDistinct {
        /// The student for whom no-overlap exam constraints were asserted.
        student: StudentId,
        /// The exams that must be pairwise non-overlapping for the student.
        exams: Vec<ExamId>,
    },
}

/// Shared, non-classifying debug context attached to solver errors.
///
/// This struct intentionally contains only supporting details. The actual
/// failure category is represented by `SolverError`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverDebugInfo {
    /// A textual dump of the optimizer state at the time diagnostics were built.
    pub optimization_state: String,
    /// All hard constraints that were tracked and asserted in the optimizer.
    pub all_tracked_constraints: Vec<ConstraintError>,
}
