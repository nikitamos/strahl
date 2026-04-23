use std::f32::consts::FRAC_1_PI;

use crate::{
  Spectrum,
  material::bsdf::{BSDF, BsdfMetadata},
};

#[repr(transparent)]
#[derive(Debug)]
pub struct Lambertian {
  pub s: Spectrum,
}

impl BSDF for Lambertian {
  fn bsdf(&self, out: crate::VecHit, inc: crate::VecHit, tm: super::BSDFSampleContext) -> Spectrum {
    if out.z * inc.z < 0.0 {
      Spectrum::ZERO
    } else {
      self.s * FRAC_1_PI
    }
  }

  fn sample_bsdf(
    &self,
    out: crate::VecHit,
    u: crate::SampleState,
    _tm: super::BSDFSampleContext,
  ) -> Option<crate::Sample<Spectrum, super::BsdfMetadata>> {
    Some(u.hemisphere_cosine().map_all(|mut inc, _| {
      if out.z < 0.0 {
        inc.z *= -1.0; // Why?
      }
      (self.s * FRAC_1_PI, BsdfMetadata {
        inc,
        ..Default::default()
      })
    }))
  }

  fn bsdf2(
    &self,
    out: crate::VecHit,
    inc: crate::VecHit,
    ctx: super::BSDFSampleContext,
  ) -> Option<crate::Sample<Spectrum, BsdfMetadata>> {
    Some(crate::Sample {
      prob:     self.pdf(out, inc, ctx),
      sample:   self.s * FRAC_1_PI,
      metadata: BsdfMetadata {
        inc,
        ..Default::default()
      },
    })
  }

  fn pdf(&self, out: crate::VecHit, inc: crate::VecHit, tm: super::BSDFSampleContext) -> f32 {
    if out.z * inc.z < 0.0 {
      0.0
    } else {
      inc.z.abs() * FRAC_1_PI
    }
  }
}
