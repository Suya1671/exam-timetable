//! Diesel models for the `exam` table.

use crate::id::{ExamId, SubjectId};
use crate::schema::exam;

/// AI-generated (GPT-5.2-codex).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    diesel::AsExpression,
    diesel::FromSqlRow,
    serde::Serialize,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum TimeslotRestrictionMode {
    /// AI-generated (GPT-5.2-codex).
    Allow,
    /// AI-generated (GPT-5.2-codex).
    Deny,
}

impl diesel::serialize::ToSql<diesel::sql_types::Text, diesel::sqlite::Sqlite>
    for TimeslotRestrictionMode
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::sqlite::Sqlite>,
    ) -> diesel::serialize::Result {
        out.set_value(match self {
            TimeslotRestrictionMode::Allow => "allow",
            TimeslotRestrictionMode::Deny => "deny",
        });
        Ok(diesel::serialize::IsNull::No)
    }
}

impl diesel::deserialize::FromSql<diesel::sql_types::Text, diesel::sqlite::Sqlite>
    for TimeslotRestrictionMode
{
    fn from_sql(
        value: diesel::sqlite::SqliteValue<'_, '_, '_>,
    ) -> diesel::deserialize::Result<Self> {
        let value = <String as diesel::deserialize::FromSql<
            diesel::sql_types::Text,
            diesel::sqlite::Sqlite,
        >>::from_sql(value)?;

        match value.as_str() {
            "allow" => Ok(Self::Allow),
            "deny" => Ok(Self::Deny),
            _ => Err(format!("Invalid timeslot restriction mode: {value}").into()),
        }
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    diesel::Queryable,
    diesel::Selectable,
    diesel::Identifiable,
    diesel::Associations,
    serde::Serialize,
)]
#[diesel(table_name = exam)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(belongs_to(crate::subjects::Subject))]
#[diesel(belongs_to(crate::subject_grade::SubjectGrade, foreign_key = subject_id))]
pub struct Exam {
    pub id: ExamId,
    pub subject_id: SubjectId,
    pub grade: i32,
    pub paper: i32,
    pub duration_hours: f32,
    pub priority: i32,
    pub slots_required: i32,
    pub timeslot_restriction_mode: Option<TimeslotRestrictionMode>,
}
