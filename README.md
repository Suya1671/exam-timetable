# Exam Timetable

**WIP. Mostly personal project. Some bits (A lot of bits, currentlty) are messy and partially vibe-coded/AI generated with little review other than "yeah that code is _fine_ and it works". I am slowly reviewing of all AI-marked areas. I am not happy with the code quality. Bleh :p**

This repository contains a Rust workspace for building an exam scheduling system backed by Z3, along with a sveltekit frontend.

## Overview

- `crates/model`: database models and shared ID types.
- `crates/entity`: SeaORM entities generated from the SQLite schema.
- `crates/solver`: Z3-backed scheduler and constraints.
- `crates/backend`: database access, data prep, and wiring to the solver.
- `frontend`: Sveltekit frontend
    - Personal note: I despise some of the stuff I've pulled off in there out of principal but since I always know I have JS might as well save me brain power and use cursed stuff (_looking at you, tanstack forms_)
    - Personal note 2: Heavily vibe-refactored and I don't like how "busy" some of the code is. I'll have to do a full review/potential rewrite later to clean stuff up. Probably more state stores and derived shenanigans.
- `frontend/app`: Tauri app for the frontend

Timeslots are ordered by `(date, slot)` when building any scheduling logic. Do not assume
`TimeslotId` is in chronological order.

## Quick start

From the repository root:

- Build workspace: `cargo build`
- Run tests: `cargo nextest run`
- Run a single test: `cargo nextest run -p solver add_pair_constraint_enforces_allowed_pairs`
- Lint: `cargo clippy --all-targets --all-features`
- Format: `cargo fmt`

From `frontend`:
- Install dependencies: `pnpm i`
- Build tauri app: `pnpm tauri build`
- Lint: `pnpm lint`
- Format: `pnpm format`

Coverage helpers are documented in `docs/coverage.md`.

## TODOs
- [ ] Better use of the Intl and Temporal APIs (they are amazing aaaa)
- [ ] Fix the severe lack of actual reactivity (thanks Codex for being react-brained. bleh. The code is ugly and I apologise for anyone who looks at it right now)
- [ ] Make the UI for saving/loading timetables better
  - [ ] Actually make it make sense
  - [ ] Make resolving work correctly
  - [ ] Categorise the UI better
  - [ ] Custom Save vs. Save As
  - [ ] Better Dialogs

## AI usage and contribution policy

AI was used in this repository. All AI-generated contributions that have not been modified/fully reviewed by
persons are marked with an explicit notice (for example `/// AI-generated.` in Rust doc comments, or `/** AI-generated */` in JSDoc).

AI-generated contributions are allowed, but they will be reviewed strictly and must be marked
as such.
