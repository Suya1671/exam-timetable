use crate::id::{TimeslotId, TimeslotSlot};
use crate::schema::timeslot;

#[derive(
    Debug, Clone, PartialEq, Eq, diesel::Queryable, diesel::Selectable, diesel::Identifiable,
)]
#[diesel(table_name = timeslot)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Timeslot {
    pub id: TimeslotId,
    pub date: String,
    pub slot: TimeslotSlot,
}

#[derive(Debug, Clone, PartialEq, Eq, diesel::Insertable)]
#[diesel(table_name = timeslot)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewTimeslot {
    pub date: String,
    pub slot: TimeslotSlot,
}
