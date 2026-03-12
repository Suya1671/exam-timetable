use sea_orm::{DeriveActiveEnum, DeriveValueType, EnumIter};

/// Identifier for an exam.
/// AI-generated (GPT-5.2-codex).
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
    DeriveValueType,
    derive_more::From,
    derive_more::Into,
)]
pub struct ExamId(pub i64);

/// Identifier for a session.
/// AI-generated (GPT-5.2-codex).
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
    DeriveValueType,
    derive_more::From,
    derive_more::Into,
)]
pub struct SessionId(pub i64);

/// Identifier for a subject.
/// AI-generated (GPT-5.2-codex).
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
    DeriveValueType,
    derive_more::From,
    derive_more::Into,
)]
pub struct SubjectId(pub i64);

/// Identifier for a timeslot.
/// AI-generated (GPT-5.2-codex).
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
    DeriveValueType,
    derive_more::From,
    derive_more::Into,
)]
pub struct TimeslotId(pub i64);

/// Identifier for a student.
/// AI-generated (GPT-5.2-codex).
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
    DeriveValueType,
    derive_more::From,
    derive_more::Into,
)]
pub struct StudentId(pub i64);

/// Timeslot slot marker.
/// AI-generated (GPT-5.2-codex).
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "u8", db_type = "Integer")]
pub enum TimeslotSlot {
    #[sea_orm(num_value = 0)]
    First,
    #[sea_orm(num_value = 1)]
    Second,
}
