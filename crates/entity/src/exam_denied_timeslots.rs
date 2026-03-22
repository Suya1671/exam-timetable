//! Diesel models for the `exam_denied_timeslot` join table.

use crate::id::{ExamId, TimeslotId};
use crate::schema::exam_denied_timeslot;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    diesel::Queryable,
    diesel::Selectable,
    diesel::Identifiable,
    diesel::Insertable,
    diesel::Associations,
)]
#[diesel(table_name = exam_denied_timeslot)]
#[diesel(primary_key(exam_id, timeslot_id))]
#[diesel(belongs_to(crate::exams::Exam, foreign_key = exam_id))]
#[diesel(belongs_to(crate::timeslots::Timeslot, foreign_key = timeslot_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ExamDeniedTimeslot {
    pub exam_id: ExamId,
    pub timeslot_id: TimeslotId,
}
