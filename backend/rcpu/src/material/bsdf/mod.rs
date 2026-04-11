use crate::{Sample, SampleState, Spectrum, VecHit, VecLocal};

pub enum RayDirection {
  Camera,
  Light,
}

pub enum ScatteringEvent {
  Reflection,
  Transmission,
}

pub struct BsdfMetadata {
  pub inc:   VecHit,
  pub eta:   f32,
  pub dirac: bool,
}

pub struct BSDFSampleContext {
  // pub out
}

/// Bidirectional scattering distribution functions
pub trait BSDF {
  fn bsdf(&self, out: VecHit, inc: VecHit, tm: RayDirection) -> Spectrum;
  fn sample_bsdf(
    &self,
    out: VecHit,
    u: SampleState,
    tm: RayDirection,
  ) -> Option<Sample<Spectrum, BsdfMetadata>>;
  fn pdf(&self, out: VecHit, inc: VecHit, tm: RayDirection) -> f32;
  /// TODO: what does it do?
  #[allow(unused)]
  fn rho(&self, out: VecHit, u: &[SampleState]) { todo!() }
}
pub struct CombinedBSDF {}

// TODO: implement BSDF for slice of BSDFs?

pub mod lambertian;
