//! Diesel models for the `student` table.

use crate::id::StudentId;
use crate::schema::student;

/// AI-generated (GPT-5.2-codex).
#[derive(
    Debug, Clone, PartialEq, Eq, diesel::Queryable, diesel::Selectable, diesel::Identifiable,
)]
#[diesel(table_name = student)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Student {
    pub id: StudentId,
    pub name: String,
    pub grade: i32,
}
