use std::{fs::File, io::Write};

use super::{Interaction, IntersectionContext, Scene, Spectrum};
use crate::{
  Castable, Point, PointGlobal, Sample, Sampler, camera::{self, CameraRay}, light::{LightSampleContext, LightSampleMetadata}, material::bsdf::BSDFSampleContext
};
use glam::{Vec3, Vec3Swizzles, Vec4Swizzles};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

pub struct Solver {
  pub(crate) sampler: Sampler,
}

impl Solver {
  pub(crate) fn new() -> Self {
    Self {
      sampler: Sampler::default(),
    }
  }
  pub fn render(&self, scene: &Scene, cam: &mut camera::Camera) {
    let rays = cam.init_rays();
    const SAMPLES: i32 = 16;
    rays.into_par_iter().enumerate().for_each(|(i, ray)| {
      for _ in 0..SAMPLES {
        self.trace_camera_ray(scene, ray);
        ray.reset_direction();
      }
      ray.color /= SAMPLES as f32;
    });
    #[cfg(false)]
    Self::dump_rays(rays);
  }

  fn dump_rays(rays: &[CameraRay]) {
    let mut file = File::create("scatter.csv").unwrap();
    for r in rays {
      let next = r.origin.xyz() + r.direction;
      let _ = writeln!(
        file,
        "{},{},{},{},{},{}",
        r.origin.x, r.origin.y, r.origin.z, next.x, next.y, next.z
      );
    }
  }

  fn trace_camera_ray(&self, scene: &Scene, ray: &mut CameraRay) {
    let mut throughput = Vec3::ONE;
    const BOUNCES: u32 = 1;
    for b in 0..BOUNCES {
      let Some(isect) = Self::closest_hit(scene, ray) else {
        // Infinite lights
        // println!("no hit, exiting {b}");
        break;
      };
      // if emissive then L += beta * interaction.emission(-ray.dir) w.r.t. spectrum
      // if b == BOUNCES break;
      let bsdf = isect.body.material.bsdf();
      let cur_ray = isect.hit.to_hit((-ray.direction).into());
      if let Some(light) = self.sample_light(scene, isect.hit.point_global())
        && light.prob != 0.0 {
          let wi = isect.hit.global_to_hit(
            (light.metadata.point.xyz() - isect.hit.point_global().xyz())
              .normalize()
              .into(),
          );
          if let Some(bsdf2) = bsdf.bsdf2(cur_ray, wi, BSDFSampleContext::Camera) {
            // In hit space wi.z corresponds to dot(wi, normal)
            // Note that if using normal mapping that's generally not the case.
            let f = bsdf2.sample * cur_ray.z.abs(); // is this really *that* angle?
            ray.color += throughput * f * light.sample / (light.prob) / 4.0; // Why 4.0?
          }
        }
      // Sample a new direction
      let Some(bs) = bsdf.sample_bsdf(cur_ray, self.sampler.sample(), BSDFSampleContext::Camera)
      else {
        break;
      };
      throughput *= bs.sample * bs.metadata.inc.z.abs() / bs.prob;
      ray.direction = isect.hit.to_global(bs.metadata.inc).normalize();
      ray.origin = (isect.hit.point_global().xyz() + ray.direction * 0.0001).into();
    }
  }
  
  fn trace_mlt(&self, scene: &Scene, ray: &mut CameraRay) {
    // Probably we'd like to create a path here?
    // 1. Sample path
    // 2.
  }

  #[allow(unused)]
  fn mlt(&self, scene: &Scene, image: ()) {
    let mut x = MLTPath::new_with_length(4, scene, MLTPathPolicy {  });
    const MUTATIONS: u32 = 4;
    for i in 0..MUTATIONS {
      let y = x.mutate(&());
      let a = y.prob;
      if self.sampler.sample().uniform_1d < a {
        x = y.sample;
      }
      // Pixels may be updating by filtering
      todo!("record sample(image, x)");
    }
  }

  #[allow(unused)]
  fn mlt_init<'s>(&self, scene: &'s Scene, path_count: u32, retain: &mut [MLTPath<'s>]) {
    
  }


  fn sample_light(
    &self,
    scene: &Scene,
    dest: PointGlobal,
  ) -> Option<Sample<Spectrum, LightSampleMetadata>> {
    scene
      .sample_light_source(&self.sampler, dest)
      .compose(|src, ()| {
        src.sample(self.sampler.sample(), LightSampleContext {
          dst: dest,
          scene,
        })
      })
  }

  pub(crate) fn closest_hit<'a>(scene: &'a Scene, r: &mut CameraRay) -> Option<Interaction<'a>> {
    scene
      .bodies
      .iter()
      .filter_map(|b| {
        let ctx = IntersectionContext {
          transform: b.transform(),
        };
        b.geometry.try_intersect(ctx, r).map(|hit| Interaction {
          body: b,
          ray_dir: hit.transform.v2local(r.direction()),
          hit,
        })
      })
      .min_by(|a, b| a.hit.ray_distance.partial_cmp(&b.hit.ray_distance).unwrap())
  }
}

struct MLTVertex(PointGlobal);

impl MLTVertex {
  pub fn global(&self) -> PointGlobal {
    self.0
  }
}

pub struct MLTPath<'a> {
  scene: &'a Scene,
  vertices: Vec<MLTVertex>
}

#[derive(Default)]
pub struct MLTPathPolicy {

}

pub struct BidirectionalMutationProb {

}

pub enum ScatterEventType {
  Diffuse, Specular
}

pub trait MutationStrategies {
  fn bidirectional(&self, path: &mut MLTPath, sampler: &Sampler) -> f32 {
    let last_del = sampler.usize(0, path.len());
    let first_del = sampler.usize(0, last_del);

    // new subpath len
    let k = 2usize;
    // Sample new subpath(s)
    // Sample the BSDF at the endpoint and cast some rays...
    // OR IF the subpath is terminal, sample a point at lens or light source.

    // Join the subpaths or reject the mutation
    // regeneration may be required?
    todo!()
  }

  fn lens_perturbation(&self, path: &mut MLTPath, sampler: &Sampler) -> f32 {

    todo!()
  }

  fn caustics_perturbation(&self, path: &mut MLTPath, sampler: &Sampler) -> f32 {

    todo!()
  }
}

impl MutationStrategy for () {

}

impl<'a> MLTPath<'a> {
  pub fn len(&self) -> usize {
    0
  }
  pub fn mutate(&self, strategy: &impl MutationStrategy) -> Sample<MLTPath<'a>> {
    todo!()
  }

  pub fn acceptance_prob(&self) -> f32 {
    todo!()
  }

  pub fn f(&self) -> f32 {
    todo!()
  }

  pub fn new_with_length(length: u32, scene: &Scene, policy: MLTPathPolicy) -> Self {
    todo!()
  }

  pub fn a(x: &Self, y: &Self) -> f32{
    (y.f() * Self::t(y, x) / x.f() / Self::t(x, y)).min(1.0)
  }

  pub fn t(x: &Self, y: &Self) -> f32{
    todo!()
  }
}

// K(x, y) is Markov chain kernel:
// \forall x \in \Omega \int_\Omega K(x \to y) \dd \mu(y) = 1 
// p_i(x) = \int_\Omega K(x \to y) p_{i-1}(y) \dd \mu(y)
// p_i converges to certain density function p^* (stationary distr) independently of ICs.

// detailed balance -- what is it?
