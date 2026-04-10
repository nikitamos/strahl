use glam::Vec3;

use crate::{
  Castable, IntersectionContext, SurfaceHit,
  points::PointLocal,
  sampling::{Sample, SampleState},
};

pub trait Geometry: Sync + Send {
  fn sample_point(&self, state: SampleState) -> Sample<PointLocal>;
  fn try_intersect(&self, ctx: IntersectionContext, ray: &dyn Castable) -> Option<SurfaceHit>;
}

pub struct Sphere {
  pub radius: f32,
}
impl Geometry for Sphere {
  fn sample_point(&self, state: SampleState) -> Sample<PointLocal> { todo!() }

  fn try_intersect(&self, ctx: IntersectionContext, ray: &dyn Castable) -> Option<SurfaceHit> {
    println!("try-isect");
    let oc: Vec3 = ray.pos().into_local(ctx.g2l).into(); // Since sphere is at origin
    let direction = ctx.g2l.transform_vector3(ray.direction());

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
    })
  }
}

pub struct Plane {}

pub struct Point {}
