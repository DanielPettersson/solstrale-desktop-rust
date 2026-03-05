# Technology Stack

## Core Technologies
- **Programming Language:** Rust (2024 Edition)
- **UI Framework:** [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) / [egui](https://github.com/emilk/egui) (v0.33.0) - A fast, easy-to-use GUI library.
- **Rendering Engine:** [solstrale](https://github.com/DanielPettersson/solstrale-rust.git) - The underlying path tracing engine.

## Libraries and Tools
- **Data Serialization:** `serde`, `serde_json`, `serde_yaml` - For managing scene configurations and data exchange.
- **Image Processing:** `image` (v0.25.8) - For handling rendered output and texture loading.
- **Syntax Highlighting:** `syntect` - Used for the YAML scene editor.
- **Caching:** `moka` - For efficient data management.
- **Templating:** `tera` - For dynamic content generation.
- **CLI Utilities:** `clap` (Command Line Argument Parser) and `indicatif` (Progress reporting).
- **Math Utilities:** Custom implementation of spherical coordinates and damping for interactive camera movement.
