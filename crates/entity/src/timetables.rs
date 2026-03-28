//! Diesel models for the `timetables` table.

use crate::id::TimetableId;
use crate::schema::timetables;

/// AI-generated (GPT-5.3-codex).
#[derive(
    Debug, Clone, PartialEq, Eq, diesel::Queryable, diesel::Selectable, diesel::Identifiable,
)]
#[diesel(table_name = timetables)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Timetable {
    pub id: TimetableId,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}
