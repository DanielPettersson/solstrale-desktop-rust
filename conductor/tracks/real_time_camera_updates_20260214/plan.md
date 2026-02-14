# Implementation Plan: Real-time Camera Updates

## Phase 1: Infrastructure & Model Updates [checkpoint: 404f40d]
Update internal data structures to support persistent camera communication and the new `ray_trace` parameters.

- [x] Task: Update `RenderControl` struct in `src/lib.rs` to include `camera_config_sender: Option<Sender<solstrale::camera::CameraConfig>>`. 1280ecc
- [x] Task: Update `render` function signature in `src/render_output.rs` to return `(Receiver<RenderMessage>, Sender<bool>, Sender<solstrale::camera::CameraConfig>)`. a023d64
- [x] Task: Conductor - User Manual Verification 'Infrastructure & Model Updates' (Protocol in workflow.md) 404f40d

## Phase 2: Core Rendering Integration [checkpoint: 5baa8db]
Integrate the new `camera_config` receiver and `idle` parameter into the rendering thread lifecycle.

- [x] Task: In `src/render_output.rs`, update the `thread::spawn` block in the `render` function to pass the `camera_config_receiver` and `idle: true` to the `ray_trace` call. 3745131
- [x] Task: Implement the logic in `render` to return the `camera_config_sender`. 3745131
- [x] Task: Write a unit test in `src/render_output.rs` or a new test file to verify that the `render` function correctly returns the expected channels. 3745131
- [x] Task: Conductor - User Manual Verification 'Core Rendering Integration' (Protocol in workflow.md) 5baa8db

## Phase 3: UI Interaction Logic [checkpoint: 8538d46]
Refactor the UI loop to use the persistent channel for camera updates.

- [x] Task: Modify `render_output` in `src/render_output.rs` to detect if a render is already active when a camera update occurs. c0b58f8
- [x] Task: If active and only the camera changed, use `render_control.camera_config_sender` to send the new `solstrale::camera::CameraConfig`. c0b58f8
- [x] Task: Ensure the `loading_scene` flag is not set to `true` when performing a camera-only update. c0b58f8
- [x] Task: Update the "Handle render restarts" logic to preserve the `render_receiver` and thread if only the camera moved. c0b58f8
- [x] Task: Conductor - User Manual Verification 'UI Interaction Logic' (Protocol in workflow.md) 8538d46

## Phase 4: Final Verification & Cleanup
Ensure stability across all interaction types (YAML change, resize, camera move).

- [x] Task: Verify that resizing the window still triggers a full restart (correctly setting `render_control.render_requested = true`). f9280f0
- [x] Task: Verify that YAML changes still trigger a full restart. 859eacd
- [ ] Task: Run full project tests and linting.
- [ ] Task: Conductor - User Manual Verification 'Final Verification & Cleanup' (Protocol in workflow.md)
