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
}

impl OrbitCamera {
    pub fn new(look_from: Vec3, look_at: Vec3, damping_factor: f64) -> Self {
        let dir = look_from - look_at;
        let distance = dir.length();
        let azimuth = dir.x.atan2(dir.z);
        let polar = (dir.y / distance).acos();

        Self {
            current_target: look_at,
            current_distance: distance,
            current_azimuth: azimuth,
            current_polar: polar,
            target_target: look_at,
            target_distance: distance,
            target_azimuth: azimuth,
            target_polar: polar,
            damping_factor,
        }
    }

    pub fn update(&mut self) -> bool {
        let mut changed = false;

        if (self.current_target - self.target_target).length() > 0.001 {
            self.current_target =
                self.current_target + (self.target_target - self.current_target) * self.damping_factor;
            changed = true;
        } else {
            self.current_target = self.target_target;
        }

        if (self.current_distance - self.target_distance).abs() > 0.001 {
            self.current_distance += (self.target_distance - self.current_distance) * self.damping_factor;
            changed = true;
        } else {
            self.current_distance = self.target_distance;
        }

        if (self.current_azimuth - self.target_azimuth).abs() > 0.001 {
            self.current_azimuth += (self.target_azimuth - self.current_azimuth) * self.damping_factor;
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
        self.target_target = self.target_target + pan_vector;
    }
}

fn normalize(v: Vec3) -> Vec3 {
    let len = v.length();
    if len > 0. {
        v / len
    } else {
        v
    }
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use solstrale::geo::vec3::Vec3;

    #[test]
    fn test_orbit_camera_init() {
        let look_from = Vec3::new(0.0, 0.0, 10.0);
        let look_at = Vec3::new(0.0, 0.0, 0.0);
        let camera = OrbitCamera::new(look_from, look_at, 0.1);

        assert!((camera.current_distance - 10.0).abs() < 0.001);
        assert!((camera.current_polar - PI / 2.0).abs() < 0.001);
        assert!((camera.current_azimuth - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_orbit_camera_look_from() {
        let look_from = Vec3::new(10.0, 0.0, 0.0);
        let look_at = Vec3::new(0.0, 0.0, 0.0);
        let camera = OrbitCamera::new(look_from, look_at, 1.0); // No damping for test

        let calculated_look_from = camera.look_from();
        assert!((calculated_look_from.x - 10.0).abs() < 0.001);
        assert!((calculated_look_from.y - 0.0).abs() < 0.001);
        assert!((calculated_look_from.z - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_orbit_camera_damping() {
        let look_from = Vec3::new(0.0, 0.0, 10.0);
        let look_at = Vec3::new(0.0, 0.0, 0.0);
        let mut camera = OrbitCamera::new(look_from, look_at, 0.1);

        camera.orbit(PI / 2.0, 0.0);
        assert!((camera.current_azimuth - 0.0).abs() < 0.001);
        assert!((camera.target_azimuth - PI / 2.0).abs() < 0.001);

        camera.update();
        assert!(camera.current_azimuth > 0.0);
        assert!(camera.current_azimuth < PI / 2.0);
    }

    #[test]
    fn test_orbit_camera_pan() {
        let look_from = Vec3::new(0.0, 0.0, 10.0);
        let look_at = Vec3::new(0.0, 0.0, 0.0);
        let mut camera = OrbitCamera::new(look_from, look_at, 1.0);

        camera.pan(1.0, 1.0, Vec3::new(0.0, 1.0, 0.0));
        camera.update();

        let new_look_at = camera.look_at();
        assert!(new_look_at.x < 0.0);
        assert!(new_look_at.y > 0.0);
    }
}
