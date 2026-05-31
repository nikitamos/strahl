use std::ops::Deref;

use glam::Vec3;

use crate::{
  Castable, Interaction, IntersectionContext, PointGlobal, RayGeneric, SurfaceHit, Transform,
  VecLocal, are_codirectional,
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

#[derive(Default, Clone, Debug)]
pub struct GeometrySampleMetadata {
  pub normal: VecLocal,
}

pub trait Geometry: Sync + Send {
  fn sample_point(&self, state: SampleState) -> Sample<PointLocal, GeometrySampleMetadata>;
  fn try_intersect<'a>(&self, ctx: IntersectionContext, ray: &dyn Castable) -> Option<SurfaceHit>;
  fn uv(&self) -> Option<&UVMap> { None }
}

pub struct Sphere {
  pub radius: f32,
}
impl Geometry for Sphere {
  fn sample_point(&self, state: SampleState) -> Sample<PointLocal, GeometrySampleMetadata> {
    let mut sample = state.sphere_uniform().map_all(|x, _| {
      ((x.deref() * self.radius).into(), GeometrySampleMetadata {
        normal: (*x).into(),
      })
    });
    sample.prob /= self.radius.powi(2);
    sample
  }

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

    // let mut fact = intersection.dot(direction).signum();
    // if fact == 0.0 {
    //   fact = 1.0;
    // }
    let fact = 1.0;

    Some(SurfaceHit::new(
      intersection.into(),
      (intersection).normalize() * fact,
      t,
      ctx.transform,
    ))
  }
}

pub struct Plane {}

pub struct Point {}

impl Geometry for Point {
  fn sample_point(&self, _state: SampleState) -> Sample<PointLocal, GeometrySampleMetadata> {
    Sample {
      prob:     1.0,
      sample:   Vec3::ZERO.into(),
      metadata: GeometrySampleMetadata {
        normal: Vec3::ZERO.into(),
      },
    }
  }

  fn try_intersect<'a>(&self, ctx: IntersectionContext, ray: &dyn Castable) -> Option<SurfaceHit> {
    let pos = ctx.transform.p2local(ray.pos());
    let dir = ctx.transform.v2local(ray.direction());
    if are_codirectional(pos.into(), -*dir) {
      Some(SurfaceHit::new(
        Vec3::ZERO.into(),
        Vec3::ZERO,
        pos.length(),
        ctx.transform,
      ))
    } else {
      None
    }
  }
}

pub trait HasGeometry {
  fn geometry(&self) -> &dyn Geometry;
}

pub trait HasTransform {
  fn transform(&self) -> Transform;
}

fn closest_hit<'a, T: HasGeometry + HasTransform>(
  geometries: impl Iterator<Item = &'a T>,
  ray: RayGeneric,
) -> Option<Interaction<'a, T>> {
  geometries
    .filter_map(|b| {
      let ctx = IntersectionContext {
        transform: b.transform(),
      };
      b.geometry()
        .try_intersect(ctx, &ray)
        .map(|hit| Interaction {
          body: b,
          ray_dir: hit.transform.v2local(ray.direction()),
          hit,
        })
    })
    .min_by(|a, b| a.hit.ray_distance.partial_cmp(&b.hit.ray_distance).unwrap())
}
