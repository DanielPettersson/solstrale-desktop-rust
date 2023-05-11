use solstrale::camera::CameraConfig;
use solstrale::geo::vec3::Vec3;
use solstrale::hittable::hittable_list::HittableList;
use solstrale::hittable::obj_model::load_obj_model;
use solstrale::hittable::rotation_y::RotationY;
use solstrale::hittable::sphere::Sphere;
use solstrale::hittable::translation::Translation;
use solstrale::hittable::Hittable;
use solstrale::material::texture::SolidColor;
use solstrale::material::{Dielectric, DiffuseLight, Lambertian};
use solstrale::renderer::{RenderConfig, Scene};

pub fn create_scene(render_config: RenderConfig) -> Scene {
    let camera = CameraConfig {
        vertical_fov_degrees: 20.,
        aperture_size: 0.1,
        focus_distance: 3.,
        look_from: Vec3::new(-0.2, 0.8, 4.),
        look_at: Vec3::new(0., 0.5, 0.),
    };

    let mut world = HittableList::new();
    let yellow = Lambertian::new(SolidColor::new(1., 1., 0.));
    let glass = Dielectric::new(SolidColor::new(1., 1., 1.), 1.5);
    let red = Lambertian::new(SolidColor::new(1., 0., 0.));
    let light = DiffuseLight::new(10., 10., 10.);
    world.add(Sphere::new(Vec3::new(50., 100., 50.), 20., light));
    world.add(Sphere::new(Vec3::new(0., 0.5, 0.), 0.5, glass));
    world.add(Sphere::new(Vec3::new(0.5, 0.3, -0.2), 0.3, yellow));
    world.add(Sphere::new(Vec3::new(0., -50., 0.), 50., red));

    let dragon = load_obj_model("/home/daniel/", "owl.obj", 0.3).unwrap();
    world.add(RotationY::new(
        Translation::new(dragon, Vec3::new(-0.8, 0., 0.3)),
        35.,
    ));

    Scene {
        world,
        camera,
        background_color: Vec3::new(0.2, 0.3, 0.5),
        render_config,
    }
}
