# Solstråle Desktop

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![CI](https://github.com/DanielPettersson/solstrale-desktop-rust/actions/workflows/ci.yaml/badge.svg)](https://github.com/DanielPettersson/solstrale-desktop-rust/actions/workflows/ci.yaml)

A desktop UI for the [Solstråle path tracer](https://github.com/DanielPettersson/solstrale-rust).

![Solstråle Desktop UI](https://github.com/DanielPettersson/solstrale-desktop-rust/assets/3603911/432b6661-716a-46ef-86ab-3789c4fb52da)

## Key Features

*   **Real-time Preview:** See your path-traced scene evolve as it renders, and re-render as you edit it.
*   **Interactive Camera:** Navigate your scene with intuitive orbit, pan, and zoom controls, featuring smooth damping for a professional feel.
*   **Scene Editor:** An outline of the scene and an inspector for the selected part of it. Numbers can be expressions, and a repeat creates its contents once per value of a loop variable.
*   **Animation Preview:** Scrub through a scene animated with `frameIndex` with the frame slider.
*   **Progress Tracking:** Visual feedback on rendering progress and estimated time remaining.
*   **Batch Rendering:** Command-line utility for rendering the frames of an animation.

## Technology Stack

*   **Language:** Rust (2024 Edition)
*   **UI Framework:** [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
*   **Rendering Engine:** [Solstråle](https://github.com/DanielPettersson/solstrale-rust)
*   **Image Processing:** [image](https://github.com/image-rs/image)
*   **Serialization:** Serde (YAML)

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

    With `SOLSTRALE_TIME_EDITS=1` set, it prints how long each edit takes to show in the viewport.

### Interactive Controls

*   **Orbit:** Left-click and drag.
*   **Pan:** Right-click and drag.
*   **Zoom:** Scroll wheel.
*   **Camera:** Moving the view moves the scene's camera. When the camera's position is an expression only the view moves, and the camera's inspector offers to reset the view or replace the expressions with it.
*   **Edit Scene:** Select a part of the scene in the outline on the left and change it in the inspector on the right. Add hittables with +, right click one to duplicate, move, wrap in a repeat or delete it, and drag to reorder. Help > How to use has the details. An edit goes to the running render as it is made, and only the part of the scene it changed is built again.
*   **Shortcuts:** Ctrl+S saves the scene, Ctrl+R restarts the render.

## Scene Files

Scenes are saved as YAML. Any number can be an expression, including each part of a position or color, and scene `variables` and `repeat` cover repetition and animation:

```yaml
variables:
  spacing: 2.5
  bounce: abs(sin(frameIndex * 0.1)) * 2
camera:
  look_from: 0, 2, -12
world:
  - repeat:
      variable: i
      from: -2
      to: 3              # stops before 3
      world:
        - sphere:
            center: i * spacing, bounce, 0
            radius: 0.8
```

Expressions support `+ - * / % ^`, parentheses and the functions `sin cos tan abs sqrt floor round pow min max len`. They can read `frameIndex`, `pi`, `e`, the scene variables and the loop variables of enclosing repeats. The editor writes the file, so comments and formatting in a hand-edited file are not kept when it is saved from the app.

Scenes from earlier versions that use Tera templates need rewriting with these, as templates are no longer supported.

## Batch Rendering

The project also includes a batch rendering utility, which renders the frames of an animation to `frame_00000000.png`, `frame_00000001.png` and so on in the current directory:

```bash
target/release/solstrale-batch-render scene.yaml --width 800 --height 600 --num-frames 100
```

## License

This project is licensed under the Apache License, Version 2.0. See the [LICENSE](LICENSE) file for details.