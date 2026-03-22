use diesel::{
    backend::Backend,
    deserialize::FromSql,
    deserialize::FromSqlRow,
    serialize::{Output, ToSql},
    sql_types::Integer,
    AsExpression,
};
use diesel_derive_newtype::DieselNewType;

/// Identifier for an exam.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    DieselNewType,
    derive_more::From,
    derive_more::Into,
)]
pub struct ExamId(pub i32);

/// Identifier for a session.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    derive_more::Display,
    DieselNewType,
    derive_more::From,
    derive_more::Into,
)]
pub struct SessionId(pub i32);

/// Identifier for a subject.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    DieselNewType,
    derive_more::From,
    derive_more::Into,
)]
pub struct SubjectId(pub i32);

/// Identifier for a timeslot.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    DieselNewType,
    derive_more::From,
    derive_more::Into,
)]
pub struct TimeslotId(pub i32);

/// Identifier for a student.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    DieselNewType,
    derive_more::From,
    derive_more::Into,
)]
pub struct StudentId(pub i32);

/// Timeslot slot marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsExpression, FromSqlRow)]
#[diesel(sql_type = Integer)]
#[repr(u8)]
pub enum TimeslotSlot {
    First,
    Second,
}

impl<DB> ToSql<Integer, DB> for TimeslotSlot
where
    DB: Backend,
    i32: ToSql<Integer, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> diesel::serialize::Result {
        static FIRST_VALUE: i32 = 0;
        static SECOND_VALUE: i32 = 1;

        let value: &i32 = match self {
            TimeslotSlot::First => &FIRST_VALUE,
            TimeslotSlot::Second => &SECOND_VALUE,
        };

        <i32 as ToSql<Integer, DB>>::to_sql(value, out)
    }
}

impl<DB> FromSql<Integer, DB> for TimeslotSlot
where
    DB: Backend,
    i32: FromSql<Integer, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let value = i32::from_sql(bytes)?;
        match value {
            0 => Ok(TimeslotSlot::First),
            1 => Ok(TimeslotSlot::Second),
            _ => Err(format!("invalid timeslot slot value: {value}").into()),
        }
    }
}
