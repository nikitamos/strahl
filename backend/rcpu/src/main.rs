use std::sync::Arc;

use glam::{Quat, Vec3};
use rcpu::{
  Geometry, RayTracer, SurfaceProperty,
  camera::Camera,
  light::LightEmissionDirection,
  material::{ConcreteMaterial, bsdf::specular::Specular, medium::UniformMedium},
};

fn main() {
  let back = RayTracer::new();
  let mut scene = back.create_scene();
  let solver = back.create_solver();
  scene.add_sphere(2.0);
  let g: Arc<dyn Geometry> = back.create_sphere(0.4);
  scene.add_light(
    Arc::clone(&g),
    SurfaceProperty::Uniform(Vec3::ONE),
    LightEmissionDirection::Omni,
  );
  scene.add_body(
    g,
    Arc::new(ConcreteMaterial {
      medium: UniformMedium { ior: 1.0 },
      bsdf:   Specular {
        r: glam::vec3(0.0, 0.4, 0.3),
      },
    }),
    rcpu::TransformParts {
      pos:      glam::vec3(0.0, 0.0, 3.0).into(),
      rotation: Quat::IDENTITY,
    },
  );
  let mut cam = Camera::new(
    (640, 480).into(),
    Vec3::Y,
    Vec3::X,
    (5.0 * Vec3::NEG_Y).into(),
    rcpu::camera::CameraType::Perspective,
  );
  solver.render(&scene, &mut cam);
  let mut img = image::Rgb32FImage::new(640, 480);
  cam.write_image(&mut img);
  img.save("out.tiff").unwrap();
}
