use super::{Interaction, IntersectionContext, Scene, Spectrum};
use crate::{
  Castable, GeometrySampleMetadata, PointGlobal, Sample, Sampler,
  camera::{self, CameraRay},
  light::LightSampleContext,
};
use glam::Vec3;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

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
    for _ in 0..SAMPLES {
      rays
        .into_par_iter()
        .for_each(|ray| self.trace_camera_ray(scene, ray));
    }
  }

  fn trace_camera_ray(&self, scene: &Scene, ray: &mut CameraRay) {
    let mut throughput: f32 = 1.0;
    if let Some(intr) = Self::closest_hit(scene, ray) {
      let local2hit = glam::Quat::from_rotation_arc(intr.hit.normal, Vec3::Z);
      let out_hit = (-local2hit.mul_vec3(intr.incoming.into())).into();
      let sample = intr.body.material.bsdf().sample_bsdf(
        out_hit,
        self.sampler.sample(),
        crate::material::bsdf::BSDFSampleContext::Camera,
      );
      if let Some(bsdf) = sample {
        let inc = intr.hit.transform.v2world(
          (-local2hit.inverse().mul_vec3(bsdf.metadata.inc.into()))
            .normalize()
            .into(),
        );
        ray.direction = inc.into();
        ray.origin = intr.hit.transform.p2world(intr.hit.point);
        if let Some(ls) = self.hit_light(scene, ray) {
          ray.color += bsdf.sample * ls; // * inc.dot(intr.hit.normal);
        }
      }
    }
  }
  fn sample_light(
    &self,
    scene: &Scene,
    dest: PointGlobal,
  ) -> Sample<Spectrum, GeometrySampleMetadata> {
    scene
      .sample_light_source(&self.sampler, dest)
      .compose(|src, ()| src.sample(self.sampler.sample(), LightSampleContext { dst: dest }))
  }
  pub(crate) fn hit_light(&self, scene: &Scene, ray: &dyn Castable) -> Option<Spectrum> {
    let light = &scene.lights[0];
    if let Some(hit) = light.try_intersect(ray) {
      Some(light.spectrum.get())
    } else {
      None
    }
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
          incoming: hit.transform.v2local(r.direction()),
          hit,
        })
      })
      .min_by(|a, b| a.hit.ray_distance.partial_cmp(&b.hit.ray_distance).unwrap())
  }
}
