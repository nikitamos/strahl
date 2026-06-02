use crate::{
  Interaction, Sample, SampleState, Spectrum, VecGlobal, VecHit, material::medium::MediumInterface,
};

#[derive(Clone, Copy)]
pub enum BSDFSampleDirection {
  Camera,
  Light,
}

pub struct BSDFSampleContext<'a> {
  direction:   BSDFSampleDirection,
  interaction: &'a Interaction<'a>,
  interface:   MediumInterface<'a, 'a>,
}

// TODO: replace with proper delegate methods
impl<'a> std::ops::Deref for BSDFSampleContext<'a> {
  type Target = BSDFSampleDirection;

  fn deref(&self) -> &Self::Target { &self.direction }
}

impl<'a> BSDFSampleContext<'a> {
  pub fn camera(interaction: &'a Interaction<'a>, interface: MediumInterface<'a, 'a>) -> Self {
    Self {
      direction: BSDFSampleDirection::Camera,
      interaction,
      interface,
    }
  }
}

impl BSDFSampleDirection {
  pub fn light_direction(&self, intr: &Interaction, meta: &BsdfMetadata) -> VecGlobal {
    match self {
      BSDFSampleDirection::Camera => intr.hit.transform.v2world(intr.ray_dir),
      BSDFSampleDirection::Light => intr.hit.to_global(meta.inc),
    }
  }
  pub fn eye_direction(&self, intr: &Interaction, meta: &BsdfMetadata) -> VecGlobal {
    match self {
      BSDFSampleDirection::Camera => intr.hit.to_global(meta.inc),
      BSDFSampleDirection::Light => intr.hit.transform.v2world(intr.ray_dir),
    }
  }
}

#[derive(Default)]
pub struct BsdfMetadata {
  /// Sampled direction of incoming light.
  pub inc:         VecHit,
  pub eta:         f32,
  pub dirac:       bool,
  /// Whether the ray was refracted or reflected.
  pub transmitted: bool,
}

impl BsdfMetadata {
  pub fn jacobian_with(&self, out: VecHit) -> f32 {
    (out.z * self.inc.z).abs() / out.distance_squared(*self.inc)
  }
}

/// Bidirectional scattering distribution functions
pub trait BSDF: Sync + Send {
  /// Evaluates the BSDF given incident and exitant direction
  fn bsdf(&self, out: VecHit, inc: VecHit, ctx: &BSDFSampleContext) -> Spectrum {
    self
      .bsdf2(out, inc, ctx)
      .map_or(Spectrum::ZERO, Sample::value)
  }
  /// Samples the BSDF given the exitant direction. It uses pre-generated state from a sampler
  #[must_use]
  fn sample_bsdf(
    &self,
    out: VecHit,
    u: SampleState,
    ctx: &BSDFSampleContext,
  ) -> Option<Sample<Spectrum, BsdfMetadata>>;
  /// Evaluates the BSDF for the incident and exitant directions and
  /// returns a Sample containing probability of such scattering event
  fn bsdf2(
    &self,
    _out: VecHit,
    _inc: VecHit,
    _ctx: &BSDFSampleContext,
  ) -> Option<Sample<Spectrum, BsdfMetadata>>;
  /// Evaluates PDF for given directions.
  ///
  /// **This function may be deleted in future**
  fn pdf(&self, out: VecHit, inc: VecHit, ctx: &BSDFSampleContext) -> f32;
  /// Samples the BSDF multiple times, returning the average (what?)
  #[allow(unused)]
  fn rho(&self, out: VecHit, u: &[SampleState]) { todo!() }
}
pub struct CombinedBSDF {}

// TODO: implement BSDF for tuple of BSDFs?

pub mod dielectric;
pub mod lambertian;
pub mod specular;
