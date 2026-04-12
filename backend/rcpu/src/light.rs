use glam::Mat4;

use crate::{Castable, Geometry, IntersectionContext, Sample, SampleState, Spectrum, SurfaceHit, SurfaceProperty, Transform, VecGlobal, geometry};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub enum LightEmissionDirection {
  Omni,
  Spot(VecGlobal, f32),
  Directed(VecGlobal),
}

pub struct LightSampleContext {

}

/// For now each point of the source emits light in direction normal
/// to the surface
pub struct LightSource {
  pub(crate) geometry: Arc<dyn Geometry>,
  pub(crate) spectrum: SurfaceProperty<Spectrum>,
  pub(crate) dir:      LightEmissionDirection,
  pub(crate) translation: glam::Vec3
}

impl LightSource {
  pub(crate) fn new(
    geometry: Arc<dyn Geometry>,
    spectrum: SurfaceProperty<Spectrum>,
    translation: glam::Vec3,
    dir: LightEmissionDirection,
  ) -> Self{
    Self {
        geometry: geometry,
        spectrum: spectrum,
        translation,
        dir,
    }
  }
  pub fn try_intersect(&self, ray: &dyn Castable) -> Option<SurfaceHit> {
    self.geometry.try_intersect(IntersectionContext {
        transform: Transform::from_w2l(Mat4::from_translation(-self.translation)),
    }, ray)
  }
  pub fn sample(&self, sample: SampleState, ctx: LightSampleContext) -> Sample<()> {
    todo!()
  }
}
