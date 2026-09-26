# Technology Stack

## Core Technologies
- **Programming Language:** Rust (2024 Edition)
- **UI Framework:** [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) / [egui](https://github.com/emilk/egui) (v0.36) - A fast, easy-to-use GUI library.
- **Rendering Engine:** [solstrale](https://github.com/DanielPettersson/solstrale-rust.git) - The underlying path tracing engine.

## Libraries and Tools
- **Data Serialization:** `serde`, `serde_yaml` - Scenes are saved as YAML.
- **Image Processing:** `image` (v0.25.8) - For handling rendered output and texture loading.
- **Caching:** `moka` - For efficient data management.
- **Expressions:** A small in-house parser and evaluator (`src/model/expr.rs`) for numbers written as expressions.
- **CLI Utilities:** `clap` (Command Line Argument Parser) and `indicatif` (Progress reporting).
- **Math Utilities:** Custom implementation of spherical coordinates and damping for interactive camera movement.
