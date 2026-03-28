//! Diesel models for the `timetable_slots` table.

use crate::id::{SessionId, TimeslotId, TimetableId};
use crate::schema::timetable_slots;
use time::Time;

/// AI-generated (GPT-5.3-codex).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    diesel::Queryable,
    diesel::Selectable,
    diesel::Identifiable,
    diesel::Associations,
)]
#[diesel(table_name = timetable_slots)]
#[diesel(primary_key(timetable_id, session_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(belongs_to(crate::timetables::Timetable, foreign_key = timetable_id))]
#[diesel(belongs_to(crate::sessions::Session, foreign_key = session_id))]
#[diesel(belongs_to(crate::timeslots::Timeslot, foreign_key = timeslot_id))]
pub struct TimetableSlot {
    pub timetable_id: TimetableId,
    pub session_id: SessionId,
    pub timeslot_id: TimeslotId,
    pub locked: bool,
    pub reading_start_time: Option<Time>,
    pub exam_start_time: Option<Time>,
    pub exam_end_time: Option<Time>,
}
