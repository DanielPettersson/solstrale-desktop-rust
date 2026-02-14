# Implementation Plan: Real-time Camera Updates

## Phase 1: Infrastructure & Model Updates
Update internal data structures to support persistent camera communication and the new `ray_trace` parameters.

- [x] Task: Update `RenderControl` struct in `src/lib.rs` to include `camera_config_sender: Option<Sender<solstrale::camera::CameraConfig>>`. 1280ecc
- [ ] Task: Update `render` function signature in `src/render_output.rs` to return `(Receiver<RenderMessage>, Sender<bool>, Sender<solstrale::camera::CameraConfig>)`.
- [ ] Task: Conductor - User Manual Verification 'Infrastructure & Model Updates' (Protocol in workflow.md)

## Phase 2: Core Rendering Integration
Integrate the new `camera_config` receiver and `idle` parameter into the rendering thread lifecycle.

- [ ] Task: In `src/render_output.rs`, update the `thread::spawn` block in the `render` function to pass the `camera_config_receiver` and `idle: true` to the `ray_trace` call.
- [ ] Task: Implement the logic in `render` to return the `camera_config_sender`.
- [ ] Task: Write a unit test in `src/render_output.rs` or a new test file to verify that the `render` function correctly returns the expected channels.
- [ ] Task: Conductor - User Manual Verification 'Core Rendering Integration' (Protocol in workflow.md)

## Phase 3: UI Interaction Logic
Refactor the UI loop to use the persistent channel for camera updates.

- [ ] Task: Modify `render_output` in `src/render_output.rs` to detect if a render is already active when a camera update occurs.
- [ ] Task: If active and only the camera changed, use `render_control.camera_config_sender` to send the new `solstrale::camera::CameraConfig`.
- [ ] Task: Ensure the `loading_scene` flag is not set to `true` when performing a camera-only update.
- [ ] Task: Update the "Handle render restarts" logic to preserve the `render_receiver` and thread if only the camera moved.
- [ ] Task: Conductor - User Manual Verification 'UI Interaction Logic' (Protocol in workflow.md)

## Phase 4: Final Verification & Cleanup
Ensure stability across all interaction types (YAML change, resize, camera move).

- [ ] Task: Verify that resizing the window still triggers a full restart (correctly setting `render_control.render_requested = true`).
- [ ] Task: Verify that YAML changes still trigger a full restart.
- [ ] Task: Run full project tests and linting.
- [ ] Task: Conductor - User Manual Verification 'Final Verification & Cleanup' (Protocol in workflow.md)
