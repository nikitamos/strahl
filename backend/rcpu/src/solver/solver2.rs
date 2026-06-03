use super::{
  super::{Interaction, Scene},
  closest_hit,
};
use crate::{
  RayGeneric, Sample, Sampler, Spectrum, VecGlobal,
  camera::Camera,
  material::{
    bsdf::BSDFSampleContext,
    medium::{Medium, MediumInterface},
  },
};
use rayon::prelude::*;

pub struct ForwardPathTracer {
  pub(crate) sampler:   Sampler,
  pub(crate) max_depth: u32,
  pub(crate) samples:   u32,
}

struct InteractionMedium<'a> {
  current: &'a Medium,
  parent:  &'a Medium,
}

impl<'a> InteractionMedium<'a> {
  fn interact(&self, body: &'a Medium) -> InteractionMedium<'a> {
    let last_ptr: *const Medium = self.current;
    let next_ptr: *const Medium = body;
    if std::ptr::addr_eq(last_ptr.cast::<()>(), next_ptr.cast::<()>()) {
      // Ray hits internal surface. Interacting medium is parent
      InteractionMedium {
        current: self.current,
        parent:  self.parent,
      }
    } else {
      // Ray hits external surface. Interacting medium is body
      InteractionMedium {
        current: body,
        parent:  self.current,
      }
    }
  }
  fn interface(&self) -> MediumInterface<'a, 'a> { MediumInterface::new(self.parent, self.current) }
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
        ray.color += self.trace_ray(&ray.as_generic(), 0, scene);
        ray.reset_direction();
      }
      ray.color /= self.samples as f32;
    });
  }

  #[must_use = "Raytracing has no side-effects except mutating thread's RNG"]
  #[inline(always)]
  pub fn trace_ray(&self, ray: &RayGeneric, depth: u32, scene: &Scene) -> Spectrum {
    let starting_medium = Medium { ior: 1.0 };
    let interaction_medium = InteractionMedium {
      current: &starting_medium,
      parent:  &starting_medium,
    };
    self.trace_ray_impl(ray, depth, scene, interaction_medium)
  }

  fn trace_ray_impl<'a>(
    &self,
    ray: &RayGeneric,
    depth: u32,
    scene: &'a Scene,
    medium: InteractionMedium<'_>,
  ) -> glam::Vec3 {
    // We assume that the last `medium` is the medium containing the ray origin
    if self.should_terminate(depth) {
      return Spectrum::ZERO;
    }
    let ray = ray.clone().step();

    let Some(interaction) = closest_hit(scene, &ray) else {
      return Spectrum::ZERO;
    };

    // culling not required
    let mut cumulative_color = Spectrum::ZERO;
    self.sample_direct_lighting(scene, &interaction, &mut cumulative_color, &medium);
    self.sample_indirect_lighting(scene, &interaction, &mut cumulative_color, depth, medium);

    cumulative_color
  }

  #[inline(always)]
  fn sample_indirect_lighting<'a>(
    &self,
    scene: &'a Scene,
    interaction: &'a Interaction<'a>,
    cumulative_color: &mut Spectrum,
    depth: u32,
    mediums: InteractionMedium<'_>,
  ) {
    let intersected_point = interaction.hit.point_global();
    let material = interaction.material();
    let mut intr_medium = mediums.interact(material.medium());
    let Some(bsdf) = material.bsdf().sample_bsdf(
      interaction.caused_by(),
      self.sampler.sample(),
      &BSDFSampleContext::camera(interaction, intr_medium.interface()),
    ) else {
      return;
    };
    if !bsdf.metadata.transmitted {
      intr_medium = mediums;
    }
    let direction = interaction.hit.to_global(bsdf.metadata.inc);
    let ray = RayGeneric::new_stepped(intersected_point, direction);
    let cosine = bsdf.metadata.inc.z.abs();
    let rt = self.trace_ray_impl(&ray, depth + 1, scene, intr_medium);
    *cumulative_color += rt * bsdf.sample * cosine / bsdf.prob; // TODO: should we divide, should we multiply by cosine?
  }

  #[inline(always)]
  fn sample_direct_lighting(
    &self,
    scene: &Scene,
    interaction: &Interaction,
    cumulative_color: &mut Spectrum,
    medium: &InteractionMedium,
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
            &BSDFSampleContext::camera(
              interaction,
              medium.interact(interaction.body_medium()).interface(),
            ), // that's completely wrong!
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
