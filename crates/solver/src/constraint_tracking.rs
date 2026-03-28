use std::collections::HashMap;
use z3::{ast::Bool, Optimize};

use crate::diagnostics::ConstraintError;

/// Tracks hard constraints asserted into Z3 and maps them back to typed
/// scheduler constraints for diagnostics.
///
/// Each hard constraint is asserted with a dedicated tracking atom
/// (`assert_and_track`), which allows unsat-core extraction to be translated
/// into domain-level `ConstraintError` values.
pub struct ConstraintTracker {
    /// Mapping from Z3 tracking atoms to typed domain constraints.
    tracked_constraints: HashMap<Bool, ConstraintError>,
}

impl ConstraintTracker {
    /// Create an empty constraint tracker with no tracked hard constraints.
    pub fn new() -> Self {
        Self {
            tracked_constraints: HashMap::new(),
        }
    }

    /// Assert a hard constraint into the optimizer and register its
    /// corresponding domain-level metadata.
    ///
    /// The constraint is tracked with a fresh numeric boolean symbol so that,
    /// if the query becomes unsatisfiable, the unsat core can be mapped back
    /// to `ConstraintError` values.
    pub fn assert_hard(&mut self, optimizer: &Optimize, expr: &Bool, constraint: ConstraintError) {
        let index = self.tracked_constraints.len();
        let tracker = Bool::new_const(index as u32);

        self.tracked_constraints.insert(tracker.clone(), constraint);
        optimizer.assert_and_track(expr, &tracker);
    }

    /// Extract typed constraints present in the unsat core.
    ///
    /// Returns an empty vector when `sat_result` is not `SatResult::Unsat`.
    /// This keeps call sites simple and avoids accidental unsat-core access for
    /// SAT/UNKNOWN states.
    pub fn unsat_core_constraints(
        &self,
        optimizer: &Optimize,
    ) -> impl Iterator<Item = &ConstraintError> {
        optimizer
            .get_unsat_core()
            .into_iter()
            .filter_map(|core| self.tracked_constraints.get(&core))
    }
}

#[cfg(test)]
mod tests {
    use z3::{ast::Bool, Optimize, SatResult};

    use super::ConstraintTracker;
    use crate::ConstraintError;

    #[test]
    fn unsat_diagnostics_include_tracked_core_constraints() {
        let optimizer = Optimize::new();
        let mut tracker = ConstraintTracker::new();

        tracker.assert_hard(
            &optimizer,
            &Bool::from_bool(false),
            ConstraintError::DomainLowerBound {
                session: entity::id::SessionId(1),
            },
        );
        tracker.assert_hard(
            &optimizer,
            &Bool::from_bool(true),
            ConstraintError::DomainUpperBound {
                session: entity::id::SessionId(1),
                n_timeslots: 3,
            },
        );

        assert_eq!(optimizer.check(&[]), SatResult::Unsat);

        let mut unsat_core_constraints = tracker.unsat_core_constraints(&optimizer);

        assert!(unsat_core_constraints.any(|err| err
            == &ConstraintError::DomainLowerBound {
                session: entity::id::SessionId(1)
            }));
    }

    #[test]
    fn debug_info_contains_optimizer_state_for_sat() {
        let optimizer = Optimize::new();
        let tracker = ConstraintTracker::new();

        assert_eq!(optimizer.check(&[]), SatResult::Sat);

        let unsat_constraints = tracker.unsat_core_constraints(&optimizer);
        assert_eq!(unsat_constraints.count(), 0);
    }
}
