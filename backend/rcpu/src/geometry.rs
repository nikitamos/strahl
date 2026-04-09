use crate::{
  Castable, IntersectionContext, sampling::Sample, sampling::SampleState, SurfaceIntersection, points::PointLocal,
};

pub trait Geometry : Sync + Send {
  fn sample_point(&self, state: SampleState) -> Sample<PointLocal>;
  fn try_intersect(
    &self,
    ctx: IntersectionContext,
    ray: &dyn Castable,
  ) -> Option<SurfaceIntersection>;
}

pub struct Sphere {}

pub struct Plane {}

pub struct Point {}
