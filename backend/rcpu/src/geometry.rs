use std::ops::Deref;

use glam::Vec3;

use crate::{
  Castable, Interaction, IntersectionContext, RayGeneric, SurfaceHit, Transform, VecGlobal,
  VecLocal,
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
  fn try_intersect<'a>(
    &self,
    ctx: IntersectionContext<'a>,
    ray: &dyn Castable,
  ) -> Option<SurfaceHit<'a>>;
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

  fn try_intersect<'a>(
    &self,
    ctx: IntersectionContext<'a>,
    ray: &dyn Castable,
  ) -> Option<SurfaceHit<'a>> {
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

pub struct Quad {
  pub origin: VecLocal,
  u:          VecLocal,
  v:          VecLocal,
  normal:     VecLocal,
  area:       f32,
}

impl Quad {
  /// Creates an arbitrarily oriented parallelogram quad.
  #[must_use]
  pub fn new(origin: VecLocal, u: VecLocal, v: VecLocal) -> Self {
    let cross = u.cross(*v);
    let area = cross.length();
    assert!(area > 1e-5, "Quad must have non-zero area");

    Self {
      origin,
      u,
      v,
      normal: (cross / area).into(),
      area,
    }
  }

  pub fn invert_normal(&mut self) { self.normal = -self.normal; }

  /// Creates an axis-aligned square centered at `center`, with side length `side`,
  /// lying in the plane perpendicular to `axis`.
  #[must_use]
  pub fn axis_aligned_square(center: VecGlobal, axis: VecGlobal, side: f32) -> Self {
    let normal = axis.normalize();

    let up = if normal.abs().z < 0.9 {
      Vec3::Z
    } else {
      Vec3::X
    };
    let u_dir = up.cross(normal).normalize();
    let v_dir = normal.cross(u_dir).normalize();

    let half_side = side * 0.5;
    let origin = *center - u_dir * half_side - v_dir * half_side;

    Self::new(origin.into(), (u_dir * side).into(), (v_dir * side).into())
  }

  /// Square in the XY plane (normal points along +Z)
  #[must_use]
  pub fn xy_square(center: VecGlobal, side: f32) -> Self {
    Self::axis_aligned_square(center, Vec3::Z.into(), side)
  }

  /// Square in the YZ plane (normal points along +X)
  #[must_use]
  pub fn yz_square(center: VecGlobal, side: f32) -> Self {
    Self::axis_aligned_square(center, Vec3::X.into(), side)
  }

  /// Square in the ZX plane (normal points along +Y)
  #[must_use]
  pub fn zx_square(center: VecGlobal, side: f32) -> Self {
    Self::axis_aligned_square(center, Vec3::Y.into(), side)
  }
}

impl Geometry for Quad {
  fn sample_point(&self, state: SampleState) -> Sample<PointLocal, GeometrySampleMetadata> {
    let [u_rand, v_rand] = state.uniform_2d.into();

    let point = *self.origin + u_rand * *self.u + v_rand * *self.v;
    let pdf = 1.0 / self.area;

    let metadata = GeometrySampleMetadata {
      normal: self.normal,
    };

    Sample {
      sample: point.into(),
      prob: pdf,
      metadata,
    }
  }

  fn try_intersect<'a>(
    &self,
    ctx: IntersectionContext<'a>,
    ray: &dyn Castable,
  ) -> Option<SurfaceHit<'a>> {
    let dir = *ctx.transform.v2local(ray.direction());
    let ray_origin = *ctx.transform.p2local(ray.pos());
    let normal = *self.normal;
    let denom = dir.dot(normal);

    // Ray is parallel to the quad plane
    if denom.abs() < 1e-5 {
      return None;
    }

    let t = (*self.origin - ray_origin).dot(normal) / denom;

    let p = ray_origin + dir * t;
    let w = p - *self.origin;

    // (w \cross v) \cdot n_unit = u_coord \cdot area
    let u_val = w.cross(*self.v).dot(normal) / self.area;
    let v_val = self.u.cross(w).dot(normal) / self.area;

    // Check if the hit lies within the quad boundaries
    if t < 0.0 || !(0.0..=1.0).contains(&u_val) || !(0.0..=1.0).contains(&v_val) {
      return None;
    }

    Some(SurfaceHit::new(p.into(), normal, t, ctx.transform))
  }

  // `uv` is omitted as requested; uses the trait's default implementation.
}

mod mesh;
pub use mesh::TriangleMesh;

pub trait HasGeometry {
  fn geometry(&self) -> &dyn Geometry;
}

// pub trait HasTransform {
//   fn transform(&self) -> Transform;
// }

// fn closest_hit<'a, T: HasGeometry + HasTransform>(
//   geometries: impl Iterator<Item = &'a T>,
//   ray: RayGeneric,
// ) -> Option<Interaction<'a, T>> {
//   geometries
//     .filter_map(|b| {
//       let ctx = IntersectionContext {
//         transform: &b.transform(),
//       };
//       b.geometry()
//         .try_intersect(ctx, &ray)
//         .map(|hit| Interaction {
//           body: b,
//           ray_dir: hit.transform.v2local(ray.direction()),
//           hit,
//         })
//     })
//     .min_by(|a, b| a.hit.ray_distance.partial_cmp(&b.hit.ray_distance).unwrap())
// }
