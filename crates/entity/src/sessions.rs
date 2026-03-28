//! Diesel models for the `session` table.

use crate::id::{ExamId, SessionId};
use crate::schema::session;

/// AI-generated (GPT-5.2-codex).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    diesel::Queryable,
    diesel::Selectable,
    diesel::Identifiable,
    diesel::Associations,
)]
#[diesel(table_name = session)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(belongs_to(crate::exams::Exam, foreign_key = exam_id))]
pub struct Session {
    pub id: SessionId,
    pub exam_id: ExamId,
    pub sequence: i32,
}
