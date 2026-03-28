//! Diesel models for the `exam_timeslot_restriction` join table.

use crate::id::{ExamId, TimeslotId};
use crate::schema::exam_timeslot_restriction;

/// AI-generated (GPT-5.2-codex).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    diesel::Queryable,
    diesel::Selectable,
    diesel::Identifiable,
    diesel::Associations,
)]
#[diesel(table_name = exam_timeslot_restriction)]
#[diesel(primary_key(exam_id, timeslot_id))]
#[diesel(belongs_to(crate::exams::Exam, foreign_key = exam_id))]
#[diesel(belongs_to(crate::timeslots::Timeslot, foreign_key = timeslot_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ExamTimeslotRestriction {
    pub exam_id: ExamId,
    pub timeslot_id: TimeslotId,
}
