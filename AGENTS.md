_note for people: this project is **not** vibe-coded. Agents were used to do stuff I did not want to do myself or not absolutely critical to functionality. I don't like making error messages._
# AGENTS Guide

This file documents how agentic coders should work in this repo: how to build/test, expected
style conventions, and how to extend the solver safely.

## Commands

All commands are run from the repo root.

- Build workspace: `cargo build`
- Build a crate: `cargo build -p solver`
- Run all tests: `cargo nextest run`
- Run a crate's tests: `cargo nextest run -p solver`
- Run a single test (by name): `cargo nextest run -p solver add_pair_constraint_enforces_allowed_pairs`
- Run tests with output: `cargo nextest run -p solver add_pair_constraint_enforces_allowed_pairs -- --nocapture`
- Lint (clippy): `cargo clippy --all-targets --all-features`
- Format: `cargo fmt`

Coverage shortcuts (see `docs/coverage.md` and `.cargo/config.toml`):

- Summary: `cargo cov`
- HTML: `cargo cov-html`
- LCOV: `cargo cov-lcov`
- Cobertura: `cargo cov-cobertura`

## Code Style

General Rust conventions used in this repo:

- Formatting: `cargo fmt` (rustfmt defaults).
- Naming: Regular rust guidelines
- Linting: `cargo clippy`
- Types:
  - IDs use newtypes in `crates/model/src/lib.rs` (e.g., `ExamId`, `TimeslotId`).
  - Solver uses `SessionId` and `TimeslotIndex` wrappers (see `crates/solver/src/lib.rs`).
  - Use `i64` for Z3 integer values and mappings (Z3 uses `Int`).
  - Use `u32` for counts when reflecting schema fields like `slots_required`.
- Error handling:
  - `thiserror` is the standard error type; keep variants descriptive.
  - For solver constraints, add new `ConstraintError` variants in
    `crates/solver/src/diagnostics.rs` and register them via `ConstraintTracker`.
- Documentation:
  - Public methods have doc comments in the solver; follow the existing tone.
  - Avoid adding comments unless the logic is non-obvious.

### AI-generated label requirement

Any newly created **function**, **struct**, or **enum variant** must include a notice:

- Add `/// AI-generated (GPT-5.2-codex).` on the item doc comment block (structs, functions, enum variants).
- If a function already has doc comments, append the line with the notice.
- Keep the notice to a single line, ASCII only.

Also include change reasoning in the final agent response (one or two bullets describing
why the change was made).

## Architecture Overview

- `crates/model`: database models and shared ID types.
- `crates/solver`: Z3-backed scheduler and constraints.
- `crates/backend`: database access, data prep, and wiring to the solver.

Timeslots are ordered by `(date, slot)` when building any scheduling logic. Do not assume
`TimeslotId` is in chronological order.

## Adding or Changing Constraints

When introducing a new hard/soft constraint, follow this checklist:

1) **Model the data**
   - Add schema fields or join tables in migrations if needed.
   - Add or reuse model types in `crates/model/src/lib.rs`.

2) **Backend data prep**
   - Load the required data (SQLx queries in `crates/backend/src/lib.rs`).
   - Build any helper maps/arrays (e.g., day groupings, consecutive windows).
   - Prefer grouping and deduping to reduce redundant solver constraints.

3) **Solver API**
   - Add a method in `crates/solver/src/lib.rs` for the new constraint.
   - Hard constraints should be asserted using `ConstraintTracker::assert_hard` so
     infeasibility diagnostics are meaningful.
   - Soft constraints should use `optimizer.assert_soft(...)` or `minimize/maximize`.

4) **Diagnostics**
   - Add a `ConstraintError` variant for new hard constraints in
     `crates/solver/src/diagnostics.rs`.

5) **Ordering & positions**
   - The solver operates on `TimeslotIndex` (ordered by date, slot). Map DB `TimeslotId`
     to solver indices in the backend before constructing constraints.
   - Never compare raw IDs for time order; always use ordered indices.

6) **Tests**
   - Add unit tests in `crates/solver/src/lib.rs` to cover the new constraint logic.
   - Use small timeslot/exam sets to keep solver tests fast and deterministic.
      - Unless writing integration tests/bigger multi-constraint tests.

### Multi-slot exams pattern (example)

- Treat the assignment variable as the **start** slot of a block.
- Build allowed start windows based on ordered timeslots.
- Constrain other exams with a positional map (e.g., “outside the block”).
