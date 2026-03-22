//! Diesel models for the `enrolled_student` join table.

use crate::id::{StudentId, SubjectId};
use crate::schema::enrolled_student;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    diesel::Queryable,
    diesel::Selectable,
    diesel::Identifiable,
    diesel::Associations,
    diesel::Insertable,
)]
#[diesel(table_name = enrolled_student)]
#[diesel(primary_key(student_id, subject_id))]
#[diesel(belongs_to(crate::students::Student, foreign_key = student_id))]
#[diesel(belongs_to(crate::subjects::Subject, foreign_key = subject_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct EnrolledStudent {
    pub student_id: StudentId,
    pub subject_id: SubjectId,
}
