# Track: Update the rust edition to 2024

## Overview
Upgrade the `solstrale-desktop-rust` project from the Rust 2021 edition to the Rust 2024 edition to leverage new language features, performance improvements, and modern idioms.

## Functional Requirements
1.  **Toolchain Upgrade:** Target Rust toolchain version 1.93 or later to support the 2024 edition.
2.  **Edition Update:** Change the `edition` field in `Cargo.toml` to `"2024"`.
3.  **Automated Migration:** Use `cargo fix --edition` to automate as much of the migration as possible.
4.  **Manual Resolution:** Address any remaining compilation errors, warnings, or lint issues manually.
5.  **Proactive Refactoring:** Refactor the codebase to use new Rust 2024 features (e.g., `gen` keyword, improved `impl Trait`, and other 2024-specific idioms) where applicable.

## Non-Functional Requirements
1.  **Stability:** The application must remain fully functional after the upgrade.
2.  **Compatibility:** All current features and integrations (e.g., `solstrale` engine, `egui` UI) must continue to work seamlessly.
3.  **Maintainability:** Code should be updated to follow the most current and recommended Rust practices.

## Acceptance Criteria
- [ ] `Cargo.toml` contains `edition = "2024"`.
- [ ] The project builds successfully with Rust 1.93+.
- [ ] All unit and integration tests pass.
- [ ] `cargo clippy` and `cargo fmt` report no issues.
- [ ] New Rust 2024 idioms have been applied where they improve the code.

## Out of Scope
- Upgrading dependencies that are not required for the 2024 edition.
- Major feature additions or architectural changes unrelated to the edition upgrade.
