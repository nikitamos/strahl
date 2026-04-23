use crate::{Sample, SampleState, Spectrum, VecHit, material::medium::MediumInterface};

pub enum BSDFSampleContext<'a> {
  Camera,
  Light,
  Context(MediumInterface<'a, 'a>),
}

pub enum ScatteringEvent {
  Reflection,
  Transmission,
}

#[derive(Default)]
pub struct BsdfMetadata {
  /// Sampled direction of incoming light.
  pub inc:   VecHit,
  pub eta:   f32,
  pub dirac: bool,
}

/// Bidirectional scattering distribution functions
pub trait BSDF {
  /// Evaluates the BSDF given incident and exitant direction
  fn bsdf(&self, out: VecHit, inc: VecHit, ctx: BSDFSampleContext) -> Spectrum;
  /// Samples the BSDF given the exitant direction. It uses pre-generated state from a sampler
  fn sample_bsdf(
    &self,
    out: VecHit,
    u: SampleState,
    ctx: BSDFSampleContext,
  ) -> Option<Sample<Spectrum, BsdfMetadata>>;
  /// Evaluates the BSDF for the incident and exitant directions and
  /// returns a Sample containing probability of such scattering event
  fn bsdf2(
    &self,
    out: VecHit,
    inc: VecHit,
    ctx: BSDFSampleContext,
  ) -> Option<Sample<Spectrum, BsdfMetadata>> {
    todo!()
  }
  /// Evaluates PDF for given directions.
  ///
  /// **This function may be deleted in future**
  fn pdf(&self, out: VecHit, inc: VecHit, ctx: BSDFSampleContext) -> f32;
  /// Samples the BSDF multiple times, returning the average (what?)
  #[allow(unused)]
  fn rho(&self, out: VecHit, u: &[SampleState]) { todo!() }
}
pub struct CombinedBSDF {}

// TODO: implement BSDF for tuple of BSDFs?

pub mod lambertian;
pub mod specular;
