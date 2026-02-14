# Specification: Real-time Camera Updates

## Overview
This track aims to optimize camera interactions (orbit, pan, zoom) by utilizing the new `camera_config` receiver and `idle` parameter in the Solstråle `ray_trace` function. Instead of aborting and restarting the entire rendering process when the camera moves, we will send updated camera configurations to the running rendering thread.

## Functional Requirements
- **Efficient Camera Updates:** When only the camera is updated (orbit, pan, zoom), the UI should send a new `CameraConfig` to the rendering thread via a dedicated channel instead of restarting the renderer.
- **Improved `RenderControl`:** Add a `camera_config_sender` to `RenderControl` to persist the connection to the active rendering thread.
- **Rendering Thread Persistence:** The `render` function will be updated to return the `camera_config_sender`, allowing `render_output` to communicate with it.
- **Idle Mode:** Enable the `idle` parameter in `ray_trace` to allow the rendering thread to wait for updates or abort signals after completing a render pass, preventing unnecessary thread destruction.
- **Full Restart on Scene Change:** Changes to the scene YAML or viewport size will still trigger a full render restart as they require re-parsing or re-allocation of resources.

## Non-Functional Requirements
- **Responsiveness:** Camera movements should feel significantly more responsive as the overhead of thread creation and scene re-parsing is removed for these interactions.
- **Resource Efficiency:** Reduced CPU/GPU overhead by avoiding redundant scene setup during interactive camera adjustments.

## Acceptance Criteria
- [ ] Rotating, panning, or zooming the camera does not cause the "Loading scene..." state if the scene has already been loaded.
- [ ] Camera updates are reflected in the render output in real-time during interaction.
- [ ] The rendering thread stays alive across multiple camera adjustments.
- [ ] Modifying the scene YAML correctly triggers a full restart and shows the loading state if applicable.
- [ ] Resizing the window correctly triggers a full restart to match the new viewport.

## Out of Scope
- Real-time updates for other scene properties (materials, lights, etc.) via this mechanism.
- Changes to the underlying `solstrale` library.
