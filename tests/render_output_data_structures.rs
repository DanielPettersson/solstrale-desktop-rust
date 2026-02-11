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
    // We want to remove texture_handle as we are moving to wgpu::Buffer
    // let _ = ri.texture_handle;
}
