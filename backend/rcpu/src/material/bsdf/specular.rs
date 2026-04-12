use glam::Vec3;

use crate::{Sample, Spectrum, VecHit, material::bsdf::BSDF};

pub struct Specular {
  pub r:   Spectrum,
  pub dir: [VecHit],
}

impl BSDF for Specular {
  fn bsdf(&self, out: VecHit, inc: VecHit, tm: super::BSDFSampleContext) -> Spectrum {
    if inc.reflect(Vec3::Z) == out.into() {
      self.r
    } else {
      Spectrum::ZERO
    }
  }

  fn sample_bsdf(
    &self,
    out: VecHit,
    _u: crate::SampleState,
    _tm: super::BSDFSampleContext,
  ) -> Option<crate::Sample<Spectrum, super::BsdfMetadata>> {
    Some(Sample {
      prob:     1.0,
      sample:   self.r,
      metadata: super::BsdfMetadata {
        inc:   (-out.reflect(Vec3::Z)).into(),
        eta:   1.0,
        dirac: true,
      },
    })
  }

  fn pdf(&self, out: VecHit, inc: VecHit, tm: super::BSDFSampleContext) -> f32 {
    if inc.reflect(Vec3::Z) == out.into() {
      1.0
    } else {
      0.0
    }
  }
}
