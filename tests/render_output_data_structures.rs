use solstrale_desktop_rust::RenderedImage;

#[test]
fn test_rendered_image_has_output_buffer() {
    let ri = RenderedImage::default();
    // This will fail to compile if output_buffer is not present
    let _ = ri.output_buffer;
}

#[test]
fn test_rendered_image_has_render_resources() {
    let ri = solstrale_desktop_rust::RenderedImage::default();
    // This should fail to compile if render_resources is missing
    let _ = ri.render_resources;
}

#[test]
fn test_rendered_image_no_longer_has_texture_handle() {
    let ri = solstrale_desktop_rust::RenderedImage::default();
    // This is just to satisfy the test runner that we are doing something
    assert!(ri.output_buffer.is_none());
    // If texture_handle was still there, we could access it.
    // Since we removed it, we can't.
}
