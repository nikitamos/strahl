use std::ops::Deref;

use glam::{Vec3, Vec4Swizzles};

macro_rules! vec_wrapper {
  {
   $(#[$outer:meta])*
   $name:ident => $underlying:ty $(; $($conv:ty,)*)?
  } => {
    #[repr(transparent)]
    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    $(#[$outer])*
    pub struct $name($underlying);
    impl Deref for $name {
      type Target = $underlying;
      fn deref(&self) -> &Self::Target { &self.0 }
    }
    impl $name {
      pub const fn new(value: $underlying) -> Self { Self(value) }
    }
    impl From<$name> for $underlying {
      fn from(value: $name) -> $underlying { value.0 }
    }
    impl From<$underlying> for $name {
      fn from(value: $underlying) -> $name { Self(value) }
    }
    $($(
      impl From<$conv> for $name {
        fn from(value: $conv) -> Self{
          Self(value.into())
        }
      }
    )*)?
  };
}

vec_wrapper! {
  /// Type-safe wrapper around point in geometry space
  PointLocal => glam::Vec4
}

impl PointLocal {
  #[deprecated]
  pub fn into_global(self, l2g: glam::Mat4) -> PointGlobal { PointGlobal(l2g * self.0) }
}

impl From<PointLocal> for glam::Vec3 {
  fn from(value: PointLocal) -> Self { value.0.xyz() }
}

impl From<glam::Vec3> for PointLocal {
  fn from(value: glam::Vec3) -> Self { PointLocal(glam::vec4(value.x, value.y, value.z, 1.0)) }
}

vec_wrapper! {
  /// Type-safe wrapper around point in world coordinates
  PointGlobal => glam::Vec4
}

impl PointGlobal {
  #[deprecated]
  pub fn into_local(self, g2l: glam::Mat4) -> PointLocal { PointLocal(g2l * self.0) }
}

impl From<PointGlobal> for glam::Vec3 {
  fn from(value: PointGlobal) -> Self { value.0.xyz() }
}

impl From<glam::Vec3> for PointGlobal {
  fn from(value: glam::Vec3) -> Self { PointGlobal(glam::vec4(value.x, value.y, value.z, 1.0)) }
}

vec_wrapper! {
  /// Type-safe wrapper around vector in geometry coordinates
  VecLocal => Vec3; glam::Vec3A,
}

vec_wrapper! {
  /// Type-safe wrapper around vector in world coordinates
  VecGlobal => Vec3; glam::Vec3A,
}
