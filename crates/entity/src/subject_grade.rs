use crate::{id::SubjectId, schema::subject_grade, subjects::Subject};

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    diesel::Queryable,
    diesel::Selectable,
    diesel::Identifiable,
    diesel::Associations,
    serde::Serialize,
    serde::Deserialize,
    specta::Type,
)]
#[diesel(table_name = subject_grade)]
#[diesel(belongs_to(Subject))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(subject_id, grade))]
pub struct SubjectGrade {
    pub subject_id: SubjectId,
    pub grade: i32,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    diesel::Insertable,
    serde::Deserialize,
    serde::Serialize,
    specta::Type,
)]
#[diesel(table_name = subject_grade)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewSubjectGrade {
    pub subject_id: SubjectId,
    pub grade: i32,
}
