//! Diesel models for the `subject` table.

use crate::id::SubjectId;
use crate::schema::subject;

/// AI-generated (GPT-5.2-codex).
#[derive(
    Debug, Clone, PartialEq, Eq, diesel::Queryable, diesel::Selectable, diesel::Identifiable,
)]
#[diesel(table_name = subject)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Subject {
    pub id: SubjectId,
    pub name: String,
    pub grade: i32,
}

/// AI-generated (GPT-5.2-codex).
#[derive(Debug, Clone, PartialEq, Eq, diesel::Insertable)]
#[diesel(table_name = subject)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewSubject {
    pub name: String,
    pub grade: i32,
}
