#![feature(iter_map_windows)]

use std::sync::Arc;

use glam::{Quat, Vec3};
use rcpu::{
  RayTracer, Sampler, Scene,
  camera::Camera,
  material::{ConcreteMaterial, bsdf::dielectric::Dielectric, medium::UniformMedium},
  scene_loader::{SceneLoadError, SceneLoader},
};

#[cfg(false)]
mod bdpt_legacy {
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
}

fn main() {
  let back = RayTracer::new();
  let mut scene = read_scene_from_file().unwrap();
  scene.add_body(
    back.create_sphere(1.0),
    Arc::new(ConcreteMaterial {
      medium: (UniformMedium { ior: 1.33 }),
      bsdf:   (Dielectric {
        transmission: Vec3::new(0.6, 0.6, 0.6),
        reflection:   Vec3::ONE,
      }),
    }),
    rcpu::TransformParts {
      pos:      glam::vec3(0.5, -1.0, 4.2).into(),
      rotation: Quat::IDENTITY,
    },
  );

  let cam = Camera::new(
    (320, 320).into(),
    Vec3::Y,
    Vec3::X,
    (6.5 * Vec3::NEG_Y + 4.0 * Vec3::Z).into(),
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
  let samples = 2048;
  let solver = back.create_solver2(depth, samples);
  solver.render(&scene, &mut cam);
  let mut img = image::Rgb32FImage::new(320, 320);
  cam.write_image(&mut img);
  img.save(format!("solver-{depth}-x{samples}.tiff")).unwrap();
}
