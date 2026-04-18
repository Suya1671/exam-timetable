_note for people: this project is **not** vibe-coded. Agents were used to do stuff I did not want to/did not have time to do myself or not absolutely critical to functionality. I don't like making error messages. All code with AI comments will eventually be reviewed._

# AGENTS Guide

This file is for agentic coding assistants working in this repository.
It covers build/test/lint commands, code style, architecture, and safe extension patterns.

## Repository Layout

- List out all projects, their paths, and descriptions using `pnpm moon projects --json`
- `migrations`: SQL migrations.
- `docs/coverage.md`: coverage workflow.

## Core Commands

- All commands and task infrastructure is handled by moonrepo
    - If a task fails, please use the debug-task skill to help the user debug the issue
- List out all tasks using `pnpm moon tasks --json` and for a specific project using `pnpm moon tasks <project> --json`
- Run a task using `pnpm moon run <project>:<task-name> --json`
    - You can run all tasks for all projects using `pnpm moon run :<task-name> --json`
    - For more guidance on the syntax, see https://moonrepo.dev/docs/concepts/target

## Testing Notes

- For targeted debugging, run a single test name first before full suite.
- Frontend currently has check/lint workflows; no dedicated unit-test runner is configured here.

## Formatting, Imports, and Naming

### Rust

- Use rustfmt defaults via `cargo fmt`.
- Keep imports grouped logically; remove unused imports.
- Naming:
  - Types/traits: `PascalCase`
  - Functions/modules/fields: `snake_case`
  - Constants: `SCREAMING_SNAKE_CASE`
- Follow crate-level conventions before introducing new patterns.

### Frontend (TypeScript/Svelte)

- Use eslint
- Prefer explicit, narrow types for form values and data transforms.
- Use existing alias conventions (`$lib`, route-local `./data`, `./forms`).
- Keep component imports stable and remove unused imports aggressively.

## Type and Data Guidelines

- IDs in Rust should use existing newtypes from `crates/model/src/lib.rs` when available.
- Solver layer:
  - Keep `SessionId` / `TimeslotIndex` wrappers where already used.
  - Use `i64` for Z3 integer-domain mappings.
  - Use `u32` for schema-like count fields (`slots_required`, etc.) when appropriate.
- Do not compare raw `TimeslotId` for chronology; use ordered `(date, slot)` or mapped indices.

## Error Handling Guidelines

- Rust:
  - Use `thiserror` for domain errors.
  - Keep variants descriptive and actionable.
  - In solver hard constraints, add/update `ConstraintError` in `crates/solver/src/diagnostics.rs`
    and register via `ConstraintTracker`.
- Frontend:
  - Prefer validating at form boundaries.
  - Keep user-facing errors concise and consistent with existing tone.

## Frontend Infrastructure Notes

- Svelte compiler async mode is enabled (`frontend/svelte.config.js`), so top-level `await` in
  component markup is valid and used in this repo.
- SvelteKit uses `@sveltejs/adapter-static` with fallback `index.html` for Tauri-friendly builds.
- Drizzle config outputs generated DB files under `frontend/src/lib/db`.
  Treat generated files carefully and avoid manual edits unless intentional.
- Current route organization increasingly uses route-local modules:
  - `+page.svelte` for orchestration/UI
  - `data.ts` for data access
  - `forms.ts` for shared validators/form helpers
  - dedicated dialog/components for complex UI blocks

## AI-Generated Annotation Requirement

Any newly created **function**, **struct**, **constant/top level variable in a typescript/svelte file**, or **enum variant** must include:

- Rust: `/// AI-generated (<Model/harness name>).`
- Frontend/TS/JSDoc: `/** AI-generated (<Model/harness name>). */`
- If doc comments already exist, append a single-line notice.
- Keep notice ASCII-only.
- Updates to functions must also have this notice, with the update annotated as a regular comment
- Also include 1-2 bullets in your comment explaining *why* the change was made/what is the purpose of the function/update.


## Constraint Extension Checklist

When adding/changing hard or soft constraints:

1. Model data
   - Add migration/schema changes if needed.
   - Reuse/add model types in `crates/model`.

2. Backend prep
   - Load required rows in backend data prep.
   - Build helper maps/windows and dedupe aggressively.

3. Solver API
   - Add/extend solver methods in `crates/solver/src/lib.rs`.
   - Hard constraints: assert via `ConstraintTracker::assert_hard`.
   - Soft constraints: optimizer soft assertions/objectives.

4. Diagnostics
   - Add/update diagnostics for new infeasibility sources.

5. Ordering
   - Always map to ordered timeslot positions before temporal logic.

6. Tests
   - Add focused unit tests with small deterministic datasets.
   - Run targeted test first, then full crate/workspace tests.

### Multi-slot exam pattern

- Model assignment as **start slot**.
- Build allowed start windows from ordered timeslots.
- Constrain overlap/position relative to the occupied block.

## Agent Workflow Expectations

- Prefer minimal, local changes that match existing style.
- Do not rewrite large files when extracting focused modules is enough.
- For big Svelte pages, prefer extracting dialogs/sections into components
    - Logic and variables should be self contained when possible, else be passed through props/events
- Before finishing:
  - run relevant checks (`cargo` and/or `pnpm check`)
  - report any pre-existing failures separately from new regressions.
