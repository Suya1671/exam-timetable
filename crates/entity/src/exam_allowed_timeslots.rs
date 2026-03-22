//! Diesel models for the `exam_allowed_timeslot` table.

use crate::id::{ExamId, TimeslotId};
use crate::schema::exam_allowed_timeslot;

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
#[diesel(table_name = exam_allowed_timeslot)]
#[diesel(primary_key(exam_id, timeslot_id))]
#[diesel(belongs_to(crate::exams::Exam, foreign_key = exam_id))]
#[diesel(belongs_to(crate::timeslots::Timeslot, foreign_key = timeslot_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ExamAllowedTimeslot {
    pub exam_id: ExamId,
    pub timeslot_id: TimeslotId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, diesel::Insertable)]
#[diesel(table_name = exam_allowed_timeslot)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewExamAllowedTimeslot {
    pub exam_id: ExamId,
    pub timeslot_id: TimeslotId,
}
