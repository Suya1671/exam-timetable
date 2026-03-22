//! Diesel models for the `same_day_exam` table.

use crate::id::ExamId;
use crate::schema::same_day_exam;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    diesel::Queryable,
    diesel::Selectable,
    diesel::Identifiable,
    diesel::Insertable,
)]
#[diesel(table_name = same_day_exam)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(first_slot_exam_id, second_slot_exam_id))]
pub struct SameDayExam {
    pub first_slot_exam_id: ExamId,
    pub second_slot_exam_id: ExamId,
    pub date: String,
}
