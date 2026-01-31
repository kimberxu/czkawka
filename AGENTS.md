# Czkawka Agent Guidelines

## 🌍 Project Context & Architecture
Czkawka is a fast, multi-functional tool to remove unnecessary files (duplicates, empty folders, similar images, etc.).
It is a Rust workspace organized as follows:
- **`czkawka_core`**: The brain. Contains all algorithms and logic. **ALL** business logic changes go here.
- **`krokiet`**: The modern GUI frontend using **Slint**. This is the primary focus for UI development.
- **`czkawka_gui`**: Legacy GTK4 frontend. Maintenance mode only.
- **`czkawka_cli`**: Command-line interface.

## 🌿 Branch: `self/pathchange-thumbsave` (Customized)
This branch contains specific customizations for personal use, diverging from upstream `master`.
**Agents must preserve these behaviors:**

1.  **Portable Paths (Forced)**
    - **Logic**: `czkawka_core/src/common/config_cache_path.rs`
    - **Behavior**: Ignores system-standard paths (XDG/AppData). Defaults `Config` and `Cache` folders to be siblings of the executable (`exe_dir/config`, `exe_dir/cache`).
    - **Override**: Still respects `CZKAWKA_CONFIG_PATH` and `CZKAWKA_CACHE_PATH` env vars if set.

2.  **Thumbnail Export in JSON**
    - **Logic**: `czkawka_core/src/tools/similar_videos/mod.rs`
    - **Behavior**: The `thumbnail_path` field in `VideosEntry` is **NOT** skipped during serialization (`#[serde(skip)]` removed).
    - **Purpose**: Allows external tools to read the absolute path of generated thumbnails from the exported JSON report.

---

## 🛠 Development Workflow

### Build & Run
**Use `just` (install via `cargo install just`) or direct cargo commands.**

- **Build Release**: `just build_all` (or `cargo build --release`)
- **Run CLI**: `just run czkawka_cli -- <args>`
- **Run GUI (Slint)**: `just run krokiet`
- **Fast Debug Run**: `just runr <bin>` (Release profile with debug info - faster execution than debug)

### Verification (MANDATORY)
Before submitting changes, ensure strictly no warnings or errors.

1.  **Format**: `cargo fmt`
2.  **Lint**: `cargo clippy -- -D warnings` (Strict! No warnings allowed)
    - *Tip*: Use `just fix` to auto-fix simple formatting and clippy issues.
3.  **Test**: `cargo test`

### Running Single Tests
To verify specific changes without running the full suite:
```bash
cargo test --package czkawka_core --lib -- video_search_test
```

---

## 📏 Code Standards

### Rust Style
- **Edition**: 2024
- **Safety**:
    - **No `unwrap()`**: Use `expect("Context why this shouldn't fail")` or propagate errors with `?`.
    - **No `panic!`**: Return `Result` types.
- **Imports**: No wildcards (`use std::fs::*;` ❌). Explicit imports only.
- **Logging**: Use `log::info!`, `log::warn!` for status updates.

### Error Handling
- Prefer `anyhow` or specific `Result` types in CLI/GUI layers.
- In `czkawka_core`, library errors should be strictly typed where possible.

### Performance
- This is a performance-critical tool.
- Avoid `.clone()` in hot loops.
- Use `release` build for any timing/benchmark checks.
- Image/Video processing can be heavy - respect `stop_receiver` signals to allow cancellation.

## 📂 Key File Locations
- **Core Logic**: `czkawka_core/src/`
    - `common/`: Shared utilities (paths, messaging, extensions).
    - `tools/`: Specific tools (duplicates, similar_images, similar_videos).
- **GUI Logic (Slint)**: `krokiet/src/`
    - `main.rs`: Entry point.
    - `ui/`: `.slint` UI definition files.

---

## 🤖 Cursor/Copilot Rules
- **Conciseness**: Write linear, easy-to-read code.
- **Context**: When editing `czkawka_core`, check if changes affect multiple frontends.
- **Refactoring**: If you see legacy patterns (e.g. `unwrap` in old code), fix them while touching the file.
