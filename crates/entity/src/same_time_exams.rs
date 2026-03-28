//! Diesel models for the `same_time_exam` table.

use crate::id::ExamId;
use crate::schema::same_time_exam;

/// AI-generated (GPT-5.3-codex).
#[derive(
    Debug, Clone, PartialEq, Eq, diesel::Queryable, diesel::Selectable, diesel::Identifiable,
)]
#[diesel(table_name = same_time_exam)]
#[diesel(primary_key(exam1_id, exam2_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SameTimeExam {
    pub exam1_id: ExamId,
    pub exam2_id: ExamId,
}
