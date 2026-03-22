//! Diesel models for the `exam` table.

use crate::id::{ExamId, SubjectId};
use crate::schema::exam;

#[derive(
    Debug,
    Clone,
    PartialEq,
    diesel::Queryable,
    diesel::Selectable,
    diesel::Identifiable,
    diesel::Associations,
)]
#[diesel(table_name = exam)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(belongs_to(crate::subjects::Subject, foreign_key = subject_id))]
pub struct Exam {
    pub id: ExamId,
    pub subject_id: SubjectId,
    pub paper: i32,
    pub duration_hours: f32,
    pub priority: i32,
    pub slots_required: i32,
}

#[derive(Debug, Clone, PartialEq, diesel::Insertable)]
#[diesel(table_name = exam)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewExam {
    pub subject_id: SubjectId,
    pub paper: i32,
    pub duration_hours: f32,
    pub priority: i32,
    pub slots_required: i32,
}
