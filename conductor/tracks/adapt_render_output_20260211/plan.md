# Implementation Plan: Adapt Render Output to wgpu::Buffer

This plan outlines the steps to transition `render_output.rs` from image-based rendering to direct `wgpu::Buffer` display using a fragment shader and `egui::PaintCallback`.

## Phase 1: Infrastructure and Shader Setup
Prepare the necessary `wgpu` structures and the embedded shader.

- [x] Task: Define Embedded WGSL Shader 8f0a7d6
    - [x] Create a constant string in `render_output.rs` containing the WGSL shader code.
    - [x] Implement a basic fragment shader that samples from a buffer/texture (depending on buffer layout).
- [x] Task: Define Render Resources Structure ec84ed9
    - [x] Define a struct to hold `wgpu` resources (Pipeline, BindGroupLayout, etc.) needed for the `PaintCallback`.
    - [x] Implement `egui_wgpu::CallbackTrait` for this struct.

## Phase 2: Refactor render_output.rs [checkpoint: bbf28fb]
Modify the existing UI component to handle the new buffer-based progress.

- [x] Task: Update Data Structures 04f01fd
    - [x] Update `RenderOutput` and related structs to store references or IDs to the `wgpu::Buffer` instead of `RetainedImage` or similar.
- [x] Task: Implement PaintCallback Logic bbf28fb
    - [x] Create the `egui::PaintCallback` in the `ui` function of `RenderOutput`.
    - [x] Ensure the callback correctly passes the current `output_buffer` to the GPU pipeline.
- [x] Task: Handle Scaling and Resizing bbf28fb
    - [x] Update the vertex/fragment shader logic or the callback's transformation matrix to implement "Stretch to Fit".

## Phase 3: Verification and Cleanup
Ensure everything works as expected and adheres to the project's quality standards.

- [ ] Task: Verify Compilation and Rendering
    - [ ] Fix any type errors arising from the `solstrale` dependency update.
    - [ ] Run the application and confirm the render output is displayed.
- [ ] Task: Test Window Resizing
    - [ ] Manually verify that the output scales correctly when the window is resized.
- [ ] Task: Conductor - User Manual Verification 'Adapt Render Output' (Protocol in workflow.md)
