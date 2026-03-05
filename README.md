# Solstråle Desktop

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![CI](https://github.com/DanielPettersson/solstrale-desktop-rust/actions/workflows/ci.yaml/badge.svg)](https://github.com/DanielPettersson/solstrale-desktop-rust/actions/workflows/ci.yaml)

A desktop UI for the [Solstråle path tracer](https://github.com/DanielPettersson/solstrale-rust).

![Solstråle Desktop UI](https://github.com/DanielPettersson/solstrale-desktop-rust/assets/3603911/432b6661-716a-46ef-86ab-3789c4fb52da)

## Key Features

*   **Real-time Preview:** See your path-traced scene evolve as it renders.
*   **Interactive Camera:** Navigate your scene with intuitive orbit, pan, and zoom controls, featuring smooth damping for a professional feel.
*   **Integrated YAML Editor:** Configure your scenes directly within the app using a built-in editor with syntax highlighting.
*   **Progress Tracking:** Visual feedback on rendering progress and estimated time remaining.
*   **Integrated Documentation:** Built-in guidance for scene creation and configuration.
*   **Batch Rendering:** Command-line utility for efficient high-volume rendering.

## Technology Stack

*   **Language:** Rust (2021 Edition)
*   **UI Framework:** [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
*   **Rendering Engine:** [Solstråle](https://github.com/DanielPettersson/solstrale-rust)
*   **Syntax Highlighting:** [syntect](https://github.com/trishume/syntect)
*   **Image Processing:** [image](https://github.com/image-rs/image)
*   **Templating** [tera](https://github.com/Keats/tera)
*   **Serialization:** Serde (JSON, YAML)

## Getting Started

### Prerequisites

*   [Rust toolchain](https://rustup.rs/) (latest stable version recommended)

### Build and Run

1.  Clone the repository:
    ```bash
    git clone https://github.com/DanielPettersson/solstrale-desktop-rust.git
    cd solstrale-desktop-rust
    ```

2.  Build the project:
    ```bash
    ./build.sh
    ```
    *Note: The first build may take some time as it compiles all dependencies.*

3.  Run the application:
    ```bash
    target/release/solstrale-desktop
    ```

### Interactive Controls

*   **Orbit:** Left-click and drag.
*   **Pan:** Right-click and drag.
*   **Zoom:** Scroll wheel.
*   **Edit Scene:** Use the YAML editor on the right side to modify scene parameters in real-time.

## Batch Rendering

The project also includes a batch rendering utility for non-interactive workloads:

```bash
target/release/solstrale-batch-render --scene scene.yaml --output output.png
```

## License

This project is licensed under the Apache License, Version 2.0. See the [LICENSE](LICENSE) file for details.