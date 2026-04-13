use crate::id::ExamId;
use crate::schema::exam_constraint;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::Output;
use diesel::serialize::ToSql;
use diesel::sql_types::Text;
use diesel::AsExpression;
use diesel::FromSqlRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum ExamConstraintType {
    SameDay,
    DifferentDay,
    SameWeek,
    DifferentWeek,
    DifferentTime,
    SameTime,
    Before,
}

impl<DB> ToSql<Text, DB> for ExamConstraintType
where
    DB: Backend,
    str: ToSql<Text, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> diesel::serialize::Result {
        let value = match self {
            ExamConstraintType::SameDay => "same_day",
            ExamConstraintType::DifferentDay => "different_day",
            ExamConstraintType::SameWeek => "same_week",
            ExamConstraintType::DifferentWeek => "different_week",
            ExamConstraintType::SameTime => "same_time",
            ExamConstraintType::DifferentTime => "different_time",
            ExamConstraintType::Before => "before",
        };

        <str as ToSql<Text, DB>>::to_sql(value, out)
    }
}

impl<DB> FromSql<Text, DB> for ExamConstraintType
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let value = String::from_sql(bytes)?;
        match value.as_str() {
            "same_day" => Ok(ExamConstraintType::SameDay),
            "different_day" => Ok(ExamConstraintType::DifferentDay),
            "same_week" => Ok(ExamConstraintType::SameWeek),
            "different_week" => Ok(ExamConstraintType::DifferentWeek),
            "same_time" => Ok(ExamConstraintType::SameTime),
            "different_time" => Ok(ExamConstraintType::DifferentTime),
            "before" => Ok(ExamConstraintType::Before),
            _ => Err(format!("invalid exam constraint type: {}", value).into()),
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, diesel::Queryable, diesel::Selectable, diesel::Identifiable,
)]
#[diesel(table_name = exam_constraint)]
#[diesel(primary_key(exam1_id, exam2_id, constraint_type))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ExamConstraint {
    pub exam1_id: ExamId,
    pub exam2_id: ExamId,
    pub constraint_type: ExamConstraintType,
}
