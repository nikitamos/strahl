use glam::Mat4;

use crate::{
  Castable, Geometry, GeometrySampleMetadata, IntersectionContext, PointGlobal, Sample,
  SampleState, Spectrum, SurfaceHit, SurfaceProperty, Transform, VecGlobal,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub enum LightEmissionDirection {
  Omni,
  Spot(VecGlobal, f32),
  Directed(VecGlobal),
}

/// The context passed to the light sampler
pub struct LightSampleContext {
  /// The point from which the light is sampled. It means
  /// that the sampled ray **must** hit this point
  pub dst: PointGlobal,
}

/// For now each point of the source emits light in direction normal
/// to the surface
pub struct LightSource {
  pub(crate) geometry:    Arc<dyn Geometry>,
  pub(crate) spectrum:    SurfaceProperty<Spectrum>,
  pub(crate) dir:         LightEmissionDirection,
  pub(crate) translation: glam::Vec3,
}

impl LightSource {
  pub(crate) fn new(
    geometry: Arc<dyn Geometry>,
    spectrum: SurfaceProperty<Spectrum>,
    translation: glam::Vec3,
    dir: LightEmissionDirection,
  ) -> Self {
    Self {
      geometry,
      spectrum,
      translation,
      dir,
    }
  }
  pub fn try_intersect(&self, ray: &dyn Castable) -> Option<SurfaceHit> {
    self.geometry.try_intersect(
      IntersectionContext {
        transform: Transform::from_w2l(Mat4::from_translation(-self.translation)),
      },
      ray,
    )
  }
  pub fn sample(
    &self,
    state: SampleState,
    ctx: LightSampleContext,
  ) -> Option<Sample<Spectrum, GeometrySampleMetadata>> {
    match self.dir {
      LightEmissionDirection::Omni => {
        Some(
          // TODO: occlusion test!
          self
            .geometry
            .sample_point(state)
            .map(|point| match self.spectrum {
              SurfaceProperty::Uniform(s) => s,
              SurfaceProperty::Texture(_) => unimplemented!(),
            }),
        )
      }
      _ => unimplemented!(),
    }
  }
}
