# Specification: Interactive Camera Controls (OrbitControls style)

## Overview
Implement interactive camera controls in the render output window, allowing users to orbit, pan, and zoom using the mouse. This feature mimics the behavior of `Three.js OrbitControls`, providing an intuitive way to explore the 3D scene.

## Functional Requirements
- **Orbiting:** Left-click and drag in the render output to rotate the camera around its target.
- **Panning:** Right-click (or middle-click) and drag to move the camera and target parallel to the view plane.
- **Zooming:** Use the mouse scroll wheel to move the camera closer to or further from the target.
- **Damping:** Implement movement smoothing (inertia) so the camera glides to a stop after mouse input ends.
- **Efficient Updates:** When the camera moves, the render must restart immediately using the mutated camera parameters. The scene should NOT be re-parsed from YAML; instead, the existing scene's camera fields should be updated directly.
- **Initial State:** The initial camera position and target (look_at) should be derived from the scene's YAML configuration.

## Non-Functional Requirements
- **Fluidity:** The UI must remain responsive while camera calculations and render restarts occur.
- **Accuracy:** Camera movements should feel natural and intuitive, matching user expectations for 3D navigation.

## Acceptance Criteria
- [ ] Dragging with the left mouse button rotates the camera around the target.
- [ ] Dragging with the right/middle mouse button pans the camera.
- [ ] Scrolling the mouse wheel zooms in and out.
- [ ] The camera continues to move slightly after releasing the mouse button (damping).
- [ ] The render output restarts as soon as the camera is moved.
- [ ] Changes in camera position are reflected in the render without a full scene reload from YAML.

## Out of Scope
- Persisting the moved camera position back to the YAML file (this can be a separate track).
- Advanced camera features like field-of-view (FOV) adjustments via mouse.
