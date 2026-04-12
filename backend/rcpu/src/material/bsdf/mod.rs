use crate::{Sample, SampleState, Spectrum, VecHit, VecLocal};

pub enum BSDFSampleContext {
  Camera,
  Light,
}

pub enum ScatteringEvent {
  Reflection,
  Transmission,
}

#[derive(Default)]
pub struct BsdfMetadata {
  pub inc:   VecHit,
  pub eta:   f32,
  pub dirac: bool,
}

/// Bidirectional scattering distribution functions
pub trait BSDF {
  fn bsdf(&self, out: VecHit, inc: VecHit, ctx: BSDFSampleContext) -> Spectrum;
  fn sample_bsdf(
    &self,
    out: VecHit,
    u: SampleState,
    ctx: BSDFSampleContext,
  ) -> Option<Sample<Spectrum, BsdfMetadata>>;
  fn pdf(&self, out: VecHit, inc: VecHit, ctx: BSDFSampleContext) -> f32;
  /// TODO: what does it do?
  #[allow(unused)]
  fn rho(&self, out: VecHit, u: &[SampleState]) { todo!() }
}
pub struct CombinedBSDF {}

// TODO: implement BSDF for tuple of BSDFs?

pub mod lambertian;
pub mod specular;
