use solstrale::camera::CameraConfig;
use solstrale::geo::vec3::Vec3;
use solstrale::hittable::hittable_list::HittableList;
use solstrale::hittable::sphere::Sphere;
use solstrale::hittable::Hittable;
use solstrale::material::texture::SolidColor;
use solstrale::material::{DiffuseLight, Lambertian};
use solstrale::renderer::{RenderConfig, Scene};

pub fn create_scene(render_config: RenderConfig) -> Scene {
    let camera = CameraConfig {
        vertical_fov_degrees: 20.,
        aperture_size: 0.1,
        focus_distance: 10.,
        look_from: Vec3::new(0., 0., 4.),
        look_at: Vec3::new(0., 0., 0.),
    };

    let mut world = HittableList::create();
    let yellow = Lambertian::create(SolidColor::create(1., 1., 0.));
    let light = DiffuseLight::create(10., 10., 10.);
    world.add(Sphere::create(Vec3::new(0., 100., 0.), 20., light));
    world.add(Sphere::create(Vec3::new(0., 0., 0.), 0.5, yellow));

    Scene {
        world,
        camera,
        background_color: Vec3::new(0.2, 0.3, 0.5),
        render_config,
    }
}
