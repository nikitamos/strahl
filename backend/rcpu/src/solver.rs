use super::{Interaction, IntersectionContext, Scene, Spectrum};
use crate::{
  Castable, Sampler,
  camera::{self, CameraRay},
};
use glam::Vec3;
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
    pub(crate) const SAMPLES: i32 = 16;
    for i in 0..SAMPLES {
      let x = rays.into_par_iter().enumerate().for_each(|(i, ray)| {
        if let Some(intr) = Self::closest_hit(scene, ray) {
          let local2hit = glam::Quat::from_rotation_arc(intr.hit.normal, Vec3::Z);
          let out_hit = (-local2hit.mul_vec3(intr.incoming.into())).into();
          // println!("out+hit -> {:?}", out_hit);
          let sample = intr.body.material.bsdf().sample_bsdf(
            out_hit,
            self.sampler.sample(),
            crate::material::bsdf::BSDFSampleContext::Camera,
          );
          // Sample the BSDF
          if let Some(bsdf) = sample {
            let inc = intr.hit.transform.v2world(
              (-local2hit.inverse().mul_vec3(bsdf.metadata.inc.into()))
                .normalize()
                .into(),
            );
            ray.direction = inc.into();
            ray.origin = intr.hit.transform.p2world(intr.hit.point);
            if let Some(ls) = self.sample_light(scene, ray) {
              ray.color += bsdf.sample * ls; // * inc.dot(intr.hit.normal);
            }
          }
          // Sample the light
          // println!("{}", ray.color);
          // cast a new ray, sample light
        }
      });
    }
  }
  pub(crate) fn sample_light(&self, scene: &Scene, ray: &mut CameraRay) -> Option<Spectrum> {
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
