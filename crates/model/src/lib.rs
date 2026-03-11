//! This crate contains the database models for the exam timetable application.

#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![warn(missing_docs)]
// #![allow(clippy::multiple_crate_versions)]

use time::Date;

/// Identifier for an exam.
pub type ExamId = i64;

/// Identifier for a subject.
pub type SubjectId = i64;

/// Identifier for a timeslot.
pub type TimeslotId = i64;

/// Identifier for a student.
pub type StudentId = i64;

/// Represents an academic subject offered in the timetable.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Subject {
    /// Unique identifier for the subject.
    pub id: SubjectId,
    /// Human-readable name of the subject (e.g. "Mathematics").
    pub name: String,
    /// Grade for which the subject is taught.
    pub grade: u8,
}

/// Represents a single exam session for a specific grade and subject.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Exam {
    /// Unique identifier for the exam.
    pub id: ExamId,
    /// Identifier of the subject for this exam.
    pub subject_id: SubjectId,
    /// The paper number of the exam. e.g. Math P1 = paper 1, Math P2 = paper 2.
    pub paper: u8,
    /// The duration of the exam in hours.
    pub duration_hours: f32,
    /// Soft priority for scheduling.
    ///
    /// Higher priority exams should have their soft constraints (e.g. preferred days) more strongly enforced than lower priority exams.
    pub priority: u8, // Grade 12 = higher priority
    /// Number of consecutive timeslots required for the exam.
    pub slots_required: u32,
}

/// Represents a student that participates in exams.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Student {
    /// Unique identifier for the student.
    pub id: StudentId,
    /// Full name of the student.
    pub name: String,
    /// Grade of the student.
    pub grade: u8,
}

/// Associates a student with a subject they are enrolled in.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnrolledStudent {
    /// Identifier of the student.
    pub student_id: StudentId,
    /// Identifier of the subject.
    pub subject_id: SubjectId,
}

/// Associates an exam with a timeslot it is explicitly allowed to use.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExamAllowedTimeslot {
    /// Identifier of the exam.
    pub exam_id: ExamId,
    /// Identifier of the allowed timeslot.
    pub timeslot_id: TimeslotId,
}

/// Associates an exam with a timeslot it is explicitly denied from using.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExamDeniedTimeslot {
    /// Identifier of the exam.
    pub exam_id: ExamId,
    /// Identifier of the denied timeslot.
    pub timeslot_id: TimeslotId,
}

/// Associates a student with a timeslot they are explicitly allowed to use.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StudentAllowedTimeslot {
    /// Identifier of the student.
    pub student_id: StudentId,
    /// Identifier of the allowed timeslot.
    pub timeslot_id: TimeslotId,
}

/// Associates a student with a timeslot they are explicitly denied from using.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StudentDeniedTimeslot {
    /// Identifier of the student.
    pub student_id: StudentId,
    /// Identifier of the denied timeslot.
    pub timeslot_id: TimeslotId,
}

/// Associates a subject with a timeslot it is explicitly allowed to use.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubjectAllowedTimeslot {
    /// Identifier of the subject.
    pub subject_id: SubjectId,
    /// Identifier of the allowed timeslot.
    pub timeslot_id: TimeslotId,
}

/// Associates a subject with a timeslot it is explicitly denied from using.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubjectDeniedTimeslot {
    /// Identifier of the subject.
    pub subject_id: SubjectId,
    /// Identifier of the denied timeslot.
    pub timeslot_id: TimeslotId,
}

/// Represents a specific timeslot during which exams can be scheduled.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Timeslot {
    /// Unique identifier for the timeslot.
    pub id: TimeslotId,
    /// Calendar date of the timeslot.
    pub date: Date,
    /// The slot within the day (first or second).
    pub slot: DaySlot,
}

/// Constrains two exams to happen on the same calendar day.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SameDayExam {
    /// Identifier of the first exam.
    pub exam1_id: ExamId,
    /// Identifier of the second exam.
    pub exam2_id: ExamId,
    /// Required calendar date for both exams.
    pub date: Date,
}

/// Constrains two exams to happen in different calendar weeks.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DifferentWeekExam {
    /// Identifier of the first exam.
    pub exam1_id: ExamId,
    /// Identifier of the second exam.
    pub exam2_id: ExamId,
}

/// A day is divided into two possible slots for exams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum DaySlot {
    /// The first slot of the day, typically the morning session.
    First = 0,
    /// The second slot of the day, typically the afternoon session.
    Second,
}

impl From<DaySlot> for u8 {
    fn from(value: DaySlot) -> Self {
        match value {
            DaySlot::First => 0,
            DaySlot::Second => 1,
        }
    }
}
