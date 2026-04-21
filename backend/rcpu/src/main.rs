use glam::Vec3;
use rcpu::{RayTracer, SurfaceProperty, camera::Camera, light::LightEmissionDirection};

fn main() {
  let back = RayTracer::new();
  let mut scene = back.create_scene();
  let solver = back.create_solver();
  scene.add_sphere(2.0);
  let g = back.create_sphere(0.4);
  scene.add_light(
    g,
    SurfaceProperty::Uniform(Vec3::ONE),
    LightEmissionDirection::Omni,
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
