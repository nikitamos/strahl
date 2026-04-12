use glam::Vec3;

use crate::{
  Castable, IntersectionContext, SurfaceHit,
  points::PointLocal,
  sampling::{Sample, SampleState},
};

pub struct UVMap {
  uv: Vec<glam::Vec2>,
}

impl From<Vec<glam::Vec2>> for UVMap {
  fn from(uv: Vec<glam::Vec2>) -> Self { Self { uv } }
}

impl UVMap {
  pub fn new(uv: &[glam::Vec2]) -> Self { Self { uv: uv.into() } }
}

pub trait Geometry: Sync + Send {
  fn sample_point(&self, state: SampleState) -> Sample<PointLocal>;
  fn try_intersect<'a>(&self, ctx: IntersectionContext, ray: &dyn Castable) -> Option<SurfaceHit>;
  fn uv(&self) -> Option<&UVMap> { None }
}

pub struct Sphere {
  pub radius: f32,
}
impl Geometry for Sphere {
  fn sample_point(&self, state: SampleState) -> Sample<PointLocal> { todo!() }

  fn try_intersect(&self, ctx: IntersectionContext, ray: &dyn Castable) -> Option<SurfaceHit> {
    let oc: Vec3 = ctx.transform.p2local(ray.pos()).into();
    let direction: Vec3 = ctx.transform.v2local(ray.direction()).into();

    let a = direction.length_squared();
    let b = 2.0 * oc.dot(direction);
    let c = oc.length_squared() - self.radius * self.radius;

    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
      return None;
    }

    let sqrt_disc = discriminant.sqrt();
    let t1 = (-b - sqrt_disc) / (2.0 * a);
    let t2 = (-b + sqrt_disc) / (2.0 * a);

    let t = if t1 > 0.0 && t2 > 0.0 {
      t1.min(t2)
    } else if t1 > 0.0 {
      t1
    } else if t2 > 0.0 {
      t2
    } else {
      return None;
    };
    let intersection = oc + t * direction;

    Some(SurfaceHit {
      point:        intersection.into(),
      normal:       (intersection).normalize(),
      ray_distance: t,
      transform:    ctx.transform,
    })
  }
}

pub struct Plane {}

pub struct Point {}
