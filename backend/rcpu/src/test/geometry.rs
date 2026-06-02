use crate::RayGeneric;

pub type TestRay = RayGeneric;

mod sphere {
  use std::assert_matches;

  use glam::{Mat4, Vec3};

  use crate::{Geometry, PointLocal, Sphere, SurfaceHit, Transform, test::geometry::TestRay};

  #[test]
  fn direct_origin_collision() {
    let s = Sphere { radius: 1.0 };
    let ray = TestRay {
      position:  (2.0 * Vec3::Y).into(),
      direction: Vec3::NEG_Y.into(),
    };
    let binding = Transform::from_w2l(Mat4::IDENTITY);
    let isect = s.try_intersect(
      crate::IntersectionContext {
        transform: &binding,
      },
      &ray,
    );
    const POINT: PointLocal = PointLocal::new(Vec3::new(0.0, 1.0, 0.0));
    assert_matches!(
      isect,
      Some(SurfaceHit {
        normal: Vec3::Y,
        ray_distance: 1.0,
        ..
      })
    );
    assert_eq!(isect.unwrap().point, POINT, "wrong collision point");
  }
  #[test]
  fn miss() {
    let s = Sphere { radius: 1.0 };
    let ray = TestRay {
      position:  (2.0 * Vec3::Y).into(),
      direction: Vec3::Y.into(),
    };
    let binding = Transform::from_w2l(Mat4::IDENTITY);
    let isect = s.try_intersect(
      crate::IntersectionContext {
        transform: &binding,
      },
      &ray,
    );
    assert_matches!(isect, None, "miss expected");
  }
  #[test]
  fn translated_hit() {
    let s = Sphere { radius: 1.0 };
    let ray = TestRay {
      position:  (2.0 * Vec3::Y).into(),
      direction: Vec3::NEG_Y.into(),
    };
    let binding = Transform::from_w2l(Mat4::from_translation(Vec3::Y));
    let isect = s.try_intersect(
      crate::IntersectionContext {
        transform: &binding,
      },
      &ray,
    );
    const POINT: PointLocal = PointLocal::new(Vec3::new(0.0, 1.0, 0.0));
    assert_matches!(
      isect,
      Some(SurfaceHit {
        normal: Vec3::Y,
        ray_distance: 2.0,
        ..
      })
    );
    assert_eq!(isect.unwrap().point, POINT, "wrong collision point");
  }
  #[test]
  fn tangential_hit() {
    let s = Sphere { radius: 1.0 };
    let ray = TestRay {
      position:  (2.0 * Vec3::Y).into(),
      direction: Vec3::NEG_Y.into(),
    };
    let binding = Transform::from_w2l(Mat4::from_translation(Vec3::X));
    let isect = s.try_intersect(
      crate::IntersectionContext {
        transform: &binding,
      },
      &ray,
    );
    const POINT: PointLocal = PointLocal::new(Vec3::new(1.0, 0.0, 0.0));
    assert_matches!(
      isect,
      Some(SurfaceHit {
        normal: Vec3::X,
        ray_distance: 2.0,
        ..
      })
    );
    assert_eq!(isect.unwrap().point, POINT, "wrong collision point");
  }
}
