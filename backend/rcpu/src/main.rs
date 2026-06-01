#![feature(iter_map_windows)]

use std::{io::Write, sync::Arc};

use glam::{Quat, Vec3};
use rcpu::{
  Geometry, Quad, RayTracer, Sampler, Scene, SurfaceProperty, TransformParts,
  camera::{Camera, CameraRay},
  light::LightEmissionDirection,
  material::{
    ConcreteMaterial,
    bsdf::{lambertian::Lambertian, specular::Specular},
    medium::UniformMedium,
  },
  scene_loader::{SceneLoadError, SceneLoader},
  solver::bdpt::PathTerminator,
};

struct UniformLEqPath {
  n:    usize,
  prob: f32,
}

impl UniformLEqPath {
  fn new(n: usize, prob: f32) -> Self { Self { n, prob } }
}

impl PathTerminator for UniformLEqPath {
  fn should_terminate(
    &self,
    vertices: usize,
    _gen_vertex: &rcpu::solver::bdpt::PathVertex,
    sampler: &Sampler,
  ) -> bool {
    sampler.sample().uniform_1d >= self.prob || self.n <= vertices
  }
}

fn main() {
  let back = RayTracer::new();
  let scene = read_scene_from_file().unwrap();

  let cam = Camera::new(
    (320, 320).into(),
    Vec3::Y,
    Vec3::X,
    (5.0 * Vec3::NEG_Y + 4.0 * Vec3::Z).into(),
    rcpu::camera::CameraType::Perspective,
  );
  // bdpt_solve(back, scene, cam);
  solve2(back, scene, cam);
}

fn read_scene_from_file() -> Result<Scene, SceneLoadError> {
  let mut loader = SceneLoader::new();
  loader.load("test-scene.toml")
}

fn solve2(back: RayTracer, scene: rcpu::Scene, mut cam: Camera) {
  let depth = 8;
  let samples = 256;
  let solver = back.create_solver2(depth, samples);
  solver.render(&scene, &mut cam);
  let mut img = image::Rgb32FImage::new(320, 320);
  cam.write_image(&mut img);
  img.save(format!("solver-{depth}-x{samples}.tiff")).unwrap();
}
