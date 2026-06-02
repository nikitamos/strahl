use glam::{Mat4, Quat, Vec3, Vec3Swizzles};

use crate::{
  Castable, Geometry, GeometrySampleMetadata, IntersectionContext, PointGlobal, PointLocal, Sample,
  SampleState, Sampler, Scene, Spectrum, SurfaceHit, SurfaceProperty, Transform, VecGlobal, VecHit,
  VecLocal,
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
  pub(crate) geometry:  Arc<dyn Geometry>,
  pub(crate) spectrum:  SurfaceProperty<Spectrum>,
  pub(crate) dir:       LightEmissionDirection,
  pub(crate) transform: Transform,
}

#[derive(Clone, Default, Debug)]
pub struct LightSampleMetadata {
  pub geometry:   GeometrySampleMetadata,
  pub point:      PointGlobal,
  pub direction:  VecGlobal,
  pub point_prob: f32,
  pub dir_prob:   f32,
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
      transform: Transform::from_w2l(Mat4::from_translation(-translation)),
      dir,
    }
  }
  pub fn try_intersect(&self, ray: &dyn Castable) -> Option<SurfaceHit> {
    self.geometry.try_intersect(
      IntersectionContext {
        transform: &self.transform,
      },
      ray,
    )
  }
  pub fn transform(&self) -> &Transform { &self.transform }
  pub fn sample_point(&self, state: SampleState) -> Sample<PointLocal, GeometrySampleMetadata> {
    self.geometry.sample_point(state)
  }
  pub fn sample(
    &self,
    state: SampleState,
    ctx: LightSampleContext,
  ) -> Option<Sample<Spectrum, LightSampleMetadata>> {
    match self.dir {
      LightEmissionDirection::Omni => {
        let point = self.geometry.sample_point(state);
        let point_prob = point.prob;
        point.and_then(|point, geometry| {
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
                  dir_prob: 1.0,
                  direction: Vec3::Z.into(),
                  point_prob,
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

  fn sample_direction_at(
    &self,
    _point: PointLocal,
    sample: SampleState,
  ) -> Sample<VecHit, Spectrum> {
    match self.spectrum {
      SurfaceProperty::Uniform(s) => sample.hemisphere_cosine().with_metadata(s),
      SurfaceProperty::Texture(_) => todo!(),
    }
  }
  /// Returns the radiance emitted in given direction from given
  /// point at the surface. This is equivalent to
  ///
  /// $$ \texttt{color}(\texttt{origin}) \cdot \langle\texttt{direction}, \texttt{normal}\rangle $$
  pub fn emitted_radiance(
    &self,
    _origin: PointLocal,
    direction: VecLocal,
    normal: VecLocal,
  ) -> Spectrum {
    let factor = direction.dot(*normal);
    if direction.dot(*normal) < 0.0 {
      return Spectrum::ZERO;
    }
    let surface = match self.spectrum {
      SurfaceProperty::Uniform(s) => match self.dir {
        LightEmissionDirection::Omni => s,
        LightEmissionDirection::Spot(_, _) => todo!(),
        LightEmissionDirection::Directed(_) => todo!(),
      },
      SurfaceProperty::Texture(_) => todo!(),
    };
    surface * factor
  }

  pub fn sample_point_and_direction(
    &self,
    sampler: &Sampler,
    _ctx: LightSampleContext,
  ) -> Sample<Spectrum, LightSampleMetadata> {
    let ps = self.geometry.sample_point(sampler.sample());
    let point_prob = ps.prob;
    ps.compose(|point, geometry| {
      let hit2local = Quat::from_rotation_arc(Vec3::Z, geometry.normal.into());
      let ds = self.sample_direction_at(point, sampler.sample());
      let dir_prob = ds.prob;
      ds.map_all(|hit, spec| {
        (spec, LightSampleMetadata {
          geometry,
          point: self.transform().p2world(point),
          direction: self.transform().v2world((hit2local * hit.xyz()).into()),
          point_prob,
          dir_prob,
        })
      })
    })
  }
}
