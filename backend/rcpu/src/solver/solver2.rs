use super::{
  super::{Interaction, Scene},
  closest_hit,
};
use crate::{
  RayGeneric, Sample, Sampler, Spectrum, VecGlobal, camera::Camera,
  material::bsdf::BSDFSampleContext,
};
use rayon::prelude::*;

pub struct ForwardPathTracer {
  pub(crate) sampler:   Sampler,
  pub(crate) max_depth: u32,
  pub(crate) samples:   u32,
}

impl ForwardPathTracer {
  pub fn new(sampler: Sampler, max_depth: u32, samples: u32) -> Self {
    Self {
      sampler,
      max_depth,
      samples,
    }
  }

  pub fn render(&self, scene: &Scene, cam: &mut Camera) {
    let rays = cam.init_rays();
    rays.into_par_iter().enumerate().for_each(|(_i, ray)| {
      for _ in 0..self.samples {
        ray.color += self.trace_ray(&RayGeneric::from_castable(ray), 0, scene);
        ray.reset_direction();
      }
      ray.color /= self.samples as f32;
    });
  }

  #[must_use = "Raytracing has no side-effects except mutating thread's RNG"]
  pub fn trace_ray(&self, ray: &RayGeneric, depth: u32, scene: &Scene) -> Spectrum {
    if self.should_terminate(depth) {
      return Spectrum::ZERO;
    }
    let ray = ray.clone().step();

    let Some(interaction) = closest_hit(scene, &ray) else {
      return Spectrum::ZERO;
    };

    // culling not required
    // TODO: emission?
    let mut cumulative_color = Spectrum::ZERO;
    self.sample_direct_lighting(scene, &interaction, &mut cumulative_color);
    self.sample_indirect_lighting(scene, &interaction, &mut cumulative_color, depth);

    cumulative_color
  }

  #[inline(always)]
  pub(crate) fn sample_indirect_lighting(
    &self,
    scene: &Scene,
    interaction: &Interaction,
    cumulative_color: &mut Spectrum,
    depth: u32,
  ) {
    let intersected_point = interaction.hit.point_global();
    let material = interaction.material();
    let Some(bsdf) = material.bsdf().sample_bsdf(
      interaction.caused_by(),
      self.sampler.sample(),
      BSDFSampleContext::Camera,
    ) else {
      return;
    };
    let direction = interaction.hit.to_global(bsdf.metadata.inc);
    let ray = RayGeneric::new_stepped(intersected_point, direction);
    let cosine = bsdf.metadata.inc.z.abs();
    let rt = self.trace_ray(&ray, depth + 1, scene);
    *cumulative_color += rt * bsdf.sample * cosine / bsdf.prob; // TODO: should we divide, should we multiply by cosine?
  }

  #[inline(always)]
  pub(crate) fn sample_direct_lighting(
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
            interaction.caused_by(),
            BSDFSampleContext::Camera,
          )
          .map(Sample::value)
          .unwrap_or_default();
        *cumulative_color += bsdf * radiance * cosine / point.prob;
      }
    }
  }

  #[inline(always)]
  pub(crate) fn should_terminate(&self, depth: u32) -> bool { depth == self.max_depth }

  pub fn max_depth(&self) -> u32 { self.max_depth }
  pub fn set_max_depth(&mut self, max_depth: u32) { self.max_depth = max_depth; }
}
