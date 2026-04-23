use glam::{Mat4, Vec3};

use crate::{
  Castable, Geometry, GeometrySampleMetadata, IntersectionContext, PointGlobal, PointLocal, Sample,
  SampleState, Scene, Spectrum, SurfaceHit, SurfaceProperty, Transform, VecGlobal,
};
use std::{ops::Deref, sync::Arc};

#[derive(Clone, Copy, Debug)]
pub enum LightEmissionDirection {
  Omni,
  Spot(VecGlobal, f32),
  Directed(VecGlobal),
}

/// The context passed to the light sampler
pub struct LightSampleContext<'s> {
  /// The point from which the light is sampled. It means
  /// that the sampled ray **must** hit this point
  pub dst:   PointGlobal,
  pub scene: &'s Scene,
}

/// For now each point of the source emits light in direction normal
/// to the surface
pub struct LightSource {
  pub(crate) geometry:    Arc<dyn Geometry>,
  pub(crate) spectrum:    SurfaceProperty<Spectrum>,
  pub(crate) dir:         LightEmissionDirection,
  pub(crate) translation: glam::Vec3,
}

#[derive(Clone, Default, Debug)]
pub struct LightSampleMetadata {
  pub geometry: GeometrySampleMetadata,
  pub point:    PointGlobal,
}

impl Deref for LightSampleMetadata {
  type Target = GeometrySampleMetadata;
  fn deref(&self) -> &Self::Target { &self.geometry }
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
  pub fn transform(&self) -> Transform {
    Transform::from_w2l(Mat4::from_translation(-self.translation))
  }
  pub fn sample(
    &self,
    state: SampleState,
    ctx: LightSampleContext,
  ) -> Option<Sample<Spectrum, LightSampleMetadata>> {
    match self.dir {
      LightEmissionDirection::Omni => {
        let point = self.geometry.sample_point(state);
        point.compose(|point, geometry| {
          if ctx
            .scene
            .is_visible(self.transform().p2world(point), ctx.dst)
          {
            match self.spectrum {
              SurfaceProperty::Uniform(s) => Some(Sample {
                prob:     1.0, // TODO: fix probability?
                sample:   s,
                metadata: LightSampleMetadata {
                  geometry,
                  point: self.transform().p2world(point),
                },
              }),
              SurfaceProperty::Texture(_) => unimplemented!(),
            }
          } else {
            // println!("{:?} (dest) is occluded from {:?}(light)", ctx.dst, self.transform().p2world(point));
            None
          }
        })
      }
      _ => unimplemented!(),
    }
  }
}
