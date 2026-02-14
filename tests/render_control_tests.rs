use solstrale_desktop_rust::RenderControl;
use std::sync::mpsc::Sender;

#[test]
fn test_render_control_has_camera_config_sender() {
    let rc = RenderControl::default();
    let _: Option<Sender<solstrale::camera::CameraConfig>> = rc.camera_config_sender;
}
