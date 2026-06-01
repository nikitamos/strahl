use glam::Vec3;

use crate::{Sample, Spectrum, VecHit, material::bsdf::BSDF};

/// The specular reflection BSDF.
///
/// Specular BSDF reflects light in a deterministic fashion, so that
/// * incidence angle is equal to the reflection angle;
/// * surface normal, incident and reflected rays belong to the same plane.
pub struct Specular {
  /// Reflected spectrum
  pub r: Spectrum,
}

impl BSDF for Specular {
  fn bsdf(&self, out: VecHit, inc: VecHit, _tm: &super::BSDFSampleContext) -> Spectrum {
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
    _tm: &super::BSDFSampleContext,
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

  fn bsdf2(
    &self,
    out: VecHit,
    inc: VecHit,
    _ctx: &super::BSDFSampleContext,
  ) -> Option<Sample<Spectrum, super::BsdfMetadata>> {
    if inc.reflect(Vec3::Z) == out.into() {
      Some(Sample {
        prob:     1.0,
        sample:   self.r,
        metadata: super::BsdfMetadata {
          inc,
          eta: 1.0,
          dirac: true,
        },
      })
    } else {
      None
    }
  }

  fn pdf(&self, out: VecHit, inc: VecHit, _tm: &super::BSDFSampleContext) -> f32 {
    if inc.reflect(Vec3::Z) == out.into() {
      1.0
    } else {
      0.0
    }
  }
}
