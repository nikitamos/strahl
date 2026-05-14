#![feature(iter_map_windows)]

use std::{io::Write, sync::Arc};

use glam::{Quat, Vec3};
use rcpu::{
  Geometry, RayTracer, Sampler, SurfaceProperty,
  camera::{Camera, CameraRay},
  light::LightEmissionDirection,
  material::{ConcreteMaterial, bsdf::lambertian::Lambertian, medium::UniformMedium},
  solver::PathTerminator,
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
    _gen_vertex: &rcpu::solver::PathVertex,
    sampler: &Sampler,
  ) -> bool {
    sampler.sample().uniform_1d >= self.prob || self.n <= vertices
  }
}

fn main() {
  let back = RayTracer::new();
  let mut scene = back.create_scene();
  scene.add_sphere(1.0);
  let g: Arc<dyn Geometry> = back.create_sphere(0.4);
  scene.add_light(
    Arc::clone(&g),
    SurfaceProperty::Uniform(Vec3::ONE),
    LightEmissionDirection::Omni,
  );
  scene.add_body(
    back.create_sphere(20.0),
    Arc::new(ConcreteMaterial {
      medium: UniformMedium { ior: 1.0 },
      bsdf:   Lambertian {
        s: glam::vec3(0.76, 0.8, 0.1),
      },
    }),
    rcpu::TransformParts {
      pos:      Vec3::ZERO.into(),
      rotation: Quat::IDENTITY,
    },
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
      pos:      glam::vec3(0.0, -0.4, -3.0).into(),
      rotation: Quat::IDENTITY,
    },
  );
  /*
  let mut f = std::fs::File::create("light-paths.csv").unwrap();
  for i in 0..64 {
    let path = rcpu::solver::BidirectionalPath::sample_eye_subpath(
      &scene,
      &Sampler::new(),
      &mut CameraRay::new((5.0 * Vec3::NEG_Y).into(), Vec3::Y.into()),
      &3,
      Default::default(),
    );
    // rcpu::solver::BidirectionalPath::sample_light_subpath(&scene, &Sampler::new(), &6usize);
    path
      .vertices
      .iter()
      .map_windows(|[l, r]| {
        writeln!(
          &mut f,
          "{},{},{},{},{},{}",
          l.point.x, l.point.y, l.point.z, r.point.x, r.point.y, r.point.z
        )
        .unwrap();
      })
      .for_each(|()| ());
  }
  */

  let mut cam = Camera::new(
    (480, 480).into(),
    Vec3::Y,
    Vec3::X,
    (5.0 * Vec3::NEG_Y).into(),
    rcpu::camera::CameraType::Perspective,
  );
  for l in 1..3 {
    for e in 2..4 {
      let solver = back.create_bdpt_solver(l, e, if l + e == 3 { 24 } else { 24 });
      solver.render(&scene, &mut cam);
      let mut img = image::Rgb32FImage::new(480, 480);
      cam.write_image(&mut img);
      img.save(format!("l{l}-e{e}.tiff")).unwrap();
    }
  }
}
