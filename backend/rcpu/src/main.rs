use glam::Vec3;
use rcpu::{RayTracer, camera::Camera};

fn main() {
  let back = RayTracer::new();
  let mut scene = back.create_scene();
  let solver = back.create_solver();
  scene.add_sphere(2.0);
  let mut cam = Camera::new(
    (640, 80).into(),
    Vec3::Y,
    Vec3::X,
    (5.0 * Vec3::NEG_X).into(),
    rcpu::camera::CameraType::Perspective,
  );
  solver.render(&scene, &mut cam);
}
