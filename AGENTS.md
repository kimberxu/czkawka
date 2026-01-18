# AGENTS.md - Protocol for AI Agents

## Project Context
**Czkawka** (Polish: "hiccup") is a multi-functional app to remove unnecessary files (duplicates, empty folders, etc.).
**Krokiet** (Polish: "croquet") is the **Slint-based GUI frontend** for Czkawka.
**Core**: `czkawka_core` contains the shared logic.

## 1. Build, Lint, and Test Commands

### Build & Run
*   **Build (Debug)**: `cargo build --bin krokiet`
*   **Build (Release)**: `cargo build --release --bin krokiet`
*   **Run (Debug)**: `cargo run --bin krokiet`
*   **Run (Release)**: `cargo run --release --bin krokiet` (or `just runr krokiet`)
*   **Run with specific backend**: `SLINT_BACKEND=winit-software cargo run --bin krokiet`

### Testing
*   **Run all tests**: `cargo test --bin krokiet`
*   **Run single test**: `cargo test --bin krokiet <test_name>`
    *   Example: `cargo test --bin krokiet test_initial_state`
*   **Run library tests**: `cargo test -p czkawka_core`

### Linting & Formatting
*   **Format**: `cargo fmt` (or `just fix` for full cleanup)
*   **Lint**: `cargo clippy --all-features --all-targets`
*   **Fix Lints**: `cargo clippy --fix --allow-dirty --allow-staged --all-features --all-targets`
*   **Note**: The project uses a strict `clippy.toml` configuration. Respect existing warnings.

## 2. Code Style & Conventions

### General Rust
*   **Edition**: Rust 2024.
*   **Formatting**: Enforced by `rustfmt`. Run `cargo fmt` before committing.
*   **Naming**:
    *   `snake_case` for functions, variables, modules.
    *   `PascalCase` for structs, enums, traits.
    *   `SCREAMING_SNAKE_CASE` for constants.
*   **Imports**:
    *   Group imports: `std`, external crates, internal `crate::`.
    *   Avoid wildcard imports `use ...::*` except for prelude-like modules or when explicitly justified.
*   **Error Handling**:
    *   Prefer `Result` propagation (`?`) over `unwrap()` or `expect()`.
    *   Exception: `unwrap()`/`expect()` allowed in `main.rs` startup logic or tests.
    *   Use `anyhow` or specific error types if defined in `czkawka_core`.

### Krokiet (Slint Frontend) Specifics
*   **Architecture**:
    *   `main.rs`: Entry point, setup, and high-level wiring.
    *   `common/`: Shared utilities.
    *   `connect_*.rs`: Modules that bind UI events (Slint) to Rust logic. **Keep logic separated in these files.**
    *   `file_actions/`: Specific file operation logic (delete, move, etc.).
*   **Slint Integration**:
    *   UI definitions are in `.slint` files (likely in `ui/` or `src/ui/`).
    *   Rust code interacts with UI via `Weak<MainWindow>` or `AppWindow` handles.
    *   Use `slint::include_modules!()` in `main.rs`.
*   **State Management**:
    *   Use `Rc<VecModel<...>>` for list models.
    *   Updates to UI models should happen on the UI thread or use `invoke_from_event_loop`.

### Translations (i18n)
*   Uses `i18n-embed` with Fluent (`.ftl` files).
*   If adding new strings:
    1.  Add to `.ftl` files (English first).
    2.  Run `just translate` (if applicable) or manually update generic translation files.
    3.  Use `fl!()` macro in code.

## 3. Workflow Rules
1.  **Check Status**: `git status` before starting.
2.  **Branching**: Create feature branches (e.g., `feature/improved-selection`).
3.  **Incremental Work**:
    *   Write a test (if possible) -> Fail -> Implement -> Pass.
    *   Or: Implement small chunk -> Compile -> Verify.
4.  **Verification**:
    *   **Must run** `cargo clippy` and ensure no *new* warnings are introduced.
    *   **Must run** `cargo build` to ensure compilation.
5.  **Committing**: Use Conventional Commits (e.g., `feat(krokiet): add selection logic`, `fix(core): resolve duplicate crash`).

## 4. Known Constraints
*   **Slint Limitations**: Some advanced custom widgets might be hard to implement. Prefer standard Slint widgets where possible.
*   **Platform Specifics**: Be aware of Windows vs Linux file path handling (`\` vs `/`).
*   **Unsafe**: Avoid `unsafe` code unless interacting with FFI or strictly necessary for performance (and well-documented).

## 5. Development Tools (Justfile)
The `justfile` in the root offers shortcuts:
*   `just fix`: Runs formatter and clippy fixes.
*   `just runr krokiet`: Runs optimized release build.
*   `just test_resize <arg>`: Performance testing for image resizing.
