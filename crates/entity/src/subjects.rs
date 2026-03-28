//! Diesel models for the `subject` table.

use crate::id::SubjectId;
use crate::schema::subject;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    diesel::Queryable,
    diesel::Selectable,
    diesel::Identifiable,
    serde::Serialize,
)]
#[diesel(table_name = subject)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Subject {
    pub id: SubjectId,
    pub name: String,
}
