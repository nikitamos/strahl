use std::{fs::File, io::Write, marker::PhantomData};

use super::{Interaction, IntersectionContext, Scene, Spectrum};
use crate::{
  Castable, PointGlobal, RayGeneric, Sample, Sampler, VecGlobal,
  camera::{self, Camera, CameraRay},
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
pub struct Solver2 {
  pub(crate) sampler: Sampler,
  max_depth:          u32,
}

impl Solver2 {
  pub fn new(sampler: Sampler, max_depth: u32) -> Self { Self { sampler, max_depth } }

  pub fn render(&self, scene: &Scene, cam: &mut Camera) {
    let rays = cam.init_rays();
    const SAMPLES: i32 = 8;
    rays.into_par_iter().enumerate().for_each(|(_i, ray)| {
      for _ in 0..SAMPLES {
        ray.color += self.trace_ray(&RayGeneric::from_castable(ray), 0, scene);
        ray.reset_direction();
      }
      ray.color /= SAMPLES as f32;
    });
  }

  #[must_use = "Raytracing has no side-effects except mutating thread's RNG"]
  pub fn trace_ray(&self, ray: &RayGeneric, depth: u32, scene: &Scene) -> Spectrum {
    if self.should_terminate(depth) {
      return Spectrum::ZERO;
    }
    let ray = ray.clone().step();

    let Some(interaction) = Solver::closest_hit(scene, &ray) else {
      return Spectrum::ZERO;
    };

    // culling not required
    // TODO: emission?
    let mut cumulative_color = Spectrum::ZERO;
    self.sample_direct_lighting(scene, &interaction, &mut cumulative_color);
    self.sample_indirect_lighting(scene, &interaction, &mut cumulative_color);

    cumulative_color
  }

  #[inline(always)]
  fn sample_indirect_lighting(
    &self,
    scene: &Scene,
    interaction: &Interaction,
    cumulative_color: &mut Spectrum,
  ) {
    let intersected_point = interaction.hit.point_global();
    let hit_normal = interaction.hit.normal_global();
    let material = interaction.material();
    // here: trace refracted & reflected rays?
    // Or should BSDF sample a single direction
  }

  #[inline(always)]
  fn sample_direct_lighting(
    &self,
    scene: &Scene,
    interaction: &Interaction,
    cumulative_color: &mut Spectrum,
  ) {
    let hit_point = interaction.hit.point_global();
    for light in &scene.lights {
      let point = light.sample_point(self.sampler.sample());
      let light_point = light.transform().p2world(point.sample);
      let light_normal = light.transform().v2world(point.metadata.normal);

      if scene.is_visible(hit_point, light_point) {
        let shadow_direction: VecGlobal = (light_point - hit_point).normalize().into();
        let light_factor = -shadow_direction.dot(*light_normal);
        if light_factor <= 0.0 {
          continue;
        }
        let radiance = light.emitted_radiance(
          point.sample,
          light.transform().v2local(-shadow_direction),
          point.metadata.normal,
        );
        let cosine = interaction.hit.global_to_hit(shadow_direction).z.abs();
        let bsdf = interaction
          .bsdf()
          .bsdf2(
            interaction.hit.global_to_hit(shadow_direction),
            interaction.incoming(),
            BSDFSampleContext::Camera,
          )
          .map(Sample::value)
          .unwrap_or_default();
        *cumulative_color += bsdf * radiance * cosine / point.prob;
      }
    }
    // Who invented it?
    // *cumulative_color /= scene.lights.len().max(1) as f32;
  }

  #[inline(always)]
  fn should_terminate(&self, depth: u32) -> bool { depth == self.max_depth }

  pub fn max_depth(&self) -> u32 { self.max_depth }
  pub fn set_max_depth(&mut self, max_depth: u32) { self.max_depth = max_depth; }
}
