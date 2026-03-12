# Exam Timetable

This repository contains a Rust workspace for building an exam scheduling system backed by Z3.

## Overview

- `crates/model`: database models and shared ID types.
- `crates/entity`: SeaORM entities generated from the SQLite schema.
- `crates/solver`: Z3-backed scheduler and constraints.
- `crates/backend`: database access, data prep, and wiring to the solver.

Timeslots are ordered by `(date, slot)` when building any scheduling logic. Do not assume
`TimeslotId` is in chronological order.

## SeaORM schema sync (Tauri runtime)

Entity-first schema sync should run in the Tauri app once the database connection is created:

```rust
db.get_schema_registry("entity::entity::*").sync(&db).await?;
```

## Quick start

From the repository root:

- Build workspace: `cargo build`
- Run tests: `cargo nextest run`
- Run a single test: `cargo nextest run -p solver add_pair_constraint_enforces_allowed_pairs`
- Lint: `cargo clippy --all-targets --all-features`
- Format: `cargo fmt`

Coverage helpers are documented in `docs/coverage.md`.

## AI usage and contribution policy

AI was used in this repository. All AI-generated contributions that have not been modified/fully reviewed by
persons are marked with an explicit notice (for example `/// AI-generated.` in Rust doc comments).

AI-generated contributions are allowed, but they will be reviewed strictly and must be marked
as such.
