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
  let mut scene = back.create_scene();
  scene.add_sphere(1.0);
  let g: Arc<dyn Geometry> = back.create_sphere(0.35);
  scene.add_light(
    Arc::clone(&g),
    SurfaceProperty::Uniform(Vec3::splat(10.2)),
    LightEmissionDirection::Omni,
    -2.0 * glam::vec3(1.0, 1.0, 0.4),
  );
  scene.add_body(
    back.create_sphere(28.0),
    Arc::new(ConcreteMaterial {
      medium: UniformMedium { ior: 1.0 },
      bsdf:   Specular { r: Vec3::ONE }, // bsdf:   Lambertian {
                                         //   s: Vec3::X, //glam::vec3(1.0, 1.0, 1.0),
                                         // },
    }),
    rcpu::TransformParts {
      pos:      (glam::vec3(30.0, 0.0, 4.0)).into(),
      rotation: Quat::IDENTITY,
    },
  );
  scene.add_body(
    Arc::new(Quad::yz_square((Vec3::X * -3.0).into(), 15.0)),
    Arc::new(ConcreteMaterial {
      medium: UniformMedium { ior: 1.0 },
      bsdf:   Lambertian { s: Vec3::Z }, // bsdf:   Lambertian {
                                         //   s: Vec3::X, //glam::vec3(1.0, 1.0, 1.0),
                                         // },
    }),
    TransformParts::IDENTITY,
  );
  scene.add_body(
    g,
    Arc::new(ConcreteMaterial {
      medium: UniformMedium { ior: 1.0 },
      bsdf:   Lambertian {
        s: glam::vec3(0.0, 0.4, 0.3),
      },
    }),
    rcpu::TransformParts {
      pos:      glam::vec3(0.0, 0.4, -3.0).into(),
      rotation: Quat::IDENTITY,
    },
  );
  let s = read_scene_from_file().unwrap();

  let cam = Camera::new(
    (480, 480).into(),
    Vec3::Y,
    Vec3::X,
    (5.0 * Vec3::NEG_Y).into(),
    rcpu::camera::CameraType::Perspective,
  );
  // bdpt_solve(back, scene, cam);
  solve2(back, s, cam);
}

fn read_scene_from_file() -> Result<Scene, SceneLoadError> {
  let mut loader = SceneLoader::new();
  loader.load("test-scene.toml")
}

fn solve2(back: RayTracer, scene: rcpu::Scene, mut cam: Camera) {
  let solver = back.create_solver2(3, 64);
  solver.render(&scene, &mut cam);
  let mut img = image::Rgb32FImage::new(480, 480);
  cam.write_image(&mut img);
  img.save(format!("solver2.tiff")).unwrap();
}
