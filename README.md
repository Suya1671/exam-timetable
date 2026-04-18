# Exam Timetable


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

## Dependencies
- Tauri dependencies
- SQlite
- Z3
  - Unless you're using the `gh-release` or `vendored` feature flags
  - `gh-release` and the default featureset requires the end-user to have Z3 installed as well. `vendored` will statically link z3

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
- [ ] Make the UI for timetables better
  - [ ] Proper saving/loading
    - [ ] Custom Save vs. Save As
    - [ ] Notification when a save happened successfully
  - [ ] Store custom timetable data into the database, rather than in the export dialog
  - [ ] Categorise the UI better
  - [ ] Solver progress/explanation
  - [ ] Ability to check a timetable
  - [ ] Clearer resolve behaviour
  - [ ] Better Dialogs
  - [ ] Confirmation when a user exports

## AI usage and contribution policy

**This is a currently WIP, Mostly personal project, and some bits (A lot of bits, currentlty) are messy and AI generated with little review other than "yeah that code is _fine_ and it works". I am slowly reviewing of all AI-marked areas :p. I do not approve of vibe coding (i.e. LLM generated code without proper review), but LLMs are a useful tool to get stuff done a bit faster.**

AI was used in this repository. All AI-generated contributions that have not been modified/fully reviewed by
persons are marked with an explicit notice (for example `/// AI-generated.` in Rust doc comments, or `/** AI-generated */` in JSDoc).

AI-generated contributions are allowed, but they will be reviewed strictly and must be marked
as such.
