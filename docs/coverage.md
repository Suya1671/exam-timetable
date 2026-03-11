# Coverage Guide

This repository uses Rust LLVM coverage via `cargo-llvm-cov`, integrated with `cargo-nextest`.

## 1) Install coverage tooling

```/dev/null/install.sh#L1-2
cargo install cargo-llvm-cov
cargo install cargo-nextest
```

## 2) Run coverage from repo root

Convenient aliases are configured in `.cargo/config.toml`.

### Summary in terminal (via nextest)

```/dev/null/cov-summary.sh#L1-1
cargo cov
```

Equivalent command:

```/dev/null/cov-summary-equivalent.sh#L1-1
cargo llvm-cov nextest --workspace --all-features --summary-only
```

### HTML report (best for browsing/debugging)

```/dev/null/cov-html.sh#L1-1
cargo cov-html
```

Equivalent command:

```/dev/null/cov-html-equivalent.sh#L1-1
cargo llvm-cov nextest --workspace --all-features --html --output-dir coverage/html
```

Then open:

- `coverage/html/index.html`

### LCOV report (widely supported)

```/dev/null/cov-lcov.sh#L1-1
cargo cov-lcov
```

Equivalent command:

```/dev/null/cov-lcov-equivalent.sh#L1-1
cargo llvm-cov nextest --workspace --all-features --lcov --output-path coverage/lcov.info
```

Output:

- `coverage/lcov.info`

### Cobertura XML (editor/CI friendly)

```/dev/null/cov-cobertura.sh#L1-1
cargo cov-cobertura
```

Equivalent command:

```/dev/null/cov-cobertura-equivalent.sh#L1-1
cargo llvm-cov nextest --workspace --all-features --cobertura --output-path coverage/cobertura.xml
```

Output:

- `coverage/cobertura.xml`

---

## 3) Editor integration

### VS Code

Use an extension that reads LCOV (for example, Coverage Gutters) and point it to:

- `coverage/lcov.info`

This enables inline file-level highlights based on coverage data.

### Zed

Use coverage consumers/integrations that accept standard report formats.  
This repo emits both:

- `coverage/lcov.info`
- `coverage/cobertura.xml`

These are the most compatible formats for inline or side-panel coverage workflows.

---

## 4) Per-crate coverage (optional)

If you only want solver coverage (still through nextest):

```/dev/null/cov-solver.sh#L1-1
cargo llvm-cov nextest -p solver --all-features --html --output-dir coverage/html-solver
```

Open:

- `coverage/html-solver/index.html`

---

## 5) Nix usage

Coverage is available in the flake flow as checks/packages/apps.

- Include coverage in check flow:
  - `nix flake check`

- Build machine-readable reports:
  - `nix build .#coverage`

- Build browsable HTML report:
  - `nix build .#coverage-html`

---

## 6) Notes

- Generated coverage artifacts are ignored by git (`coverage/`, `*.profraw`, `*.profdata`).
- Run coverage from the repository root so workspace paths resolve correctly.
- The coverage commands above execute tests through `nextest`, so behavior matches your nextest test workflow.
