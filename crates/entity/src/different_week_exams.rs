use crate::id::ExamId;
use crate::schema::different_week_exams;

#[derive(
    Debug, Clone, PartialEq, Eq, diesel::Queryable, diesel::Selectable, diesel::Identifiable,
)]
#[diesel(table_name = different_week_exams)]
#[diesel(primary_key(exam1_id, exam2_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DifferentWeekExam {
    pub exam1_id: ExamId,
    pub exam2_id: ExamId,
}
