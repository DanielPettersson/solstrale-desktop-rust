use solstrale::camera::CameraConfig;
use solstrale::geo::vec3::Vec3;
use std::f64::consts::PI;

#[derive(Clone, Debug, PartialEq)]
pub struct OrbitCamera {
    pub current_target: Vec3,
    pub current_distance: f64,
    pub current_azimuth: f64,
    pub current_polar: f64,

    pub target_target: Vec3,
    pub target_distance: f64,
    pub target_azimuth: f64,
    pub target_polar: f64,

    pub damping_factor: f64,
    pub vertical_fov_degrees: f64,
    pub aperture_size: f64,
    pub up: Vec3,
}

impl OrbitCamera {
    pub fn from_config(camera_config: &CameraConfig, damping_factor: f64) -> Self {
        let dir = camera_config.look_from - camera_config.look_at;
        let distance = dir.length();
        let azimuth = dir.x.atan2(dir.z);
        let polar = (dir.y / distance).acos();

        Self {
            current_target: camera_config.look_at,
            current_distance: distance,
            current_azimuth: azimuth,
            current_polar: polar,
            target_target: camera_config.look_at,
            target_distance: distance,
            target_azimuth: azimuth,
            target_polar: polar,
            damping_factor,
            vertical_fov_degrees: camera_config.vertical_fov_degrees,
            aperture_size: camera_config.aperture_size,
            up: camera_config.up,
        }
    }

    pub fn update(&mut self) -> bool {
        let mut changed = false;

        if (self.current_target - self.target_target).length() > 0.001 {
            self.current_target = self.current_target
                + (self.target_target - self.current_target) * self.damping_factor;
            changed = true;
        } else {
            self.current_target = self.target_target;
        }

        if (self.current_distance - self.target_distance).abs() > 0.001 {
            self.current_distance +=
                (self.target_distance - self.current_distance) * self.damping_factor;
            changed = true;
        } else {
            self.current_distance = self.target_distance;
        }

        if (self.current_azimuth - self.target_azimuth).abs() > 0.001 {
            self.current_azimuth +=
                (self.target_azimuth - self.current_azimuth) * self.damping_factor;
            changed = true;
        } else {
            self.current_azimuth = self.target_azimuth;
        }

        if (self.current_polar - self.target_polar).abs() > 0.001 {
            self.current_polar += (self.target_polar - self.current_polar) * self.damping_factor;
            changed = true;
        } else {
            self.current_polar = self.target_polar;
        }

        changed
    }

    pub fn look_from(&self) -> Vec3 {
        let x = self.current_distance * self.current_polar.sin() * self.current_azimuth.sin();
        let y = self.current_distance * self.current_polar.cos();
        let z = self.current_distance * self.current_polar.sin() * self.current_azimuth.cos();

        self.current_target + Vec3::new(x, y, z)
    }

    pub fn look_at(&self) -> Vec3 {
        self.current_target
    }

    pub fn orbit(&mut self, delta_azimuth: f64, delta_polar: f64) {
        self.target_azimuth += delta_azimuth;
        self.target_polar = (self.target_polar + delta_polar).clamp(0.01, PI - 0.01);
    }

    /// Relative zoom, positive amount moves closer. Scales with the current
    /// view distance, so the perceived speed is the same in any scene.
    pub fn zoom(&mut self, amount: f64) {
        self.target_distance = (self.target_distance * (-amount).exp()).max(0.01);
    }

    pub fn pan(&mut self, delta_x: f64, delta_y: f64, up: Vec3) {
        let look_from = self.look_from();
        let look_at = self.look_at();
        let forward = normalize(look_at - look_from);
        let right = normalize(cross(forward, up));
        let actual_up = normalize(cross(right, forward));

        let pan_vector = right * -delta_x + actual_up * delta_y;
        self.target_target += pan_vector;
    }
}

