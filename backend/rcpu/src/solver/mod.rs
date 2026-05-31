use std::{fs::File, io::Write, marker::PhantomData};

use super::{Interaction, IntersectionContext, Scene, Spectrum};
use crate::{
  Castable, PointGlobal, Sample, Sampler,
  camera::{self, CameraRay},
  light::{LightSampleContext, LightSampleMetadata},
  material::bsdf::BSDFSampleContext,
};
use glam::{Vec3, Vec3Swizzles};
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
    const SAMPLES: i32 = 256;
    rays.into_par_iter().enumerate().for_each(|(_i, ray)| {
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
    for _b in 0..BOUNCES {
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
        && light.prob != 0.0
      {
        let wi = isect.hit.global_to_hit(
          (light.metadata.point.xyz() - isect.hit.point_global().xyz())
            .normalize()
            .into(),
        );
        if let Some(bsdf2) = bsdf.bsdf2(cur_ray, wi, BSDFSampleContext::Camera) {
          // In hit space wi.z corresponds to dot(wi, normal)
          // Note that if using normal mapping that's generally not the case.
          let f = bsdf2.sample * cur_ray.z.abs(); // is this really *that* angle?
          ray.color += throughput * f * light.sample / (light.prob) / 4.0;
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
  fn sample_light(
    &self,
    scene: &Scene,
    dest: PointGlobal,
  ) -> Option<Sample<Spectrum, LightSampleMetadata>> {
    scene
      .sample_light_source(&self.sampler, dest)
      .and_then(|src, ()| {
        src.sample(self.sampler.sample(), LightSampleContext {
          dst: dest,
          scene,
        })
      })
  }
  pub(crate) fn hit_light(&self, scene: &Scene, ray: &dyn Castable) -> Option<Spectrum> {
    let light = &scene.lights[0];
    light.try_intersect(ray).map(|_hit| light.spectrum.get())
  }
  pub(crate) fn closest_hit<'a>(scene: &'a Scene, r: &impl Castable) -> Option<Interaction<'a>> {
    let mut x = 0;
    scene
      .bodies
      .iter()
      .filter_map(|b| {
        let ctx = IntersectionContext {
          transform: b.transform(),
        };
        b.geometry
          .try_intersect(ctx, r)
          .map(|hit| Interaction {
            body: b,
            ray_dir: hit.transform.v2local(r.direction()),
            hit,
          })
          .inspect(|_| x += 1)
      })
      .min_by(|a, b| a.hit.ray_distance.partial_cmp(&b.hit.ray_distance).unwrap())
  }
}

pub mod bdpt;
