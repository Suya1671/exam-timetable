use crate::id::ExamId;
use crate::schema::exam_order_constraint;

#[derive(
    Debug, Clone, PartialEq, Eq, diesel::Queryable, diesel::Selectable, diesel::Identifiable,
)]
#[diesel(table_name = exam_order_constraint)]
#[diesel(primary_key(exam1_id, exam2_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ExamOrderConstraint {
    pub exam1_id: ExamId,
    pub exam2_id: ExamId,
}
