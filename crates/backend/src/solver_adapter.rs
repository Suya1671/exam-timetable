use std::collections::HashMap;

use diesel::{ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, SqliteConnection};
use entity::id::{SessionId, TimeslotId};
use entity::schema::{session, timeslot};
use itertools::Itertools;
use solver::{ExamScheduler, TimeslotIndex as SolverTimeslotIndex};

/// Chronological timeslot ordering plus a symbolic position map.
///
/// `ordered` is used to build consecutive windows, while `positions` is used
/// to build Z3 expressions that reason about "between" relationships without
/// relying on TimeslotId ordering.
struct TimeslotIndex {
    /// Chronologically ordered list of timeslot IDs.
    ordered: Vec<TimeslotId>,
    /// Mapping from timeslot ID to its chronological position index.
    positions: HashMap<TimeslotId, u64>,
}

impl TimeslotIndex {
    /// Build a chronological index for timeslots.
    pub fn build_index(db: &mut SqliteConnection) -> QueryResult<TimeslotIndex> {
        let mut ordered = Vec::new();
        let mut positions = HashMap::new();

        let rows = timeslot::table
            .order((timeslot::date.asc(), timeslot::slot.asc()))
            .select(timeslot::id)
            .load_iter(db)?;

        for row in rows {
            let id = row?;

            positions.insert(id, ordered.len() as u64);
            ordered.push(id);
        }

        Ok(TimeslotIndex { ordered, positions })
    }
}

/// Helper for mapping DB IDs to solver IDs and back.
pub struct SolverAdapter {
    session_ids: Vec<SessionId>,
    timeslot_index: TimeslotIndex,
}

impl SolverAdapter {
    /// Build solver sessions and mapping tables for exams/timeslots.
    pub fn new(db: &mut SqliteConnection) -> QueryResult<SolverAdapter> {
        let session_ids = session::table
            .order((session::exam_id.asc(), session::sequence.asc()))
            .select(session::id)
            .load(db)?;

        let timeslot_index = TimeslotIndex::build_index(db)?;

        Ok(SolverAdapter {
            session_ids,
            timeslot_index,
        })
    }

    /// Map database timeslot ids to solver timeslot indices.
    ///
    /// Uses the chronological position map from `BackendTimeslotIndex` so ordering is stable.
    fn map_timeslots_to_indices(
        &self,
        timeslots: impl Iterator<Item = TimeslotId>,
    ) -> Vec<SolverTimeslotIndex> {
        timeslots
            .map(|timeslot| {
                let pos = self
                    .timeslot_index
                    .positions
                    .get(&timeslot)
                    .expect("timeslot missing from positions map");
                SolverTimeslotIndex(*pos)
            })
            .collect()
    }

    /// Map a single timeslot id to a solver timeslot index.
    pub fn timeslot_index_for_id(&self, timeslot_id: TimeslotId) -> SolverTimeslotIndex {
        let pos = self
            .timeslot_index
            .positions
            .get(&timeslot_id)
            .expect("timeslot missing from positions map");
        SolverTimeslotIndex(*pos)
    }

    /// Build all same-day solver timeslot pairs.
    /// AI-generated (GPT-5.2-codex).
    pub fn day_pairs<I>(&self, day_groups: I) -> Vec<(SolverTimeslotIndex, SolverTimeslotIndex)>
    where
        I: IntoIterator<Item = Vec<TimeslotId>>,
    {
        day_groups
            .into_iter()
            .map(|timeslots| self.map_timeslots_to_indices(timeslots.iter().copied()))
            .flat_map(|timeslots| timeslots.into_iter().tuple_combinations())
            .collect()
    }

    /// Map timeslot week groupings to solver indices.
    /// AI-generated (GPT-5.2-codex).
    pub fn week_map<I>(&self, entries: I) -> HashMap<SolverTimeslotIndex, u64>
    where
        I: IntoIterator<Item = (TimeslotId, u8)>,
    {
        entries
            .into_iter()
            .map(|(timeslot_id, week)| (self.timeslot_index_for_id(timeslot_id), week.into()))
            .collect()
    }

    /// Map solver results back to session/timeslot ids.
    ///
    /// Uses the solver index to select from the ordered timeslot list.
    /// AI-generated (GPT-5.2-codex).
    pub fn map_solution(
        &self,
        results: HashMap<SessionId, SolverTimeslotIndex>,
    ) -> HashMap<SessionId, TimeslotId> {
        results
            .into_iter()
            .filter_map(|(session, index)| {
                self.timeslot_index
                    .ordered
                    .get(index.0 as usize)
                    .copied()
                    .map(|timeslot| (session, timeslot))
            })
            .collect()
    }

