# Specification: Adapt Render Output to wgpu::Buffer

## Overview
The `solstrale` library has updated its `RenderProgress` structure, replacing the `render_image` field with an `output_buffer` of type `wgpu::Buffer`. This track involves updating `src/render_output.rs` to consume this buffer directly and display it efficiently using a fragment shader integrated into the `egui` UI.

## Functional Requirements
- **Efficient Display:** Replace CPU-side image processing with a GPU-side fragment shader that reads directly from the `wgpu::Buffer`.
- **egui Integration:** Utilize `egui::PaintCallback` to execute the custom `wgpu` rendering logic within the standard `egui` render pass.
- **Embedded Shader:** The WGSL shader code for displaying the buffer will be embedded directly within the Rust source code.
- **Aspect Ratio/Scaling:** The fragment shader will handle scaling the rendered buffer to fit the available UI area (Stretch to Fit).

## Non-Functional Requirements
- **Performance:** Minimize CPU-to-GPU data transfers by utilizing the buffer already present on the GPU.
- **Responsiveness:** Ensure the UI remains fluid while rendering and displaying the high-frequency updates from the path tracer.

## Acceptance Criteria
- [ ] `src/render_output.rs` successfully compiles after adapting to the `output_buffer` field in `RenderProgress`.
- [ ] The rendered output is visible in the application viewport.
- [ ] Resizing the application window correctly scales the rendered output.
- [ ] No significant performance regressions are observed compared to the previous image-based approach.

## Out of Scope
- Modifying the internal logic of the `solstrale` rendering engine.
- Adding new post-processing effects in this specific update (unless required for display).