impl From<&OrbitCamera> for CameraConfig {
    fn from(c: &OrbitCamera) -> CameraConfig {
        CameraConfig {
            vertical_fov_degrees: c.vertical_fov_degrees,
            aperture_size: c.aperture_size,
            look_from: c.look_from(),
            look_at: c.look_at(),
            up: c.up,
        }
    }
}

fn normalize(v: Vec3) -> Vec3 {
    let len = v.length();
    if len > 0. { v / len } else { v }
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

/// A comparable copy of a camera config, which the library type is not.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraSnapshot {
    pub vertical_fov_degrees: f64,
    pub aperture_size: f64,
    pub look_from: Vec3,
    pub look_at: Vec3,
    pub up: Vec3,
}

impl CameraSnapshot {
    /// Equal but for the rounding that going through orbit angles adds
    pub fn approx_eq(&self, other: &CameraSnapshot) -> bool {
        let scale = 1e-6 * (1. + (self.look_from - self.look_at).length());
        (self.look_from - other.look_from).length() < scale
            && (self.look_at - other.look_at).length() < scale
            && self.up == other.up
            && self.vertical_fov_degrees == other.vertical_fov_degrees
            && self.aperture_size == other.aperture_size
    }
}

impl From<&CameraConfig> for CameraSnapshot {
    fn from(c: &CameraConfig) -> Self {
        CameraSnapshot {
            vertical_fov_degrees: c.vertical_fov_degrees,
            aperture_size: c.aperture_size,
            look_from: c.look_from,
            look_at: c.look_at,
            up: c.up,
        }
    }
}

impl From<&CameraSnapshot> for CameraConfig {
    fn from(c: &CameraSnapshot) -> Self {
        CameraConfig {
            vertical_fov_degrees: c.vertical_fov_degrees,
            aperture_size: c.aperture_size,
            look_from: c.look_from,
            look_at: c.look_at,
            up: c.up,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ViewAction {
    /// Keep the view the user has orbited to
    Keep,
    /// Move the view to the scene's camera
    Reset,
}

/// Whether a full re-render keeps the orbited view. It is kept unless the
/// scene's camera itself changed since the view was last set from it, so that
/// editing a material does not throw away where the user has moved to.
pub fn view_action(
    applied: Option<&CameraSnapshot>,
    scene_camera: &CameraSnapshot,
    has_view: bool,
    force_reset: bool,
) -> ViewAction {
    if force_reset || !has_view || applied != Some(scene_camera) {
        ViewAction::Reset
    } else {
        ViewAction::Keep
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(x: f64) -> CameraSnapshot {
        CameraSnapshot {
            vertical_fov_degrees: 60.,
            aperture_size: 0.,
            look_from: Vec3::new(x, 0., -10.),
            look_at: Vec3::new(0., 0., 0.),
            up: Vec3::new(0., 1., 0.),
        }
    }

    #[test]
    fn view_is_kept_unless_the_scene_camera_changed() {
        let a = snapshot(0.);
        assert_eq!(ViewAction::Keep, view_action(Some(&a), &a, true, false));
        assert_eq!(
            ViewAction::Reset,
            view_action(Some(&a), &snapshot(1.), true, false)
        );
        assert_eq!(ViewAction::Reset, view_action(None, &a, true, false));
        assert_eq!(ViewAction::Reset, view_action(Some(&a), &a, false, false));
        assert_eq!(ViewAction::Reset, view_action(Some(&a), &a, true, true));
    }

    #[test]
    fn from_config_round_trips() {
        let c = CameraConfig::from(&snapshot(3.));
        let orbit = OrbitCamera::from_config(&c, 1.);
        let back = CameraSnapshot::from(&CameraConfig::from(&orbit));
        let expected = snapshot(3.);
        assert!(
            (back.look_from - expected.look_from).length() < 1e-9,
            "{:?}",
            back
        );
        assert!(
            (back.look_at - expected.look_at).length() < 1e-9,
            "{:?}",
            back
        );
        assert_eq!(expected.vertical_fov_degrees, back.vertical_fov_degrees);
    }
}
