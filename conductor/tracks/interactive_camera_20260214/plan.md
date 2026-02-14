# Implementation Plan: Interactive Camera Controls

Implement OrbitControls-style mouse interactions (orbit, pan, zoom) with damping and efficient render restarts.

## Phase 1: Camera Model and Logic Extensions
Extend the internal camera model to support the target-based transformations required for OrbitControls and implement the damping logic.

- [x] Task: Create `src/model/orbit_camera.rs` to wrap existing camera with orbit logic. dbfb7bf
- [x] Task: Implement `OrbitCamera` struct with properties for `target`, `distance`, `azimuth`, `polar`, and damping state. dbfb7bf
- [x] Task: Implement conversion logic between `OrbitCamera` spherical coordinates and the underlying camera's `look_from`/`look_at`. dbfb7bf
- [x] Task: Write Tests: Verify `OrbitCamera` correctly calculates positions based on spherical coordinates. dbfb7bf
- [x] Task: Implement: `OrbitCamera` transformation logic. dbfb7bf
- [x] Task: Write Tests: Verify damping calculations correctly interpolate values over time. dbfb7bf
- [x] Task: Implement: Damping logic in `OrbitCamera::update()`. dbfb7bf
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Camera Model and Logic Extensions' (Protocol in workflow.md)

## Phase 2: Input Handling and State Management
Capture mouse events in the render output area and map them to `OrbitCamera` mutations.

- [ ] Task: Update `RenderOutput` state to hold an optional `OrbitCamera` instance.
- [ ] Task: Implement input capturing in `src/render_output.rs` for `PointerButton::Primary` (Orbit), `Secondary` (Pan), and `Scroll` (Zoom).
- [ ] Task: Write Tests: Verify mouse drag deltas are correctly converted to angle/position changes in `OrbitCamera`.
- [ ] Task: Implement: Input to `OrbitCamera` mapping.
- [ ] Task: Integrate `OrbitCamera` update loop into the UI `update` function to handle damping/animations.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Input Handling and State Management' (Protocol in workflow.md)

## Phase 3: Efficient Render Restart
Enable mutating the camera in the active render without re-parsing the scene YAML.

- [ ] Task: Add a method to `RenderConfig` or `Scene` to update only the camera configuration.
- [ ] Task: Modify the rendering loop in `src/lib.rs` (or the relevant render controller) to accept camera mutations and trigger a reset of the `solstrale` engine.
- [ ] Task: Write Tests: Verify that updating the camera and restarting the render doesn't trigger a full scene reload.
- [ ] Task: Implement: Camera mutation and render restart trigger.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Efficient Render Restart' (Protocol in workflow.md)

## Phase 4: Final Integration and Polishing
Connect all components and ensure a smooth user experience.

- [ ] Task: Initialize `OrbitCamera` from the scene's initial YAML camera settings.
- [ ] Task: Ensure the UI remains fluid during rapid camera movements.
- [ ] Task: Write Tests: End-to-end verification of camera movement triggering render updates.
- [ ] Task: Implement: Final wiring and performance optimization.
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Final Integration and Polishing' (Protocol in workflow.md)
