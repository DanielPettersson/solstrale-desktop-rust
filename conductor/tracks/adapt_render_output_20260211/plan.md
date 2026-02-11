# Implementation Plan: Adapt Render Output to wgpu::Buffer

This plan outlines the steps to transition `render_output.rs` from image-based rendering to direct `wgpu::Buffer` display using a fragment shader and `egui::PaintCallback`.

## Phase 1: Infrastructure and Shader Setup
Prepare the necessary `wgpu` structures and the embedded shader.

- [ ] Task: Define Embedded WGSL Shader
    - [ ] Create a constant string in `render_output.rs` containing the WGSL shader code.
    - [ ] Implement a basic fragment shader that samples from a buffer/texture (depending on buffer layout).
- [ ] Task: Define Render Resources Structure
    - [ ] Define a struct to hold `wgpu` resources (Pipeline, BindGroupLayout, etc.) needed for the `PaintCallback`.
    - [ ] Implement `egui_wgpu::CallbackTrait` for this struct.

## Phase 2: Refactor render_output.rs
Modify the existing UI component to handle the new buffer-based progress.

- [ ] Task: Update Data Structures
    - [ ] Update `RenderOutput` and related structs to store references or IDs to the `wgpu::Buffer` instead of `RetainedImage` or similar.
- [ ] Task: Implement PaintCallback Logic
    - [ ] Create the `egui::PaintCallback` in the `ui` function of `RenderOutput`.
    - [ ] Ensure the callback correctly passes the current `output_buffer` to the GPU pipeline.
- [ ] Task: Handle Scaling and Resizing
    - [ ] Update the vertex/fragment shader logic or the callback's transformation matrix to implement "Stretch to Fit".

## Phase 3: Verification and Cleanup
Ensure everything works as expected and adheres to the project's quality standards.

- [ ] Task: Verify Compilation and Rendering
    - [ ] Fix any type errors arising from the `solstrale` dependency update.
    - [ ] Run the application and confirm the render output is displayed.
- [ ] Task: Test Window Resizing
    - [ ] Manually verify that the output scales correctly when the window is resized.
- [ ] Task: Conductor - User Manual Verification 'Adapt Render Output' (Protocol in workflow.md)
