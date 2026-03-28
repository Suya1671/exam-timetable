use crate::id::{TimeslotId, TimeslotSlot};
use crate::schema::timeslot;
use time::{Date, Time};

#[derive(
    Debug, Clone, PartialEq, Eq, diesel::Queryable, diesel::Selectable, diesel::Identifiable,
)]
#[diesel(table_name = timeslot)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Timeslot {
    pub id: TimeslotId,
    pub date: Date,
    pub slot: TimeslotSlot,
    pub start_time: Time,
}

/// AI-generated (GPT-5.3-codex).
pub struct NewTimeslot {
    pub date: Date,
    pub slot: TimeslotSlot,
    pub start_time: Time,
}
