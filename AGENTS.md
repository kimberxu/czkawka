# Czkawka Agent Guidelines

This document provides instructions and guidelines for AI agents and developers working on the Czkawka codebase.

## 1. Project Structure & Overview

Czkawka is a multi-crate Rust workspace organized as follows:
- **`czkawka_core`**: The core logic (finding duplicates, empty folders, etc.). Shared by all frontends.
- **`czkawka_cli`**: Command-line interface.
- **`czkawka_gui`**: GTK 4 graphical interface.
- **`krokiet`**: Slint-based graphical interface (new generation).
- **`misc`**: Miscellaneous scripts (Python, etc.).

## 2. Build, Lint, and Test

The project uses `just` as a command runner. Always prefer `just` over raw cargo commands when available.

### Common Commands
- **Build All**: `just build_all` (builds release/debug, runs clippy and tests)
- **Run CLI**: `just run <args>` (e.g., `just run -h`)
- **Fix Code**: `just fix` (Runs `ruff`, `mypy`, `cargo fmt`, `cargo clippy --fix`)
- **Clean**: `just clean`

### Testing
- **Run All Tests**: `cargo test`
- **Run Specific Test**: `cargo test <test_name>`
  - Example: `cargo test test_find_duplicates_by_hash`
- **Run Tests for Specific Package**: `cargo test -p czkawka_core`
- **Benchmarks**: `cargo bench` (in `czkawka_core`)

### Linting
- **Clippy**: Strict adherence is required. The project enables many pedantic and restriction lints in `clippy.toml`.
- **Run Clippy**: `cargo clippy` (or `just build_all` which includes it).
- **Format**: `cargo fmt` must always be run before committing.

## 3. Code Style & Conventions

### General Rust Guidelines
- **Formatting**: Standard `rustfmt` rules apply.
- **Naming**: 
  - `snake_case` for variables, functions, modules.
  - `CamelCase` for Structs, Enums, Traits.
  - `SCREAMING_SNAKE_CASE` for constants/statics.
- **Imports**: Group imports in the following order:
  1. `std` / `core`
  2. External crates (e.g., `rayon`, `crossbeam`)
  3. Internal crates (`crate::...`)
  4. Alphabetical order within groups.

### Error Handling
- **Avoid `unwrap()`**: Do not use `unwrap()` in `czkawka_core` logic unless it's impossible to fail (and documented why).
- **Use `expect()`**: If you must panic on a `None` or `Err`, use `expect("reason")` to provide context.
- **Result Types**: Return `Result<T, Box<dyn Error>>` or specific error types for fallible operations.
- **Panics**: Explicit panics are generally discouraged in library code; propagate errors instead.

### Concurrency & Performance
- **Parallelism**: Use `rayon` for data parallelism (e.g., `par_iter()`) when processing large file sets.
- **Cancellation**: Long-running tasks in `czkawka_core` MUST support cancellation.
  - Pass `stop_flag: &Arc<AtomicBool>` to heavy functions.
  - Check `stop_flag.load(Ordering::Relaxed)` periodically (e.g., inside loops).
- **Progress Reporting**: Use `progress_sender: Option<&Sender<ProgressData>>` to report status back to the UI.
- **Vectors vs Iterators**: Prefer iterators and `collect()` over manual `Vec::push` loops for better optimization potential.

### Testing Patterns
- **Unit Tests**: Place unit tests in a `tests` module within the source file or in a separate `tests.rs` file in the module folder (e.g., `czkawka_core/src/tools/duplicate/tests.rs`).
- **Fixtures**: Use `tempfile::TempDir` for creating temporary files and directories in tests. Do not write to the persistent filesystem.
- **Assertions**: Use informative assertion messages: `assert_eq!(actual, expected, "message")`.

### Frontend Guidelines (GUI)
- **GTK4 (czkawka_gui)**: Follow GTK4-rs patterns. Keep UI logic separate from core business logic.
- **Slint (krokiet)**: Respect Slint's `.slint` file structure and Rust bindings.
- **Translations**: Text strings exposed to users should be translatable.

## 4. Documentation
- **Comments**: Public functions in `czkawka_core` should have doc comments (`///`).
- **TODOs**: If functionality is incomplete, use `// TODO: description` comments.
- **Changelog**: Significant changes should be reflected in `Changelog.md` (if modifying an existing feature significantly).

## 5. Development Workflow
1.  **Analyze**: Understand the existing code patterns in the specific module you are touching.
2.  **Plan**: Identify necessary changes.
3.  **Implement**: Write code following the style guides above.
4.  **Lint**: Run `cargo clippy` and fix all warnings.
5.  **Format**: Run `cargo fmt`.
6.  **Test**: Run relevant tests to ensure no regressions.
7.  **Verify**: If fixing a bug, add a regression test case.

## 6. Specific Configurations
- **clippy.toml**: Check this file to understand enabled/disabled lints.
- **Cargo.toml**: Dependency management. Add new dependencies only if strictly necessary.

## 7. AI Agent Specifics
- **Context**: When reading code, prefer reading the module definition (`mod.rs`) and the specific implementation file.
- **Tools**: Use `czkawka_core/src/tools/` as the reference for implementing new scanning tools.
- **Refactoring**: When refactoring, ensure that the `Trait` implementations (e.g., `DuplicateFinder` implementing `CommonData`) remain consistent.

---
*Generated by Antigravity Agent*
