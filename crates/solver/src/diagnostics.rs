use entity::id::StudentId;
use std::collections::HashMap;

use crate::TimeslotIndex;
use entity::id::SessionId;
use serde::{Deserialize, Serialize};

/// Structured description of a hard constraint attached to Z3.
///
/// These values are used to map unsat-core atoms back to domain-level scheduler
/// constraints for human-readable diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum ConstraintError {
    /// A session was constrained to be assigned to a valid timeslot (0 <= x < n_timeslots).
    DomainBounds {
        /// The session for which the domain bounds constraint was asserted.
        session: SessionId,
        /// The total number of available timeslots.
        n_timeslots: u64,
    },
    /// A session was constrained to be assigned to one of a finite set of allowed timeslots.
    AllowedTimeslots {
        /// The session whose assignment was restricted.
        session: SessionId,
        /// The set of timeslot identifiers that are permitted for this exam.
        timeslots: Vec<TimeslotIndex>,
    },
    /// A session was constrained to avoid a specific timeslot.
    DisallowedTimeslots {
        /// The session whose assignment was restricted.
        session: SessionId,
        /// A timeslot identifier that is forbidden for the exam.
        timeslots: Vec<TimeslotIndex>,
    },
    /// A student's exams were constrained to all occur in distinct timeslots.
    StudentDistinct {
        /// The students for whom no-overlap exam constraints were asserted.
        students: Vec<StudentId>,
        /// The sessions that must be pairwise non-overlapping for the student.
        sessions: Vec<SessionId>,
    },
    /// An exam was constrained to start only at valid multi-slot windows.
    /// AI-generated (GPT-5.2-codex).
    MultiSlotStart {
        /// The session whose start time was restricted.
        session: SessionId,
        /// The number of consecutive slots required.
        slots_required: u32,
        /// Allowed start timeslots for the exam.
        allowed_starts: Vec<TimeslotIndex>,
    },
    /// Another exam was constrained to not fall inside a multi-slot block.
    BlockExclusion {
        /// The multi-slot session defining the blocked window.
        block_session: SessionId,
        /// The exam that must remain outside the block.
        other_session: SessionId,
        /// The number of consecutive slots required by the block exam.
        all_slots: Vec<SessionId>,
    },
    /// Two exams were constrained to be assigned on the same day, consecutively.
    PairConstraint {
        /// The first session in the pairwise constraint.
        session1: SessionId,
        /// The second exam in the pairwise constraint.
        session2: SessionId,
        allowed_pairs: Vec<(TimeslotIndex, TimeslotIndex)>,
    },
    /// Two exams were constrained to be assigned in the exact same timeslot.
    /// AI-generated (GPT-5.3-codex).
    SameTime {
        /// The first session in the same-time constraint.
        session1: SessionId,
        /// The second session in the same-time constraint.
        session2: SessionId,
    },
    /// Two exams were constrained to be assigned in different timeslots.
    DifferentTime {
        /// The first session in the different-time constraint.
        session1: SessionId,
        /// The second session in the different-time constraint.
        session2: SessionId,
    },
    /// The first session was constrained to run before the second session.
    Before {
        /// The first session in the before constraint.
        first_session_id: SessionId,
        /// The second session in the before constraint.
        second_session_id: SessionId,
    },
    /// The second session was constrained to run immediately after the first session.
    Consecutive {
        /// The first session.
        first_session_id: SessionId,
        /// The second session (must be exactly first + 1).
        second_session_id: SessionId,
    },
    /// A group of exams was constrained to be scheduled in separate timeslots, but were assigned to the same one.
    SeparateExamGroups {
        session1: SessionId,
        session2: SessionId,
        mapping: HashMap<TimeslotIndex, u64>,
    },
}
