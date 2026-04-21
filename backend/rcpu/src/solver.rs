use super::{Interaction, IntersectionContext, Scene, Spectrum};
use crate::{
  Castable, GeometrySampleMetadata, PointGlobal, Sample, Sampler,
  camera::{self, CameraRay},
  light::LightSampleContext,
};
use glam::{FloatExt, Vec3, Vec3Swizzles, Vec4Swizzles};
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
    const SAMPLES: i32 = 1;
    for _ in 0..SAMPLES {
      rays.into_par_iter().enumerate().for_each(|(i, ray)| {
        self.trace_camera_ray(scene, ray);
      });
    }
  }

  fn trace_camera_ray(&self, scene: &Scene, ray: &mut CameraRay) {
    let mut local_ray = ray.clone();
    let mut throughput: f32 = 1.0;
    if let Some(intr) = Self::closest_hit(scene, ray) {
      let local2hit = glam::Quat::from_rotation_arc(intr.hit.normal, Vec3::Z);
      let out_hit = (local2hit.mul_vec3(-*intr.incoming)).into();
      // println!("out+hit -> {:?}", out_hit);
      let sample = intr.body.material.bsdf().sample_bsdf(
        out_hit,
        self.sampler.sample(),
        crate::material::bsdf::BSDFSampleContext::Camera,
      );
      // Sample the BSDF
      if let Some(bsdf) = sample {
        let light_incoming = intr.hit.transform.v2world(
          (-local2hit.inverse().mul_vec3(bsdf.metadata.inc.into()))
            .normalize()
            .into(),
        );
        // Ray direction in world coordinates
        local_ray.direction = light_incoming.into();
        local_ray.origin = intr.hit.transform.p2world(intr.hit.point);
        // ray.color = Vec3::ONE;
        if let Some(ls) = self.sample_light(scene, intr.hit.point_global()) {
          // ray.color = 0.5 * out_hit.xyz() + 1.0;
          local_ray.color +=
            bsdf.sample * ls.sample * ((-*intr.incoming).dot(intr.hit.normal)).abs();
        }
      }
    }
    ray.color = local_ray.color;
  }
  fn sample_light(
    &self,
    scene: &Scene,
    dest: PointGlobal,
  ) -> Option<Sample<Spectrum, GeometrySampleMetadata>> {
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
          incoming: hit.transform.v2local(r.direction()),
          hit,
        })
      })
      .min_by(|a, b| a.hit.ray_distance.partial_cmp(&b.hit.ray_distance).unwrap())
  }
}
