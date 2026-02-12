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
fn test_render_progress_fields() {
    let (output_sender, output_receiver) = std::sync::mpsc::channel();
    // We can't easily create a RenderProgress without running ray_trace,
    // but we can check if it has certain fields by trying to access them in code that won't run.
    if false {
        let rp: solstrale::renderer::RenderProgress = unsafe { std::mem::zeroed() };
        let _ = rp.output_buffer;
        let _ = rp.device;
        let _ = rp.queue;
    }
}