    /// Apply allowed/disallowed timeslots for a session.
    /// AI-generated (GPT-5.2-codex).
    pub fn apply_timeslot_restrictions(
        &self,
        scheduler: &mut ExamScheduler,
        session_id: SessionId,
        allowed_timeslots: impl Iterator<Item = TimeslotId>,
        disallowed_timeslots: impl Iterator<Item = TimeslotId>,
    ) {
        let timeslot_indices = self.map_timeslots_to_indices(allowed_timeslots);
        if !timeslot_indices.is_empty() {
            scheduler.add_allowed_timeslots(session_id, timeslot_indices);
        }

        let timeslot_indices = self.map_timeslots_to_indices(disallowed_timeslots);
        if !timeslot_indices.is_empty() {
            scheduler.add_disallowed_timeslots(session_id, timeslot_indices);
        }
    }

    pub fn session_ids(&self) -> &[SessionId] {
        &self.session_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_timeslots_to_solver_indices_uses_positions() {
        let timeslots = [TimeslotId(7), TimeslotId(3), TimeslotId(9)];
        let timeslot_index = TimeslotIndex {
            ordered: vec![TimeslotId(3), TimeslotId(9), TimeslotId(7)],
            positions: HashMap::from([(TimeslotId(7), 2), (TimeslotId(3), 0), (TimeslotId(9), 1)]),
        };

        let mappings = SolverAdapter {
            session_ids: vec![SessionId(1)],
            timeslot_index,
        };
        let mapped = mappings.map_timeslots_to_indices(timeslots.iter().copied());
        assert_eq!(
            mapped,
            vec![
                SolverTimeslotIndex(2),
                SolverTimeslotIndex(0),
                SolverTimeslotIndex(1)
            ]
        );
    }

    #[test]
    fn map_solver_solution_returns_ordered_timeslots() {
        let results = HashMap::from([
            (SessionId(0), SolverTimeslotIndex(1)),
            (SessionId(1), SolverTimeslotIndex(0)),
        ]);
        let timeslot_index = TimeslotIndex {
            ordered: vec![TimeslotId(50), TimeslotId(60)],
            positions: HashMap::from([(TimeslotId(50), 0), (TimeslotId(60), 1)]),
        };

        let mappings = SolverAdapter {
            session_ids: vec![SessionId(0), SessionId(1)],
            timeslot_index,
        };
        let mapped = mappings.map_solution(results);
        assert_eq!(mapped.get(&SessionId(0)), Some(&TimeslotId(60)));
        assert_eq!(mapped.get(&SessionId(1)), Some(&TimeslotId(50)));
    }

    /// AI-generated (GPT-5.2-codex).
    #[test]
    fn day_pairs_expands_groups_into_combinations() {
        let timeslot_index = TimeslotIndex {
            ordered: vec![
                TimeslotId(10),
                TimeslotId(11),
                TimeslotId(12),
                TimeslotId(20),
            ],
            positions: HashMap::from([
                (TimeslotId(10), 0),
                (TimeslotId(11), 1),
                (TimeslotId(12), 2),
                (TimeslotId(20), 3),
            ]),
        };

        let mappings = SolverAdapter {
            session_ids: vec![SessionId(1)],
            timeslot_index,
        };

        let day_groups = vec![
            vec![TimeslotId(10), TimeslotId(12), TimeslotId(11)],
            vec![TimeslotId(20)],
        ];
        let pairs = mappings.day_pairs(day_groups);

        assert_eq!(
            pairs,
            vec![
                (SolverTimeslotIndex(0), SolverTimeslotIndex(2)),
                (SolverTimeslotIndex(0), SolverTimeslotIndex(1)),
                (SolverTimeslotIndex(2), SolverTimeslotIndex(1)),
            ]
        );
    }

    /// AI-generated (GPT-5.2-codex).
    #[test]
    fn week_map_translates_timeslot_ids_to_indices() {
        let timeslot_index = TimeslotIndex {
            ordered: vec![TimeslotId(1), TimeslotId(2), TimeslotId(3)],
            positions: HashMap::from([(TimeslotId(1), 0), (TimeslotId(2), 1), (TimeslotId(3), 2)]),
        };

        let mappings = SolverAdapter {
            session_ids: vec![SessionId(1)],
            timeslot_index,
        };

        let week_entries = vec![(TimeslotId(3), 5), (TimeslotId(1), 4)];
        let week_map = mappings.week_map(week_entries);

        assert_eq!(week_map.get(&SolverTimeslotIndex(2)), Some(&5));
        assert_eq!(week_map.get(&SolverTimeslotIndex(0)), Some(&4));
    }
}
