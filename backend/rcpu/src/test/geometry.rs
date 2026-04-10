use glam::Vec3;

use crate::{Castable, PointGlobal};

struct TestRay {
  origin: PointGlobal,
  dir:    Vec3,
}

impl Castable for TestRay {
  fn pos(&self) -> PointGlobal { self.origin }

  fn direction(&self) -> glam::Vec3 { self.dir }
}

mod sphere {
  use std::assert_matches::assert_matches;

  use glam::{Mat4, Vec3, Vec4};

  use crate::{
    Geometry, PointLocal, Sphere, SurfaceHit, camera::CameraRay, test::geometry::TestRay,
  };

  #[test]
  fn direct_origin_collision() {
    let s = Sphere { radius: 1.0 };
    let ray = TestRay {
      origin: (2.0 * Vec3::Y).into(),
      dir:    Vec3::NEG_Y,
    };
    let isect = s.try_intersect(
      crate::IntersectionContext {
        g2l: Mat4::IDENTITY,
      },
      &ray,
    );
    const POINT: PointLocal = PointLocal::new(Vec4::new(0., 1., 0., 1.));
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
      origin: (2.0 * Vec3::Y).into(),
      dir:    Vec3::Y,
    };
    let isect = s.try_intersect(
      crate::IntersectionContext {
        g2l: Mat4::IDENTITY,
      },
      &ray,
    );
    assert_matches!(isect, None, "miss expected");
  }
  #[test]
  fn translated_hit() {
    let s = Sphere { radius: 1.0 };
    let ray = TestRay {
      origin: (2.0 * Vec3::Y).into(),
      dir:    Vec3::NEG_Y,
    };
    let isect = s.try_intersect(
      crate::IntersectionContext {
        g2l: Mat4::from_translation(Vec3::NEG_Y),
      },
      &ray,
    );
    const POINT: PointLocal = PointLocal::new(Vec4::new(0., 1., 0., 1.));
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
}
