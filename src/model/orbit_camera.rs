use crate::model::{Creator, CreatorContext};
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
    pub fn new(
        cc: &crate::model::camera_config::CameraConfig,
        ctx: &CreatorContext,
        damping_factor: f64,
    ) -> Self {
        let camera_config = cc.create(ctx).unwrap();

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

    pub fn zoom(&mut self, delta_distance: f64) {
        self.target_distance = (self.target_distance + delta_distance).max(0.01);
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
